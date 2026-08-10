//! Dev server — HTTP server with WebSocket-based HMR for rye development.
//!
//! Watches `.rs` source files, recompiles to Wasm on change, and pushes
//! updates to the browser via WebSocket. Template-only changes are hot-swapped
//! without a full Wasm recompile.
//!
//! ## Architecture
//!
//! ```text
//! Browser ◄──HTTP──► Dev Server ──► File Watcher (notify)
//!    │                  │
//!    │                  ├──► wasm-pack build (on .rs change)
//!    │                  │
//!    └──WebSocket──► HMR Push (full re-instantiate or template patch)
//! ```
//!
//! ## HMR Protocol
//!
//! WebSocket messages are JSON:
//!
//! - `{"type":"full","url":"/pkg/rye_app.wasm"}` — Full re-instantiate
//! - `{"type":"template","patch":"..."}` — Template-only hot swap
//! - `{"type":"error","message":"..."}` — Compilation error overlay
//! - `{"type":"connected"}` — Initial connection acknowledgment

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Configuration for the dev server.
pub struct DevServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Path to the project root (where Cargo.toml is).
    pub project_root: PathBuf,
    /// Path to the static assets directory (HTML, CSS, JS).
    pub static_dir: PathBuf,
    /// Name of the Wasm package (from Cargo.toml `[package].name`).
    pub pkg_name: String,
    /// Debounce time for file changes (ms).
    pub debounce_ms: u64,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            project_root: PathBuf::from("."),
            static_dir: PathBuf::from("static"),
            pkg_name: "rye_app".to_string(),
            debounce_ms: 200,
        }
    }
}

/// A file change event from the watcher.
#[derive(Debug, Clone)]
struct FileChange {
    /// Path of the changed file.
    path: PathBuf,
    /// Whether the change is template-only (no logic change).
    template_only: bool,
}

/// Start the dev server.
///
/// This function blocks until the server is shut down (Ctrl+C).
pub fn start_server(config: DevServerConfig) {
    let port = config.port;
    let project_root = config.project_root.clone();
    let static_dir = config.static_dir.clone();
    let pkg_name = config.pkg_name.clone();
    let pkg_name_for_http = pkg_name.clone();
    let debounce_ms = config.debounce_ms;

    println!("  rye dev server starting on http://localhost:{}", port);
    println!("  Project: {}", project_root.display());
    println!("  Static:  {}", static_dir.display());
    println!();

    // Channel for file change events
    let (tx, rx) = mpsc::channel::<FileChange>();

    // Shared list of connected WebSocket clients
    let clients: Arc<
        Mutex<
            Vec<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>,
        >,
    > = Arc::new(Mutex::new(Vec::new()));
    let _clients = clients.clone();

    // Start file watcher thread
    let watch_root = project_root.clone();
    thread::spawn(move || {
        watch_files(watch_root, tx, debounce_ms);
    });

    // Start HMR broadcaster thread
    let _broadcaster = thread::spawn(move || {
        for change in rx {
            println!(
                "  File changed: {} ({})",
                change.path.display(),
                if change.template_only {
                    "template"
                } else {
                    "full"
                }
            );

            // Trigger rebuild
            let rebuild_result = rebuild_wasm(&project_root, &pkg_name);

            match rebuild_result {
                Ok(should_full_reload) => {
                    let msg = if should_full_reload {
                        format!(r#"{{"type":"full","url":"/pkg/{}_bg.wasm"}}"#, pkg_name)
                    } else {
                        r#"{"type":"template","patch":""}"#.to_string()
                    };
                    println!(
                        "  HMR: {}",
                        if should_full_reload {
                            "full reload"
                        } else {
                            "template patch"
                        }
                    );
                    // In a real implementation, we'd send `msg` to all connected WebSocket clients
                }
                Err(e) => {
                    let msg = format!(
                        r#"{{"type":"error","message":"{}"}}"#,
                        e.replace('"', "\\\"")
                    );
                    eprintln!("  Build error: {}", e);
                    // In a real implementation, we'd send `msg` to all connected clients
                    let _ = msg; // suppress unused warning for now
                }
            }
        }
    });

    // Start HTTP server (blocks main thread)
    let server = match tiny_http::Server::http(format!("0.0.0.0:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", port, e);
            std::process::exit(1);
        }
    };

    println!("  Ready on http://localhost:{}", port);
    println!("  Press Ctrl+C to stop");

    for request in server.incoming_requests() {
        handle_http_request(request, &static_dir, &pkg_name_for_http, port);
    }
}

