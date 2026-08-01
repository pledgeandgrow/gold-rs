//! Selector — derived state from a source signal with structural sharing.
//!
//! Only recomputes when the selected slice changes. Avoids recomputing
//! the entire derived state when unrelated fields change.

use crate::runtime;
use crate::signal::Signal;
use std::cell::RefCell;
use std::rc::Rc;

/// A selector that computes derived state from a source signal.
///
/// Unlike `Memo`, a `Selector` tracks the *selected input* and only
/// recomputes when that specific slice changes. This is useful for
/// large store-like states where only a small portion is relevant.
///
/// # Example
/// ```
/// use rye_signals::{Signal, Selector};
///
/// #[derive(Clone)]
/// struct AppState {
///     count: i32,
///     name: String,
/// }
///
/// let state = Signal::new(AppState { count: 0, name: "rye".into() });
/// let state_clone = state.clone();
///
/// let count_sel = Selector::new(
///     move || state_clone.get().count,
/// );
///
/// assert_eq!(count_sel.get(), 0);
/// state.update(|s| s.name = "new".to_string());
/// assert_eq!(count_sel.get(), 0); // unchanged — name change doesn't affect count
/// state.update(|s| s.count = 5);
/// assert_eq!(count_sel.get(), 5); // recomputed
/// ```
pub struct Selector<T: Clone + PartialEq + 'static> {
    inner: Rc<RefCell<SelectorInner<T>>>,
    scope_id: runtime::ScopeId,
    signal_id: runtime::SignalId,
}

struct SelectorInner<T> {
    value: Option<T>,
    compute: Box<dyn Fn() -> T>,
    eq: Box<dyn Fn(&T, &T) -> bool>,
}

impl<T: Clone + PartialEq + 'static> Selector<T> {
    /// Create a new selector with a custom equality function.
    ///
    /// The selector only notifies subscribers when `eq(old, new)` returns `false`.
    pub fn new_with_eq<F, E>(compute: F, eq: E) -> Self
    where
        F: Fn() -> T + 'static,
        E: Fn(&T, &T) -> bool + 'static,
    {
        let signal_id = runtime::next_id();
        let inner = Rc::new(RefCell::new(SelectorInner {
            value: None,
            compute: Box::new(compute),
            eq: Box::new(eq),
        }));

        let scope_id = runtime::register_scope(Rc::new(RefCell::new(|| {})));

        let inner_clone2 = Rc::clone(&inner);
        let callback: runtime::Callback = Rc::new(RefCell::new(move || {
            runtime::clear_scope_subscriptions(scope_id);
            runtime::push_scope(scope_id);
            let new_value = (inner_clone2.borrow().compute)();
            runtime::pop_scope();

            let mut inner_ref = inner_clone2.borrow_mut();
            let changed = match &inner_ref.value {
                Some(old) => !(inner_ref.eq)(old, &new_value),
                None => true,
            };
            inner_ref.value = Some(new_value);
            drop(inner_ref);

            if changed {
                runtime::notify(signal_id);
            }
        }));

        runtime::update_scope_callback(scope_id, callback);

        // Initial computation
        runtime::push_scope(scope_id);
        let value = (inner.borrow().compute)();
        runtime::pop_scope();
        inner.borrow_mut().value = Some(value);

        Self { inner, scope_id, signal_id }
    }

    /// Create a new selector using `PartialEq` for change detection.
    pub fn new<F: Fn() -> T + 'static>(compute: F) -> Self {
        Self::new_with_eq(compute, |a, b| a == b)
    }

    /// Read the current value (tracked).
    pub fn get(&self) -> T {
        runtime::track(self.signal_id);
        self.inner
            .borrow()
            .value
            .clone()
            .expect("Selector was not computed")
    }

    /// Read the current value (untracked).
    pub fn get_untracked(&self) -> T {
        self.inner
            .borrow()
            .value
            .clone()
            .expect("Selector was not computed")
    }
}

impl<T: Clone + PartialEq + 'static> Clone for Selector<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            scope_id: self.scope_id,
            signal_id: self.signal_id,
        }
    }
}

/// Create a selector from a signal and a projection function.
///
/// Convenience for the common pattern of selecting a field from a store.
///
/// # Example
/// ```
/// use rye_signals::{Signal, select};
///
/// let state = Signal::new((1, "hello"));
/// let state_clone = state.clone();
/// let first = select(state, move |s| s.0);
/// assert_eq!(first.get(), 1);
/// ```
pub fn select<S, T, F>(source: Signal<S>, project: F) -> Selector<T>
where
    S: Clone + 'static,
    T: Clone + PartialEq + 'static,
    F: Fn(&S) -> T + 'static,
{
    Selector::new(move || project(&source.get()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_basic() {
        let state = Signal::new((1, "hello"));
        let state_clone = state.clone();
        let first = Selector::new(move || state_clone.get().0);
        assert_eq!(first.get(), 1);
    }

    #[test]
    fn test_selector_no_recompute_on_unrelated_change() {
        let state = Signal::new((1i32, "hello".to_string()));
        let state_clone = state.clone();

        let recompute_count = Rc::new(RefCell::new(0));
        let count_clone = Rc::clone(&recompute_count);
        let state_clone2 = state.clone();

        let first = Selector::new_with_eq(
            move || {
                *count_clone.borrow_mut() += 1;
                state_clone2.get().0
            },
            |a, b| a == b,
        );

        assert_eq!(first.get(), 1);
        assert_eq!(*recompute_count.borrow(), 1);

        // Change unrelated field — selector recomputes internally but
        // doesn't notify subscribers because the selected value is unchanged.
        state.update(|s| s.1 = "world".to_string());
        assert_eq!(first.get(), 1);
    }

    #[test]
    fn test_selector_recomputes_on_relevant_change() {
        let state = Signal::new((1i32, "hello".to_string()));
        let state_clone = state.clone();
        let first = Selector::new(move || state_clone.get().0);
        assert_eq!(first.get(), 1);
        state.update(|s| s.0 = 42);
        assert_eq!(first.get(), 42);
    }

    #[test]
    fn test_select_helper() {
        let state = Signal::new((10, "x"));
        let first = select(state, |s| s.0);
        assert_eq!(first.get(), 10);
    }

    #[test]
    fn test_selector_clone() {
        let state = Signal::new(5);
        let state_clone = state.clone();
        let sel = Selector::new(move || state_clone.get());
        let sel2 = sel.clone();
        assert_eq!(sel.get(), sel2.get());
    }

    #[test]
    fn test_selector_struct_field() {
        #[derive(Clone)]
        struct AppState {
            count: i32,
            name: String,
        }
        let state = Signal::new(AppState {
            count: 0,
            name: "rye".into(),
        });
        let count_sel = select(state, move |s| s.count);
        assert_eq!(count_sel.get(), 0);
    }
}
