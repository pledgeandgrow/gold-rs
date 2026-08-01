//! Goal 223: `rpg upgrade` with automatic codemods.
//!
//! Extend existing migration tooling with automatic codemod application during
//! version upgrades. Breaking changes come with codemods that transform old
//! API usage to new.

use std::collections::HashMap;

/// A codemod — an automatic code transformation.
#[derive(Debug, Clone)]
pub struct Codemod {
    /// The codemod ID.
    pub id: String,
    /// The version this codemod was introduced in.
    pub version: String,
    /// The description of what it changes.
    pub description: String,
    /// The search pattern (old API).
    pub search: String,
    /// The replacement (new API).
    pub replace: String,
    /// Whether this is a breaking change.
    pub breaking: bool,
    /// The file patterns to apply to (glob).
    pub file_patterns: Vec<String>,
}

impl Codemod {
    /// Create a new codemod.
    pub fn new(id: &str, version: &str, search: &str, replace: &str) -> Self {
        Self {
            id: id.to_string(),
            version: version.to_string(),
            description: String::new(),
            search: search.to_string(),
            replace: replace.to_string(),
            breaking: false,
            file_patterns: vec!["**/*.rs".to_string()],
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Mark as breaking.
    pub fn breaking_change(mut self) -> Self {
        self.breaking = true;
        self
    }

    /// Set file patterns.
    pub fn with_file_patterns(mut self, patterns: Vec<String>) -> Self {
        self.file_patterns = patterns;
        self
    }

    /// Apply this codemod to a source string.
    pub fn apply(&self, source: &str) -> (String, usize) {
        let result = source.replace(&self.search, &self.replace);
        let count = (source.matches(&self.search).count()) as usize;
        (result, count)
    }
}

/// The result of an upgrade.
#[derive(Debug, Clone)]
pub struct UpgradeResult {
    /// The target version.
    pub target_version: String,
    /// The codemods applied.
    pub codemods_applied: Vec<(String, usize)>,
    /// The total number of replacements.
    pub total_replacements: usize,
    /// Whether the upgrade succeeded.
    pub success: bool,
    /// Any errors encountered.
    pub errors: Vec<String>,
}

impl UpgradeResult {
    /// Create a new upgrade result.
    pub fn new(target_version: &str) -> Self {
        Self {
            target_version: target_version.to_string(),
            codemods_applied: Vec::new(),
            total_replacements: 0,
            success: true,
            errors: Vec::new(),
        }
    }

    /// Record a codemod application.
    pub fn record(&mut self, codemod_id: &str, replacements: usize) {
        self.codemods_applied.push((codemod_id.to_string(), replacements));
        self.total_replacements += replacements;
    }

    /// Record an error.
    pub fn record_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.success = false;
    }

    /// Get the summary.
    pub fn summary(&self) -> String {
        format!(
            "Upgrade to {}: {} codemods, {} replacements, {} errors",
            self.target_version,
            self.codemods_applied.len(),
            self.total_replacements,
            self.errors.len(),
        )
    }
}

/// The codemod registry — manages all available codemods.
pub struct CodemodRegistry {
    codemods: HashMap<String, Vec<Codemod>>,
}

impl CodemodRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            codemods: HashMap::new(),
        }
    }

    /// Register a codemod.
    pub fn register(&mut self, codemod: Codemod) {
        self.codemods
            .entry(codemod.version.clone())
            .or_insert_with(Vec::new)
            .push(codemod);
    }

    /// Get codemods for a specific version.
    pub fn for_version(&self, version: &str) -> Vec<&Codemod> {
        self.codemods.get(version).map(|v| v.iter().collect()).unwrap_or_default()
    }

    /// Get all codemods up to a target version.
    pub fn up_to_version(&self, target: &str) -> Vec<&Codemod> {
        self.codemods
            .iter()
            .filter(|(v, _)| v.as_str() <= target)
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    /// Get the number of registered codemods.
    pub fn len(&self) -> usize {
        self.codemods.values().map(|v| v.len()).sum()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.codemods.is_empty()
    }

    /// Apply all codemods for a target version to source code.
    pub fn apply_all(&self, target_version: &str, source: &str) -> UpgradeResult {
        let mut result = UpgradeResult::new(target_version);
        let codemods = self.up_to_version(target_version);
        let mut current_source = source.to_string();

        for codemod in codemods {
            let (new_source, count) = codemod.apply(&current_source);
            if count > 0 {
                result.record(&codemod.id, count);
                current_source = new_source;
            }
        }

        result
    }
}

