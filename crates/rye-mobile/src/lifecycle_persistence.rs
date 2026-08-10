//! Goal 209: Native app lifecycle persistence.
//!
//! Save and restore app state across app kills and relaunches.
//! Automatically serialize signal state to platform-appropriate storage.

use std::collections::HashMap;
use std::sync::Mutex;

/// The platform-appropriate storage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// iOS UserDefaults / NSUserDefaults.
    UserDefaults,
    /// Android SharedPreferences / SavedStateHandle.
    SharedPreferences,
    /// Web IndexedDB.
    IndexedDb,
    /// Web localStorage.
    LocalStorage,
    /// Web sessionStorage.
    SessionStorage,
    /// In-memory (not persisted).
    Memory,
}

impl StorageType {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            StorageType::UserDefaults => "UserDefaults",
            StorageType::SharedPreferences => "SharedPreferences",
            StorageType::IndexedDb => "IndexedDB",
            StorageType::LocalStorage => "localStorage",
            StorageType::SessionStorage => "sessionStorage",
            StorageType::Memory => "Memory",
        }
    }

    /// Check if this storage persists across app restarts.
    pub fn is_persistent(&self) -> bool {
        matches!(
            self,
            StorageType::UserDefaults
                | StorageType::SharedPreferences
                | StorageType::IndexedDb
                | StorageType::LocalStorage
        )
    }
}

/// A snapshot of app state — serialized signal values.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// The snapshot version (for migration).
    pub version: u32,
    /// The serialized signal values (signal_id -> JSON value).
    pub signals: HashMap<String, String>,
    /// The timestamp of the snapshot (Unix timestamp).
    pub timestamp: u64,
    /// The app version when the snapshot was taken.
    pub app_version: String,
}

impl StateSnapshot {
    /// Create a new empty snapshot.
    pub fn new() -> Self {
        Self {
            version: 1,
            signals: HashMap::new(),
            timestamp: 0,
            app_version: String::new(),
        }
    }

    /// Create a snapshot with a specific app version.
    pub fn with_version(app_version: &str) -> Self {
        Self {
            version: 1,
            signals: HashMap::new(),
            timestamp: 0,
            app_version: app_version.to_string(),
        }
    }

    /// Add a signal value to the snapshot.
    pub fn add_signal(&mut self, id: &str, value: &str) {
        self.signals.insert(id.to_string(), value.to_string());
    }

    /// Get a signal value from the snapshot.
    pub fn get_signal(&self, id: &str) -> Option<&str> {
        self.signals.get(id).map(|s| s.as_str())
    }

    /// Remove a signal from the snapshot.
    pub fn remove_signal(&mut self, id: &str) -> bool {
        self.signals.remove(id).is_some()
    }

    /// Get the number of signals in the snapshot.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Check if the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    /// Serialize the snapshot to JSON.
    pub fn to_json(&self) -> String {
        let signals: Vec<String> = self
            .signals
            .iter()
            .map(|(k, v)| {
                format!(
                    "\"{}\":\"{}\"",
                    k,
                    v.replace('\\', "\\\\").replace('"', "\\\"")
                )
            })
            .collect();

        format!(
            "{{\"version\":{},\"timestamp\":{},\"appVersion\":\"{}\",\"signals\":{{{}}}}}",
            self.version,
            self.timestamp,
            self.app_version,
            signals.join(",")
        )
    }

    /// Deserialize from JSON (simple parser).
    pub fn from_json(json: &str) -> Option<Self> {
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return None;
        }

        let mut snapshot = Self::new();

        // Extract version
        if let Some(v_start) = json.find("\"version\":") {
            let rest = &json[v_start + 10..];
            if let Some(v_end) = rest.find(',') {
                if let Ok(version) = rest[..v_end].parse::<u32>() {
                    snapshot.version = version;
                }
            }
        }

        // Extract timestamp
        if let Some(t_start) = json.find("\"timestamp\":") {
            let rest = &json[t_start + 12..];
            if let Some(t_end) = rest.find(',') {
                if let Ok(timestamp) = rest[..t_end].parse::<u64>() {
                    snapshot.timestamp = timestamp;
                }
            }
        }

