//! GlobalSignal — app-wide reactive state without context providers.

use crate::runtime;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any::TypeId;

/// A global signal accessible from any component without context.
///
/// Uses `TypeId`-keyed storage for lazy initialization.
///
/// # Example
/// ```
/// use rye_signals::GlobalSignal;
///
/// static THEME: GlobalSignal<String> = GlobalSignal::new(|| "light".to_string());
///
/// // Read
/// assert_eq!(THEME.get(), "light");
/// // Write
/// THEME.set("dark".to_string());
/// assert_eq!(THEME.get(), "dark");
/// ```
pub struct GlobalSignal<T: Send + Sync + Clone + 'static> {
    init: fn() -> T,
}

// Global storage for global signal values.
// Each global signal is identified by its TypeId + address.
thread_local! {
    static GLOBAL_STORE: RefCell<HashMap<(TypeId, usize), Box<dyn std::any::Any>>> =
        RefCell::new(HashMap::new());
    static GLOBAL_IDS: RefCell<HashMap<(TypeId, usize), usize>> =
        RefCell::new(HashMap::new());
}

impl<T: Send + Sync + Clone + 'static> GlobalSignal<T> {
    /// Create a new global signal with a lazy initializer.
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            init,
        }
    }

    fn key(&self) -> (TypeId, usize) {
        let ptr = &self.init as *const _ as *const () as usize;
        (TypeId::of::<T>(), ptr)
    }

    fn signal_id(&self) -> usize {
        let key = self.key();
        GLOBAL_IDS.with(|ids| {
            let mut ids = ids.borrow_mut();
            if let Some(&id) = ids.get(&key) {
                id
            } else {
                let id = runtime::next_id();
                ids.insert(key, id);
                id
            }
        })
    }

    fn ensure_init(&self) {
        let key = self.key();
        GLOBAL_STORE.with(|store| {
            let mut store = store.borrow_mut();
            if !store.contains_key(&key) {
                store.insert(key, Box::new((self.init)()));
            }
        });
    }

    /// Get a clone of the current value (tracked).
    pub fn get(&self) -> T {
        self.ensure_init();
        runtime::track(self.signal_id());
        let key = self.key();
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&key)
                .and_then(|v| v.downcast_ref::<T>())
                .cloned()
                .expect("GlobalSignal not initialized")
        })
    }

    /// Get a clone of the current value (untracked).
    pub fn get_untracked(&self) -> T {
        self.ensure_init();
        let key = self.key();
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&key)
                .and_then(|v| v.downcast_ref::<T>())
                .cloned()
                .expect("GlobalSignal not initialized")
        })
    }

    /// Set a new value and notify subscribers.
    pub fn set(&self, value: T) {
        self.ensure_init();
        let key = self.key();
        GLOBAL_STORE.with(|store| {
            *store
                .borrow_mut()
                .get_mut(&key)
                .and_then(|v| v.downcast_mut::<T>())
                .expect("GlobalSignal not initialized") = value;
        });
        runtime::notify(self.signal_id());
    }

    /// Functional update.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        self.ensure_init();
        let key = self.key();
        GLOBAL_STORE.with(|store| {
            let mut store = store.borrow_mut();
            let v = store
                .get_mut(&key)
                .and_then(|v| v.downcast_mut::<T>())
                .expect("GlobalSignal not initialized");
            f(v);
        });
        runtime::notify(self.signal_id());
    }
}

unsafe impl<T: Send + Sync + Clone + 'static> Sync for GlobalSignal<T> {}
unsafe impl<T: Send + Sync + Clone + 'static> Send for GlobalSignal<T> {}
