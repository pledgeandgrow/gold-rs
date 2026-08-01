//! # rye-macros
//!
//! Procedural macros for rye — `template!`, `#[component]`, custom diagnostics.

mod template;
mod component;
mod server;

use proc_macro::TokenStream;

/// The `template!` macro — HTML-like template syntax that compiles to
/// optimized static templates with dynamic signal bindings.
///
/// # Example
/// ```ignore
/// template! {
///     div {
///         class: "container",
///         h1 { "Hello, " {name} "!" }
///         button { onclick: move |_| increment(), "Click me" }
///     }
/// }
/// ```
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let tokens = proc_macro2::TokenStream::from(input);
    match template::parse_template(tokens) {
        Ok(node) => {
            let code = template::generate_code(&node);
            TokenStream::from(code)
        }
        Err(msg) => {
            // Generate a compile error
            let msg = format!("template! macro error: {}", msg);
            TokenStream::from(quote::quote! {
                compile_error!(#msg);
            })
        }
    }
}

/// The `#[component]` attribute macro — wraps a Rust function into a
/// typed component with validated props.
///
/// # Example
/// ```ignore
/// #[component]
/// fn Counter(props: CounterProps) -> Element {
///     template! { ... }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component_impl(item)
}

/// The `#[server]` attribute macro — transforms a function into a type-safe
/// server action with automatic serialization.
///
/// On the server, the function runs directly and is registered in the
/// action registry. On the client (Wasm), the function body is replaced
/// with a stub that serializes arguments and calls the server via HTTP.
///
/// # Example
/// ```ignore
/// #[server]
/// async fn get_user(id: u32) -> Result<String, ServerError> {
///     db.find_user(id).await
/// }
/// ```
#[proc_macro_attribute]
pub fn server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    server::server_impl(item)
}