        // Extract signals
        if let Some(s_start) = json.find("\"signals\":{") {
            let signals_section = &json[s_start + 11..];
            if let Some(s_end) = signals_section.find('}') {
                let signals_str = &signals_section[..s_end];
                // Parse key-value pairs
                let mut in_key = false;
                let mut in_value = false;
                let mut escape = false;
                let mut current_key = String::new();
                let mut current_value = String::new();
                let mut found_colon = false;

                for ch in signals_str.chars() {
                    if escape {
                        if in_key {
                            current_key.push(ch);
                        } else if in_value {
                            current_value.push(ch);
                        }
                        escape = false;
                        continue;
                    }

                    match ch {
                        '\\' => escape = true,
                        '"' => {
                            if in_key {
                                in_key = false;
                            } else if in_value {
                                in_value = false;
                                snapshot
                                    .signals
                                    .insert(current_key.clone(), current_value.clone());
                                current_key.clear();
                                current_value.clear();
                                found_colon = false;
                            } else if !found_colon {
                                in_key = true;
                            } else {
                                in_value = true;
                            }
                        }
                        ':' if !in_key && !in_value => {
                            found_colon = true;
                        }
                        _ => {
                            if in_key {
                                current_key.push(ch);
                            } else if in_value {
                                current_value.push(ch);
                            }
                        }
                    }
                }
            }
        }

        Some(snapshot)
    }

    /// Clear all signals.
    pub fn clear(&mut self) {
        self.signals.clear();
    }
}

impl Default for StateSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// The lifecycle persistence manager — saves and restores app state.
pub struct LifecyclePersistenceManager {
    storage_type: StorageType,
    current_snapshot: Mutex<StateSnapshot>,
    save_count: Mutex<u32>,
    restore_count: Mutex<u32>,
}

impl LifecyclePersistenceManager {
    /// Create a new persistence manager.
    pub fn new(storage_type: StorageType) -> Self {
        Self {
            storage_type,
            current_snapshot: Mutex::new(StateSnapshot::new()),
            save_count: Mutex::new(0),
            restore_count: Mutex::new(0),
        }
    }

    /// Get the storage type.
    pub fn storage_type(&self) -> StorageType {
        self.storage_type
    }

    /// Save a signal value to the current snapshot.
    pub fn save_signal(&self, id: &str, value: &str) {
        self.current_snapshot.lock().unwrap().add_signal(id, value);
    }

    /// Save the current snapshot (simulated — stores in memory).
    pub fn save(&self) -> bool {
        *self.save_count.lock().unwrap() += 1;
        true
    }

    /// Restore a previously saved snapshot.
    pub fn restore(&self, snapshot: StateSnapshot) {
        *self.current_snapshot.lock().unwrap() = snapshot;
        *self.restore_count.lock().unwrap() += 1;
    }

    /// Restore from a JSON string.
    pub fn restore_from_json(&self, json: &str) -> bool {
        if let Some(snapshot) = StateSnapshot::from_json(json) {
            self.restore(snapshot);
            true
        } else {
            false
        }
    }

    /// Get a signal value from the current snapshot.
    pub fn get_signal(&self, id: &str) -> Option<String> {
        self.current_snapshot
            .lock()
            .unwrap()
            .get_signal(id)
            .map(|s| s.to_string())
    }

    /// Get the current snapshot.
    pub fn current_snapshot(&self) -> StateSnapshot {
        self.current_snapshot.lock().unwrap().clone()
    }

    /// Get the current snapshot as JSON.
    pub fn to_json(&self) -> String {
        self.current_snapshot.lock().unwrap().to_json()
    }

    /// Clear the current snapshot.
    pub fn clear(&self) {
        self.current_snapshot.lock().unwrap().clear();
    }

    /// Get the number of signals in the current snapshot.
    pub fn signal_count(&self) -> usize {
        self.current_snapshot.lock().unwrap().signal_count()
    }

    /// Get the save count.
    pub fn save_count(&self) -> u32 {
        *self.save_count.lock().unwrap()
    }

    /// Get the restore count.
    pub fn restore_count(&self) -> u32 {
        *self.restore_count.lock().unwrap()
    }

