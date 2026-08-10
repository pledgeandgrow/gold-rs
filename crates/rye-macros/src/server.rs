//! Server action macro — transforms a function into a type-safe RPC.
//!
//! On the server target, the function runs directly and is registered
//! in the global action registry. On the client (Wasm) target, the
//! function body is replaced with a stub that serializes arguments
//! and calls the server via `rye_core::server_action::call_server`.
//!
//! ## Example
//!
//! ```ignore
//! #[server]
//! async fn create_user(name: String, email: String) -> Result<User, ServerError> {
//!     db.insert(name, email).await
//! }
//! ```
//!
//! On the client, `create_user("Alice".into(), "a@b.com".into())` makes
//! an HTTP POST to `/api/actions/create_user` with the serialized args.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::Type;
use syn::{parse_macro_input, FnArg, GenericArgument, ItemFn, PatType, PathArguments, ReturnType};

/// Implementation of the `#[server]` attribute macro.
pub fn server_impl(item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Extract argument types
    let arg_types: Vec<TokenStream2> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                Some(ty.to_token_stream())
            } else {
                None
            }
        })
        .collect();

    // Extract return type — we need the Ok type for deserialization
    let return_inner_type = extract_return_inner_type(&input_fn.sig.output);

    // Collect argument identifiers
    let arg_idents: Vec<syn::Ident> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(PatType { pat, .. }) = arg {
                if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                    return Some(pat_ident.ident.clone());
                }
            }
            None
        })
        .collect();

    // Generate the tuple type for arguments
    let args_tuple_type = if arg_types.len() == 1 {
        let t = &arg_types[0];
        quote! { (#t,) }
    } else if arg_types.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#arg_types),*) }
    };

    // Generate argument collection expression (for client stub)
    let args_expr = if arg_idents.is_empty() {
        quote! { () }
    } else if arg_idents.len() == 1 {
        let id = &arg_idents[0];
        quote! { (#id,) }
    } else {
        quote! { (#(#arg_idents),*) }
    };

    // Generate tuple destructuring for server-side call
    // e.g., for 2 args: let (a, b) = args; fn_name(a, b)
    let destructure_and_call = if arg_idents.is_empty() {
        quote! { #fn_name() }
    } else {
        let n = arg_idents.len();
        let indices: Vec<syn::Ident> = (0..n)
            .map(|i| syn::Ident::new(&format!("__arg{}", i), proc_macro2::Span::call_site()))
            .collect();
        quote! {
            let (#(#indices),*) = args;
            #fn_name(#(#indices),*)
        }
    };

    // Generate the function signature for the client stub
    let inputs = &input_fn.sig.inputs;
    let asyncness = &input_fn.sig.asyncness;

    let expanded = quote! {
        // === Server target: keep original function + register it ===
        #[cfg(not(target_arch = "wasm32"))]
        #input_fn

        // Register the action on the server at startup
        #[cfg(not(target_arch = "wasm32"))]
        ::rye_core::server_action::register_action(#fn_name_str, Box::new(|input: &str| {
            let input = input.to_string();
            Box::pin(async move {
                let args: #args_tuple_type = ::rye_serialize::deserialize(&input)
                    .ok_or_else(|| ::rye_core::server_action::ServerError::Deserialize(
                        "Failed to deserialize server action input".to_string()
                    ))?;
                let result = #destructure_and_call .await;
                match result {
                    Ok(val) => {
                        let serialized = ::rye_serialize::serialize(&val);
                        Ok(serialized)
                    }
                    Err(e) => {
                        let serialized = ::rye_serialize::serialize(&e);
                        Ok(serialized)
                    }
                }
            })
        }));

        // === Client target (Wasm): stub that calls the server ===
        #[cfg(target_arch = "wasm32")]
        #asyncness fn #fn_name(
            #inputs
        ) -> Result<#return_inner_type, ::rye_core::server_action::ServerError> {
            let args: #args_tuple_type = #args_expr;
            let input = ::rye_serialize::serialize(&args);
            let output = ::rye_core::server_action::call_server(#fn_name_str, &input).await?;
            if let Some(result) = ::rye_serialize::deserialize::<Result<#return_inner_type, ::rye_core::server_action::ServerError>>(&output) {
                return result;
            }
            ::rye_serialize::deserialize::<#return_inner_type>(&output)
                .ok_or_else(|| ::rye_core::server_action::ServerError::Deserialize(
                    "Failed to deserialize server action response".to_string()
                ))
        }
    };

    TokenStream::from(expanded)
}

/// Extract the inner type from a `Result<T, E>` return type.
fn extract_return_inner_type(output: &ReturnType) -> TokenStream2 {
    match output {
        ReturnType::Type(_, ty) => {
            // Try to extract T from Result<T, E>
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                                return inner_ty.to_token_stream();
                            }
                        }
                    }
                }
            }
            // If not Result, return the type itself
            ty.to_token_stream()
        }
        ReturnType::Default => quote! { () },
    }
}
