//! Hooks — component-local state for the rye framework.
//!
//! Since `mount()` calls the render function exactly once, hooks like
//! `use_signal` create persistent state during that single render pass.
//! The state is stored in a `HookScope` that lives for the lifetime of
//! the `RenderScope`, so signals created by `use_signal` remain valid
//! and reactive after rendering completes.
//!
//! ## How it works
//!
//! 1. `mount()` sets up a thread-local `HookContext` before calling
//!    the render function.
//! 2. `use_signal()` creates a `Signal<T>`, stores it in the context,
//!    and returns a clone.
//! 3. After the render function returns, `mount()` collects the context
//!    into the `RenderScope` to keep the signals alive.
//! 4. Subsequent `Effect`s created by the render loop can read these
//!    signals and react to changes.
//!
//! ## Example
//!
//! ```ignore
//! use rye_core::{mount, use_signal};
//!
//! let scope = mount(|| {
//!     let count = use_signal(|| 0);
//!     // count is Signal<i32> — can be read in reactive bindings
//!     template! {
//!         div { "Count: " {count.get()} }
//!     }
//! });
//! ```
//!
//! ## Nesting
//!
//! `use_signal` can be called multiple times within a single render pass.
//! Each call creates a new independent signal. The order of calls matters
//! only for identification — each `use_signal` call gets a unique index.

use rye_signals::Signal;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static HOOK_CONTEXT: RefCell<Option<HookContext>> = RefCell::new(None);
}

/// Internal context for hooks during a render pass.
struct HookContext {
    /// Signals created by `use_signal` — kept alive as `Rc<dyn Any>`.
    signals: Vec<Rc<dyn std::any::Any>>,
}

impl HookContext {
    fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }
}

/// Set up the hook context for a render pass.
///
/// Called by `mount()` before invoking the render function.
/// Returns a guard that, when dropped, collects all created signals
/// and clears the thread-local.
pub(crate) struct HookGuard {
    context: Option<HookContext>,
}

impl HookGuard {
    /// Take ownership of the signals created during the render pass.
    pub(crate) fn into_signals(self) -> Vec<Rc<dyn std::any::Any>> {
        self.context.map(|c| c.signals).unwrap_or_default()
    }
}

/// Enter a hook scope — sets up the thread-local context.
///
/// Returns a guard that must be kept alive for the duration of the render pass.
/// When dropped, the context is cleared.
pub(crate) fn enter_hook_scope() -> HookGuard {
    HOOK_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(HookContext::new());
    });
    HookGuard {
        context: Some(HookContext::new()),
    }
}

/// Drain the thread-local context into the guard.
///
/// Called after the render function returns to collect signals.
pub(crate) fn drain_hook_context(guard: &mut HookGuard) {
    HOOK_CONTEXT.with(|ctx| {
        if let Some(hook_ctx) = ctx.borrow_mut().take() {
            if let Some(guard_ctx) = &mut guard.context {
                guard_ctx.signals.extend(hook_ctx.signals);
            }
        }
    });
}

/// Create a signal with an initial value computed by the closure.
///
/// The signal persists for the lifetime of the `RenderScope` returned by
/// `mount()`. Reading the signal inside an `Effect` (e.g. in a reactive
/// template binding) creates a dependency — the effect re-runs when the
/// signal changes.
///
/// # Panics
///
/// Panics if called outside of a `mount()` render pass.
///
/// # Example
///
/// ```ignore
/// use rye_core::{mount, use_signal};
///
/// let scope = mount(|| {
///     let count = use_signal(|| 0);
///     template! {
///         div { "Count: " {count.get()} }
///     }
/// });
/// ```
pub fn use_signal<T, F>(init: F) -> Signal<T>
where
    T: Clone + 'static,
    F: FnOnce() -> T,
{
    let signal = Signal::new(init());

    HOOK_CONTEXT.with(|ctx| {
        let mut borrow = ctx.borrow_mut();
        if let Some(hook_ctx) = borrow.as_mut() {
            hook_ctx.signals.push(Rc::new(signal.clone()));
        } else {
            panic!("use_signal called outside of a mount() render pass");
        }
    });

    signal
}

/// Create a signal with a default value.
///
/// Convenience for `use_signal(T::default())`.
pub fn use_signal_default<T>() -> Signal<T>
where
    T: Clone + Default + 'static,
{
    use_signal(T::default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::Element;
    use crate::renderer::Renderer;
    use crate::template::Template;

    /// A minimal test renderer.
    struct TestRenderer;
    impl Renderer for TestRenderer {
        type Node = String;
        type Text = String;
        type Element = String;
        fn create_element(&mut self, tag: &str) -> Self::Element {
            tag.to_string()
        }
        fn create_text(&mut self, content: &str) -> Self::Text {
            content.to_string()
        }
        fn set_text(&mut self, _node: &Self::Text, _content: &str) {}
        fn set_attribute(&mut self, _el: &Self::Element, _name: &str, _value: &str) {}
        fn remove_attribute(&mut self, _el: &Self::Element, _name: &str) {}
        fn insert_child(&mut self, _parent: &Self::Element, _child: &Self::Node, _index: usize) {}
        fn remove_child(&mut self, _parent: &Self::Element, _index: usize) {}
        fn replace_child(&mut self, _parent: &Self::Element, _new: &Self::Node, _index: usize) {}
        fn move_child(&mut self, _parent: &Self::Element, _from: usize, _to: usize) {}
        fn set_event_listener(
            &mut self,
            _el: &Self::Element,
            _event: &str,
            _handler: crate::renderer::EventHandler,
        ) {
        }
        fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {}
        fn root(&self) -> Self::Element {
            "root".to_string()
        }
        fn text_to_node(&self, text: &Self::Text) -> Self::Node {
            text.clone()
        }
        fn element_to_node(&self, el: &Self::Element) -> Self::Node {
            el.clone()
        }
    }

    #[test]
    fn test_use_signal_in_mount() {
        let renderer = TestRenderer;
        let scope = crate::render_loop::mount(
            || {
                let count = use_signal(|| 42);
                assert_eq!(count.get(), 42);
                Element::Template(Template::text("hello"))
            },
            renderer,
        );
        // Scope keeps signals alive
        drop(scope);
    }

    #[test]
    fn test_use_signal_multiple() {
        let renderer = TestRenderer;
        let scope = crate::render_loop::mount(
            || {
                let a = use_signal(|| 1);
                let b = use_signal(|| 2);
                assert_eq!(a.get(), 1);
                assert_eq!(b.get(), 2);
                Element::Template(Template::text("ok"))
            },
            renderer,
        );
        drop(scope);
    }

    #[test]
    fn test_use_signal_default() {
        let renderer = TestRenderer;
        let scope = crate::render_loop::mount(
            || {
                let count: Signal<i32> = use_signal_default();
                assert_eq!(count.get(), 0);
                Element::Template(Template::text("ok"))
            },
            renderer,
        );
        drop(scope);
    }

    #[test]
    #[should_panic(expected = "outside of a mount")]
    fn test_use_signal_outside_mount_panics() {
        let _ = use_signal(|| 42);
    }

    #[test]
    fn test_use_signal_persists_after_render() {
        let renderer = TestRenderer;
        let scope = crate::render_loop::mount(
            || {
                let count = use_signal(|| 10);
                // Return a template that reads the signal
                let count_clone = count.clone();
                Element::Template(Template::new(vec![
                    crate::template::TemplateNode::Reactive(std::rc::Rc::new(move || {
                        count_clone.get().to_string()
                    })),
                ]))
            },
            renderer,
        );
        // The scope should keep the signal and effect alive
        drop(scope);
    }
}
