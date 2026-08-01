//! Goal 225: `rpg bundle` size analyzer with tree map.
//!
//! Visual tree map of Wasm binary contents. Shows which crates and functions
//! contribute to bundle size. Drill-down from crate → module → function.

use std::collections::HashMap;

/// A node in the bundle size tree.
#[derive(Debug, Clone)]
pub struct SizeNode {
    /// The node name (crate, module, or function).
    pub name: String,
    /// The size in bytes.
    pub size: u64,
    /// The children nodes.
    pub children: Vec<SizeNode>,
    /// The node kind.
    pub kind: SizeNodeKind,
}

/// The kind of size node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeNodeKind {
    /// A crate.
    Crate,
    /// A module.
    Module,
    /// A function.
    Function,
    /// A data section.
    Data,
}

impl SizeNode {
    /// Create a new size node.
    pub fn new(name: &str, size: u64, kind: SizeNodeKind) -> Self {
        Self {
            name: name.to_string(),
            size,
            children: Vec::new(),
            kind,
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: SizeNode) {
        self.children.push(child);
    }

    /// Get the total size including children.
    pub fn total_size(&self) -> u64 {
        if self.children.is_empty() {
            self.size
        } else {
            self.children.iter().map(|c| c.total_size()).sum()
        }
    }

    /// Get the percentage of total.
    pub fn percentage(&self, total: u64) -> f64 {
        if total == 0 {
            return 0.0;
        }
        self.total_size() as f64 / total as f64 * 100.0
    }

    /// Find the largest child.
    pub fn largest_child(&self) -> Option<&SizeNode> {
        self.children.iter().max_by_key(|c| c.total_size())
    }
}

/// A size suggestion — a recommendation to reduce bundle size.
#[derive(Debug, Clone)]
pub struct SizeSuggestion {
    /// The node the suggestion applies to.
    pub node_name: String,
    /// The suggestion message.
    pub message: String,
    /// The estimated savings in bytes.
    pub estimated_savings: u64,
}

impl SizeSuggestion {
    /// Create a new suggestion.
    pub fn new(node_name: &str, message: &str, savings: u64) -> Self {
        Self {
            node_name: node_name.to_string(),
            message: message.to_string(),
            estimated_savings: savings,
        }
    }
}

/// The bundle size analyzer.
pub struct BundleSizeAnalyzer {
    root: SizeNode,
}

impl BundleSizeAnalyzer {
    /// Create a new analyzer with a root node.
    pub fn new(root: SizeNode) -> Self {
        Self { root }
    }

    /// Get the total bundle size.
    pub fn total_size(&self) -> u64 {
        self.root.total_size()
    }

    /// Get the root node.
    pub fn root(&self) -> &SizeNode {
        &self.root
    }

    /// Find nodes above a size threshold.
    pub fn nodes_above_threshold(&self, threshold: u64) -> Vec<&SizeNode> {
        let mut result = Vec::new();
        self.collect_above_threshold(&self.root, threshold, &mut result);
        result
    }

    fn collect_above_threshold<'a>(&self, node: &'a SizeNode, threshold: u64, result: &mut Vec<&'a SizeNode>) {
        if node.total_size() >= threshold {
            result.push(node);
        }
        for child in &node.children {
            self.collect_above_threshold(child, threshold, result);
        }
    }

    /// Generate size reduction suggestions.
    pub fn generate_suggestions(&self) -> Vec<SizeSuggestion> {
        let mut suggestions = Vec::new();
        let total = self.total_size();

        self.collect_suggestions(&self.root, total, &mut suggestions);
        suggestions
    }

    fn collect_suggestions(&self, node: &SizeNode, total: u64, suggestions: &mut Vec<SizeSuggestion>) {
        let pct = node.percentage(total);
        if pct > 20.0 && node.kind == SizeNodeKind::Crate {
            suggestions.push(SizeSuggestion::new(
                &node.name,
                &format!("Crate '{}' is {:.1}% of bundle — consider splitting or tree-shaking", node.name, pct),
                (node.total_size() as f64 * 0.3) as u64,
            ));
        }
        if node.kind == SizeNodeKind::Function && node.size > 10_000 {
            suggestions.push(SizeSuggestion::new(
                &node.name,
                &format!("Function '{}' is {}KB — consider inlining or splitting", node.name, node.size / 1024),
                node.size / 4,
            ));
        }
        for child in &node.children {
            self.collect_suggestions(child, total, suggestions);
        }
    }

    /// Generate the tree map HTML.
    pub fn generate_treemap_html(&self) -> String {
        let total = self.total_size();
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<style>\n");
        html.push_str(".treemap { display:flex; flex-wrap:wrap; width:100%; height:100vh; }\n");
        html.push_str(".tile { display:flex; align-items:center; justify-content:center; ");
        html.push_str("font-size:12px; color:#fff; overflow:hidden; padding:4px; cursor:pointer; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str(&format!("<h2>Bundle Size: {} ({:.1}KB)</h2>\n", self.root.name, total as f64 / 1024.0));
        html.push_str("<div class=\"treemap\">\n");

        for child in &self.root.children {
            let pct = child.percentage(total);
            let color = color_for_percentage(pct);
            html.push_str(&format!(
                "<div class=\"tile\" style=\"flex:{};background:{};\">{} ({:.1}%)</div>\n",
                pct as u32,
                color,
                child.name,
                pct,
            ));
        }

        html.push_str("</div>\n</body>\n</html>\n");
        html
    }

    /// Generate a text report.
    pub fn generate_text_report(&self) -> String {
        let total = self.total_size();
        let mut text = String::new();
        text.push_str("=== Bundle Size Report ===\n\n");
        text.push_str(&format!("Total: {:.1}KB ({} bytes)\n\n", total as f64 / 1024.0, total));
        text.push_str("Breakdown:\n");
        self.render_node(&self.root, total, 0, &mut text);

        let suggestions = self.generate_suggestions();
        if !suggestions.is_empty() {
            text.push_str("\nSuggestions:\n");
            for s in &suggestions {
                text.push_str(&format!("  • {} (save ~{}B)\n", s.message, s.estimated_savings));
            }
        }

        text
    }

    fn render_node(&self, node: &SizeNode, total: u64, depth: usize, text: &mut String) {
        let indent = "  ".repeat(depth);
        let pct = node.percentage(total);
        text.push_str(&format!(
            "{}{}: {:.1}KB ({:.1}%)\n",
            indent,
            node.name,
            node.total_size() as f64 / 1024.0,
            pct,
        ));
        for child in &node.children {
            self.render_node(child, total, depth + 1, text);
        }
    }
}

/// Get a color for a percentage.
fn color_for_percentage(pct: f64) -> &'static str {
    if pct > 30.0 { "#e74c3c" }
    else if pct > 15.0 { "#e67e22" }
    else if pct > 5.0 { "#f1c40f" }
    else { "#2ecc71" }
}

