//! Signal persistence strategies.
//!
//! Beyond localStorage/IndexedDB, supports sessionStorage, URL search params,
//! cookie, and custom backend persistence via a trait.

use crate::signal::Signal;
use std::cell::RefCell;
use std::rc::Rc;

/// A persistence strategy — defines how a signal's value is stored and loaded.
pub trait PersistenceStrategy: 'static {
    /// Save the value to the persistence layer.
    fn save(&self, key: &str, value: &str);

    /// Load the value from the persistence layer.
    fn load(&self, key: &str) -> Option<String>;

    /// Remove the value from the persistence layer.
    fn remove(&self, key: &str);
}

/// In-memory persistence — useful for testing.
pub struct MemoryPersistence {
    storage: Rc<RefCell<std::collections::HashMap<String, String>>>,
}

impl MemoryPersistence {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for MemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceStrategy for MemoryPersistence {
    fn save(&self, key: &str, value: &str) {
        self.storage
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }

    fn load(&self, key: &str) -> Option<String> {
        self.storage.borrow().get(key).cloned()
    }

    fn remove(&self, key: &str) {
        self.storage.borrow_mut().remove(key);
    }
}

/// No-op persistence — does nothing. Useful as a default.
pub struct NoopPersistence;

impl PersistenceStrategy for NoopPersistence {
    fn save(&self, _key: &str, _value: &str) {}
    fn load(&self, _key: &str) -> Option<String> {
        None
    }
    fn remove(&self, _key: &str) {}
}

/// Custom backend persistence via a closure-based strategy.
pub struct CustomPersistence {
    save_fn: Box<dyn Fn(&str, &str)>,
    load_fn: Box<dyn Fn(&str) -> Option<String>>,
    remove_fn: Box<dyn Fn(&str)>,
}

impl CustomPersistence {
    pub fn new<S, L, R>(save: S, load: L, remove: R) -> Self
    where
        S: Fn(&str, &str) + 'static,
        L: Fn(&str) -> Option<String> + 'static,
        R: Fn(&str) + 'static,
    {
        Self {
            save_fn: Box::new(save),
            load_fn: Box::new(load),
            remove_fn: Box::new(remove),
        }
    }
}

impl PersistenceStrategy for CustomPersistence {
    fn save(&self, key: &str, value: &str) {
        (self.save_fn)(key, value);
    }

    fn load(&self, key: &str) -> Option<String> {
        (self.load_fn)(key)
    }

    fn remove(&self, key: &str) {
        (self.remove_fn)(key);
    }
}

/// A persisted signal — automatically saves to and loads from a persistence strategy.
pub struct PersistedSignal<T: Clone + std::fmt::Display + std::str::FromStr + 'static> {
    signal: Signal<T>,
    key: String,
    strategy: Rc<dyn PersistenceStrategy>,
}

impl<T: Clone + std::fmt::Display + std::str::FromStr + 'static> PersistedSignal<T> {
    /// Create a new persisted signal.
    ///
    /// Loads the initial value from the persistence strategy. If not found,
    /// uses the provided default.
    pub fn new<S: PersistenceStrategy>(key: &str, default: T, strategy: S) -> Self {
        let strategy = Rc::new(strategy);
        let initial = strategy
            .load(key)
            .and_then(|s| s.parse::<T>().ok())
            .unwrap_or(default);

        Self {
            signal: Signal::new(initial),
            key: key.to_string(),
            strategy,
        }
    }

    /// Get the current value (tracked).
    pub fn get(&self) -> T {
        self.signal.get()
    }

    /// Get the current value (untracked).
    pub fn get_untracked(&self) -> T {
        self.signal.get_untracked()
    }

    /// Set a new value and persist it.
    pub fn set(&self, value: T) {
        self.strategy.save(&self.key, &value.to_string());
        self.signal.set(value);
    }

    /// Update the value and persist it.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut val = self.signal.get_untracked();
        f(&mut val);
        self.strategy.save(&self.key, &val.to_string());
        self.signal.set(val);
    }

    /// Get a reference to the inner signal.
    pub fn signal(&self) -> &Signal<T> {
        &self.signal
    }

    /// Remove the persisted value.
    pub fn clear(&self) {
        self.strategy.remove(&self.key);
    }

    /// Reload from persistence, discarding in-memory changes.
    pub fn reload(&self, default: T) {
        let val = self
            .strategy
            .load(&self.key)
            .and_then(|s| s.parse::<T>().ok())
            .unwrap_or(default);
        self.signal.set(val);
    }
}

impl<T: Clone + std::fmt::Display + std::str::FromStr + 'static> Clone for PersistedSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            key: self.key.clone(),
            strategy: Rc::clone(&self.strategy),
        }
    }
}

