//! Goal 148: `rye inspect` CLI command.
//!
//! Inspect a rye project: component tree, bundle size, dependencies,
//! signal graph, and performance metrics.

use std::collections::HashMap;
use std::path::PathBuf;

/// What to inspect.
#[derive(Debug, Clone, PartialEq)]
pub enum InspectTarget {
    /// Component tree.
    Components,
    /// Bundle size breakdown.
    BundleSize,
    /// Dependency graph.
    Dependencies,
    /// Signal graph.
    Signals,
    /// Performance metrics.
    Performance,
    /// All of the above.
    All,
}

impl InspectTarget {
    /// Parse from a string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "components" => Self::Components,
            "bundle" | "size" => Self::BundleSize,
            "deps" | "dependencies" => Self::Dependencies,
            "signals" => Self::Signals,
            "perf" | "performance" => Self::Performance,
            "all" => Self::All,
            _ => Self::All,
        }
    }
}

/// Component inspection info.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component name.
    pub name: String,
    /// Source file path.
    pub source: PathBuf,
    /// Number of child components.
    pub child_count: usize,
    /// Number of signals used.
    pub signal_count: usize,
    /// Whether it's an island.
    pub is_island: bool,
    /// Estimated render cost (relative).
    pub render_cost: u32,
}

/// Bundle size info.
#[derive(Debug, Clone)]
pub struct BundleSizeInfo {
    /// Total bundle size in bytes.
    pub total_bytes: usize,
    /// Wasm size.
    pub wasm_bytes: usize,
    /// JS glue size.
    pub js_bytes: usize,
    /// CSS size.
    pub css_bytes: usize,
    /// HTML size.
    pub html_bytes: usize,
    /// Per-crate breakdown.
    pub per_crate: HashMap<String, usize>,
}

impl BundleSizeInfo {
    /// Format as a human-readable string.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Bundle Size ===\n");
        out.push_str(&format!("  Total:  {}\n", format_bytes(self.total_bytes)));
        out.push_str(&format!("  Wasm:   {}\n", format_bytes(self.wasm_bytes)));
        out.push_str(&format!("  JS:     {}\n", format_bytes(self.js_bytes)));
        out.push_str(&format!("  CSS:    {}\n", format_bytes(self.css_bytes)));
        out.push_str(&format!("  HTML:   {}\n", format_bytes(self.html_bytes)));

        if !self.per_crate.is_empty() {
            out.push_str("\n  Per-crate breakdown:\n");
            let mut crates: Vec<_> = self.per_crate.iter().collect();
            crates.sort_by(|a, b| b.1.cmp(a.1));
            for (name, size) in crates {
                out.push_str(&format!("    {}: {}\n", name, format_bytes(*size)));
            }
        }

        out
    }
}

/// Format bytes as human-readable.
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Dependency info.
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Dependency name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Whether it's a direct dependency.
    pub direct: bool,
    /// Dependencies of this dependency.
    pub dependencies: Vec<String>,
}

/// Signal graph info.
#[derive(Debug, Clone)]
pub struct SignalGraphInfo {
    /// Signal name/ID.
    pub name: String,
    /// Number of subscribers.
    pub subscriber_count: usize,
    /// Whether the signal is currently active.
    pub active: bool,
    /// Dependencies (other signals this one derives from).
    pub depends_on: Vec<String>,
}

/// Performance info.
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    /// Average render time in microseconds.
    pub avg_render_us: u64,
    /// Max render time in microseconds.
    pub max_render_us: u64,
    /// Number of renders.
    pub render_count: u64,
    /// Average reconcile time in microseconds.
    pub avg_reconcile_us: u64,
    /// Number of bridge calls per frame.
    pub bridge_calls_per_frame: u32,
}

impl PerformanceInfo {
    /// Format as a human-readable string.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Performance ===\n");
        out.push_str(&format!(
            "  Avg render:    {:.2} ms\n",
            self.avg_render_us as f64 / 1000.0
        ));
        out.push_str(&format!(
            "  Max render:    {:.2} ms\n",
            self.max_render_us as f64 / 1000.0
        ));
        out.push_str(&format!("  Render count:  {}\n", self.render_count));
        out.push_str(&format!(
            "  Avg reconcile: {:.2} ms\n",
            self.avg_reconcile_us as f64 / 1000.0
        ));
        out.push_str(&format!(
            "  Bridge/frame:  {}\n",
            self.bridge_calls_per_frame
        ));
        out
    }
}

/// Full inspection report.
#[derive(Debug, Clone)]
pub struct InspectReport {
    /// Components.
    pub components: Vec<ComponentInfo>,
    /// Bundle size.
    pub bundle: Option<BundleSizeInfo>,
    /// Dependencies.
    pub dependencies: Vec<DependencyInfo>,
    /// Signal graph.
    pub signals: Vec<SignalGraphInfo>,
    /// Performance.
    pub performance: Option<PerformanceInfo>,
}

