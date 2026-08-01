//! Goal 191: Server-side signal hydration.
//!
//! Server-side signals that serialize their state into the HTML, client-side
//! signals that deserialize and continue from the same state. No "flash" of
//! loading state on hydration.

use std::collections::HashMap;

/// A snapshot of signal state for hydration.
#[derive(Debug, Clone)]
pub struct SignalHydrationData {
    /// Map of signal ID to serialized value.
    pub signals: HashMap<String, String>,
    /// The component tree state (serialized).
    pub component_state: String,
}

impl SignalHydrationData {
    /// Create new empty hydration data.
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            component_state: String::new(),
        }
    }

    /// Add a signal's serialized state.
    pub fn add_signal(&mut self, id: &str, value: &str) {
        self.signals.insert(id.to_string(), value.to_string());
    }

    /// Get a signal's serialized state.
    pub fn get_signal(&self, id: &str) -> Option<&str> {
        self.signals.get(id).map(|s| s.as_str())
    }

    /// Serialize to JSON for embedding in HTML.
    pub fn to_json(&self) -> String {
        let signals_json: Vec<String> = self
            .signals
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
            .collect();

        format!(
            "{{\"signals\":{{{}}},\"componentState\":\"{}\"}}",
            signals_json.join(","),
            escape_json(&self.component_state),
        )
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Option<Self> {
        // Simple parsing for test purposes
        let mut data = Self::new();

        // Extract signals
        if let Some(signals_start) = json.find("\"signals\":{") {
            let signals_section = &json[signals_start + 11..];
            if let Some(signals_end) = signals_section.find('}') {
                let signals_str = &signals_section[..signals_end];
                // Parse key-value pairs
                let mut depth = 0;
                let mut current = String::new();
                let mut in_string = false;
                let mut escape = false;

                for ch in signals_str.chars() {
                    if escape {
                        current.push(ch);
                        escape = false;
                        continue;
                    }
                    match ch {
                        '\\' => {
                            escape = true;
                            current.push(ch);
                        }
                        '"' if depth == 0 => {
                            in_string = !in_string;
                            current.push(ch);
                        }
                        ':' if !in_string => {
                            current.push(ch);
                        }
                        ',' if !in_string && depth == 0 => {
                            parse_signal_pair(&current, &mut data);
                            current.clear();
                        }
                        _ => {
                            current.push(ch);
                        }
                    }
                }
                if !current.trim().is_empty() {
                    parse_signal_pair(&current, &mut data);
                }
            }
        }

        Some(data)
    }

    /// Generate the HTML script tag to embed hydration data.
    pub fn to_script_tag(&self) -> String {
        format!(
            r#"<script>window.__RYE_HYDRATION__={};</script>"#,
            self.to_json()
        )
    }

    /// Get the number of signals.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Check if there are no signals.
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty() && self.component_state.is_empty()
    }
}

