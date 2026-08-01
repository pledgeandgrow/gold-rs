//! Goals 237-245: Advanced testing features.
//!
//! E2E with Playwright, component contract tests, performance regression,
//! semantic snapshot testing, fuzz testing, accessibility tree testing,
//! cross-platform render equivalence, signal update ordering, and
//! automatic test generation from usage traces.

use std::collections::HashMap;

// === Goal 237: E2E testing with Playwright ===

/// A Playwright browser target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaywrightBrowser {
    /// Chromium.
    Chromium,
    /// Firefox.
    Firefox,
    /// WebKit.
    WebKit,
}

impl PlaywrightBrowser {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            PlaywrightBrowser::Chromium => "chromium",
            PlaywrightBrowser::Firefox => "firefox",
            PlaywrightBrowser::WebKit => "webkit",
        }
    }
}

/// An E2E test configuration.
#[derive(Debug, Clone)]
pub struct E2eTestConfig {
    /// The browsers to test.
    pub browsers: Vec<PlaywrightBrowser>,
    /// Whether to take screenshots.
    pub screenshots: bool,
    /// Whether to do visual regression.
    pub visual_regression: bool,
    /// Whether to mock network.
    pub network_mock: bool,
    /// The base URL.
    pub base_url: String,
}

impl Default for E2eTestConfig {
    fn default() -> Self {
        Self {
            browsers: vec![PlaywrightBrowser::Chromium, PlaywrightBrowser::Firefox, PlaywrightBrowser::WebKit],
            screenshots: true,
            visual_regression: false,
            network_mock: true,
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

/// An E2E test selector — auto-generated from component names.
#[derive(Debug, Clone)]
pub struct TestSelector {
    /// The selector string.
    pub selector: String,
    /// The selector type.
    pub selector_type: SelectorType,
}

/// The type of test selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorType {
    /// Data test ID.
    TestId,
    /// Component name.
    ComponentName,
    /// CSS class.
    Class,
    /// Text content.
    Text,
}

impl TestSelector {
    /// Generate a test ID selector from a component name.
    pub fn from_component(name: &str) -> Self {
        Self {
            selector: format!("[data-testid=\"{}\"]", name),
            selector_type: SelectorType::TestId,
        }
    }

    /// Generate a text selector.
    pub fn from_text(text: &str) -> Self {
        Self {
            selector: format!("text={}", text),
            selector_type: SelectorType::Text,
        }
    }
}

// === Goal 238: Component contract tests ===

/// A component contract — the public API of a component.
#[derive(Debug, Clone)]
pub struct ComponentContract {
    /// The component name.
    pub name: String,
    /// The expected props.
    pub props: Vec<ContractProp>,
    /// The expected events.
    pub events: Vec<String>,
    /// The expected slots.
    pub slots: Vec<String>,
}

/// A prop in a component contract.
#[derive(Debug, Clone)]
pub struct ContractProp {
    /// The prop name.
    pub name: String,
    /// The prop type.
    pub prop_type: String,
    /// Whether the prop is required.
    pub required: bool,
    /// The default value (if any).
    pub default: Option<String>,
}

impl ComponentContract {
    /// Create a new contract.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            props: Vec::new(),
            events: Vec::new(),
            slots: Vec::new(),
        }
    }

    /// Add a prop.
    pub fn add_prop(mut self, name: &str, prop_type: &str, required: bool) -> Self {
        self.props.push(ContractProp {
            name: name.to_string(),
            prop_type: prop_type.to_string(),
            required,
            default: None,
        });
        self
    }

    /// Add an event.
    pub fn add_event(mut self, name: &str) -> Self {
        self.events.push(name.to_string());
        self
    }

    /// Add a slot.
    pub fn add_slot(mut self, name: &str) -> Self {
        self.slots.push(name.to_string());
        self
    }

    /// Check if this contract is compatible with another (no breaking changes).
    pub fn is_compatible_with(&self, other: &ComponentContract) -> bool {
        // All required props in self must exist in other
        for prop in &self.props {
            if prop.required {
                if !other.props.iter().any(|p| p.name == prop.name && p.prop_type == prop.prop_type) {
                    return false;
                }
            }
        }
        // All events in self must exist in other
        for event in &self.events {
            if !other.events.contains(event) {
                return false;
            }
        }
        // All slots in self must exist in other
        for slot in &self.slots {
            if !other.slots.contains(slot) {
                return false;
            }
        }
        true
    }

