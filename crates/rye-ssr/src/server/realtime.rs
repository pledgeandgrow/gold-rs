//! Goal 128: WebSocket / SSE integration.
//!
//! `use_websocket()` and `use_sse()` hooks for real-time data. Server-side
//! helpers for WebSocket upgrade and SSE streaming.

/// WebSocket connection state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WsState {
    /// Connecting.
    Connecting,
    /// Connected.
    Open,
    /// Closing.
    Closing,
    /// Closed.
    Closed,
}

impl WsState {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            WsState::Connecting => "connecting",
            WsState::Open => "open",
            WsState::Closing => "closing",
            WsState::Closed => "closed",
        }
    }
}

/// WebSocket configuration.
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// WebSocket URL (ws:// or wss://).
    pub url: String,
    /// Protocols (subprotocols).
    pub protocols: Vec<String>,
    /// Reconnect on close.
    pub reconnect: bool,
    /// Reconnect delay in milliseconds.
    pub reconnect_delay_ms: u32,
    /// Max reconnect attempts.
    pub max_reconnects: u32,
}

impl WsConfig {
    /// Create a new WebSocket config.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            protocols: Vec::new(),
            reconnect: true,
            reconnect_delay_ms: 1000,
            max_reconnects: 5,
        }
    }

    /// Add a subprotocol.
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocols.push(protocol.into());
        self
    }

    /// Disable auto-reconnect.
    pub fn no_reconnect(mut self) -> Self {
        self.reconnect = false;
        self
    }
}

/// WebSocket message.
#[derive(Debug, Clone)]
pub struct WsMessage {
    /// Message data (text or binary).
    pub data: WsData,
    /// Whether the message is binary.
    pub is_binary: bool,
}

/// WebSocket message data.
#[derive(Debug, Clone)]
pub enum WsData {
    /// Text message.
    Text(String),
    /// Binary message.
    Binary(Vec<u8>),
}

/// SSE (Server-Sent Events) configuration.
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// SSE endpoint URL.
    pub url: String,
    /// Last event ID (for reconnection).
    pub last_event_id: Option<String>,
    /// Whether to auto-reconnect.
    pub reconnect: bool,
}

impl SseConfig {
    /// Create a new SSE config.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            last_event_id: None,
            reconnect: true,
        }
    }

    /// Set last event ID for reconnection.
    pub fn with_last_event_id(mut self, id: impl Into<String>) -> Self {
        self.last_event_id = Some(id.into());
        self
    }
}

/// SSE event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// Event type (empty string for default).
    pub event_type: String,
    /// Event data.
    pub data: String,
    /// Event ID.
    pub id: Option<String>,
    /// Retry interval in milliseconds.
    pub retry: Option<u32>,
}

impl SseEvent {
    /// Create a new data-only event.
    pub fn data(data: impl Into<String>) -> Self {
        Self {
            event_type: String::new(),
            data: data.into(),
            id: None,
            retry: None,
        }
    }

    /// Create a named event.
    pub fn named(event_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            data: data.into(),
            id: None,
            retry: None,
        }
    }

    /// Set event ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set retry interval.
    pub fn with_retry(mut self, retry_ms: u32) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Serialize to SSE format.
    pub fn to_sse_string(&self) -> String {
        let mut output = String::new();

        if !self.event_type.is_empty() {
            output.push_str(&format!("event: {}\n", self.event_type));
        }

        if let Some(id) = &self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        // Data can be multi-line
        for line in self.data.lines() {
            output.push_str(&format!("data: {}\n", line));
        }

        output.push('\n'); // Empty line terminates the event
        output
    }
}

/// Generate the JS for WebSocket hook.
pub fn websocket_script() -> &'static str {
    r#"<script>
(function() {
  var sockets = {};
  var nextId = 0;

  window.__rye_websocket_connect = function(url, protocols, callbackId) {
    var id = 'ws_' + (nextId++);
    var ws = protocols && protocols.length > 0
      ? new WebSocket(url, protocols)
      : new WebSocket(url);

    ws.onopen = function() {
      window.__rye_signal_update(callbackId, { state: 'open' });
    };
    ws.onclose = function() {
      window.__rye_signal_update(callbackId, { state: 'closed' });
    };
    ws.onerror = function() {
      window.__rye_signal_update(callbackId, { state: 'closed', error: true });
    };
    ws.onmessage = function(event) {
      window.__rye_signal_update(callbackId, {
        state: 'open',
        message: event.data,
        isBinary: event.data instanceof ArrayBuffer || event.data instanceof Blob
      });
    };

    sockets[id] = ws;
    return id;
  };

  window.__rye_websocket_send = function(id, data) {
    if (sockets[id] && sockets[id].readyState === 1) {
      sockets[id].send(data);
    }
  };

  window.__rye_websocket_close = function(id) {
    if (sockets[id]) {
      sockets[id].close();
      delete sockets[id];
    }
  };
})();
</script>"#
}

/// Generate the JS for SSE hook.
pub fn sse_script() -> &'static str {
    r#"<script>
(function() {
  var sources = {};
  var nextId = 0;

  window.__rye_sse_connect = function(url, lastEventId, callbackId) {
    var id = 'sse_' + (nextId++);
    var es = new EventSource(url);

    es.onmessage = function(event) {
      window.__rye_signal_update(callbackId, {
        data: event.data,
        lastEventId: event.lastEventId
      });
    };

    es.onerror = function() {
      window.__rye_signal_update(callbackId, { error: true });
    };

    sources[id] = es;
    return id;
  };

  window.__rye_sse_close = function(id) {
    if (sources[id]) {
      sources[id].close();
      delete sources[id];
    }
  };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_state() {
        assert_eq!(WsState::Open.as_str(), "open");
        assert_eq!(WsState::Closed.as_str(), "closed");
    }

    #[test]
    fn test_ws_config() {
        let config = WsConfig::new("wss://api.example.com/ws")
            .protocol("rye-protocol")
            .no_reconnect();
        assert_eq!(config.url, "wss://api.example.com/ws");
        assert_eq!(config.protocols, vec!["rye-protocol"]);
        assert!(!config.reconnect);
    }

    #[test]
    fn test_sse_config() {
        let config = SseConfig::new("/api/events").with_last_event_id("42");
        assert_eq!(config.url, "/api/events");
        assert_eq!(config.last_event_id, Some("42".to_string()));
    }

    #[test]
    fn test_sse_event_data() {
        let event = SseEvent::data("hello world");
        let sse = event.to_sse_string();
        assert!(sse.contains("data: hello world"));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_sse_event_named() {
        let event = SseEvent::named("update", r#"{"count":5}"#)
            .with_id("123")
            .with_retry(3000);
        let sse = event.to_sse_string();
        assert!(sse.contains("event: update"));
        assert!(sse.contains("data: {\"count\":5}"));
        assert!(sse.contains("id: 123"));
        assert!(sse.contains("retry: 3000"));
    }

    #[test]
    fn test_sse_event_multiline() {
        let event = SseEvent::data("line1\nline2");
        let sse = event.to_sse_string();
        assert!(sse.contains("data: line1"));
        assert!(sse.contains("data: line2"));
    }

    #[test]
    fn test_websocket_script() {
        let script = websocket_script();
        assert!(script.contains("new WebSocket"));
        assert!(script.contains("__rye_websocket_connect"));
        assert!(script.contains("__rye_websocket_send"));
    }

    #[test]
    fn test_sse_script() {
        let script = sse_script();
        assert!(script.contains("EventSource"));
        assert!(script.contains("__rye_sse_connect"));
    }
}
