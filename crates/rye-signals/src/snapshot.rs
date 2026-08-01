//! Snapshot — export and import reactive state for debugging and testing.
//!
//! Snapshots can be shared, loaded in tests, or used to reproduce bugs.

use crate::runtime::SignalId;
use crate::signal::Signal;
use std::cell::RefCell;
use std::collections::HashMap;

/// A captured snapshot of signal states.
///
/// Stores serialized representations of signal values keyed by signal ID.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub entries: Vec<SnapshotEntry>,
    pub timestamp: u64,
    pub label: String,
}

/// A single signal value in a snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotEntry {
    pub signal_id: SignalId,
    pub type_name: String,
    pub serialized: String,
}

thread_local! {
    /// Registry of snapshot-eligible signals — maps signal ID to a serializer function.
    static SERIALIZERS: RefCell<HashMap<SignalId, Box<dyn Fn() -> String>>> = RefCell::new(HashMap::new());

    /// Registry of deserializers — maps signal ID to a deserializer function.
    static DESERIALIZERS: RefCell<HashMap<SignalId, Box<dyn Fn(&str) -> Result<(), String>>>> = RefCell::new(HashMap::new());

    /// Counter for timestamps.
    static TIMESTAMP: RefCell<u64> = const { RefCell::new(0) };

    /// History of snapshots for time-travel.
    static HISTORY: RefCell<Vec<Snapshot>> = const { RefCell::new(Vec::new()) };

    /// Maximum history size.
    static MAX_HISTORY: RefCell<usize> = const { RefCell::new(100) };
}

/// Register a signal for snapshotting.
///
/// The signal's value will be serialized using the provided serializer function
/// when a snapshot is taken, and can be restored using the deserializer.
pub fn register<T: Clone + std::fmt::Display + 'static>(
    signal: &Signal<T>,
    type_name: &str,
) where
    T: std::str::FromStr,
{
    let id = signal.id();
    let signal_clone = signal.clone();
    let type_name = type_name.to_string();

    SERIALIZERS.with(|s| {
        s.borrow_mut().insert(
            id,
            Box::new(move || {
                let _ = type_name.clone();
                format!("{}", signal_clone.get_untracked())
            }),
        );
    });

    let signal_clone2 = signal.clone();
    DESERIALIZERS.with(|d| {
        d.borrow_mut().insert(
            id,
            Box::new(move |s: &str| {
                match s.parse::<T>() {
                    Ok(val) => {
                        signal_clone2.set(val);
                        Ok(())
                    }
                    Err(_) => Err(format!("Failed to parse '{}' as {}", s, std::any::type_name::<T>())),
                }
            }),
        );
    });
}

/// Export a snapshot of all registered signals.
pub fn export() -> Snapshot {
    let timestamp = TIMESTAMP.with(|t| {
        let mut t = t.borrow_mut();
        *t += 1;
        *t
    });

    let entries: Vec<SnapshotEntry> = SERIALIZERS.with(|s| {
        s.borrow()
            .iter()
            .map(|(id, serializer)| SnapshotEntry {
                signal_id: *id,
                type_name: std::any::type_name::<String>().to_string(),
                serialized: serializer(),
            })
            .collect()
    });

    Snapshot {
        entries,
        timestamp,
        label: format!("snapshot-{}", timestamp),
    }
}

/// Export a snapshot with a custom label.
pub fn export_labeled(label: &str) -> Snapshot {
    let mut snap = export();
    snap.label = label.to_string();
    snap
}