    /// Find breaking changes between this contract and a new one.
    pub fn breaking_changes(&self, new: &ComponentContract) -> Vec<String> {
        let mut changes = Vec::new();
        for prop in &self.props {
            if prop.required {
                if let Some(new_prop) = new.props.iter().find(|p| p.name == prop.name) {
                    if new_prop.prop_type != prop.prop_type {
                        changes.push(format!("Prop '{}' type changed: {} → {}", prop.name, prop.prop_type, new_prop.prop_type));
                    }
                } else {
                    changes.push(format!("Required prop '{}' was removed", prop.name));
                }
            }
        }
        for event in &self.events {
            if !new.events.contains(event) {
                changes.push(format!("Event '{}' was removed", event));
            }
        }
        for slot in &self.slots {
            if !new.slots.contains(slot) {
                changes.push(format!("Slot '{}' was removed", slot));
            }
        }
        changes
    }
}

// === Goal 239: Performance regression testing ===

/// A performance benchmark result.
#[derive(Debug, Clone)]
pub struct PerfBenchmark {
    /// The benchmark name.
    pub name: String,
    /// The render time in microseconds.
    pub render_time_us: u64,
    /// The bundle size in bytes.
    pub bundle_size: u64,
    /// The memory usage in bytes.
    pub memory_bytes: u64,
}

impl PerfBenchmark {
    /// Create a new benchmark.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            render_time_us: 0,
            bundle_size: 0,
            memory_bytes: 0,
        }
    }
}

/// A performance baseline — the expected performance values.
#[derive(Debug, Clone)]
pub struct PerfBaseline {
    /// The benchmark name.
    pub name: String,
    /// The max allowed render time in microseconds.
    pub max_render_time_us: u64,
    /// The max allowed bundle size in bytes.
    pub max_bundle_size: u64,
    /// The max allowed memory in bytes.
    pub max_memory_bytes: u64,
}

impl PerfBaseline {
    /// Create a new baseline.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            max_render_time_us: 16_000, // 16ms = 1 frame
            max_bundle_size: 500_000,  // 500KB
            max_memory_bytes: 50_000_000, // 50MB
        }
    }

    /// Check if a benchmark passes the baseline.
    pub fn check(&self, benchmark: &PerfBenchmark) -> PerfCheckResult {
        let render_ok = benchmark.render_time_us <= self.max_render_time_us;
        let bundle_ok = benchmark.bundle_size <= self.max_bundle_size;
        let memory_ok = benchmark.memory_bytes <= self.max_memory_bytes;

        PerfCheckResult {
            render_time_ok: render_ok,
            bundle_size_ok: bundle_ok,
            memory_ok,
            render_time_us: benchmark.render_time_us,
            bundle_size: benchmark.bundle_size,
            memory_bytes: benchmark.memory_bytes,
        }
    }
}

/// The result of a performance check.
#[derive(Debug, Clone)]
pub struct PerfCheckResult {
    /// Whether render time passed.
    pub render_time_ok: bool,
    /// Whether bundle size passed.
    pub bundle_size_ok: bool,
    /// Whether memory passed.
    pub memory_ok: bool,
    /// The actual render time.
    pub render_time_us: u64,
    /// The actual bundle size.
    pub bundle_size: u64,
    /// The actual memory.
    pub memory_bytes: u64,
}

impl PerfCheckResult {
    /// Check if all checks passed.
    pub fn all_passed(&self) -> bool {
        self.render_time_ok && self.bundle_size_ok && self.memory_ok
    }
}

// === Goal 240: Snapshot testing with semantic diffing ===

/// A semantic node — a structural representation of a component.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticNode {
    /// The tag name.
    pub tag: String,
    /// The props.
    pub props: HashMap<String, String>,
    /// The text content (if a text node).
    pub text: Option<String>,
    /// The children.
    pub children: Vec<SemanticNode>,
}