impl Default for SignalHydrationData {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_signal_pair(pair: &str, data: &mut SignalHydrationData) {
    // Extract "key":"value" format
    let pair = pair.trim();
    if !pair.starts_with('"') {
        return;
    }
    if let Some(key_end) = pair[1..].find('"') {
        let key = &pair[1..key_end + 1];
        let rest = &pair[key_end + 2..]; // skip ":"
        let rest = rest.trim_start_matches(':').trim();
        if rest.starts_with('"') {
            if let Some(val_end) = rest[1..].find('"') {
                let value = &rest[1..val_end + 1];
                data.add_signal(key, value);
            }
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// The server-side signal serializer — collects signal state for hydration.
pub struct ServerSignalSerializer {
    data: SignalHydrationData,
}

impl ServerSignalSerializer {
    /// Create a new serializer.
    pub fn new() -> Self {
        Self {
            data: SignalHydrationData::new(),
        }
    }

    /// Register a signal's state.
    pub fn register<T: std::fmt::Display>(&mut self, id: &str, value: &T) {
        self.data.add_signal(id, &value.to_string());
    }

    /// Set the component state.
    pub fn set_component_state(&mut self, state: &str) {
        self.data.component_state = state.to_string();
    }

    /// Get the hydration data.
    pub fn data(&self) -> &SignalHydrationData {
        &self.data
    }

    /// Generate the HTML script tag.
    pub fn to_script_tag(&self) -> String {
        self.data.to_script_tag()
    }

    /// Get the JSON representation.
    pub fn to_json(&self) -> String {
        self.data.to_json()
    }
}

impl Default for ServerSignalSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// The client-side signal deserializer — restores signal state from hydration data.
pub struct ClientSignalDeserializer {
    data: SignalHydrationData,
}

impl ClientSignalDeserializer {
    /// Create a new deserializer from hydration data.
    pub fn from_data(data: SignalHydrationData) -> Self {
        Self { data }
    }

    /// Create from JSON string.
    pub fn from_json(json: &str) -> Option<Self> {
        Some(Self {
            data: SignalHydrationData::from_json(json)?,
        })
    }

    /// Get a signal's value as a string.
    pub fn get_signal(&self, id: &str) -> Option<&str> {
        self.data.get_signal(id)
    }

    /// Get a signal's value parsed as a type.
    pub fn get_signal_parsed<T: std::str::FromStr>(&self, id: &str) -> Option<T> {
        self.data.get_signal(id).and_then(|s| s.parse().ok())
    }

    /// Get all signal IDs.
    pub fn signal_ids(&self) -> Vec<String> {
        self.data.signals.keys().cloned().collect()
    }

    /// Get the component state.
    pub fn component_state(&self) -> &str {
        &self.data.component_state
    }

    /// Get the number of signals.
    pub fn signal_count(&self) -> usize {
        self.data.signal_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_data_add_get() {
        let mut data = SignalHydrationData::new();
        data.add_signal("counter", "42");
        data.add_signal("name", "Alice");
        assert_eq!(data.get_signal("counter"), Some("42"));
        assert_eq!(data.get_signal("name"), Some("Alice"));
        assert_eq!(data.signal_count(), 2);
    }

    #[test]
    fn test_hydration_data_to_json() {
        let mut data = SignalHydrationData::new();
        data.add_signal("count", "10");
        let json = data.to_json();
        assert!(json.contains("\"signals\""));
        assert!(json.contains("\"count\":\"10\""));
    }

    #[test]
    fn test_hydration_data_to_script_tag() {
        let mut data = SignalHydrationData::new();
        data.add_signal("x", "1");
        let tag = data.to_script_tag();
        assert!(tag.contains("<script>"));
        assert!(tag.contains("__RYE_HYDRATION__"));
        assert!(tag.contains("</script>"));
    }

    #[test]
    fn test_hydration_data_is_empty() {
        let data = SignalHydrationData::new();
        assert!(data.is_empty());
    }

    #[test]
    fn test_server_serializer_register() {
        let mut serializer = ServerSignalSerializer::new();
        serializer.register("counter", &42i32);
        serializer.register("name", &"Alice");
        assert_eq!(serializer.data().get_signal("counter"), Some("42"));
        assert_eq!(serializer.data().get_signal("name"), Some("Alice"));
    }

    #[test]
    fn test_server_serializer_component_state() {
        let mut serializer = ServerSignalSerializer::new();
        serializer.set_component_state("{\"rendered\":true}");
        assert!(serializer.to_json().contains("componentState"));
    }

    #[test]
    fn test_server_serializer_to_script_tag() {
        let mut serializer = ServerSignalSerializer::new();
        serializer.register("count", &5);
        let tag = serializer.to_script_tag();
        assert!(tag.contains("__RYE_HYDRATION__"));
        assert!(tag.contains("count"));
    }

    #[test]
    fn test_client_deserializer_from_data() {
        let mut data = SignalHydrationData::new();
        data.add_signal("count", "42");
        data.add_signal("name", "Bob");

        let deserializer = ClientSignalDeserializer::from_data(data);
        assert_eq!(deserializer.get_signal("count"), Some("42"));
        assert_eq!(deserializer.get_signal_parsed::<i32>("count"), Some(42));
    }

    #[test]
    fn test_client_deserializer_signal_ids() {
        let mut data = SignalHydrationData::new();
        data.add_signal("a", "1");
        data.add_signal("b", "2");

        let deserializer = ClientSignalDeserializer::from_data(data);
        let ids = deserializer.signal_ids();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }

    #[test]
    fn test_client_deserializer_component_state() {
        let mut data = SignalHydrationData::new();
        data.component_state = "state-data".to_string();
        let deserializer = ClientSignalDeserializer::from_data(data);
        assert_eq!(deserializer.component_state(), "state-data");
    }

    #[test]
    fn test_roundtrip_json() {
        let mut data = SignalHydrationData::new();
        data.add_signal("count", "42");
        data.add_signal("name", "Alice");
        let json = data.to_json();

        let parsed = SignalHydrationData::from_json(&json).unwrap();
        assert_eq!(parsed.get_signal("count"), Some("42"));
    }
}