/// Import a snapshot — restores all signal values from the snapshot.
pub fn import(snapshot: &Snapshot) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for entry in &snapshot.entries {
        DESERIALIZERS.with(|d| {
            if let Some(deserializer) = d.borrow().get(&entry.signal_id) {
                if let Err(e) = deserializer(&entry.serialized) {
                    errors.push(e);
                }
            }
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Save the current state to history.
pub fn checkpoint() {
    let snap = export();
    HISTORY.with(|h| {
        let mut h = h.borrow_mut();
        h.push(snap);
        let max = MAX_HISTORY.with(|m| *m.borrow());
        if h.len() > max {
            h.remove(0);
        }
    });
}

/// Save the current state to history with a label.
pub fn checkpoint_labeled(label: &str) {
    let snap = export_labeled(label);
    HISTORY.with(|h| {
        let mut h = h.borrow_mut();
        h.push(snap);
        let max = MAX_HISTORY.with(|m| *m.borrow());
        if h.len() > max {
            h.remove(0);
        }
    });
}

/// Get the number of snapshots in history.
pub fn history_len() -> usize {
    HISTORY.with(|h| h.borrow().len())
}

/// Get a snapshot from history by index.
pub fn get_snapshot(index: usize) -> Option<Snapshot> {
    HISTORY.with(|h| h.borrow().get(index).cloned())
}

/// Restore a snapshot from history by index.
pub fn restore(index: usize) -> Result<(), Vec<String>> {
    let snap = get_snapshot(index);
    match snap {
        Some(s) => import(&s),
        None => Err(vec!["Snapshot index out of bounds".to_string()]),
    }
}

/// Clear all history.
pub fn clear_history() {
    HISTORY.with(|h| h.borrow_mut().clear());
}

/// Set the maximum history size.
pub fn set_max_history(max: usize) {
    MAX_HISTORY.with(|m| *m.borrow_mut() = max);
}

/// Reset all snapshot state.
pub fn reset() {
    SERIALIZERS.with(|s| s.borrow_mut().clear());
    DESERIALIZERS.with(|d| d.borrow_mut().clear());
    HISTORY.with(|h| h.borrow_mut().clear());
    TIMESTAMP.with(|t| *t.borrow_mut() = 0);
}

/// Export a snapshot as a JSON string.
pub fn export_json() -> String {
    let snap = export();
    let entries: Vec<String> = snap
        .entries
        .iter()
        .map(|e| {
            format!(
                r#"{{"signal_id":{},"type":"{}","value":"{}"}}"#,
                e.signal_id,
                e.type_name.replace('"', "\\\""),
                e.serialized.replace('"', "\\\"")
            )
        })
        .collect();

    format!(
        r#"{{"label":"{}","timestamp":{},"entries":[{}]}}"#,
        snap.label,
        snap.timestamp,
        entries.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_export() {
        reset();
        let sig = Signal::new(42i32);
        register(&sig, "i32");
        let snap = export();
        assert_eq!(snap.entries.len(), 1);
        assert_eq!(snap.entries[0].serialized, "42");
    }

    #[test]
    fn test_import_restores_values() {
        reset();
        let sig = Signal::new(10i32);
        register(&sig, "i32");
        sig.set(99);
        let snap = export();
        sig.set(0);
        assert_eq!(sig.get_untracked(), 0);
        import(&snap).unwrap();
        assert_eq!(sig.get_untracked(), 99);
    }

    #[test]
    fn test_checkpoint_and_restore() {
        reset();
        let sig = Signal::new(1i32);
        register(&sig, "i32");

        sig.set(10);
        checkpoint();
        sig.set(20);
        checkpoint();
        sig.set(30);
        checkpoint();

        assert_eq!(history_len(), 3);
        restore(0).unwrap();
        assert_eq!(sig.get_untracked(), 10);
        restore(2).unwrap();
        assert_eq!(sig.get_untracked(), 30);
    }

    #[test]
    fn test_labeled_snapshot() {
        reset();
        let sig = Signal::new(5i32);
        register(&sig, "i32");
        let snap = export_labeled("before-bug");
        assert_eq!(snap.label, "before-bug");
    }

    #[test]
    fn test_checkpoint_labeled() {
        reset();
        let sig = Signal::new(1i32);
        register(&sig, "i32");
        sig.set(100);
        checkpoint_labeled("milestone");
        let snap = get_snapshot(0).unwrap();
        assert_eq!(snap.label, "milestone");
        assert_eq!(snap.entries[0].serialized, "100");
    }

    #[test]
    fn test_export_json() {
        reset();
        let sig = Signal::new(42i32);
        register(&sig, "i32");
        let json = export_json();
        assert!(json.contains("\"value\":\"42\""));
        assert!(json.contains("\"entries\""));
    }

    #[test]
    fn test_max_history() {
        reset();
        set_max_history(3);
        let sig = Signal::new(0i32);
        register(&sig, "i32");
        for i in 0..5 {
            sig.set(i);
            checkpoint();
        }
        assert_eq!(history_len(), 3);
    }

    #[test]
    fn test_clear_history() {
        reset();
        let sig = Signal::new(1i32);
        register(&sig, "i32");
        checkpoint();
        checkpoint();
        assert_eq!(history_len(), 2);
        clear_history();
        assert_eq!(history_len(), 0);
    }
}
