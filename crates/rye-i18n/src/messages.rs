//! Messages — translation message store.

use std::collections::HashMap;

/// A store of translation messages keyed by message ID.
pub struct MessageStore {
    messages: HashMap<String, String>,
}

impl MessageStore {
    /// Create a new empty message store.
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    /// Add a message to the store.
    pub fn add(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.messages.insert(id.into(), message.into());
    }

    /// Get a message by ID.
    pub fn get(&self, id: &str) -> Option<&str> {
        self.messages.get(id).map(|s| s.as_str())
    }
}

impl Default for MessageStore {
    fn default() -> Self {
        Self::new()
    }
}
