//! Goal 109: Bridge call counter.
//!
//! DevTools panel that counts Wasm→JS bridge calls per frame. Highlights
//! components that make excessive DOM calls. Suggests batching opportunities.
//!
//! ## Design
//!
//! - `BridgeCounter` tracks calls per frame and per component
//! - `FrameStats` holds per-frame call counts
//! - `BridgeCall` records a single Wasm→JS bridge call
//! - At end of frame, `frame_report()` identifies batching opportunities

use std::collections::HashMap;

/// A single Wasm→JS bridge call record.
#[derive(Debug, Clone)]
pub struct BridgeCall {
    /// The operation type (e.g. "createElement", "setAttribute", "appendChild").
    pub op: String,
    /// The component that triggered the call.
    pub component: String,
    /// Size of the call payload in bytes (for batch optimization analysis).
    pub payload_bytes: usize,
}

/// Per-frame bridge call statistics.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Frame number.
    pub frame: u64,
    /// Total bridge calls in this frame.
    pub total_calls: usize,
    /// Calls per operation type.
    pub calls_by_op: HashMap<String, usize>,
    /// Calls per component.
    pub calls_by_component: HashMap<String, usize>,
    /// Total payload bytes transferred.
    pub total_payload: usize,
}

impl FrameStats {
    /// Create a new empty frame stats for the given frame number.
    pub fn new(frame: u64) -> Self {
        Self {
            frame,
            ..Default::default()
        }
    }

    /// Record a bridge call.
    pub fn record(&mut self, call: &BridgeCall) {
        self.total_calls += 1;
        *self.calls_by_op.entry(call.op.clone()).or_insert(0) += 1;
        *self.calls_by_component.entry(call.component.clone()).or_insert(0) += 1;
        self.total_payload += call.payload_bytes;
    }

