//! Goal 104: Streaming Wasm compilation.
//!
//! Use `WebAssembly.instantiateStreaming()` instead of `fetch + instantiate`.
//! The browser compiles Wasm while downloading, reducing time-to-interactive.
//! Also generates `<link rel="preload">` hints for `.wasm` files.

/// Generate HTML preload hints for Wasm files.
///
/// Insert these in `<head>` so the browser starts downloading `.wasm`
/// before the JS glue code requests it.
pub fn generate_preload_hints(wasm_paths: &[&str]) -> String {
    wasm_paths
        .iter()
        .map(|path| {
            format!(
                r#"<link rel="preload" href="{}" as="fetch" crossorigin>"#,
                path
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate the JS bootstrap for streaming Wasm instantiation.
///
/// Uses `WebAssembly.instantiateStreaming()` when available (faster —
/// browser compiles while downloading), falls back to `fetch + arrayBuffer +
/// instantiate` for older browsers.
pub fn streaming_instantiate_script(wasm_url: &str) -> String {
    format!(
        r#"<script>
(function() {{
  var wasmUrl = '{url}';

  function streamingSupported() {{
    return 'instantiateStreaming' in WebAssembly;
  }}

  function instantiateStreaming() {{
    return WebAssembly.instantiateStreaming(fetch(wasmUrl), {{}})
      .then(function(result) {{
        return result.instance;
      }});
  }}

  function instantiateFallback() {{
    return fetch(wasmUrl)
      .then(function(resp) {{ return resp.arrayBuffer(); }})
      .then(function(bytes) {{
        return WebAssembly.instantiate(bytes, {{}});
      }})
      .then(function(result) {{
        return result.instance;
      }});
  }}

  window.__rye_instantiate_wasm = function() {{
    if (streamingSupported()) {{
      return instantiateStreaming();
    }} else {{
      return instantiateFallback();
    }}
  }};
}})();
</script>"#,
        url = wasm_url
    )
}

/// Configuration for streaming Wasm compilation.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// URL of the Wasm file.
    pub wasm_url: String,
    /// Whether to generate preload hints.
    pub preload: bool,
    /// Whether to use streaming instantiation (falls back if unsupported).
    pub streaming: bool,
}

impl StreamingConfig {
    /// Create a new streaming config for the given Wasm URL.
    pub fn new(wasm_url: impl Into<String>) -> Self {
        Self {
            wasm_url: wasm_url.into(),
            preload: true,
            streaming: true,
        }
    }

    /// Disable preload hints.
    pub fn without_preload(mut self) -> Self {
        self.preload = false;
        self
    }

    /// Disable streaming (force fallback to fetch + instantiate).
    pub fn without_streaming(mut self) -> Self {
        self.streaming = false;
        self
    }

    /// Generate the full HTML head content for this config.
    pub fn to_head_html(&self) -> String {
        let mut html = String::new();

        if self.preload {
            html.push_str(&generate_preload_hints(&[&self.wasm_url]));
            html.push('\n');
        }

        if self.streaming {
            html.push_str(&streaming_instantiate_script(&self.wasm_url));
        }

        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_hints() {
        let hints = generate_preload_hints(&["/app.wasm", "/chunks/settings.wasm"]);
        assert!(hints.contains(r#"href="/app.wasm""#));
        assert!(hints.contains(r#"href="/chunks/settings.wasm""#));
        assert!(hints.contains(r#"rel="preload""#));
        assert!(hints.contains(r#"as="fetch""#));
        assert!(hints.contains("crossorigin"));
    }

    #[test]
    fn test_streaming_script() {
        let script = streaming_instantiate_script("/app.wasm");
        assert!(script.contains("instantiateStreaming"));
        assert!(script.contains("arrayBuffer"));
        assert!(script.contains("__rye_instantiate_wasm"));
        assert!(script.contains("/app.wasm"));
    }

    #[test]
    fn test_streaming_config_head_html() {
        let config = StreamingConfig::new("/app.wasm");
        let html = config.to_head_html();
        assert!(html.contains("preload"));
        assert!(html.contains("instantiateStreaming"));
    }

    #[test]
    fn test_streaming_config_no_preload() {
        let config = StreamingConfig::new("/app.wasm").without_preload();
        let html = config.to_head_html();
        assert!(!html.contains("preload"));
        assert!(html.contains("instantiateStreaming"));
    }

    #[test]
    fn test_streaming_config_no_streaming() {
        let config = StreamingConfig::new("/app.wasm").without_streaming();
        let html = config.to_head_html();
        assert!(html.contains("preload"));
        assert!(!html.contains("instantiateStreaming"));
    }
}
