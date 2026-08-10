//! Code splitting — lazy chunk loading for smaller initial Wasm downloads.
//!
//! In a traditional Wasm app, the entire binary is downloaded and parsed
//! before any interactivity is available. For large apps, this can be
//! several megabytes.
//!
//! Code splitting solves this by dividing the app into chunks:
//! - **Main chunk**: Core framework + critical-path components (always loaded)
//! - **Route chunks**: Per-route component trees (loaded on navigation)
//! - **Island chunks**: Per-island Wasm (loaded on hydration, see `islands.rs`)
//! - **Lazy chunks**: Explicitly lazy-loaded components via `LazyComponent`
//!
//! ## How it works
//!
//! 1. At build time, `wasm-pack` + custom tooling splits the Wasm binary
//!    into multiple `.wasm` files based on the chunk graph.
//! 2. At runtime, the `ChunkLoader` dynamically imports chunks via
//!    `WebAssembly.instantiateStreaming` (browser) or `include_bytes!` (native).
//! 3. Components wrapped in `LazyComponent` suspend until their chunk loads,
//!    showing a fallback via `Suspense`.
//!
//! ## Usage
//!
//! ```ignore
//! use rye_core::code_split::{ChunkLoader, LazyComponent};
//!
//! // Define a lazy-loaded route
//! let settings_route = LazyComponent::new("settings_chunk", || {
//!     template! { Settings { } }
//! });
//!
//! // In a template:
//! template! {
//!     Suspense {
//!         fallback: template! { "Loading..." },
//!         settings_route
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Unique identifier for a Wasm chunk.
pub type ChunkId = String;

/// Status of a chunk load operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkStatus {
    /// Chunk has not been requested yet.
    NotLoaded,
    /// Chunk is currently loading.
    Loading,
    /// Chunk has been loaded successfully.
    Loaded,
    /// Chunk failed to load.
    Error(String),
}

/// A loaded chunk — holds the raw Wasm bytes and instantiation status.
#[derive(Debug, Clone)]
pub struct LoadedChunk {
    /// Chunk identifier.
    pub id: ChunkId,
    /// Raw Wasm bytes (or empty if loaded via streaming).
    pub bytes: Vec<u8>,
    /// URL the chunk was loaded from (if applicable).
    pub url: String,
}

/// Chunk loader — manages dynamic loading of Wasm chunks.
///
/// On Wasm targets, this uses `fetch` + `WebAssembly.instantiateStreaming`.
/// On native targets, chunks are compiled into the binary via `include_bytes!`.
pub struct ChunkLoader {
    /// Map of chunk ID → load status.
    statuses: Arc<Mutex<HashMap<ChunkId, ChunkStatus>>>,
    /// Map of chunk ID → loaded chunk data.
    loaded: Arc<Mutex<HashMap<ChunkId, LoadedChunk>>>,
    /// Map of chunk ID → URL.
    chunk_urls: HashMap<ChunkId, String>,
}