impl InspectReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            bundle: None,
            dependencies: Vec::new(),
            signals: Vec::new(),
            performance: None,
        }
    }

    /// Format the full report.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Rye Project Inspection ===\n\n");

        if !self.components.is_empty() {
            out.push_str(&format!("Components ({}):\n", self.components.len()));
            for comp in &self.components {
                let island = if comp.is_island { " [island]" } else { "" };
                out.push_str(&format!(
                    "  {}{} — {} children, {} signals, cost: {}\n",
                    comp.name, island, comp.child_count, comp.signal_count, comp.render_cost
                ));
            }
            out.push('\n');
        }

        if let Some(bundle) = &self.bundle {
            out.push_str(&bundle.format());
            out.push('\n');
        }

        if !self.dependencies.is_empty() {
            out.push_str(&format!("Dependencies ({}):\n", self.dependencies.len()));
            for dep in &self.dependencies {
                let kind = if dep.direct { "direct" } else { "transitive" };
                out.push_str(&format!("  {} {} ({})\n", dep.name, dep.version, kind));
            }
            out.push('\n');
        }

        if !self.signals.is_empty() {
            out.push_str(&format!("Signals ({}):\n", self.signals.len()));
            for sig in &self.signals {
                let active = if sig.active { "active" } else { "inactive" };
                out.push_str(&format!(
                    "  {} — {} subscribers ({})\n",
                    sig.name, sig.subscriber_count, active
                ));
            }
            out.push('\n');
        }

        if let Some(perf) = &self.performance {
            out.push_str(&perf.format());
        }

        out
    }
}

impl Default for InspectReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_target_from_str() {
        assert_eq!(
            InspectTarget::from_str("components"),
            InspectTarget::Components
        );
        assert_eq!(InspectTarget::from_str("bundle"), InspectTarget::BundleSize);
        assert_eq!(InspectTarget::from_str("deps"), InspectTarget::Dependencies);
        assert_eq!(InspectTarget::from_str("signals"), InspectTarget::Signals);
        assert_eq!(InspectTarget::from_str("perf"), InspectTarget::Performance);
        assert_eq!(InspectTarget::from_str("all"), InspectTarget::All);
        assert_eq!(InspectTarget::from_str("unknown"), InspectTarget::All);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_bundle_size_format() {
        let info = BundleSizeInfo {
            total_bytes: 1048576,
            wasm_bytes: 700000,
            js_bytes: 200000,
            css_bytes: 50000,
            html_bytes: 98576,
            per_crate: {
                let mut m = HashMap::new();
                m.insert("rye-core".to_string(), 300000);
                m.insert("rye-html".to_string(), 150000);
                m
            },
        };
        let formatted = info.format();
        assert!(formatted.contains("1.0 MB"));
        assert!(formatted.contains("Wasm"));
        assert!(formatted.contains("rye-core"));
    }

    #[test]
    fn test_performance_info_format() {
        let info = PerformanceInfo {
            avg_render_us: 1500,
            max_render_us: 5000,
            render_count: 1000,
            avg_reconcile_us: 800,
            bridge_calls_per_frame: 12,
        };
        let formatted = info.format();
        assert!(formatted.contains("1.50 ms"));
        assert!(formatted.contains("5.00 ms"));
        assert!(formatted.contains("1000"));
        assert!(formatted.contains("12"));
    }

    #[test]
    fn test_inspect_report_format() {
        let mut report = InspectReport::new();
        report.components.push(ComponentInfo {
            name: "Button".to_string(),
            source: PathBuf::from("src/button.rs"),
            child_count: 0,
            signal_count: 2,
            is_island: false,
            render_cost: 10,
        });
        report.components.push(ComponentInfo {
            name: "Dashboard".to_string(),
            source: PathBuf::from("src/dashboard.rs"),
            child_count: 5,
            signal_count: 10,
            is_island: true,
            render_cost: 80,
        });
        report.bundle = Some(BundleSizeInfo {
            total_bytes: 500000,
            wasm_bytes: 300000,
            js_bytes: 100000,
            css_bytes: 50000,
            html_bytes: 50000,
            per_crate: HashMap::new(),
        });
        report.performance = Some(PerformanceInfo {
            avg_render_us: 2000,
            max_render_us: 8000,
            render_count: 500,
            avg_reconcile_us: 1000,
            bridge_calls_per_frame: 8,
        });

        let formatted = report.format();
        assert!(formatted.contains("Button"));
        assert!(formatted.contains("[island]"));
        assert!(formatted.contains("Dashboard"));
        assert!(formatted.contains("488.3 KB"));
        assert!(formatted.contains("2.00 ms"));
    }
}
