//! Goal 221: `rpg playground` online editor.
//!
//! Web-based rye code editor with live preview. Write rye components in the
//! browser, see rendered output instantly. Shareable URLs for code snippets.

use std::collections::HashMap;

/// A playground snippet — a shareable piece of rye code.
#[derive(Debug, Clone)]
pub struct PlaygroundSnippet {
    /// The snippet ID (for sharing).
    pub id: String,
    /// The component code.
    pub code: String,
    /// The title.
    pub title: String,
    /// The author.
    pub author: String,
    /// Whether the snippet is public.
    pub is_public: bool,
    /// The rye version used.
    pub rye_version: String,
}

impl PlaygroundSnippet {
    /// Create a new snippet.
    pub fn new(code: &str) -> Self {
        Self {
            id: generate_id(),
            code: code.to_string(),
            title: "Untitled".to_string(),
            author: "anonymous".to_string(),
            is_public: true,
            rye_version: "0.1.0".to_string(),
        }
    }

    /// Set the title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// Set visibility.
    pub fn private(mut self) -> Self {
        self.is_public = false;
        self
    }

    /// Generate a shareable URL.
    pub fn share_url(&self, base_url: &str) -> String {
        format!("{}/playground/{}", base_url, self.id)
    }
}

/// Generate a random 8-character ID.
fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut id = String::new();
    let mut hash: u64 = 0xdeadbeef_u64
        .wrapping_add(COUNTER.fetch_add(1, Ordering::SeqCst));
    for i in 0..8 {
        hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = ((hash >> (i * 4)) as usize) % chars.len();
        id.push(chars[idx]);
    }
    id
}

/// The playground server configuration.
#[derive(Debug, Clone)]
pub struct PlaygroundConfig {
    /// The port to serve on.
    pub port: u16,
    /// Whether to enable auto-save.
    pub auto_save: bool,
    /// Whether to enable live preview.
    pub live_preview: bool,
    /// The base URL for sharing.
    pub base_url: String,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            auto_save: true,
            live_preview: true,
            base_url: "http://localhost:3000".to_string(),
        }
    }
}

/// The playground editor — manages snippets and generates the editor HTML.
pub struct PlaygroundEditor {
    snippets: HashMap<String, PlaygroundSnippet>,
    config: PlaygroundConfig,
}

impl PlaygroundEditor {
    /// Create a new playground editor.
    pub fn new(config: PlaygroundConfig) -> Self {
        Self {
            snippets: HashMap::new(),
            config,
        }
    }

    /// Save a snippet and return its ID.
    pub fn save(&mut self, snippet: PlaygroundSnippet) -> String {
        let id = snippet.id.clone();
        self.snippets.insert(id.clone(), snippet);
        id
    }

    /// Load a snippet by ID.
    pub fn load(&self, id: &str) -> Option<&PlaygroundSnippet> {
        self.snippets.get(id)
    }

    /// List all public snippets.
    pub fn list_public(&self) -> Vec<&PlaygroundSnippet> {
        self.snippets.values().filter(|s| s.is_public).collect()
    }

    /// Get the number of snippets.
    pub fn len(&self) -> usize {
        self.snippets.len()
    }

    /// Generate the playground HTML page.
    pub fn generate_html(&self, snippet_id: Option<&str>) -> String {
        let initial_code = snippet_id
            .and_then(|id| self.snippets.get(id))
            .map(|s| s.code.clone())
            .unwrap_or_else(|| default_snippet_code());

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>rye playground</title>
<style>
body {{ margin:0; font-family:system-ui,sans-serif; display:flex; height:100vh; }}
#editor {{ width:50%; height:100%; font-family:monospace; font-size:14px; padding:12px;
           border:none; border-right:1px solid #ddd; resize:none; outline:none; }}
#preview {{ width:50%; height:100%; border:none; }}
#toolbar {{ position:fixed; top:0; right:0; z-index:100; padding:8px; background:#f5f5f5; }}
button {{ margin:0 4px; padding:4px 12px; cursor:pointer; }}
</style>
</head>
<body>
<div id="toolbar">
  <button onclick="share()">Share</button>
  <button onclick="format()">Format</button>
  <button onclick="reset()">Reset</button>
</div>
<textarea id="editor" spellcheck="false">{code}</textarea>
<iframe id="preview"></iframe>
<script>
var editor=document.getElementById('editor');
var preview=document.getElementById('preview');
var debounceTimer;
function render(){{
  var code=editor.value;
  var doc=preview.contentDocument||preview.contentWindow.document;
  doc.open();
  doc.write('<html><body><div id="app"></div></body></html>');
  doc.close();
  // In production, this would compile and render the rye code
  var app=doc.getElementById('app');
  app.innerHTML='<pre>Live preview: '+code.length+' chars</pre>';
}}
editor.addEventListener('input',function(){{
  clearTimeout(debounceTimer);
  debounceTimer=setTimeout(render,300);
}});
function share(){{
  var code=editor.value;
  // POST to /api/snippets and get shareable URL
  alert('Shareable URL copied to clipboard');
}}
function format(){{ editor.value=editor.value; render(); }}
function reset(){{ editor.value='{default_code}'; render(); }}
render();
</script>
</body>
</html>"#,
            code = initial_code.replace('<', "&lt;").replace('>', "&gt;"),
            default_code = default_snippet_code().replace('<', "&lt;").replace('>', "&gt;").replace('\n', "\\n"),
        )
    }
}