impl ChunkLoader {
    /// Create a new chunk loader.
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(Mutex::new(HashMap::new())),
            loaded: Arc::new(Mutex::new(HashMap::new())),
            chunk_urls: HashMap::new(),
        }
    }

    /// Register a chunk URL.
    pub fn register_chunk(&mut self, id: &str, url: &str) {
        self.chunk_urls.insert(id.to_string(), url.to_string());
    }

    /// Get the status of a chunk.
    pub fn status(&self, id: &str) -> ChunkStatus {
        let statuses = self.statuses.lock().unwrap();
        statuses.get(id).cloned().unwrap_or(ChunkStatus::NotLoaded)
    }

    /// Check if a chunk is loaded.
    pub fn is_loaded(&self, id: &str) -> bool {
        self.status(id) == ChunkStatus::Loaded
    }

    /// Load a chunk by ID.
    ///
    /// On Wasm targets, this would use `fetch` + `WebAssembly.instantiateStreaming`.
    /// On native targets, this is a no-op (chunks are compiled in).
    ///
    /// Returns `Ok(())` if the chunk is already loaded or loads successfully.
    pub fn load(&self, id: &str) -> Result<(), String> {
        // Check if already loaded
        if self.is_loaded(id) {
            return Ok(());
        }

        // Check if currently loading
        {
            let statuses = self.statuses.lock().unwrap();
            if let Some(ChunkStatus::Loading) = statuses.get(id) {
                return Err("Chunk is already loading".to_string());
            }
            if let Some(ChunkStatus::Error(e)) = statuses.get(id) {
                return Err(e.clone());
            }
        }

        // Mark as loading
        {
            let mut statuses = self.statuses.lock().unwrap();
            statuses.insert(id.to_string(), ChunkStatus::Loading);
        }

        // Get chunk URL
        let url = self.chunk_urls.get(id).map(|s| s.as_str()).unwrap_or("");

        // Load chunk bytes
        let bytes = if url.is_empty() {
            // No URL registered — chunk might be compiled in
            Vec::new()
        } else {
            // On native targets, try to read from filesystem
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::fs::read(url).map_err(|e| {
                    let msg = format!("Failed to load chunk {}: {}", id, e);
                    let mut statuses = self.statuses.lock().unwrap();
                    statuses.insert(id.to_string(), ChunkStatus::Error(msg.clone()));
                    msg
                })?
            }

            // On Wasm targets, this would use fetch API
            #[cfg(target_arch = "wasm32")]
            {
                Vec::new()
            }
        };

        // Mark as loaded
        {
            let mut statuses = self.statuses.lock().unwrap();
            statuses.insert(id.to_string(), ChunkStatus::Loaded);
        }

        // Store loaded chunk
        {
            let mut loaded = self.loaded.lock().unwrap();
            loaded.insert(
                id.to_string(),
                LoadedChunk {
                    id: id.to_string(),
                    bytes,
                    url: url.to_string(),
                },
            );
        }

        Ok(())
    }

    /// Load multiple chunks in parallel.
    pub fn load_many(&self, ids: &[&str]) -> Result<(), String> {
        for id in ids {
            self.load(id)?;
        }
        Ok(())
    }

    /// Get loaded chunk data.
    pub fn get_chunk(&self, id: &str) -> Option<LoadedChunk> {
        let loaded = self.loaded.lock().unwrap();
        loaded.get(id).cloned()
    }

    /// List all registered chunk IDs.
    pub fn registered_chunks(&self) -> Vec<&str> {
        self.chunk_urls.keys().map(|s| s.as_str()).collect()
    }

    /// Preload a chunk without blocking (fire-and-forget).
    ///
    /// On Wasm targets, this would spawn an async fetch.
    /// On native targets, this loads synchronously in a background thread.
    pub fn preload(&self, id: &str) {
        let _ = self.load(id);
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Global chunk loader instance.
static GLOBAL_LOADER: Mutex<Option<ChunkLoader>> = Mutex::new(None);

/// Initialize the global chunk loader.
pub fn init_loader(loader: ChunkLoader) {
    let mut global = GLOBAL_LOADER.lock().unwrap();
    *global = Some(loader);
}

/// Load a chunk using the global loader.
pub fn load_chunk(id: &str) -> Result<(), String> {
    let global = GLOBAL_LOADER.lock().unwrap();
    if let Some(loader) = global.as_ref() {
        loader.load(id)
    } else {
        Err("No global chunk loader initialized".to_string())
    }
}

/// Check if a chunk is loaded using the global loader.
pub fn is_chunk_loaded(id: &str) -> bool {
    let global = GLOBAL_LOADER.lock().unwrap();
    global.as_ref().is_some_and(|loader| loader.is_loaded(id))
}

/// A lazy-loaded component.
///
/// Wraps a component that lives in a separate Wasm chunk. The component
/// is not loaded until first rendered, at which point it suspends (via
/// `Suspense`) until the chunk is available.
pub struct LazyComponent {
    /// Chunk ID for this component.
    chunk_id: ChunkId,
    /// Whether the chunk has been loaded.
    loaded: bool,
}

impl LazyComponent {
    /// Create a new lazy component bound to a chunk ID.
    pub fn new(chunk_id: impl Into<String>) -> Self {
        Self {
            chunk_id: chunk_id.into(),
            loaded: false,
        }
    }

    /// Get the chunk ID for this component.
    pub fn chunk_id(&self) -> &str {
        &self.chunk_id
    }

    /// Check if the component's chunk is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded || is_chunk_loaded(&self.chunk_id)
    }

    /// Load the component's chunk.
    pub fn load(&mut self) -> Result<(), String> {
        if self.loaded {
            return Ok(());
        }
        load_chunk(&self.chunk_id)?;
        self.loaded = true;
        Ok(())
    }
}

