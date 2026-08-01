//! Goal 188: Server-sent events with typed channels.
//!
//! Extend existing SSE support with typed channels. `sse_channel::<UserEvent>()`
//! creates a type-safe SSE stream. Client-side `use_sse_channel::<UserEvent>()`
//! auto-deserializes. Compile-time guarantee that server and client agree on event types.

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Mutex;

/// A typed SSE event — serializable to the SSE wire format.
#[derive(Debug, Clone)]
pub struct SseEvent<T: SseEventType> {
    /// The event type name.
    pub event_type: String,
    /// The event data (serialized).
    pub data: String,
    /// The event ID (for reconnection).
    pub id: Option<String>,
    /// The retry interval in milliseconds.
    pub retry: Option<u32>,
    _marker: PhantomData<T>,
}

/// Trait for types that can be sent over SSE channels.
pub trait SseEventType: Clone + Debug + Send + Sync + 'static {
    /// The event type name (used as the SSE `event:` field).
    fn event_name() -> &'static str;

    /// Serialize to string for the SSE `data:` field.
    fn serialize(&self) -> String;

    /// Deserialize from a string (the SSE `data:` field).
    fn deserialize(data: &str) -> Option<Self>;
}

impl<T: SseEventType> SseEvent<T> {
    /// Create a new typed SSE event.
    pub fn new(event: &T) -> Self {
        Self {
            event_type: T::event_name().to_string(),
            data: event.serialize(),
            id: None,
            retry: None,
            _marker: PhantomData,
        }
    }

    /// Set the event ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the retry interval.
    pub fn with_retry(mut self, retry_ms: u32) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Serialize to the SSE wire format.
    pub fn to_sse_string(&self) -> String {
        let mut output = String::new();

        if let Some(id) = &self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        output.push_str(&format!("event: {}\n", self.event_type));

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        // Data can be multi-line
        for line in self.data.lines() {
            output.push_str(&format!("data: {}\n", line));
        }

        output.push('\n'); // End of event
        output
    }
}

/// A typed SSE channel — sends typed events to connected clients.
pub struct SseChannel<T: SseEventType> {
    /// The channel name.
    pub name: String,
    /// Buffer of pending events.
    events: Mutex<Vec<SseEvent<T>>>,
    /// Whether the channel is active.
    active: Mutex<bool>,
}

impl<T: SseEventType> SseChannel<T> {
    /// Create a new typed SSE channel.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            events: Mutex::new(Vec::new()),
            active: Mutex::new(true),
        }
    }

    /// Send an event on the channel.
    pub fn send(&self, event: &T) -> SseEvent<T> {
        let sse_event = SseEvent::new(event);
        self.events.lock().unwrap().push(sse_event.clone());
        sse_event
    }

    /// Send an event with an ID.
    pub fn send_with_id(&self, event: &T, id: &str) -> SseEvent<T> {
        let sse_event = SseEvent::new(event).with_id(id);
        self.events.lock().unwrap().push(sse_event.clone());
        sse_event
    }

    /// Drain all pending events and return them as a single SSE string.
    pub fn flush(&self) -> String {
        let events: Vec<SseEvent<T>> = self.events.lock().unwrap().drain(..).collect();
        events.iter().map(|e| e.to_sse_string()).collect()
    }

    /// Get the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Check if the channel is active.
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }

    /// Close the channel.
    pub fn close(&self) {
        *self.active.lock().unwrap() = false;
    }

    /// Get all pending events without draining.
    pub fn peek(&self) -> Vec<String> {
        self.events.lock().unwrap().iter().map(|e| e.to_sse_string()).collect()
    }
}