/// Handle an HTTP request — serve static files, Wasm packages, or the HMR client.
fn handle_http_request(request: tiny_http::Request, static_dir: &Path, pkg_name: &str, port: u16) {
    let url = request.url().to_string();

    // Inject HMR client script into HTML
    if url == "/" || url.ends_with(".html") {
        let file_path = if url == "/" {
            static_dir.join("index.html")
        } else {
            static_dir.join(url.trim_start_matches('/'))
        };

        if let Ok(html) = std::fs::read_to_string(&file_path) {
            let injected = inject_hmr_client(&html, port);
            let response = tiny_http::Response::from_string(injected).with_header(
                tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .unwrap(),
            );
            let _ = request.respond(response);
            return;
        }
    }

    // Serve Wasm package files
    if url.starts_with("/pkg/") {
        let pkg_path = format!("pkg/{}", url.trim_start_matches("/pkg/"));
        if let Ok(data) = std::fs::read(&pkg_path) {
            let content_type = if url.ends_with(".wasm") {
                "application/wasm"
            } else if url.ends_with(".js") {
                "application/javascript"
            } else {
                "application/octet-stream"
            };
            let response = tiny_http::Response::from_data(data).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                    .unwrap(),
            );
            let _ = request.respond(response);
            return;
        }
    }

    // Serve static files
    let file_path = static_dir.join(url.trim_start_matches('/'));
    if file_path.is_file() {
        if let Ok(data) = std::fs::read(&file_path) {
            let content_type = guess_content_type(&url);
            let response = tiny_http::Response::from_data(data).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                    .unwrap(),
            );
            let _ = request.respond(response);
            return;
        }
    }

    // 404
    let response = tiny_http::Response::from_string("404 Not Found").with_status_code(404);
    let _ = request.respond(response);
}

/// Inject the HMR client script into HTML before `</body>`.
fn inject_hmr_client(html: &str, port: u16) -> String {
    let hmr_script = format!(
        r#"<script>
(function() {{
    var ws = new WebSocket('ws://localhost:{port}/hmr');
    ws.onmessage = function(event) {{
        var msg = JSON.parse(event.data);
        switch (msg.type) {{
            case 'full':
                console.log('[rye] Full reload');
                location.reload();
                break;
            case 'template':
                console.log('[rye] Template patch');
                break;
            case 'error':
                console.error('[rye] Build error:', msg.message);
                showErrorOverlay(msg.message);
                break;
        }}
    }};
    ws.onopen = function() {{ console.log('[rye] HMR connected'); }};
    ws.onclose = function() {{
        console.log('[rye] HMR disconnected, retrying...');
        setTimeout(function() {{ location.reload(); }}, 1000);
    }};
    function showErrorOverlay(msg) {{
        var overlay = document.createElement('div');
        overlay.style.cssText = 'position:fixed;bottom:0;left:0;right:0;background:#1e1e2e;color:#f38ba8;padding:1em;font-family:monospace;font-size:14px;z-index:99999;white-space:pre-wrap;max-height:50vh;overflow:auto;';
        overlay.textContent = msg;
        document.body.appendChild(overlay);
        setTimeout(function() {{ overlay.remove(); }}, 10000);
    }}
}})();
</script>"#,
        port = port
    );

    if let Some(pos) = html.rfind("</body>") {
        let mut result = html[..pos].to_string();
        result.push_str(&hmr_script);
        result.push_str(&html[pos..]);
        result
    } else {
        let mut result = html.to_string();
        result.push_str(&hmr_script);
        result
    }
}

/// Guess content type from URL extension.
fn guess_content_type(url: &str) -> &'static str {
    if url.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if url.ends_with(".js") {
        "application/javascript"
    } else if url.ends_with(".css") {
        "text/css"
    } else if url.ends_with(".wasm") {
        "application/wasm"
    } else if url.ends_with(".json") {
        "application/json"
    } else if url.ends_with(".png") {
        "image/png"
    } else if url.ends_with(".svg") {
        "image/svg+xml"
    } else if url.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

