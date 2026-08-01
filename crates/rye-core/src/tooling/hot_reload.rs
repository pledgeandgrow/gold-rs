//! Goal 147: Template hot reload.
//!
//! Hot Module Replacement for rye templates. When a template file changes,
//! the updated component is recompiled and swapped in without full page reload.

use std::collections::HashMap;
use std::path::PathBuf;

/// A hot reload event.
#[derive(Debug, Clone)]
pub struct HotReloadEvent {
    /// File that changed.
    pub path: PathBuf,
    /// Type of change.
    pub change: FileChange,
    /// Timestamp.
    pub timestamp: u64,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileChange {
    /// File was created.
    Created,
    /// File was modified.
    Modified,
    /// File was deleted.
    Deleted,
}

/// Hot reload configuration.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Directories to watch for changes.
    pub watch_dirs: Vec<PathBuf>,
    /// File extensions to watch.
    pub extensions: Vec<String>,
    /// Debounce time in milliseconds.
    pub debounce_ms: u32,
    /// Whether to reload CSS without full HMR.
    pub css_fast_refresh: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watch_dirs: vec![PathBuf::from("src")],
            extensions: vec!["rs".to_string(), "html".to_string(), "css".to_string()],
            debounce_ms: 100,
            css_fast_refresh: true,
        }
    }
}

/// Hot reload manager — tracks file changes and triggers updates.
pub struct HotReloadManager {
    /// Configuration.
    config: HotReloadConfig,
    /// Registered components and their source files.
    component_sources: HashMap<String, PathBuf>,
    /// Pending reload events.
    pending_events: Vec<HotReloadEvent>,
    /// Last reload timestamp (for debouncing).
    last_reload: u64,
}

impl HotReloadManager {
    /// Create a new hot reload manager.
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            component_sources: HashMap::new(),
            pending_events: Vec::new(),
            last_reload: 0,
        }
    }

    /// Register a component with its source file.
    pub fn register(&mut self, component: impl Into<String>, path: PathBuf) {
        self.component_sources.insert(component.into(), path);
    }

    /// Handle a file change event.
    pub fn on_change(&mut self, path: PathBuf, change: FileChange) {
        let timestamp = current_timestamp();
        // Debounce: skip if too soon after last reload
        if timestamp - self.last_reload < self.config.debounce_ms as u64 {
            return;
        }

        // Check if the file extension is watched
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !self.config.extensions.iter().any(|e| e == ext) {
                return;
            }
        }

        self.pending_events.push(HotReloadEvent {
            path: path.clone(),
            change,
            timestamp,
        });
    }

    /// Drain pending events and return affected components.
    pub fn drain(&mut self) -> Vec<String> {
        let mut affected = Vec::new();
        let events = std::mem::take(&mut self.pending_events);

        for event in &events {
            // Find components whose source file changed
            for (component, source) in &self.component_sources {
                if source == &event.path {
                    affected.push(component.clone());
                }
            }
        }

        if !events.is_empty() {
            self.last_reload = current_timestamp();
        }

        affected
    }

    /// Number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Number of registered components.
    pub fn component_count(&self) -> usize {
        self.component_sources.len()
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Generate the JS for client-side hot reload via WebSocket.
pub fn hot_reload_script(port: u16) -> String {
    format!(
        r#"<script>
(function() {{
  var ws = new WebSocket('ws://localhost:{port}');
  var pendingReload = false;

  ws.onmessage = function(event) {{
    var msg = JSON.parse(event.data);
    if (msg.type === 'reload') {{
      if (!pendingReload) {{
        pendingReload = true;
        setTimeout(function() {{
          location.reload();
        }}, 50);
      }}
    }} else if (msg.type === 'css-update') {{
      // Hot-swap CSS without full reload
      var links = document.querySelectorAll('link[rel="stylesheet"]');
      links.forEach(function(link) {{
        var href = link.getAttribute('href');
        if (href && href.indexOf(msg.href) !== -1) {{
          var newLink = link.cloneNode();
          newLink.href = href + '?v=' + Date.now();
          link.parentNode.replaceChild(newLink, link);
        }}
      }});
    }} else if (msg.type === 'component-update') {{
      // Try to hot-swap a single component
      if (window.__rye_hot_update_component) {{
        window.__rye_hot_update_component(msg.component);
      }} else {{
        location.reload();
      }}
    }}
  }};

  ws.onclose = function() {{
    // Reconnect after 1 second
    setTimeout(function() {{
      location.reload();
    }}, 1000);
  }};
}})();
</script>"#,
        port = port
    )
}

/// A reload message sent to the client.
#[derive(Debug, Clone)]
pub enum ReloadMessage {
    /// Full page reload.
    FullReload,
    /// CSS-only update.
    CssUpdate { href: String },
    /// Component-level update.
    ComponentUpdate { component: String },
}

impl ReloadMessage {
    /// Serialize to JSON for WebSocket transmission.
    pub fn to_json(&self) -> String {
        match self {
            ReloadMessage::FullReload => {
                r#"{"type":"reload"}"#.to_string()
            }
            ReloadMessage::CssUpdate { href } => {
                format!(r#"{{"type":"css-update","href":"{}"}}"#, href)
            }
            ReloadMessage::ComponentUpdate { component } => {
                format!(r#"{{"type":"component-update","component":"{}"}}"#, component)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadConfig::default();
        assert_eq!(config.extensions, vec!["rs", "html", "css"]);
        assert!(config.css_fast_refresh);
    }

    #[test]
    fn test_hot_reload_manager_register() {
        let mut mgr = HotReloadManager::new(HotReloadConfig::default());
        mgr.register("Button", PathBuf::from("src/components/button.rs"));
        mgr.register("Card", PathBuf::from("src/components/card.rs"));
        assert_eq!(mgr.component_count(), 2);
    }

    #[test]
    fn test_hot_reload_manager_on_change() {
        let mut mgr = HotReloadManager::new(HotReloadConfig::default());
        mgr.register("Button", PathBuf::from("src/components/button.rs"));

        mgr.on_change(PathBuf::from("src/components/button.rs"), FileChange::Modified);
        assert_eq!(mgr.pending_count(), 1);

        let affected = mgr.drain();
        assert_eq!(affected, vec!["Button"]);
    }

    #[test]
    fn test_hot_reload_manager_ignored_extension() {
        let mut mgr = HotReloadManager::new(HotReloadConfig::default());
        mgr.on_change(PathBuf::from("README.md"), FileChange::Modified);
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_hot_reload_script() {
        let script = hot_reload_script(3000);
        assert!(script.contains("ws://localhost:3000"));
        assert!(script.contains("WebSocket"));
        assert!(script.contains("component-update"));
        assert!(script.contains("css-update"));
    }

    #[test]
    fn test_reload_message_json() {
        let full = ReloadMessage::FullReload;
        assert_eq!(full.to_json(), r#"{"type":"reload"}"#);

        let css = ReloadMessage::CssUpdate { href: "/style.css".to_string() };
        assert!(css.to_json().contains("css-update"));
        assert!(css.to_json().contains("/style.css"));

        let comp = ReloadMessage::ComponentUpdate { component: "Button".to_string() };
        assert!(comp.to_json().contains("component-update"));
        assert!(comp.to_json().contains("Button"));
    }

    #[test]
    fn test_file_change_variants() {
        assert_ne!(FileChange::Created, FileChange::Modified);
        assert_ne!(FileChange::Modified, FileChange::Deleted);
    }
}