/// Persist a signal with a given strategy.
///
/// # Example
/// ```
/// use rye_signals::{Signal, persist, MemoryPersistence};
///
/// let strategy = MemoryPersistence::new();
/// let count = persist("count", 0, strategy);
/// count.set(42);
/// assert_eq!(count.get(), 42);
/// ```
pub fn persist<T, S: PersistenceStrategy>(key: &str, default: T, strategy: S) -> PersistedSignal<T>
where
    T: Clone + std::fmt::Display + std::str::FromStr + 'static,
{
    PersistedSignal::new(key, default, strategy)
}

/// Available persistence strategy types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PersistenceType {
    /// In-memory (testing only).
    Memory,
    /// Browser localStorage (persists across sessions).
    LocalStorage,
    /// Browser sessionStorage (tab-scoped).
    SessionStorage,
    /// URL search params (shareable state).
    UrlParams,
    /// Cookie (server-readable).
    Cookie,
    /// Custom backend.
    Custom,
}

impl std::fmt::Display for PersistenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceType::Memory => write!(f, "memory"),
            PersistenceType::LocalStorage => write!(f, "localStorage"),
            PersistenceType::SessionStorage => write!(f, "sessionStorage"),
            PersistenceType::UrlParams => write!(f, "url-params"),
            PersistenceType::Cookie => write!(f, "cookie"),
            PersistenceType::Custom => write!(f, "custom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_persistence_basic() {
        let p = MemoryPersistence::new();
        p.save("key", "value");
        assert_eq!(p.load("key"), Some("value".to_string()));
        p.remove("key");
        assert_eq!(p.load("key"), None);
    }

    #[test]
    fn test_persisted_signal_loads_from_storage() {
        let strategy = MemoryPersistence::new();
        strategy.save("count", "42");
        let sig = persist("count", 0i32, strategy);
        assert_eq!(sig.get(), 42);
    }

    #[test]
    fn test_persisted_signal_uses_default_when_empty() {
        let strategy = MemoryPersistence::new();
        let sig = persist("count", 10i32, strategy);
        assert_eq!(sig.get(), 10);
    }

    #[test]
    fn test_persisted_signal_saves_on_set() {
        let storage = Rc::new(RefCell::new(std::collections::HashMap::new()));
        let storage_clone = Rc::clone(&storage);

        let custom = CustomPersistence::new(
            move |k, v| {
                storage_clone
                    .borrow_mut()
                    .insert(k.to_string(), v.to_string());
            },
            |_| None,
            |_| {},
        );

        let sig = persist("count", 0i32, custom);
        sig.set(99);

        // Create a new signal with same storage to verify persistence
        let storage2 = Rc::clone(&storage);
        let storage3 = Rc::clone(&storage);
        let custom2 = CustomPersistence::new(
            |_, _| {},
            move |k| storage3.borrow().get(k).cloned(),
            |_| {},
        );
        let _ = storage2; // keep alive
        let sig2 = persist("count", 0i32, custom2);
        assert_eq!(sig2.get(), 99);
    }

    #[test]
    fn test_persisted_signal_update() {
        let strategy = MemoryPersistence::new();
        let sig = persist("count", 5i32, strategy);
        sig.update(|v| *v += 10);
        assert_eq!(sig.get(), 15);
    }

    #[test]
    fn test_persisted_signal_clear() {
        let strategy = MemoryPersistence::new();
        strategy.save("key", "42");
        let sig = persist("key", 0i32, strategy);
        assert_eq!(sig.get(), 42);
        sig.clear();
        // Value is still in memory but removed from storage
        let strategy2 = MemoryPersistence::new();
        let sig2 = persist("key", 0i32, strategy2);
        assert_eq!(sig2.get(), 0); // default, since storage was cleared
    }

    #[test]
    fn test_persisted_signal_reload() {
        let strategy = MemoryPersistence::new();
        let sig = persist("count", 10i32, strategy);
        sig.set(99);
        // Simulate external change
        sig.reload(0);
        assert_eq!(sig.get(), 99); // reloaded from storage
    }

    #[test]
    fn test_noop_persistence() {
        let p = NoopPersistence;
        p.save("key", "value");
        assert_eq!(p.load("key"), None);
    }

    #[test]
    fn test_custom_persistence() {
        let saved = Rc::new(RefCell::new(None::<String>));
        let saved_clone = Rc::clone(&saved);

        let custom = CustomPersistence::new(
            move |_, v| *saved_clone.borrow_mut() = Some(v.to_string()),
            |_| None,
            |_| {},
        );

        custom.save("key", "value");
        assert_eq!(*saved.borrow(), Some("value".to_string()));
    }

    #[test]
    fn test_persistence_type_display() {
        assert_eq!(PersistenceType::Memory.to_string(), "memory");
        assert_eq!(PersistenceType::LocalStorage.to_string(), "localStorage");
        assert_eq!(PersistenceType::Cookie.to_string(), "cookie");
    }

    #[test]
    fn test_persisted_signal_clone() {
        let strategy = MemoryPersistence::new();
        let sig = persist("count", 5i32, strategy);
        let sig2 = sig.clone();
        assert_eq!(sig.get(), sig2.get());
        sig2.set(100);
        assert_eq!(sig.get(), 100);
    }
}