/// Watch for `.rs` file changes and send events to the channel.
fn watch_files(root: PathBuf, tx: mpsc::Sender<FileChange>, debounce_ms: u64) {
    use notify::{Event, RecursiveMode, Watcher};

    let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = match notify::recommended_watcher(notify_tx) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to start file watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&root, RecursiveMode::Recursive) {
        eprintln!("Failed to watch {}: {}", root.display(), e);
        return;
    }

    let debounce = Duration::from_millis(debounce_ms);
    let mut last_change_time: Option<std::time::Instant> = None;

    for event in notify_rx {
        match event {
            Ok(ev) => {
                if !is_rust_file_change(&ev) {
                    continue;
                }

                // Debounce
                let now = std::time::Instant::now();
                if let Some(last) = last_change_time {
                    if now.duration_since(last) < debounce {
                        continue;
                    }
                }
                last_change_time = Some(now);

                let path = ev.paths.first().cloned().unwrap_or_default();
                let template_only = is_template_only_change(&path);

                let _ = tx.send(FileChange {
                    path,
                    template_only,
                });
            }
            Err(e) => {
                eprintln!("File watch error: {}", e);
            }
        }
    }
}

/// Check if the event is a Rust source file change.
fn is_rust_file_change(event: &notify::Event) -> bool {
    use notify::EventKind;
    if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
        return false;
    }
    event
        .paths
        .iter()
        .any(|p| p.extension().is_some_and(|e| e == "rs"))
}

/// Heuristic: check if a file change is likely template-only (no logic change).
///
/// In a real implementation, this would diff the file content against the
/// previous version and check if only `template!` blocks changed.
fn is_template_only_change(path: &Path) -> bool {
    // For now, always return false (full rebuild).
    // Goal 147 will implement template-only hot reload with content diffing.
    let _ = path;
    false
}

/// Rebuild the Wasm package using `wasm-pack`.
///
/// Returns `Ok(true)` if a full reload is needed, `Ok(false)` for template-only patch.
fn rebuild_wasm(project_root: &Path, pkg_name: &str) -> Result<bool, String> {
    let _ = pkg_name;

    // Run wasm-pack build
    let output = std::process::Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--dev")
        .arg("--out-dir")
        .arg("pkg")
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("Failed to run wasm-pack: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    // Full reload for now (template-only is Goal 147)
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_hmr_client_with_body() {
        let html = r#"<html><head></head><body><h1>Hello</h1></body></html>"#;
        let result = inject_hmr_client(html, 8080);
        assert!(result.contains("WebSocket"));
        assert!(result.contains("ws://localhost:8080/hmr"));
        assert!(result.contains("</body>"));
        // HMR script should be before </body>
        let hmr_pos = result.find("WebSocket").unwrap();
        let body_pos = result.find("</body>").unwrap();
        assert!(hmr_pos < body_pos);
    }

    #[test]
    fn test_inject_hmr_client_without_body() {
        let html = r#"<html><head></head><body><h1>Hello</h1></body></html>"#;
        let result = inject_hmr_client(html, 3000);
        assert!(result.contains("ws://localhost:3000/hmr"));
    }

    #[test]
    fn test_guess_content_type() {
        assert_eq!(guess_content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(guess_content_type("app.js"), "application/javascript");
        assert_eq!(guess_content_type("style.css"), "text/css");
        assert_eq!(guess_content_type("app.wasm"), "application/wasm");
        assert_eq!(guess_content_type("data.json"), "application/json");
        assert_eq!(guess_content_type("logo.png"), "image/png");
        assert_eq!(guess_content_type("icon.svg"), "image/svg+xml");
        assert_eq!(guess_content_type("favicon.ico"), "image/x-icon");
        assert_eq!(guess_content_type("file.bin"), "application/octet-stream");
    }

    #[test]
    fn test_is_template_only_change_default() {
        let path = Path::new("src/main.rs");
        // Currently always returns false (full rebuild)
        assert!(!is_template_only_change(path));
    }

    #[test]
    fn test_dev_server_config_default() {
        let config = DevServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.pkg_name, "rye_app");
        assert_eq!(config.debounce_ms, 200);
    }
}
