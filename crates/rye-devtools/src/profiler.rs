//! Profiler — render performance profiler.

/// Performance profiler — tracks render times, signal updates, and frame metrics.
pub struct Profiler {
    // TODO: render time records, signal update counts, flame chart data
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {}
    }

    /// Start a profiling session.
    pub fn start(&mut self) {
        // TODO
    }

    /// Stop profiling and return results.
    pub fn stop(&mut self) -> ProfileResult {
        ProfileResult::default()
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiling results — render times, update counts, etc.
#[derive(Debug, Default)]
pub struct ProfileResult {
    /// Total render time in microseconds.
    pub total_render_time_us: u64,
    /// Number of renders.
    pub render_count: u32,
    /// Number of signal updates.
    pub signal_update_count: u32,
}