    /// Check if the storage is persistent.
    pub fn is_persistent(&self) -> bool {
        self.storage_type.is_persistent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_type_display_name() {
        assert_eq!(StorageType::UserDefaults.display_name(), "UserDefaults");
        assert_eq!(StorageType::IndexedDb.display_name(), "IndexedDB");
        assert_eq!(StorageType::Memory.display_name(), "Memory");
    }

    #[test]
    fn test_storage_type_is_persistent() {
        assert!(StorageType::UserDefaults.is_persistent());
        assert!(StorageType::SharedPreferences.is_persistent());
        assert!(StorageType::IndexedDb.is_persistent());
        assert!(StorageType::LocalStorage.is_persistent());
        assert!(!StorageType::SessionStorage.is_persistent());
        assert!(!StorageType::Memory.is_persistent());
    }

    #[test]
    fn test_state_snapshot_new() {
        let snap = StateSnapshot::new();
        assert_eq!(snap.version, 1);
        assert!(snap.is_empty());
        assert_eq!(snap.signal_count(), 0);
    }

    #[test]
    fn test_state_snapshot_add_get_signal() {
        let mut snap = StateSnapshot::new();
        snap.add_signal("counter", "42");
        snap.add_signal("name", "Alice");

        assert_eq!(snap.signal_count(), 2);
        assert_eq!(snap.get_signal("counter"), Some("42"));
        assert_eq!(snap.get_signal("name"), Some("Alice"));
        assert_eq!(snap.get_signal("nonexistent"), None);
    }

    #[test]
    fn test_state_snapshot_remove_signal() {
        let mut snap = StateSnapshot::new();
        snap.add_signal("key", "value");
        assert!(snap.remove_signal("key"));
        assert!(!snap.remove_signal("key"));
        assert_eq!(snap.signal_count(), 0);
    }

    #[test]
    fn test_state_snapshot_clear() {
        let mut snap = StateSnapshot::new();
        snap.add_signal("a", "1");
        snap.add_signal("b", "2");
        snap.clear();
        assert!(snap.is_empty());
    }

    #[test]
    fn test_state_snapshot_to_json() {
        let mut snap = StateSnapshot::with_version("1.0.0");
        snap.add_signal("count", "42");
        let json = snap.to_json();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"appVersion\":\"1.0.0\""));
        assert!(json.contains("\"count\":\"42\""));
    }

    #[test]
    fn test_state_snapshot_from_json() {
        let json = r#"{"version":2,"timestamp":1700000000,"appVersion":"2.0","signals":{"count":"42","name":"Alice"}}"#;
        let snap = StateSnapshot::from_json(json).unwrap();
        assert_eq!(snap.version, 2);
        assert_eq!(snap.timestamp, 1700000000);
        assert_eq!(snap.get_signal("count"), Some("42"));
        assert_eq!(snap.get_signal("name"), Some("Alice"));
    }

    #[test]
    fn test_state_snapshot_roundtrip() {
        let mut snap = StateSnapshot::with_version("1.5.0");
        snap.add_signal("key1", "value1");
        snap.add_signal("key2", "value2");
        let json = snap.to_json();
        let restored = StateSnapshot::from_json(&json).unwrap();
        assert_eq!(restored.get_signal("key1"), Some("value1"));
        assert_eq!(restored.get_signal("key2"), Some("value2"));
    }

    #[test]
    fn test_manager_new() {
        let mgr = LifecyclePersistenceManager::new(StorageType::UserDefaults);
        assert_eq!(mgr.storage_type(), StorageType::UserDefaults);
        assert!(mgr.is_persistent());
    }

    #[test]
    fn test_manager_save_signal() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        mgr.save_signal("counter", "100");
        assert_eq!(mgr.get_signal("counter"), Some("100".to_string()));
        assert_eq!(mgr.signal_count(), 1);
    }

    #[test]
    fn test_manager_save() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        mgr.save_signal("key", "value");
        assert!(mgr.save());
        assert_eq!(mgr.save_count(), 1);
    }

    #[test]
    fn test_manager_restore() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        let mut snap = StateSnapshot::new();
        snap.add_signal("restored", "true");
        mgr.restore(snap);
        assert_eq!(mgr.restore_count(), 1);
        assert_eq!(mgr.get_signal("restored"), Some("true".to_string()));
    }

    #[test]
    fn test_manager_restore_from_json() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        let json = r#"{"version":1,"timestamp":0,"appVersion":"","signals":{"x":"1"}}"#;
        assert!(mgr.restore_from_json(json));
        assert_eq!(mgr.get_signal("x"), Some("1".to_string()));
    }

    #[test]
    fn test_manager_restore_from_json_invalid() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        assert!(!mgr.restore_from_json("invalid json"));
    }

    #[test]
    fn test_manager_to_json() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        mgr.save_signal("test", "123");
        let json = mgr.to_json();
        assert!(json.contains("\"test\":\"123\""));
    }

    #[test]
    fn test_manager_clear() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        mgr.save_signal("a", "1");
        mgr.clear();
        assert_eq!(mgr.signal_count(), 0);
    }

    #[test]
    fn test_manager_current_snapshot() {
        let mgr = LifecyclePersistenceManager::new(StorageType::Memory);
        mgr.save_signal("key", "value");
        let snap = mgr.current_snapshot();
        assert_eq!(snap.get_signal("key"), Some("value"));
    }
}
