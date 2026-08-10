//! Goal 108: Memory profiling tools.
//!
//! `rye inspect memory` CLI command that dumps Wasm linear memory usage,
//! allocation hotspots, and arena statistics. DevTools panel shows memory
//! growth over time and identifies leak sources.
//!
//! ## Design
//!
//! - `MemorySnapshot` captures allocation counts and sizes at a point in time
//! - `MemoryTracker` records snapshots over time for trend analysis
//! - `AllocationHotspot` identifies functions with the most allocations
//! - `LeakDetector` tracks growing allocation patterns

use std::collections::HashMap;

/// A snapshot of memory usage at a point in time.
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Total bytes allocated.
    pub total_allocated: usize,
    /// Total bytes freed.
    pub total_freed: usize,
    /// Current live allocations (allocated - freed).
    pub live_bytes: usize,
    /// Number of allocation calls.
    pub alloc_count: usize,
    /// Number of free calls.
    pub free_count: usize,
    /// Bytes allocated per category.
    pub by_category: HashMap<String, usize>,
    /// Timestamp (monotonic counter).
    pub timestamp: u64,
}

impl MemorySnapshot {
    /// Create a new empty snapshot.
    pub fn new(timestamp: u64) -> Self {
        Self {
            total_allocated: 0,
            total_freed: 0,
            live_bytes: 0,
            alloc_count: 0,
            free_count: 0,
            by_category: HashMap::new(),
            timestamp,
        }
    }

    /// Record an allocation.
    pub fn record_alloc(&mut self, size: usize, category: &str) {
        self.total_allocated += size;
        self.live_bytes += size;
        self.alloc_count += 1;
        *self.by_category.entry(category.to_string()).or_insert(0) += size;
    }

    /// Record a deallocation.
    pub fn record_free(&mut self, size: usize) {
        self.total_freed += size;
        self.live_bytes = self.live_bytes.saturating_sub(size);
        self.free_count += 1;
    }

    /// Net allocation rate (bytes per allocation).
    pub fn avg_alloc_size(&self) -> f64 {
        if self.alloc_count == 0 {
            0.0
        } else {
            self.total_allocated as f64 / self.alloc_count as f64
        }
    }
}

/// Tracks memory snapshots over time for trend analysis.
pub struct MemoryTracker {
    /// Recorded snapshots.
    snapshots: Vec<MemorySnapshot>,
    /// Current snapshot being built.
    current: MemorySnapshot,
    /// Counter for timestamps.
    tick: u64,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            current: MemorySnapshot::new(0),
            tick: 0,
        }
    }

    /// Record an allocation in the current snapshot.
    pub fn alloc(&mut self, size: usize, category: &str) {
        self.current.record_alloc(size, category);
    }

    /// Record a deallocation in the current snapshot.
    pub fn free(&mut self, size: usize) {
        self.current.record_free(size);
    }

    /// Take a snapshot and start a new one.
    pub fn snapshot(&mut self) -> &MemorySnapshot {
        let snap = std::mem::replace(&mut self.current, MemorySnapshot::new(self.tick + 1));
        self.snapshots.push(snap);
        self.tick += 1;
        self.snapshots.last().unwrap()
    }

    /// Get all recorded snapshots.
    pub fn snapshots(&self) -> &[MemorySnapshot] {
        &self.snapshots
    }

    /// Detect potential leaks — categories with monotonic growth.
    pub fn detect_leaks(&self) -> Vec<LeakReport> {
        if self.snapshots.len() < 2 {
            return Vec::new();
        }

        let mut leaks = Vec::new();
        let categories: std::collections::HashSet<&String> = self
            .snapshots
            .iter()
            .flat_map(|s| s.by_category.keys())
            .collect();

        for category in categories {
            let mut growing = true;
            let mut prev = 0usize;

            for snap in &self.snapshots {
                let current = snap.by_category.get(category).copied().unwrap_or(0);
                if current < prev {
                    growing = false;
                    break;
                }
                prev = current;
            }

            if growing && prev > 0 {
                let first = self
                    .snapshots
                    .first()
                    .unwrap()
                    .by_category
                    .get(category)
                    .copied()
                    .unwrap_or(0);
                leaks.push(LeakReport {
                    category: category.clone(),
                    growth_bytes: prev.saturating_sub(first),
                    final_bytes: prev,
                });
            }
        }

        leaks
    }

    /// Generate a human-readable memory report.
    pub fn report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Memory Report ===\n\n");

        if let Some(latest) = self.snapshots.last() {
            report.push_str(&format!("Live bytes: {}\n", latest.live_bytes));
            report.push_str(&format!("Total allocated: {}\n", latest.total_allocated));
            report.push_str(&format!("Total freed: {}\n", latest.total_freed));
            report.push_str(&format!("Allocations: {}\n", latest.alloc_count));
            report.push_str(&format!("Frees: {}\n", latest.free_count));
            report.push_str(&format!(
                "Avg alloc size: {:.1} bytes\n",
                latest.avg_alloc_size()
            ));

            report.push_str("\nBy category:\n");
            let mut cats: Vec<_> = latest.by_category.iter().collect();
            cats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, size) in cats {
                report.push_str(&format!("  {}: {} bytes\n", cat, size));
            }
        }

        let leaks = self.detect_leaks();
        if !leaks.is_empty() {
            report.push_str("\n=== Potential Leaks ===\n");
            for leak in leaks {
                report.push_str(&format!(
                    "  {} (grew by {} bytes to {})\n",
                    leak.category, leak.growth_bytes, leak.final_bytes
                ));
            }
        }

        report
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A detected potential memory leak.
#[derive(Debug, Clone)]
pub struct LeakReport {
    /// Category that is growing.
    pub category: String,
    /// How much it grew across all snapshots.
    pub growth_bytes: usize,
    /// Final size in bytes.
    pub final_bytes: usize,
}

