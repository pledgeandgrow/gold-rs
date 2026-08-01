//! Component trait and props abstraction.

use crate::element::Element;

/// A component is a function that produces an Element.
///
/// Components are the building blocks of a rye application.
/// They can be function components (via `#[component]` macro)
/// or trait implementations.
pub trait Component: 'static {
    /// The typed props for this component.
    type Props: ComponentProps;

    /// Render the component to an Element.
    fn render(props: Self::Props) -> Element;
}

/// A marker trait for component props.
///
/// Props are plain Rust structs that implement this trait.
/// The `#[component]` macro auto-generates this implementation.
pub trait ComponentProps: 'static {}

/// Function component wrapper — used by the `#[component]` macro.
pub struct FunctionComponent<F, P>
where
    F: Fn(P) -> Element + 'static,
    P: ComponentProps,
{
    func: F,
    _marker: std::marker::PhantomData<P>,
}

impl<F, P> FunctionComponent<F, P>
where
    F: Fn(P) -> Element + 'static,
    P: ComponentProps,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn render(&self, props: P) -> Element {
        (self.func)(props)
    }
}