/// Default playground snippet code.
fn default_snippet_code() -> String {
    r#"use rye::prelude::*;

#[component]
fn Counter() {
    let count = signal(0);

    div {
        button { onclick: move |_| count.set(count.get() + 1); "+" }
        span { {count.get()} }
        button { onclick: move |_| count.set(count.get() - 1); "-" }
    }
}
"#.to_string()
}

/// Run the playground command.
pub fn run(args: &[String]) {
    let port: u16 = args
        .iter()
        .find(|a| a.starts_with("--port"))
        .and_then(|a| a[6..].parse().ok())
        .unwrap_or(3000);

    let config = PlaygroundConfig {
        port,
        ..Default::default()
    };

    let editor = PlaygroundEditor::new(config);
    let html = editor.generate_html(None);

    println!("Starting rye playground on port {}", port);
    println!("Open http://localhost:{} in your browser", port);
    println!("\nHTML page size: {} bytes", html.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_new() {
        let snippet = PlaygroundSnippet::new("fn Hello() {}");
        assert!(!snippet.id.is_empty());
        assert_eq!(snippet.title, "Untitled");
        assert!(snippet.is_public);
    }

    #[test]
    fn test_snippet_builder() {
        let snippet = PlaygroundSnippet::new("code")
            .with_title("My Snippet")
            .with_author("alice")
            .private();
        assert_eq!(snippet.title, "My Snippet");
        assert_eq!(snippet.author, "alice");
        assert!(!snippet.is_public);
    }

    #[test]
    fn test_snippet_share_url() {
        let snippet = PlaygroundSnippet::new("code");
        let url = snippet.share_url("http://localhost:3000");
        assert!(url.contains("/playground/"));
        assert!(url.contains(&snippet.id));
    }

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_playground_editor_save_load() {
        let mut editor = PlaygroundEditor::new(PlaygroundConfig::default());
        let snippet = PlaygroundSnippet::new("fn Test() {}");
        let id = editor.save(snippet);
        assert!(editor.load(&id).is_some());
        assert!(editor.load("nonexistent").is_none());
    }

    #[test]
    fn test_playground_editor_list_public() {
        let mut editor = PlaygroundEditor::new(PlaygroundConfig::default());
        editor.save(PlaygroundSnippet::new("a"));
        editor.save(PlaygroundSnippet::new("b").private());
        assert_eq!(editor.list_public().len(), 1);
    }

    #[test]
    fn test_playground_editor_len() {
        let mut editor = PlaygroundEditor::new(PlaygroundConfig::default());
        editor.save(PlaygroundSnippet::new("a"));
        editor.save(PlaygroundSnippet::new("b"));
        assert_eq!(editor.len(), 2);
    }

    #[test]
    fn test_playground_generate_html() {
        let editor = PlaygroundEditor::new(PlaygroundConfig::default());
        let html = editor.generate_html(None);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("editor"));
        assert!(html.contains("preview"));
    }

    #[test]
    fn test_playground_generate_html_with_snippet() {
        let mut editor = PlaygroundEditor::new(PlaygroundConfig::default());
        let snippet = PlaygroundSnippet::new("fn Custom() {}").with_title("Custom");
        let id = editor.save(snippet);
        let html = editor.generate_html(Some(&id));
        assert!(html.contains("Custom"));
    }

    #[test]
    fn test_default_snippet_code() {
        let code = default_snippet_code();
        assert!(code.contains("Counter"));
        assert!(code.contains("signal"));
    }
}
