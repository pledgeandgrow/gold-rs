//! Component macro implementation — wraps a function into a typed component.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident, ToTokens};
use syn::{ItemFn, FnArg, PatType, parse_macro_input};

/// Implementation of the `#[component]` attribute macro.
///
/// Transforms a function like:
/// ```ignore
/// #[component]
/// fn Counter(props: CounterProps) -> Element {
///     template! { ... }
/// }
/// ```
///
/// Into:
/// ```ignore
/// fn Counter(props: CounterProps) -> Element {
///     template! { ... }
/// }
///
/// struct CounterComponent;
/// impl Component for CounterComponent {
///     type Props = CounterProps;
///     fn render(props: Self::Props) -> Element {
///         Counter(props)
///     }
/// }
/// ```
pub fn component_impl(item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Extract props type from the first argument
    let props_type = extract_props_type(&input_fn);

    // Generate the component struct and impl
    let component_struct_name = format_ident!("{}Component", fn_name);

    let expanded = quote! {
        // Keep the original function
        #input_fn

        // Generate a component struct
        pub struct #component_struct_name;

        impl ::rye_core::Component for #component_struct_name {
            type Props = #props_type;

            fn render(props: Self::Props) -> ::rye_core::Element {
                #fn_name(props)
            }
        }

        // Implement ComponentProps for the props type if not already implemented
        // (The user should derive or implement ComponentProps manually)
    };

    TokenStream::from(expanded)
}

/// Extract the props type from the function's first argument.
fn extract_props_type(item_fn: &ItemFn) -> TokenStream2 {
    if let Some(FnArg::Typed(PatType { ty, .. })) = item_fn.sig.inputs.first() {
        return ty.to_token_stream();
    }

    // Fallback: use a generic type
    quote! { () }
}