/// An allocation hotspot — a function/location with many allocations.
#[derive(Debug, Clone)]
pub struct AllocationHotspot {
    /// Function or location name.
    pub location: String,
    /// Number of allocations from this location.
    pub count: usize,
    /// Total bytes allocated from this location.
    pub total_bytes: usize,
}

/// Allocation hotspot tracker.
pub struct HotspotTracker {
    /// Map of location → (count, total_bytes).
    hotspots: HashMap<String, (usize, usize)>,
}

impl HotspotTracker {
    /// Create a new hotspot tracker.
    pub fn new() -> Self {
        Self {
            hotspots: HashMap::new(),
        }
    }

    /// Record an allocation at a location.
    pub fn record(&mut self, location: &str, size: usize) {
        let entry = self.hotspots.entry(location.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += size;
    }

    /// Get the top N allocation hotspots by total bytes.
    pub fn top_n(&self, n: usize) -> Vec<AllocationHotspot> {
        let mut spots: Vec<_> = self
            .hotspots
            .iter()
            .map(|(loc, (count, bytes))| AllocationHotspot {
                location: loc.clone(),
                count: *count,
                total_bytes: *bytes,
            })
            .collect();
        spots.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        spots.truncate(n);
        spots
    }
}

impl Default for HotspotTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_snapshot() {
        let mut snap = MemorySnapshot::new(0);
        snap.record_alloc(100, "render");
        snap.record_alloc(200, "render");
        snap.record_free(100);

        assert_eq!(snap.total_allocated, 300);
        assert_eq!(snap.live_bytes, 200);
        assert_eq!(snap.alloc_count, 2);
        assert_eq!(snap.free_count, 1);
    }

    #[test]
    fn test_memory_tracker_snapshot() {
        let mut tracker = MemoryTracker::new();
        tracker.alloc(100, "render");
        tracker.alloc(200, "layout");
        tracker.snapshot();
        tracker.alloc(50, "render");
        tracker.snapshot();

        let snaps = tracker.snapshots();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].live_bytes, 300);
        assert_eq!(snaps[1].live_bytes, 50);
    }

    #[test]
    fn test_memory_tracker_leak_detection() {
        let mut tracker = MemoryTracker::new();
        tracker.alloc(100, "render");
        tracker.snapshot();
        tracker.alloc(200, "render");
        tracker.snapshot();
        tracker.alloc(300, "render");
        tracker.snapshot();

        let leaks = tracker.detect_leaks();
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].category, "render");
        assert_eq!(leaks[0].final_bytes, 300);
    }

    #[test]
    fn test_memory_tracker_no_leak() {
        let mut tracker = MemoryTracker::new();
        tracker.alloc(100, "render");
        tracker.snapshot();
        tracker.alloc(50, "render");
        tracker.snapshot();

        let leaks = tracker.detect_leaks();
        // "render" went 100 → 50, not growing
        assert_eq!(leaks.len(), 0);
    }

    #[test]
    fn test_memory_tracker_report() {
        let mut tracker = MemoryTracker::new();
        tracker.alloc(100, "render");
        tracker.alloc(200, "layout");
        tracker.snapshot();

        let report = tracker.report();
        assert!(report.contains("Live bytes: 300"));
        assert!(report.contains("render: 100 bytes"));
        assert!(report.contains("layout: 200 bytes"));
    }

    #[test]
    fn test_hotspot_tracker() {
        let mut ht = HotspotTracker::new();
        ht.record("render_component", 100);
        ht.record("render_component", 200);
        ht.record("layout_flex", 500);

        let top = ht.top_n(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].location, "layout_flex");
        assert_eq!(top[0].total_bytes, 500);
        assert_eq!(top[1].location, "render_component");
        assert_eq!(top[1].count, 2);
        assert_eq!(top[1].total_bytes, 300);
    }

    #[test]
    fn test_avg_alloc_size() {
        let mut snap = MemorySnapshot::new(0);
        snap.record_alloc(100, "a");
        snap.record_alloc(200, "a");
        assert_eq!(snap.avg_alloc_size(), 150.0);
    }
}
