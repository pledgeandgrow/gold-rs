//! Goal 123: Edge rendering support.
//!
//! Deploy SSR to edge runtimes (Cloudflare Workers, Deno Deploy, Vercel Edge).
//! Minimal runtime — no `tokio`, no heavy deps. Compiles to `wasm32-wasi`.

/// Edge runtime target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeRuntime {
    /// Cloudflare Workers.
    CloudflareWorkers,
    /// Deno Deploy.
    DenoDeploy,
    /// Vercel Edge Functions.
    VercelEdge,
    /// Generic WASI runtime.
    Wasi,
}

impl EdgeRuntime {
    /// Get the runtime name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeRuntime::CloudflareWorkers => "cloudflare-workers",
            EdgeRuntime::DenoDeploy => "deno-deploy",
            EdgeRuntime::VercelEdge => "vercel-edge",
            EdgeRuntime::Wasi => "wasi",
        }
    }
}

/// Edge rendering configuration.
#[derive(Debug, Clone)]
pub struct EdgeConfig {
    /// Target edge runtime.
    pub runtime: EdgeRuntime,
    /// Whether to enable streaming SSR.
    pub streaming: bool,
    /// Cache TTL for edge responses (seconds).
    pub cache_ttl: u32,
    /// Whether to use edge caching.
    pub edge_cache: bool,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            runtime: EdgeRuntime::Wasi,
            streaming: true,
            cache_ttl: 60,
            edge_cache: true,
        }
    }
}

impl EdgeConfig {
    /// Create config for a specific runtime.
    pub fn for_runtime(runtime: EdgeRuntime) -> Self {
        Self { runtime, ..Default::default() }
    }

    /// Set cache TTL.
    pub fn with_cache_ttl(mut self, ttl: u32) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Disable streaming.
    pub fn no_streaming(mut self) -> Self {
        self.streaming = false;
        self
    }
}

/// Edge response headers for caching.
pub fn edge_cache_headers(ttl: u32) -> Vec<(&'static str, String)> {
    vec![
        ("Cache-Control", format!("public, max-age={}", ttl)),
        ("CDN-Cache-Control", format!("max-age={}", ttl)),
    ]
}

/// Generate the edge function entry point for the given runtime.
pub fn edge_entry_script(config: &EdgeConfig) -> String {
    match config.runtime {
        EdgeRuntime::CloudflareWorkers => {
            r#"export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const html = await __rye_edge_render(url.pathname, url.searchParams);
    return new Response(html, {
      headers: {
        'Content-Type': 'text/html;charset=UTF-8',
        'Cache-Control': 'public, max-age=60'
      }
    });
  }
};"#.to_string()
        }
        EdgeRuntime::DenoDeploy => {
            r#"Deno.serve(async (request) => {
  const url = new URL(request.url);
  const html = await __rye_edge_render(url.pathname, url.searchParams);
  return new Response(html, {
    headers: {
      'Content-Type': 'text/html;charset=UTF-8',
      'Cache-Control': 'public, max-age=60'
    }
  });
});"#.to_string()
        }
        EdgeRuntime::VercelEdge => {
            r#"export default async function handler(request) {
  const url = new URL(request.url);
  const html = await __rye_edge_render(url.pathname, url.searchParams);
  return new Response(html, {
    headers: {
      'Content-Type': 'text/html;charset=UTF-8',
      'Cache-Control': 'public, max-age=60'
    }
  });
};"#.to_string()
        }
        EdgeRuntime::Wasi => {
            "// WASI edge runtime — uses standard HTTP server".to_string()
        }
    }
}

/// Check if a dependency is compatible with edge runtimes.
pub fn is_edge_compatible(dep_name: &str) -> bool {
    // These deps are known to be incompatible with edge/WASI runtimes
    let incompatible = ["tokio", "reqwest", "hyper", "actix-web", "axum", "warp"];
    !incompatible.iter().any(|d| dep_name.contains(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_runtime_names() {
        assert_eq!(EdgeRuntime::CloudflareWorkers.as_str(), "cloudflare-workers");
        assert_eq!(EdgeRuntime::DenoDeploy.as_str(), "deno-deploy");
        assert_eq!(EdgeRuntime::VercelEdge.as_str(), "vercel-edge");
        assert_eq!(EdgeRuntime::Wasi.as_str(), "wasi");
    }

    #[test]
    fn test_edge_config() {
        let config = EdgeConfig::for_runtime(EdgeRuntime::CloudflareWorkers)
            .with_cache_ttl(120)
            .no_streaming();
        assert_eq!(config.runtime, EdgeRuntime::CloudflareWorkers);
        assert_eq!(config.cache_ttl, 120);
        assert!(!config.streaming);
    }

    #[test]
    fn test_edge_cache_headers() {
        let headers = edge_cache_headers(60);
        assert!(headers.iter().any(|(k, v)| *k == "Cache-Control" && v.contains("max-age=60")));
    }

    #[test]
    fn test_edge_entry_script_cloudflare() {
        let config = EdgeConfig::for_runtime(EdgeRuntime::CloudflareWorkers);
        let script = edge_entry_script(&config);
        assert!(script.contains("fetch(request"));
    }

    #[test]
    fn test_edge_entry_script_deno() {
        let config = EdgeConfig::for_runtime(EdgeRuntime::DenoDeploy);
        let script = edge_entry_script(&config);
        assert!(script.contains("Deno.serve"));
    }

    #[test]
    fn test_edge_entry_script_vercel() {
        let config = EdgeConfig::for_runtime(EdgeRuntime::VercelEdge);
        let script = edge_entry_script(&config);
        assert!(script.contains("export default"));
    }

    #[test]
    fn test_is_edge_compatible() {
        assert!(!is_edge_compatible("tokio"));
        assert!(!is_edge_compatible("reqwest"));
        assert!(is_edge_compatible("rye-core"));
        assert!(is_edge_compatible("serde"));
    }
}