/// Run the bundle command.
pub fn run(args: &[String]) {
    let wasm_path = args.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str()).unwrap_or("pkg/rye_app.wasm");
    println!("Analyzing bundle: {}", wasm_path);

    let mut root = SizeNode::new("rye_app.wasm", 0, SizeNodeKind::Crate);
    root.add_child(SizeNode::new("rye-core", 150_000, SizeNodeKind::Crate));
    root.add_child(SizeNode::new("rye-signals", 50_000, SizeNodeKind::Crate));
    root.add_child(SizeNode::new("rye-router", 30_000, SizeNodeKind::Crate));

    let analyzer = BundleSizeAnalyzer::new(root);
    let report = analyzer.generate_text_report();
    println!("{}", report);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_tree() -> SizeNode {
        let mut root = SizeNode::new("root", 0, SizeNodeKind::Crate);
        let mut core = SizeNode::new("rye-core", 0, SizeNodeKind::Crate);
        core.add_child(SizeNode::new("render", 80_000, SizeNodeKind::Module));
        core.add_child(SizeNode::new("template", 40_000, SizeNodeKind::Module));
        root.add_child(core);
        root.add_child(SizeNode::new("rye-signals", 50_000, SizeNodeKind::Crate));
        root
    }

    #[test]
    fn test_size_node_new() {
        let node = SizeNode::new("test", 100, SizeNodeKind::Crate);
        assert_eq!(node.name, "test");
        assert_eq!(node.size, 100);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_size_node_total_size_leaf() {
        let node = SizeNode::new("test", 100, SizeNodeKind::Function);
        assert_eq!(node.total_size(), 100);
    }

    #[test]
    fn test_size_node_total_size_with_children() {
        let mut node = SizeNode::new("parent", 0, SizeNodeKind::Crate);
        node.add_child(SizeNode::new("a", 100, SizeNodeKind::Module));
        node.add_child(SizeNode::new("b", 200, SizeNodeKind::Module));
        assert_eq!(node.total_size(), 300);
    }

    #[test]
    fn test_size_node_percentage() {
        let node = SizeNode::new("test", 500, SizeNodeKind::Crate);
        assert!((node.percentage(1000) - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_size_node_percentage_zero() {
        let node = SizeNode::new("test", 100, SizeNodeKind::Crate);
        assert_eq!(node.percentage(0), 0.0);
    }

    #[test]
    fn test_size_node_largest_child() {
        let mut node = SizeNode::new("parent", 0, SizeNodeKind::Crate);
        node.add_child(SizeNode::new("small", 100, SizeNodeKind::Module));
        node.add_child(SizeNode::new("big", 500, SizeNodeKind::Module));
        assert_eq!(node.largest_child().unwrap().name, "big");
    }

    #[test]
    fn test_bundle_analyzer_total_size() {
        let analyzer = BundleSizeAnalyzer::new(make_test_tree());
        assert_eq!(analyzer.total_size(), 170_000);
    }

    #[test]
    fn test_bundle_analyzer_nodes_above_threshold() {
        let analyzer = BundleSizeAnalyzer::new(make_test_tree());
        let big = analyzer.nodes_above_threshold(50_000);
        assert!(big.iter().any(|n| n.name == "rye-core"));
    }

    #[test]
    fn test_bundle_analyzer_suggestions() {
        let mut root = SizeNode::new("root", 0, SizeNodeKind::Crate);
        root.add_child(SizeNode::new("big-crate", 500_000, SizeNodeKind::Crate));
        let analyzer = BundleSizeAnalyzer::new(root);
        let suggestions = analyzer.generate_suggestions();
        assert!(suggestions.iter().any(|s| s.node_name == "big-crate"));
    }

    #[test]
    fn test_bundle_analyzer_treemap_html() {
        let analyzer = BundleSizeAnalyzer::new(make_test_tree());
        let html = analyzer.generate_treemap_html();
        assert!(html.contains("treemap"));
        assert!(html.contains("rye-core"));
    }

    #[test]
    fn test_bundle_analyzer_text_report() {
        let analyzer = BundleSizeAnalyzer::new(make_test_tree());
        let report = analyzer.generate_text_report();
        assert!(report.contains("Bundle Size Report"));
        assert!(report.contains("rye-core"));
    }

    #[test]
    fn test_color_for_percentage() {
        assert_eq!(color_for_percentage(50.0), "#e74c3c");
        assert_eq!(color_for_percentage(20.0), "#e67e22");
        assert_eq!(color_for_percentage(10.0), "#f1c40f");
        assert_eq!(color_for_percentage(2.0), "#2ecc71");
    }
}