    /// Find the component with the most calls.
    pub fn hottest_component(&self) -> Option<(&str, usize)> {
        self.calls_by_component
            .iter()
            .max_by_key(|(_, &c)| c)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Find the operation with the most calls.
    pub fn most_frequent_op(&self) -> Option<(&str, usize)> {
        self.calls_by_op
            .iter()
            .max_by_key(|(_, &c)| c)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

/// Bridge call counter — tracks Wasm→JS calls per frame.
pub struct BridgeCounter {
    /// Current frame being recorded.
    current_frame: FrameStats,
    /// Completed frames.
    frames: Vec<FrameStats>,
    /// Current frame number.
    frame_number: u64,
    /// Whether counting is enabled.
    enabled: bool,
}

impl BridgeCounter {
    /// Create a new bridge counter.
    pub fn new() -> Self {
        Self {
            current_frame: FrameStats::new(0),
            frames: Vec::new(),
            frame_number: 0,
            enabled: false,
        }
    }

    /// Enable bridge call counting.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable bridge call counting.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Whether counting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a bridge call (if enabled).
    pub fn record(&mut self, call: BridgeCall) {
        if self.enabled {
            self.current_frame.record(&call);
        }
    }

    /// End the current frame and start a new one.
    pub fn end_frame(&mut self) {
        if self.enabled {
            let frame = std::mem::replace(&mut self.current_frame, FrameStats::new(self.frame_number + 1));
            self.frames.push(frame);
            self.frame_number += 1;
        }
    }

    /// Get all recorded frames.
    pub fn frames(&self) -> &[FrameStats] {
        &self.frames
    }

    /// Get the last N frames.
    pub fn last_n_frames(&self, n: usize) -> &[FrameStats] {
        let start = self.frames.len().saturating_sub(n);
        &self.frames[start..]
    }

    /// Generate a report for the last frame.
    pub fn frame_report(&self) -> String {
        let mut report = String::new();

        if let Some(frame) = self.frames.last() {
            report.push_str(&format!("=== Frame {} Bridge Call Report ===\n\n", frame.frame));
            report.push_str(&format!("Total calls: {}\n", frame.total_calls));
            report.push_str(&format!("Total payload: {} bytes\n\n", frame.total_payload));

            if let Some((comp, count)) = frame.hottest_component() {
                report.push_str(&format!("Hottest component: {} ({} calls)\n", comp, count));
            }

            if let Some((op, count)) = frame.most_frequent_op() {
                report.push_str(&format!("Most frequent op: {} ({} calls)\n", op, count));
            }

            // Batching opportunities
            let batchable = self.find_batching_opportunities(frame);
            if !batchable.is_empty() {
                report.push_str("\n=== Batching Opportunities ===\n");
                for opp in batchable {
                    report.push_str(&format!(
                        "  {} ({} calls in {} — could be batched to 1)\n",
                        opp.op, opp.call_count, opp.component
                    ));
                }
            }

            // Warning for excessive calls
            if frame.total_calls > 50 {
                report.push_str("\n⚠ Warning: >50 bridge calls in single frame.\n");
                report.push_str("  Consider using begin_batch() / flush_batch().\n");
            }
        } else {
            report.push_str("No frames recorded.\n");
        }

        report
    }

    /// Find operations that are called multiple times for the same component
    /// (candidates for batching).
    fn find_batching_opportunities(&self, frame: &FrameStats) -> Vec<BatchingOpportunity> {
        let mut opportunities = Vec::new();

        // Group by (component, op) and count
        let mut groups: HashMap<(String, String), usize> = HashMap::new();
        // We don't have per-call records in FrameStats, so we approximate:
        // if a component has N calls and an op has M calls, and both are > 1,
        // it's likely some calls overlap.

        for (comp, comp_count) in &frame.calls_by_component {
            for (op, op_count) in &frame.calls_by_op {
                if *comp_count > 2 && *op_count > 2 {
                    groups.insert((comp.clone(), op.clone()), *op_count.min(comp_count));
                }
            }
        }

        for ((component, op), call_count) in groups {
            if call_count > 1 {
                opportunities.push(BatchingOpportunity {
                    component,
                    op,
                    call_count,
                });
            }
        }

        opportunities.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        opportunities
    }
}

impl Default for BridgeCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// A batching opportunity — multiple calls that could be combined.
#[derive(Debug, Clone)]
pub struct BatchingOpportunity {
    /// The component making the calls.
    pub component: String,
    /// The operation being repeated.
    pub op: String,
    /// Number of repeated calls.
    pub call_count: usize,
}

/// Generate the DevTools bridge counter panel script.
pub fn bridge_counter_script() -> &'static str {
    r#"<script>
(function() {
    var calls = [];
    var frameCalls = [];
    var enabled = false;

    window.__rye_bridge_counter = {
        enable: function() { enabled = true; },
        disable: function() { enabled = false; },
        record: function(op, component, payloadSize) {
            if (!enabled) return;
            frameCalls.push({ op: op, component: component, payload: payloadSize || 0 });
        },
        endFrame: function() {
            if (!enabled) return;
            calls.push(frameCalls);
            frameCalls = [];
            if (calls.length > 60) calls.shift(); // Keep last 60 frames
        },
        report: function() {
            var last = calls[calls.length - 1] || [];
            var total = last.length;
            var byOp = {};
            var byComp = {};
            last.forEach(function(c) {
                byOp[c.op] = (byOp[c.op] || 0) + 1;
                byComp[c.component] = (byComp[c.component] || 0) + 1;
            });
            return { total: total, byOp: byOp, byComp: byComp };
        },
        frames: function() { return calls.length; }
    };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_counter_basic() {
        let mut counter = BridgeCounter::new();
        counter.enable();

        counter.record(BridgeCall {
            op: "createElement".to_string(),
            component: "App".to_string(),
            payload_bytes: 32,
        });
        counter.record(BridgeCall {
            op: "setAttribute".to_string(),
            component: "App".to_string(),
            payload_bytes: 16,
        });
        counter.end_frame();

        assert_eq!(counter.frames().len(), 1);
        assert_eq!(counter.frames()[0].total_calls, 2);
    }

    #[test]
    fn test_bridge_counter_disabled() {
        let mut counter = BridgeCounter::new();
        counter.record(BridgeCall {
            op: "createElement".to_string(),
            component: "App".to_string(),
            payload_bytes: 32,
        });
        counter.end_frame();
        assert_eq!(counter.frames().len(), 0);
    }

    #[test]
    fn test_frame_stats_hottest() {
        let mut frame = FrameStats::new(0);
        frame.record(&BridgeCall {
            op: "createElement".into(),
            component: "App".into(),
            payload_bytes: 10,
        });
        frame.record(&BridgeCall {
            op: "setAttribute".into(),
            component: "App".into(),
            payload_bytes: 10,
        });
        frame.record(&BridgeCall {
            op: "appendChild".into(),
            component: "List".into(),
            payload_bytes: 10,
        });

        assert_eq!(frame.hottest_component(), Some(("App", 2)));
        // Both ops have count 1 except one has count 1 — actually createElement=1, setAttribute=1, appendChild=1
        // No single op has more than 1
    }

    #[test]
    fn test_frame_report() {
        let mut counter = BridgeCounter::new();
        counter.enable();

        for i in 0..10 {
            counter.record(BridgeCall {
                op: "setAttribute".into(),
                component: "Form".into(),
                payload_bytes: 16,
            });
        }
        counter.end_frame();

        let report = counter.frame_report();
        assert!(report.contains("Total calls: 10"));
        assert!(report.contains("setAttribute"));
        assert!(report.contains("Form"));
    }

    #[test]
    fn test_frame_report_warning() {
        let mut counter = BridgeCounter::new();
        counter.enable();

        for _ in 0..60 {
            counter.record(BridgeCall {
                op: "setAttribute".into(),
                component: "Form".into(),
                payload_bytes: 8,
            });
        }
        counter.end_frame();

        let report = counter.frame_report();
        assert!(report.contains("Warning"));
        assert!(report.contains("begin_batch"));
    }

    #[test]
    fn test_last_n_frames() {
        let mut counter = BridgeCounter::new();
        counter.enable();

        for _ in 0..5 {
            counter.record(BridgeCall {
                op: "op".into(),
                component: "C".into(),
                payload_bytes: 1,
            });
            counter.end_frame();
        }

        let last = counter.last_n_frames(3);
        assert_eq!(last.len(), 3);
    }

    #[test]
    fn test_bridge_counter_script() {
        let script = bridge_counter_script();
        assert!(script.contains("__rye_bridge_counter"));
        assert!(script.contains("endFrame"));
        assert!(script.contains("record"));
    }
}