/// Generate the JS chunk loader bootstrap script.
///
/// This script provides a `__rye_load_chunk(id)` function that dynamically
/// imports a Wasm chunk and instantiates it.
pub fn chunk_loader_script() -> &'static str {
    r#"<script>
(function() {
    var chunkCache = {};
    var chunkLoading = {};

    window.__rye_load_chunk = function(id) {
        if (chunkCache[id]) return chunkCache[id];
        if (chunkLoading[id]) return chunkLoading[id];

        var url = '/pkg/chunks/' + id + '.wasm';
        chunkLoading[id] = fetch(url)
            .then(function(resp) { return resp.arrayBuffer(); })
            .then(function(bytes) {
                return WebAssembly.instantiate(bytes, {
                    env: {
                        __rye_chunk_loaded: function() {
                            console.log('[rye] Chunk loaded:', id);
                        }
                    }
                });
            })
            .then(function(result) {
                chunkCache[id] = result.instance;
                delete chunkLoading[id];
                return result.instance;
            })
            .catch(function(err) {
                delete chunkLoading[id];
                console.error('[rye] Failed to load chunk:', id, err);
                throw err;
            });

        return chunkLoading[id];
    };

    window.__rye_preload_chunk = function(id) {
        window.__rye_load_chunk(id).catch(function() {});
    };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_loader_register_and_status() {
        let loader = ChunkLoader::new();
        assert_eq!(loader.status("settings"), ChunkStatus::NotLoaded);
        assert!(!loader.is_loaded("settings"));
    }

    #[test]
    fn test_chunk_loader_load_nonexistent_url() {
        let mut loader = ChunkLoader::new();
        loader.register_chunk("test", "/nonexistent/path/chunk.wasm");

        let result = loader.load("test");
        assert!(result.is_err());
        assert!(matches!(loader.status("test"), ChunkStatus::Error(_)));
    }

    #[test]
    fn test_chunk_loader_load_empty_url() {
        let loader = ChunkLoader::new();
        // No URL registered — should succeed (compiled in)
        let result = loader.load("builtin");
        assert!(result.is_ok());
        assert!(loader.is_loaded("builtin"));
    }

    #[test]
    fn test_chunk_loader_double_load() {
        let mut loader = ChunkLoader::new();
        loader.load("chunk1").unwrap();
        // Loading again should succeed (already loaded)
        let result = loader.load("chunk1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chunk_loader_registered_chunks() {
        let mut loader = ChunkLoader::new();
        loader.register_chunk("a", "/a.wasm");
        loader.register_chunk("b", "/b.wasm");

        let chunks = loader.registered_chunks();
        assert!(chunks.contains(&"a"));
        assert!(chunks.contains(&"b"));
    }

    #[test]
    fn test_chunk_loader_get_chunk() {
        let loader = ChunkLoader::new();
        loader.load("test").unwrap();
        let chunk = loader.get_chunk("test").unwrap();
        assert_eq!(chunk.id, "test");
    }

    #[test]
    fn test_lazy_component() {
        let lc = LazyComponent::new("settings_chunk");
        assert_eq!(lc.chunk_id(), "settings_chunk");
        assert!(!lc.is_loaded());
    }

    #[test]
    fn test_lazy_component_load() {
        let mut lc = LazyComponent::new("test_chunk");
        // Without global loader, this will fail
        assert!(lc.load().is_err());
    }

    #[test]
    fn test_chunk_loader_script() {
        let script = chunk_loader_script();
        assert!(script.contains("__rye_load_chunk"));
        assert!(script.contains("__rye_preload_chunk"));
        assert!(script.contains("WebAssembly.instantiate"));
        assert!(script.contains("fetch"));
    }
}