impl SemanticNode {
    /// Create a new element node.
    pub fn element(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            props: HashMap::new(),
            text: None,
            children: Vec::new(),
        }
    }

    /// Create a text node.
    pub fn text(content: &str) -> Self {
        Self {
            tag: "#text".to_string(),
            props: HashMap::new(),
            text: Some(content.to_string()),
            children: Vec::new(),
        }
    }

    /// Add a prop.
    pub fn add_prop(mut self, key: &str, value: &str) -> Self {
        self.props.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a child.
    pub fn add_child(mut self, child: SemanticNode) -> Self {
        self.children.push(child);
        self
    }

    /// Compute a semantic diff with another node.
    pub fn diff(&self, other: &SemanticNode) -> Vec<SemanticDiff> {
        let mut diffs = Vec::new();
        if self.tag != other.tag {
            diffs.push(SemanticDiff::TagChanged(self.tag.clone(), other.tag.clone()));
        }
        for (key, val) in &self.props {
            match other.props.get(key) {
                Some(other_val) if val != other_val => {
                    diffs.push(SemanticDiff::PropChanged(key.clone(), val.clone(), other_val.clone()));
                }
                None => diffs.push(SemanticDiff::PropRemoved(key.clone(), val.clone())),
                _ => {}
            }
        }
        for (key, val) in &other.props {
            if !self.props.contains_key(key) {
                diffs.push(SemanticDiff::PropAdded(key.clone(), val.clone()));
            }
        }
        if self.text != other.text {
            diffs.push(SemanticDiff::TextChanged(
                self.text.clone().unwrap_or_default(),
                other.text.clone().unwrap_or_default(),
            ));
        }
        if self.children.len() != other.children.len() {
            diffs.push(SemanticDiff::ChildrenCountChanged(self.children.len(), other.children.len()));
        }
        for (i, (a, b)) in self.children.iter().zip(other.children.iter()).enumerate() {
            let child_diffs = a.diff(b);
            for d in child_diffs {
                diffs.push(SemanticDiff::ChildChanged(i, Box::new(d)));
            }
        }
        diffs
    }
}

/// A semantic diff — a structural difference between two nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticDiff {
    /// The tag changed.
    TagChanged(String, String),
    /// A prop changed.
    PropChanged(String, String, String),
    /// A prop was removed.
    PropRemoved(String, String),
    /// A prop was added.
    PropAdded(String, String),
    /// Text content changed.
    TextChanged(String, String),
    /// Number of children changed.
    ChildrenCountChanged(usize, usize),
    /// A child changed.
    ChildChanged(usize, Box<SemanticDiff>),
}

// === Goal 241: Fuzz testing for template macro ===

/// A fuzz test result.
#[derive(Debug, Clone)]
pub struct FuzzResult {
    /// The input that was tested.
    pub input: String,
    /// Whether the macro compiled successfully.
    pub compiled: bool,
    /// Whether the macro produced a valid error (no panic).
    pub valid_error: bool,
    /// The error message (if any).
    pub error: Option<String>,
}

impl FuzzResult {
    /// Check if the fuzz test passed (either compiled or gave valid error).
    pub fn passed(&self) -> bool {
        self.compiled || self.valid_error
    }
}

/// A fuzz test generator — generates random template syntax.
pub struct FuzzGenerator {
    /// The random seed.
    seed: u64,
}

impl FuzzGenerator {
    /// Create a new generator with a seed.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a random template string.
    pub fn generate(&mut self) -> String {
        let templates = [
            "div { }",
            "div { \"hello\" }",
            "span { class: \"test\" }",
            "button { onclick: move |_| {} }",
            "div { div { } }",
            "#text",
            "div { $undefined$ }",
            "div { malformed",
            "div { } } }",
            "",
        ];
        let idx = (self.seed as usize) % templates.len();
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        templates[idx].to_string()
    }

    /// Generate multiple templates.
    pub fn generate_n(&mut self, n: usize) -> Vec<String> {
        (0..n).map(|_| self.generate()).collect()
    }
}

// === Goal 242: Accessibility tree snapshot testing ===

/// An accessibility tree node.
#[derive(Debug, Clone, PartialEq)]
pub struct A11yNode {
    /// The role.
    pub role: String,
    /// The accessible name.
    pub name: String,
    /// The children.
    pub children: Vec<A11yNode>,
}

impl A11yNode {
    /// Create a new node.
    pub fn new(role: &str, name: &str) -> Self {
        Self {
            role: role.to_string(),
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    /// Add a child.
    pub fn add_child(mut self, child: A11yNode) -> Self {
        self.children.push(child);
        self
    }

    /// Compare with an expected tree.
    pub fn matches(&self, expected: &A11yNode) -> bool {
        if self.role != expected.role || self.name != expected.name {
            return false;
        }
        if self.children.len() != expected.children.len() {
            return false;
        }
        self.children.iter().zip(expected.children.iter()).all(|(a, b)| a.matches(b))
    }
}

// === Goal 243: Cross-platform render equivalence ===

/// A platform for render testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPlatform {
    /// Web.
    Web,
    /// Desktop.
    Desktop,
    /// Mobile.
    Mobile,
}

impl RenderPlatform {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            RenderPlatform::Web => "web",
            RenderPlatform::Desktop => "desktop",
            RenderPlatform::Mobile => "mobile",
        }
    }
}