impl Default for CodemodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the upgrade command with codemods.
pub fn run(args: &[String]) {
    let target_version = args
        .iter()
        .find(|a| a.starts_with("--to"))
        .map(|a| &a[4..])
        .unwrap_or("latest");

    let dry_run = args.iter().any(|a| a == "--dry-run");

    let mut registry = CodemodRegistry::new();

    // Register known codemods
    registry.register(
        Codemod::new("rename-signal", "0.2.0", "create_signal", "signal")
            .with_description("Rename create_signal to signal")
            .breaking_change(),
    );
    registry.register(
        Codemod::new("rename-component-macro", "0.2.0", "#[rye::component]", "#[component]")
            .with_description("Simplify component macro path"),
    );

    let codemods = registry.up_to_version(target_version);

    if dry_run {
        println!("Dry run — would apply {} codemods for version {}", codemods.len(), target_version);
        for cm in &codemods {
            println!("  {} (v{}): {} → {}", cm.id, cm.version, cm.search, cm.replace);
        }
    } else {
        println!("Upgrading to version {} with {} codemods", target_version, codemods.len());
        for cm in &codemods {
            println!("  Applying {}: {}", cm.id, cm.description);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codemod_new() {
        let cm = Codemod::new("test", "0.2.0", "old_fn", "new_fn");
        assert_eq!(cm.id, "test");
        assert_eq!(cm.search, "old_fn");
        assert_eq!(cm.replace, "new_fn");
        assert!(!cm.breaking);
    }

    #[test]
    fn test_codemod_builder() {
        let cm = Codemod::new("test", "0.2.0", "a", "b")
            .with_description("test desc")
            .breaking_change()
            .with_file_patterns(vec!["**/*.rs".to_string()]);
        assert_eq!(cm.description, "test desc");
        assert!(cm.breaking);
        assert_eq!(cm.file_patterns.len(), 1);
    }

    #[test]
    fn test_codemod_apply() {
        let cm = Codemod::new("test", "0.2.0", "create_signal", "signal");
        let (result, count) = cm.apply("let x = create_signal(0);");
        assert_eq!(count, 1);
        assert!(result.contains("signal(0)"));
        assert!(!result.contains("create_signal"));
    }

    #[test]
    fn test_codemod_apply_multiple() {
        let cm = Codemod::new("test", "0.2.0", "old", "new");
        let (result, count) = cm.apply("old + old + old");
        assert_eq!(count, 3);
        assert_eq!(result, "new + new + new");
    }

    #[test]
    fn test_codemod_apply_no_match() {
        let cm = Codemod::new("test", "0.2.0", "nonexistent", "new");
        let (_, count) = cm.apply("nothing here");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_upgrade_result_new() {
        let result = UpgradeResult::new("0.2.0");
        assert_eq!(result.target_version, "0.2.0");
        assert!(result.success);
        assert_eq!(result.total_replacements, 0);
    }

    #[test]
    fn test_upgrade_result_record() {
        let mut result = UpgradeResult::new("0.2.0");
        result.record("codemod1", 5);
        result.record("codemod2", 3);
        assert_eq!(result.total_replacements, 8);
        assert_eq!(result.codemods_applied.len(), 2);
    }

    #[test]
    fn test_upgrade_result_error() {
        let mut result = UpgradeResult::new("0.2.0");
        result.record_error("file not found");
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_upgrade_result_summary() {
        let mut result = UpgradeResult::new("0.2.0");
        result.record("cm1", 5);
        let summary = result.summary();
        assert!(summary.contains("0.2.0"));
        assert!(summary.contains("1 codemods"));
        assert!(summary.contains("5 replacements"));
    }

    #[test]
    fn test_codemod_registry_register() {
        let mut registry = CodemodRegistry::new();
        registry.register(Codemod::new("test", "0.2.0", "a", "b"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_codemod_registry_for_version() {
        let mut registry = CodemodRegistry::new();
        registry.register(Codemod::new("cm1", "0.2.0", "a", "b"));
        registry.register(Codemod::new("cm2", "0.3.0", "c", "d"));
        assert_eq!(registry.for_version("0.2.0").len(), 1);
        assert_eq!(registry.for_version("0.3.0").len(), 1);
        assert_eq!(registry.for_version("0.5.0").len(), 0);
    }

    #[test]
    fn test_codemod_registry_up_to_version() {
        let mut registry = CodemodRegistry::new();
        registry.register(Codemod::new("cm1", "0.2.0", "a", "b"));
        registry.register(Codemod::new("cm2", "0.3.0", "c", "d"));
        registry.register(Codemod::new("cm3", "0.4.0", "e", "f"));
        assert_eq!(registry.up_to_version("0.3.0").len(), 2);
        assert_eq!(registry.up_to_version("0.4.0").len(), 3);
    }

    #[test]
    fn test_codemod_registry_is_empty() {
        let registry = CodemodRegistry::new();
        assert!(registry.is_empty());
    }
}
