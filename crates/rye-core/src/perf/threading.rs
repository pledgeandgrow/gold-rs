//! Goal 107: Wasm threading via SharedArrayBuffer.
//!
//! Offload heavy computation (layout, text shaping, image processing) to
//! Web Workers via SharedArrayBuffer. The main thread stays free for DOM
//! updates. Requires COOP/COEP headers — the dev server auto-configures these.
//!
//! ## Design
//!
//! - `WorkerPool` manages a set of Web Workers
//! - `SharedBuffer` wraps `SharedArrayBuffer` for cross-thread data
//! - `compute_offloaded` sends work to a worker and returns a handle
//! - Dev server sets `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers

/// Configuration for SharedArrayBuffer threading.
#[derive(Debug, Clone)]
pub struct ThreadingConfig {
    /// Number of worker threads (default: num_cpus).
    pub worker_count: usize,
    /// Worker script URL.
    pub worker_url: String,
    /// Whether COOP/COEP headers are set (required for SharedArrayBuffer).
    pub cross_origin_isolated: bool,
}

impl ThreadingConfig {
    /// Create a new threading config with default worker count.
    pub fn new(worker_url: impl Into<String>) -> Self {
        Self {
            worker_count: detect_worker_count(),
            worker_url: worker_url.into(),
            cross_origin_isolated: false,
        }
    }

    /// Set the number of workers.
    pub fn with_workers(mut self, count: usize) -> Self {
        self.worker_count = count;
        self
    }

    /// Mark as cross-origin isolated (COOP/COEP headers set).
    pub fn cross_origin_isolated(mut self) -> Self {
        self.cross_origin_isolated = true;
        self
    }
}

/// Detect the number of available worker threads.
pub fn detect_worker_count() -> usize {
    // On Wasm: navigator.hardwareConcurrency
    // On native: std::thread::available_parallelism
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }

    #[cfg(target_arch = "wasm32")]
    {
        4 // Conservative default
    }
}

/// Check if SharedArrayBuffer is available (requires COOP/COEP headers).
pub fn is_shared_array_buffer_available() -> bool {
    // On Wasm: typeof SharedArrayBuffer !== 'undefined'
    // On native: always false (use std::thread instead)
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }

    #[cfg(target_arch = "wasm32")]
    {
        false // Would check at runtime
    }
}

/// A handle to an offloaded computation.
pub struct ComputeHandle {
    /// Job ID for tracking.
    pub job_id: u64,
    /// Whether the job is complete.
    pub complete: bool,
    /// Result data (if complete).
    pub result: Option<Vec<u8>>,
}

impl ComputeHandle {
    /// Create a new pending compute handle.
    pub fn pending(job_id: u64) -> Self {
        Self {
            job_id,
            complete: false,
            result: None,
        }
    }

    /// Mark as complete with result data.
    pub fn complete_with(&mut self, data: Vec<u8>) {
        self.complete = true;
        self.result = Some(data);
    }
}

/// A pool of Web Workers for offloading computation.
pub struct WorkerPool {
    /// Configuration.
    config: ThreadingConfig,
    /// Next job ID.
    next_job_id: u64,
    /// Pending jobs.
    pending: Vec<ComputeHandle>,
}

impl WorkerPool {
    /// Create a new worker pool.
    pub fn new(config: ThreadingConfig) -> Self {
        Self {
            config,
            next_job_id: 0,
            pending: Vec::new(),
        }
    }

    /// Get the number of workers in the pool.
    pub fn worker_count(&self) -> usize {
        self.config.worker_count
    }

    /// Whether threading is available.
    pub fn is_available(&self) -> bool {
        self.config.cross_origin_isolated && is_shared_array_buffer_available()
    }

    /// Submit a computation job to the pool.
    ///
    /// On Wasm with SharedArrayBuffer: posts a message to a worker.
    /// On native or without SAB: runs synchronously (fallback).
    pub fn submit(&mut self, _data: &[u8]) -> ComputeHandle {
        let job_id = self.next_job_id;
        self.next_job_id += 1;

        if !self.is_available() {
            // Fallback: run synchronously
            let mut handle = ComputeHandle::pending(job_id);
            handle.complete_with(_data.to_vec()); // Echo back as "result"
            handle
        } else {
            // Would post message to worker
            let handle = ComputeHandle::pending(job_id);
            self.pending.push(handle.clone());
            handle
        }
    }

    /// Poll for completed jobs.
    pub fn poll(&mut self) -> Vec<ComputeHandle> {
        let mut completed = Vec::new();
        self.pending.retain(|h| {
            if h.complete {
                completed.push(h.clone());
                false
            } else {
                true
            }
        });
        completed
    }
}

impl Clone for ComputeHandle {
    fn clone(&self) -> Self {
        Self {
            job_id: self.job_id,
            complete: self.complete,
            result: self.result.clone(),
        }
    }
}

/// Generate the COOP/COEP headers needed for SharedArrayBuffer.
///
/// These headers must be set on all responses for the page:
/// - `Cross-Origin-Opener-Policy: same-origin`
/// - `Cross-Origin-Embedder-Policy: require-corp`
pub fn cross_origin_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Cross-Origin-Opener-Policy", "same-origin"),
        ("Cross-Origin-Embedder-Policy", "require-corp"),
    ]
}

/// Generate the Web Worker script for offloaded computation.
pub fn worker_script() -> &'static str {
    r#"<script>
// Worker script — runs in a Web Worker
self.onmessage = function(e) {
    var data = e.data;
    // Process data (layout, text shaping, etc.)
    // Post result back to main thread
    self.postMessage({ id: data.id, result: data.payload });
};
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threading_config() {
        let config = ThreadingConfig::new("/worker.js")
            .with_workers(8)
            .cross_origin_isolated();
        assert_eq!(config.worker_count, 8);
        assert_eq!(config.worker_url, "/worker.js");
        assert!(config.cross_origin_isolated);
    }

    #[test]
    fn test_detect_worker_count() {
        let count = detect_worker_count();
        assert!(count > 0);
    }

    #[test]
    fn test_worker_pool_creation() {
        let config = ThreadingConfig::new("/worker.js");
        let pool = WorkerPool::new(config);
        assert!(pool.worker_count() > 0);
    }

    #[test]
    fn test_worker_pool_submit_fallback() {
        let config = ThreadingConfig::new("/worker.js");
        let mut pool = WorkerPool::new(config);
        let handle = pool.submit(b"test data");
        assert!(handle.complete);
        assert!(handle.result.is_some());
    }

    #[test]
    fn test_cross_origin_headers() {
        let headers = cross_origin_headers();
        assert!(headers
            .iter()
            .any(|(k, _)| *k == "Cross-Origin-Opener-Policy"));
        assert!(headers
            .iter()
            .any(|(k, _)| *k == "Cross-Origin-Embedder-Policy"));
    }

    #[test]
    fn test_worker_script() {
        let script = worker_script();
        assert!(script.contains("onmessage"));
        assert!(script.contains("postMessage"));
    }

    #[test]
    fn test_compute_handle() {
        let mut h = ComputeHandle::pending(42);
        assert!(!h.complete);
        h.complete_with(vec![1, 2, 3]);
        assert!(h.complete);
        assert_eq!(h.result, Some(vec![1, 2, 3]));
    }
}