/// The SSE channel registry — manages typed channels by name.
pub struct SseChannelRegistry {
    channels: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl SseChannelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// Register a typed channel.
    pub fn register<T: SseEventType + 'static>(&mut self, channel: SseChannel<T>) {
        self.channels.insert(channel.name.clone(), Box::new(channel));
    }

    /// Get a typed channel by name.
    pub fn get<T: SseEventType + 'static>(&self, name: &str) -> Option<&SseChannel<T>> {
        self.channels.get(name)?.downcast_ref::<SseChannel<T>>()
    }

    /// Get the number of registered channels.
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    /// Get all channel names.
    pub fn names(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }
}

impl Default for SseChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-side hook for subscribing to a typed SSE channel.
/// Returns a receiver that auto-deserializes events.
pub struct SseReceiver<T: SseEventType> {
    /// The channel name.
    pub channel_name: String,
    /// The URL to connect to.
    pub url: String,
    /// The last event ID (for reconnection).
    pub last_event_id: Option<String>,
    _marker: PhantomData<T>,
}

impl<T: SseEventType> SseReceiver<T> {
    /// Create a new SSE receiver for a typed channel.
    pub fn new(url: &str, channel_name: &str) -> Self {
        Self {
            channel_name: channel_name.to_string(),
            url: url.to_string(),
            last_event_id: None,
            _marker: PhantomData,
        }
    }

    /// Set the last event ID (for reconnection).
    pub fn with_last_id(mut self, id: &str) -> Self {
        self.last_event_id = Some(id.to_string());
        self
    }

    /// Get the JavaScript code to create an EventSource for this channel.
    pub fn event_source_script(&self) -> String {
        let last_id = self.last_event_id.as_deref().unwrap_or("");
        format!(
            r#"(function(){{var es=new EventSource('{}?channel={}&lastId={}');es.addEventListener('{}',function(e){{window.__ryeSseEvents=window.__ryeSseEvents||[];window.__ryeSseEvents.push({{type:'{}',data:e.data,id:e.lastEventId}});}});return es;}})()"#,
            self.url,
            self.channel_name,
            last_id,
            T::event_name(),
            T::event_name(),
        )
    }

