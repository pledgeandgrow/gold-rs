//! Goal 120: Multi-window support (web).
//!
//! Use `window.open()` + `BroadcastChannel` for multi-window web apps.
//! Share signal state across windows via `BroadcastChannel`.
//! On desktop, use native multi-window. Same component API for both.

/// Multi-window channel configuration.
#[derive(Debug, Clone)]
pub struct MultiWindowConfig {
    /// The BroadcastChannel name for cross-window communication.
    pub channel_name: String,
    /// Whether to share signal state across windows.
    pub share_signals: bool,
}

impl MultiWindowConfig {
    /// Create a new multi-window config.
    pub fn new(channel_name: impl Into<String>) -> Self {
        Self {
            channel_name: channel_name.into(),
            share_signals: true,
        }
    }

    /// Disable signal sharing (windows are independent).
    pub fn no_signal_sharing(mut self) -> Self {
        self.share_signals = false;
        self
    }
}

/// A message sent between windows.
#[derive(Debug, Clone)]
pub struct WindowMessage {
    /// Message type.
    pub msg_type: WindowMessageType,
    /// Serialized payload.
    pub payload: String,
    /// Source window ID.
    pub source_id: String,
}

/// Type of inter-window message.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowMessageType {
    /// Signal state update.
    SignalUpdate,
    /// Window opened.
    WindowOpened,
    /// Window closed.
    WindowClosed,
    /// Custom message.
    Custom,
}

/// Window handle — reference to a secondary window.
pub struct WindowHandle {
    /// Window ID.
    pub id: String,
    /// Window URL.
    pub url: String,
    /// Whether the window is still open.
    pub open: bool,
}

impl WindowHandle {
    /// Create a new window handle.
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            open: true,
        }
    }

    /// Mark the window as closed.
    pub fn close(&mut self) {
        self.open = false;
    }
}

/// Multi-window manager — coordinates windows and signal sharing.
pub struct MultiWindowManager {
    /// Configuration.
    config: MultiWindowConfig,
    /// This window's unique ID.
    pub window_id: String,
    /// Open secondary windows.
    pub windows: Vec<WindowHandle>,
    /// Pending messages to send.
    pub pending_messages: Vec<WindowMessage>,
}

impl MultiWindowManager {
    /// Create a new multi-window manager.
    pub fn new(config: MultiWindowConfig) -> Self {
        let window_id = format!("win-{}", std::process::id());
        Self {
            config,
            window_id,
            windows: Vec::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Open a new window with the given URL.
    pub fn open_window(&mut self, url: &str) -> WindowHandle {
        let id = format!("win-{}-{}", std::process::id(), self.windows.len());
        let handle = WindowHandle::new(id.clone(), url.to_string());
        self.windows.push(handle);

        self.pending_messages.push(WindowMessage {
            msg_type: WindowMessageType::WindowOpened,
            payload: url.to_string(),
            source_id: self.window_id.clone(),
        });

        WindowHandle::new(id, url)
    }

    /// Broadcast a signal update to other windows.
    pub fn broadcast_signal(&mut self, signal_id: &str, value: &str) {
        if !self.config.share_signals {
            return;
        }
        let payload = format!("{}:{}", signal_id, value);
        self.pending_messages.push(WindowMessage {
            msg_type: WindowMessageType::SignalUpdate,
            payload,
            source_id: self.window_id.clone(),
        });
    }

    /// Drain pending messages.
    pub fn drain_messages(&mut self) -> Vec<WindowMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Close a window by ID.
    pub fn close_window(&mut self, id: &str) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.close();
        }
        self.pending_messages.push(WindowMessage {
            msg_type: WindowMessageType::WindowClosed,
            payload: id.to_string(),
            source_id: self.window_id.clone(),
        });
    }

    /// Number of open windows.
    pub fn open_count(&self) -> usize {
        self.windows.iter().filter(|w| w.open).count()
    }
}

/// Generate the JS for multi-window support.
pub fn multi_window_script() -> &'static str {
    r#"<script>
(function() {
  var channel = null;
  var windowId = 'win-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
  var openWindows = {};

  function initChannel(name) {
    if (typeof BroadcastChannel !== 'undefined') {
      channel = new BroadcastChannel(name);
      channel.onmessage = function(e) {
        var msg = e.data;
        if (msg.sourceId === windowId) return; // Skip own messages
        if (msg.type === 'signal-update' && window.__rye_signal_update) {
          var parts = msg.payload.split(':');
          window.__rye_signal_update(parts[0], parts[1]);
        }
        if (msg.type === 'window-opened' && window.__rye_window_opened) {
          window.__rye_window_opened(msg.payload, msg.sourceId);
        }
        if (msg.type === 'window-closed' && window.__rye_window_closed) {
          window.__rye_window_closed(msg.payload);
        }
      };
    }
  }

  window.__rye_multi_window_init = function(channelName) {
    initChannel(channelName);
    return windowId;
  };

  window.__rye_open_window = function(url, features) {
    var w = window.open(url, '_blank', features || '');
    var id = 'win-' + Date.now();
    openWindows[id] = w;
    if (channel) {
      channel.postMessage({
        type: 'window-opened',
        payload: url,
        sourceId: windowId
      });
    }
    return id;
  };

  window.__rye_broadcast_signal = function(signalId, value) {
    if (channel) {
      channel.postMessage({
        type: 'signal-update',
        payload: signalId + ':' + value,
        sourceId: windowId
      });
    }
  };

  window.__rye_close_window = function(id) {
    if (openWindows[id]) {
      openWindows[id].close();
      delete openWindows[id];
      if (channel) {
        channel.postMessage({
          type: 'window-closed',
          payload: id,
          sourceId: windowId
        });
      }
    }
  };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_window_config() {
        let config = MultiWindowConfig::new("rye-app").no_signal_sharing();
        assert_eq!(config.channel_name, "rye-app");
        assert!(!config.share_signals);
    }

    #[test]
    fn test_window_handle() {
        let mut h = WindowHandle::new("win-1", "/popup");
        assert!(h.open);
        h.close();
        assert!(!h.open);
    }

    #[test]
    fn test_multi_window_manager() {
        let config = MultiWindowConfig::new("rye-app");
        let mut mgr = MultiWindowManager::new(config);

        assert_eq!(mgr.open_count(), 0);
        mgr.open_window("/popup");
        assert_eq!(mgr.open_count(), 1);
    }

    #[test]
    fn test_broadcast_signal() {
        let config = MultiWindowConfig::new("rye-app");
        let mut mgr = MultiWindowManager::new(config);

        mgr.broadcast_signal("count", "42");
        let msgs = mgr.drain_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].msg_type, WindowMessageType::SignalUpdate);
        assert_eq!(msgs[0].payload, "count:42");
    }

    #[test]
    fn test_broadcast_signal_disabled() {
        let config = MultiWindowConfig::new("rye-app").no_signal_sharing();
        let mut mgr = MultiWindowManager::new(config);

        mgr.broadcast_signal("count", "42");
        let msgs = mgr.drain_messages();
        assert_eq!(msgs.len(), 0);
    }

    #[test]
    fn test_close_window() {
        let config = MultiWindowConfig::new("rye-app");
        let mut mgr = MultiWindowManager::new(config);

        let handle = mgr.open_window("/popup");
        mgr.close_window(&handle.id);
        assert_eq!(mgr.open_count(), 0);
    }

    #[test]
    fn test_multi_window_script() {
        let script = multi_window_script();
        assert!(script.contains("BroadcastChannel"));
        assert!(script.contains("__rye_open_window"));
        assert!(script.contains("__rye_broadcast_signal"));
    }
}