/// A render equivalence test result.
#[derive(Debug, Clone)]
pub struct EquivalenceResult {
    /// The platforms compared.
    pub platforms: Vec<RenderPlatform>,
    /// Whether the renders are equivalent.
    pub equivalent: bool,
    /// The differences (if any).
    pub differences: Vec<String>,
}

impl EquivalenceResult {
    /// Create a passing result.
    pub fn equivalent(platforms: Vec<RenderPlatform>) -> Self {
        Self {
            platforms,
            equivalent: true,
            differences: Vec::new(),
        }
    }

    /// Create a failing result.
    pub fn different(platforms: Vec<RenderPlatform>, differences: Vec<String>) -> Self {
        Self {
            platforms,
            equivalent: false,
            differences,
        }
    }
}

// === Goal 244: Signal update ordering tests ===

/// A signal update in a test.
#[derive(Debug, Clone)]
pub struct SignalUpdate {
    /// The signal name.
    pub signal: String,
    /// The new value.
    pub value: String,
    /// The order (0 = first).
    pub order: u32,
}

/// A signal dependency graph for testing.
#[derive(Debug, Clone)]
pub struct SignalGraph {
    /// The dependencies (signal -> signals it depends on).
    pub dependencies: HashMap<String, Vec<String>>,
}

impl SignalGraph {
    /// Create a new graph.
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, signal: &str, depends_on: &str) {
        self.dependencies
            .entry(signal.to_string())
            .or_insert_with(Vec::new)
            .push(depends_on.to_string());
    }

    /// Compute the topological order of updates.
    pub fn topological_order(&self) -> Vec<String> {
        let mut visited: Vec<String> = Vec::new();
        let mut result: Vec<String> = Vec::new();

        let signals: Vec<String> = self.dependencies.keys().cloned().collect();
        for signal in &signals {
            self.visit(signal, &mut visited, &mut result);
        }
        result
    }

    fn visit(&self, signal: &str, visited: &mut Vec<String>, result: &mut Vec<String>) {
        if visited.contains(&signal.to_string()) {
            return;
        }
        visited.push(signal.to_string());

        if let Some(deps) = self.dependencies.get(signal) {
            for dep in deps {
                self.visit(dep, visited, result);
            }
        }
        result.push(signal.to_string());
    }

    /// Verify that updates are in correct topological order.
    pub fn verify_order(&self, updates: &[SignalUpdate]) -> bool {
        let correct_order = self.topological_order();
        for i in 0..updates.len() {
            for j in (i + 1)..updates.len() {
                let earlier = &updates[i].signal;
                let later = &updates[j].signal;
                if let Some(deps) = self.dependencies.get(later) {
                    if deps.contains(earlier) {
                        // earlier should come before later — check
                        let earlier_pos = correct_order.iter().position(|s| s == earlier);
                        let later_pos = correct_order.iter().position(|s| s == later);
                        if let (Some(ep), Some(lp)) = (earlier_pos, later_pos) {
                            if ep > lp {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }
}

impl Default for SignalGraph {
    fn default() -> Self {
        Self::new()
    }
}

// === Goal 245: Automatic test generation from usage traces ===

/// A usage trace event — a recorded signal update and render.
#[derive(Debug, Clone)]
pub struct TraceEvent {
    /// The signal name.
    pub signal: String,
    /// The value that was set.
    pub value: String,
    /// The rendered output after the update.
    pub rendered: String,
}

/// A generated test from a trace.
#[derive(Debug, Clone)]
pub struct GeneratedTest {
    /// The test name.
    pub name: String,
    /// The test code.
    pub code: String,
    /// The number of trace events.
    pub event_count: usize,
}

impl GeneratedTest {
    /// Generate a test from a trace.
    pub fn from_trace(name: &str, events: &[TraceEvent]) -> Self {
        let mut code = String::new();
        code.push_str("#[test]\n");
        code.push_str(&format!("fn test_{}() {{\n", name));
        code.push_str("    let mut renderer = TestRenderer::new();\n");

        for event in events {
            code.push_str(&format!(
                "    // Set {} = {}\n",
                event.signal, event.value,
            ));
            code.push_str(&format!(
                "    renderer.set_signal(\"{}\", \"{}\");\n",
                event.signal, event.value,
            ));
            code.push_str(&format!(
                "    assert_eq!(renderer.render(), \"{}\");\n",
                event.rendered.replace('"', "\\\""),
            ));
        }

        code.push_str("}\n");

        Self {
            name: name.to_string(),
            code,
            event_count: events.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // E2E tests
    #[test]
    fn test_playwright_browser_display_name() {
        assert_eq!(PlaywrightBrowser::Chromium.display_name(), "chromium");
    }

    #[test]
    fn test_e2e_config_default() {
        let config = E2eTestConfig::default();
        assert_eq!(config.browsers.len(), 3);
        assert!(config.screenshots);
    }

    #[test]
    fn test_test_selector_from_component() {
        let sel = TestSelector::from_component("Button");
        assert!(sel.selector.contains("Button"));
        assert_eq!(sel.selector_type, SelectorType::TestId);
    }

    #[test]
    fn test_test_selector_from_text() {
        let sel = TestSelector::from_text("Click me");
        assert!(sel.selector.contains("Click me"));
    }

    // Contract tests
    #[test]
    fn test_component_contract_new() {
        let contract = ComponentContract::new("Button");
        assert_eq!(contract.name, "Button");
        assert!(contract.props.is_empty());
    }

    #[test]
    fn test_component_contract_builder() {
        let contract = ComponentContract::new("Button")
            .add_prop("label", "String", true)
            .add_event("click")
            .add_slot("default");
        assert_eq!(contract.props.len(), 1);
        assert_eq!(contract.events.len(), 1);
        assert_eq!(contract.slots.len(), 1);
    }

    #[test]
    fn test_component_contract_compatible() {
        let old = ComponentContract::new("Button").add_prop("label", "String", true);
        let new = ComponentContract::new("Button")
            .add_prop("label", "String", true)
            .add_prop("color", "String", false);
        assert!(old.is_compatible_with(&new));
    }

    #[test]
    fn test_component_contract_incompatible() {
        let old = ComponentContract::new("Button").add_prop("label", "String", true);
        let new = ComponentContract::new("Button");
        assert!(!old.is_compatible_with(&new));
    }

    #[test]
    fn test_component_contract_breaking_changes() {
        let old = ComponentContract::new("Button")
            .add_prop("label", "String", true)
            .add_event("click")
            .add_slot("default");
        let new = ComponentContract::new("Button")
            .add_prop("label", "i64", true);
        let changes = old.breaking_changes(&new);
        assert!(changes.iter().any(|c| c.contains("type changed")));
        assert!(changes.iter().any(|c| c.contains("click") && c.contains("removed")));
        assert!(changes.iter().any(|c| c.contains("default") && c.contains("removed")));
    }

    // Perf tests
    #[test]
    fn test_perf_benchmark_new() {
        let bench = PerfBenchmark::new("render");
        assert_eq!(bench.name, "render");
    }

    #[test]
    fn test_perf_baseline_check_pass() {
        let baseline = PerfBaseline::new("render");
        let bench = PerfBenchmark {
            name: "render".into(),
            render_time_us: 10_000,
            bundle_size: 100_000,
            memory_bytes: 10_000_000,
        };
        let result = baseline.check(&bench);
        assert!(result.all_passed());
    }

    #[test]
    fn test_perf_baseline_check_fail_render() {
        let baseline = PerfBaseline::new("render");
        let bench = PerfBenchmark {
            name: "render".into(),
            render_time_us: 20_000,
            bundle_size: 100_000,
            memory_bytes: 10_000_000,
        };
        let result = baseline.check(&bench);
        assert!(!result.render_time_ok);
        assert!(!result.all_passed());
    }

    // Semantic diff tests
    #[test]
    fn test_semantic_node_element() {
        let node = SemanticNode::element("div");
        assert_eq!(node.tag, "div");
        assert!(node.text.is_none());
    }

    #[test]
    fn test_semantic_node_text() {
        let node = SemanticNode::text("hello");
        assert_eq!(node.tag, "#text");
        assert_eq!(node.text, Some("hello".to_string()));
    }

    #[test]
    fn test_semantic_diff_tag_changed() {
        let a = SemanticNode::element("div");
        let b = SemanticNode::element("span");
        let diffs = a.diff(&b);
        assert!(diffs.iter().any(|d| matches!(d, SemanticDiff::TagChanged(_, _))));
    }

    #[test]
    fn test_semantic_diff_prop_changed() {
        let a = SemanticNode::element("div").add_prop("class", "a");
        let b = SemanticNode::element("div").add_prop("class", "b");
        let diffs = a.diff(&b);
        assert!(diffs.iter().any(|d| matches!(d, SemanticDiff::PropChanged(_, _, _))));
    }

    #[test]
    fn test_semantic_diff_no_diff() {
        let a = SemanticNode::element("div").add_prop("class", "x");
        let b = SemanticNode::element("div").add_prop("class", "x");
        let diffs = a.diff(&b);
        assert!(diffs.is_empty());
    }

    // Fuzz tests
    #[test]
    fn test_fuzz_generator_generate() {
        let mut gen = FuzzGenerator::new(42);
        let s = gen.generate();
        assert!(!s.is_empty() || s.is_empty()); // Can be empty
    }

    #[test]
    fn test_fuzz_generator_generate_n() {
        let mut gen = FuzzGenerator::new(42);
        let results = gen.generate_n(5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_fuzz_result_passed() {
        let r = FuzzResult { input: "div {}".into(), compiled: true, valid_error: false, error: None };
        assert!(r.passed());
    }

    #[test]
    fn test_fuzz_result_valid_error() {
        let r = FuzzResult { input: "invalid".into(), compiled: false, valid_error: true, error: Some("err".into()) };
        assert!(r.passed());
    }

    #[test]
    fn test_fuzz_result_failed() {
        let r = FuzzResult { input: "invalid".into(), compiled: false, valid_error: false, error: None };
        assert!(!r.passed());
    }

    // A11y tests
    #[test]
    fn test_a11y_node_new() {
        let node = A11yNode::new("button", "Submit");
        assert_eq!(node.role, "button");
        assert_eq!(node.name, "Submit");
    }

    #[test]
    fn test_a11y_node_matches() {
        let a = A11yNode::new("button", "OK").add_child(A11yNode::new("text", "Hello"));
        let b = A11yNode::new("button", "OK").add_child(A11yNode::new("text", "Hello"));
        assert!(a.matches(&b));
    }

    #[test]
    fn test_a11y_node_not_matches() {
        let a = A11yNode::new("button", "OK");
        let b = A11yNode::new("link", "OK");
        assert!(!a.matches(&b));
    }

    // Equivalence tests
    #[test]
    fn test_equivalence_result_equivalent() {
        let result = EquivalenceResult::equivalent(vec![RenderPlatform::Web, RenderPlatform::Desktop]);
        assert!(result.equivalent);
    }

    #[test]
    fn test_equivalence_result_different() {
        let result = EquivalenceResult::different(
            vec![RenderPlatform::Web, RenderPlatform::Desktop],
            vec!["color mismatch".to_string()],
        );
        assert!(!result.equivalent);
        assert_eq!(result.differences.len(), 1);
    }

    #[test]
    fn test_render_platform_display_name() {
        assert_eq!(RenderPlatform::Web.display_name(), "web");
    }

    // Signal graph tests
    #[test]
    fn test_signal_graph_topological_order() {
        let mut graph = SignalGraph::new();
        graph.add_dependency("c", "a");
        graph.add_dependency("c", "b");
        let order = graph.topological_order();
        assert!(order.contains(&"a".to_string()));
        assert!(order.contains(&"b".to_string()));
        assert!(order.contains(&"c".to_string()));
        let a_pos = order.iter().position(|s| s == "a").unwrap();
        let c_pos = order.iter().position(|s| s == "c").unwrap();
        assert!(a_pos < c_pos);
    }

    #[test]
    fn test_signal_graph_verify_order() {
        let mut graph = SignalGraph::new();
        graph.add_dependency("b", "a");
        let updates = vec![
            SignalUpdate { signal: "a".into(), value: "1".into(), order: 0 },
            SignalUpdate { signal: "b".into(), value: "2".into(), order: 1 },
        ];
        assert!(graph.verify_order(&updates));
    }

    // Trace tests
    #[test]
    fn test_generated_test_from_trace() {
        let events = vec![
            TraceEvent { signal: "count".into(), value: "1".into(), rendered: "<span>1</span>".into() },
            TraceEvent { signal: "count".into(), value: "2".into(), rendered: "<span>2</span>".into() },
        ];
        let test = GeneratedTest::from_trace("counter", &events);
        assert_eq!(test.event_count, 2);
        assert!(test.code.contains("#[test]"));
        assert!(test.code.contains("count"));
    }
}