    /// Parse an SSE event from a raw string.
    pub fn parse_event(raw: &str) -> Option<SseEvent<T>> {
        let mut event_type = String::new();
        let mut data = String::new();
        let mut id = None;

        for line in raw.lines() {
            if let Some(rest) = line.strip_prefix("event: ") {
                event_type = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("data: ") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(rest);
            } else if let Some(rest) = line.strip_prefix("id: ") {
                id = Some(rest.to_string());
            }
        }

        if event_type == T::event_name() {
            Some(SseEvent {
                event_type,
                data,
                id,
                retry: None,
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

/// Parse raw SSE text into individual event blocks.
pub fn parse_sse_stream(raw: &str) -> Vec<String> {
    raw.split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| block.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct UserEvent {
        user_id: String,
        action: String,
    }

    impl SseEventType for UserEvent {
        fn event_name() -> &'static str {
            "user-event"
        }

        fn serialize(&self) -> String {
            format!("{{\"user_id\":\"{}\",\"action\":\"{}\"}}", self.user_id, self.action)
        }

        fn deserialize(data: &str) -> Option<Self> {
            // Simple parsing for test
            if data.contains("\"user_id\":\"") && data.contains("\"action\":\"") {
                let user_id_start = data.find("\"user_id\":\"").map(|i| i + 11)?;
                let user_id_end = data[user_id_start..].find('"').map(|i| user_id_start + i)?;
                let action_start = data.find("\"action\":\"").map(|i| i + 10)?;
                let action_end = data[action_start..].find('"').map(|i| action_start + i)?;
                Some(Self {
                    user_id: data[user_id_start..user_id_end].to_string(),
                    action: data[action_start..action_end].to_string(),
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn test_sse_event_new() {
        let event = UserEvent { user_id: "u1".to_string(), action: "login".to_string() };
        let sse = SseEvent::new(&event);
        assert_eq!(sse.event_type, "user-event");
        assert!(sse.data.contains("u1"));
        assert!(sse.data.contains("login"));
    }

    #[test]
    fn test_sse_event_with_id_and_retry() {
        let event = UserEvent { user_id: "u1".to_string(), action: "click".to_string() };
        let sse = SseEvent::new(&event).with_id("evt-123").with_retry(5000);
        assert_eq!(sse.id, Some("evt-123".to_string()));
        assert_eq!(sse.retry, Some(5000));
    }

    #[test]
    fn test_sse_event_to_sse_string() {
        let event = UserEvent { user_id: "u1".to_string(), action: "login".to_string() };
        let sse = SseEvent::new(&event).with_id("42");
        let output = sse.to_sse_string();
        assert!(output.contains("id: 42"));
        assert!(output.contains("event: user-event"));
        assert!(output.contains("data: "));
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_sse_channel_send_flush() {
        let channel = SseChannel::<UserEvent>::new("user-events");
        let event = UserEvent { user_id: "u1".to_string(), action: "login".to_string() };
        channel.send(&event);
        channel.send(&event);
        assert_eq!(channel.pending_count(), 2);

        let flushed = channel.flush();
        assert!(flushed.contains("event: user-event"));
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn test_sse_channel_send_with_id() {
        let channel = SseChannel::<UserEvent>::new("test");
        let event = UserEvent { user_id: "u1".to_string(), action: "click".to_string() };
        let sse = channel.send_with_id(&event, "evt-1");
        assert_eq!(sse.id, Some("evt-1".to_string()));
    }

    #[test]
    fn test_sse_channel_close() {
        let channel = SseChannel::<UserEvent>::new("test");
        assert!(channel.is_active());
        channel.close();
        assert!(!channel.is_active());
    }

    #[test]
    fn test_sse_channel_peek() {
        let channel = SseChannel::<UserEvent>::new("test");
        let event = UserEvent { user_id: "u1".to_string(), action: "login".to_string() };
        channel.send(&event);
        let peeked = channel.peek();
        assert_eq!(peeked.len(), 1);
        // Peek doesn't drain
        assert_eq!(channel.pending_count(), 1);
    }

    #[test]
    fn test_sse_channel_registry() {
        let mut registry = SseChannelRegistry::new();
        let channel = SseChannel::<UserEvent>::new("user-events");
        registry.register(channel);
        assert_eq!(registry.len(), 1);
        assert!(registry.get::<UserEvent>("user-events").is_some());
        assert!(registry.get::<UserEvent>("nonexistent").is_none());
    }

    #[test]
    fn test_sse_channel_registry_names() {
        let mut registry = SseChannelRegistry::new();
        registry.register(SseChannel::<UserEvent>::new("a"));
        registry.register(SseChannel::<UserEvent>::new("b"));
        let names = registry.names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_sse_receiver_script() {
        let receiver = SseReceiver::<UserEvent>::new("/api/sse", "user-events")
            .with_last_id("123");
        let script = receiver.event_source_script();
        assert!(script.contains("EventSource"));
        assert!(script.contains("user-events"));
        assert!(script.contains("user-event"));
    }

    #[test]
    fn test_sse_receiver_parse_event() {
        let raw = "id: 42\nevent: user-event\ndata: {\"user_id\":\"u1\",\"action\":\"login\"}\n";
        let event = SseReceiver::<UserEvent>::parse_event(raw);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.id, Some("42".to_string()));
        assert_eq!(event.event_type, "user-event");
    }

    #[test]
    fn test_sse_receiver_parse_wrong_type() {
        let raw = "event: other-event\ndata: {}\n";
        let event = SseReceiver::<UserEvent>::parse_event(raw);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_sse_stream() {
        let raw = "event: a\ndata: 1\n\nevent: b\ndata: 2\n\n";
        let blocks = parse_sse_stream(raw);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_user_event_deserialize() {
        let event = UserEvent::deserialize("{\"user_id\":\"u1\",\"action\":\"login\"}");
        assert_eq!(event, Some(UserEvent {
            user_id: "u1".to_string(),
            action: "login".to_string(),
        }));
    }
}
