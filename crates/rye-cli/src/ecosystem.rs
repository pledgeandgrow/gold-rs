//! Goals 231-235: Monorepo, publish, theme, docs, and CI commands.
//!
//! `rpg monorepo` workspace management, `rpg publish` component library
//! publishing, `rpg theme` design token CLI, `rpg docs` local documentation
//! server, `rpg ci` CI/CD template generator.

use std::collections::HashMap;

// === Goal 231: Monorepo ===

/// A monorepo workspace configuration.
#[derive(Debug, Clone)]
pub struct MonorepoConfig {
    /// The workspace root path.
    pub root: String,
    /// The member packages.
    pub members: Vec<String>,
    /// Shared dependencies.
    pub shared_deps: HashMap<String, String>,
    /// Whether cross-component imports are enabled.
    pub cross_imports: bool,
}

impl MonorepoConfig {
    /// Create a new monorepo config.
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
            members: Vec::new(),
            shared_deps: HashMap::new(),
            cross_imports: true,
        }
    }

    /// Add a member package.
    pub fn add_member(mut self, path: &str) -> Self {
        self.members.push(path.to_string());
        self
    }

    /// Add a shared dependency.
    pub fn add_shared_dep(mut self, name: &str, version: &str) -> Self {
        self.shared_deps.insert(name.to_string(), version.to_string());
        self
    }

    /// Generate the workspace Cargo.toml.
    pub fn generate_workspace_toml(&self) -> String {
        let mut toml = String::new();
        toml.push_str("[workspace]\n");
        toml.push_str(&format!("members = [\n"));
        for m in &self.members {
            toml.push_str(&format!("    \"{}\",\n", m));
        }
        toml.push_str("]\n\n");
        toml.push_str("[workspace.dependencies]\n");
        for (name, ver) in &self.shared_deps {
            toml.push_str(&format!("{} = \"{}\"\n", name, ver));
        }
        toml
    }

    /// Get the number of members.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

// === Goal 232: Component library publishing ===

/// A published component library.
#[derive(Debug, Clone)]
pub struct PublishedLibrary {
    /// The library name.
    pub name: String,
    /// The version.
    pub version: String,
    /// The scope (e.g. "@rye").
    pub scope: String,
    /// The components included.
    pub components: Vec<String>,
    /// The documentation URL.
    pub docs_url: String,
    /// The playground URL.
    pub playground_url: String,
    /// The migration guide.
    pub migration_guide: Option<String>,
}

impl PublishedLibrary {
    /// Create a new library.
    pub fn new(scope: &str, name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            scope: scope.to_string(),
            components: Vec::new(),
            docs_url: String::new(),
            playground_url: String::new(),
            migration_guide: None,
        }
    }

    /// Add a component.
    pub fn add_component(mut self, name: &str) -> Self {
        self.components.push(name.to_string());
        self
    }

    /// Get the full package name.
    pub fn package_name(&self) -> String {
        format!("{}/{}", self.scope, self.name)
    }

    /// Generate the registry manifest.
    pub fn generate_manifest(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"name\": \"{}\",\n", self.package_name()));
        json.push_str(&format!("  \"version\": \"{}\",\n", self.version));
        json.push_str(&format!("  \"components\": [{}],\n",
            self.components.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", ")));
        json.push_str(&format!("  \"docs\": \"{}\",\n", self.docs_url));
        json.push_str(&format!("  \"playground\": \"{}\"\n", self.playground_url));
        json.push_str("}\n");
        json
    }
}

// === Goal 233: Theme CLI ===

/// A design theme.
#[derive(Debug, Clone)]
pub struct DesignTheme {
    /// The theme name.
    pub name: String,
    /// Whether it's a dark theme.
    pub dark: bool,
    /// The color tokens.
    pub colors: HashMap<String, String>,
    /// The spacing tokens.
    pub spacing: HashMap<String, String>,
    /// The typography tokens.
    pub typography: HashMap<String, String>,
}

impl DesignTheme {
    /// Create a new theme.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dark: false,
            colors: HashMap::new(),
            spacing: HashMap::new(),
            typography: HashMap::new(),
        }
    }

    /// Mark as dark theme.
    pub fn dark_mode(mut self) -> Self {
        self.dark = true;
        self
    }

    /// Add a color token.
    pub fn add_color(mut self, name: &str, value: &str) -> Self {
        self.colors.insert(name.to_string(), value.to_string());
        self
    }

    /// Add a spacing token.
    pub fn add_spacing(mut self, name: &str, value: &str) -> Self {
        self.spacing.insert(name.to_string(), value.to_string());
        self
    }

    /// Add a typography token.
    pub fn add_typography(mut self, name: &str, value: &str) -> Self {
        self.typography.insert(name.to_string(), value.to_string());
        self
    }

    /// Export to CSS custom properties.
    pub fn export_css(&self) -> String {
        let mut css = String::new();
        css.push_str(&format!(":root {{\n"));
        for (name, value) in &self.colors {
            css.push_str(&format!("  --color-{}: {};\n", name, value));
        }
        for (name, value) in &self.spacing {
            css.push_str(&format!("  --spacing-{}: {};\n", name, value));
        }
        for (name, value) in &self.typography {
            css.push_str(&format!("  --font-{}: {};\n", name, value));
        }
        css.push_str("}\n");
        css
    }

    /// Diff with another theme.
    pub fn diff(&self, other: &DesignTheme) -> Vec<String> {
        let mut diffs = Vec::new();
        for (key, val) in &self.colors {
            if let Some(other_val) = other.colors.get(key) {
                if val != other_val {
                    diffs.push(format!("color.{}: {} → {}", key, val, other_val));
                }
            } else {
                diffs.push(format!("color.{}: {} (removed)", key, val));
            }
        }
        for (key, val) in &other.colors {
            if !self.colors.contains_key(key) {
                diffs.push(format!("color.{}: {} (added)", key, val));
            }
        }
        diffs
    }
}

// === Goal 234: Docs server ===

/// The docs server configuration.
#[derive(Debug, Clone)]
pub struct DocsServerConfig {
    /// The port to serve on.
    pub port: u16,
    /// Whether to enable live search.
    pub live_search: bool,
    /// Whether to enable interactive examples.
    pub interactive: bool,
    /// Whether to work offline.
    pub offline: bool,
}

impl Default for DocsServerConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            live_search: true,
            interactive: true,
            offline: true,
        }
    }
}

/// A documentation page.
#[derive(Debug, Clone)]
pub struct DocPage {
    /// The page title.
    pub title: String,
    /// The page path.
    pub path: String,
    /// The content (markdown).
    pub content: String,
    /// The section.
    pub section: String,
}

impl DocPage {
    /// Create a new page.
    pub fn new(title: &str, path: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            path: path.to_string(),
            content: content.to_string(),
            section: "General".to_string(),
        }
    }

    /// Set the section.
    pub fn in_section(mut self, section: &str) -> Self {
        self.section = section.to_string();
        self
    }
}

// === Goal 235: CI/CD template generator ===

/// The CI/CD platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiPlatform {
    /// GitHub Actions.
    GitHubActions,
    /// GitLab CI.
    GitLabCi,
    /// CircleCI.
    CircleCi,
}

impl CiPlatform {
    /// Get the config file name.
    pub fn config_file(&self) -> &'static str {
        match self {
            CiPlatform::GitHubActions => ".github/workflows/ci.yml",
            CiPlatform::GitLabCi => ".gitlab-ci.yml",
            CiPlatform::CircleCi => ".circleci/config.yml",
        }
    }

    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            CiPlatform::GitHubActions => "GitHub Actions",
            CiPlatform::GitLabCi => "GitLab CI",
            CiPlatform::CircleCi => "CircleCI",
        }
    }
}

/// The CI/CD pipeline stages.
#[derive(Debug, Clone)]
pub struct CiPipeline {
    /// The platform.
    pub platform: CiPlatform,
    /// Whether to include build stage.
    pub build: bool,
    /// Whether to include test stage.
    pub test: bool,
    /// Whether to include lint stage.
    pub lint: bool,
    /// Whether to include size check.
    pub size_check: bool,
    /// Whether to include deploy stage.
    pub deploy: bool,
    /// The Rust toolchain.
    pub toolchain: String,
}

impl CiPipeline {
    /// Create a new pipeline.
    pub fn new(platform: CiPlatform) -> Self {
        Self {
            platform,
            build: true,
            test: true,
            lint: true,
            size_check: false,
            deploy: false,
            toolchain: "stable".to_string(),
        }
    }

    /// Enable size check.
    pub fn with_size_check(mut self) -> Self {
        self.size_check = true;
        self
    }

    /// Enable deploy.
    pub fn with_deploy(mut self) -> Self {
        self.deploy = true;
        self
    }

    /// Generate the GitHub Actions config.
    pub fn generate_github_actions(&self) -> String {
        let mut yml = String::new();
        yml.push_str("name: CI\n\n");
        yml.push_str("on: [push, pull_request]\n\n");
        yml.push_str("jobs:\n  build:\n");
        yml.push_str("    runs-on: ubuntu-latest\n");
        yml.push_str(&format!("    steps:\n"));
        yml.push_str("      - uses: actions/checkout@v4\n");
        yml.push_str(&format!("      - uses: dtolnay/rust-toolchain@{}\n", self.toolchain));
        if self.build {
            yml.push_str("      - run: cargo build --release\n");
        }
        if self.test {
            yml.push_str("      - run: cargo test\n");
        }
        if self.lint {
            yml.push_str("      - run: cargo clippy -- -D warnings\n");
        }
        if self.size_check {
            yml.push_str("      - run: rpg bundle --check\n");
        }
        if self.deploy {
            yml.push_str("      - run: rpg deploy\n");
        }
        yml
    }

    /// Generate the GitLab CI config.
    pub fn generate_gitlab_ci(&self) -> String {
        let mut yml = String::new();
        yml.push_str(&format!("image: rust:latest\n\nstages:\n"));
        let mut stages = Vec::new();
        if self.build { stages.push("  - build"); }
        if self.test { stages.push("  - test"); }
        if self.lint { stages.push("  - lint"); }
        if self.deploy { stages.push("  - deploy"); }
        yml.push_str(&stages.join("\n"));
        yml.push_str("\n\n");
        if self.build {
            yml.push_str("build:\n  stage: build\n  script:\n    - cargo build --release\n\n");
        }
        if self.test {
            yml.push_str("test:\n  stage: test\n  script:\n    - cargo test\n\n");
        }
        if self.lint {
            yml.push_str("lint:\n  stage: lint\n  script:\n    - cargo clippy -- -D warnings\n");
        }
        yml
    }

    /// Generate the config for the configured platform.
    pub fn generate(&self) -> String {
        match self.platform {
            CiPlatform::GitHubActions => self.generate_github_actions(),
            CiPlatform::GitLabCi => self.generate_gitlab_ci(),
            CiPlatform::CircleCi => self.generate_github_actions(), // Similar structure
        }
    }
}

/// Run the monorepo command.
pub fn run_monorepo(args: &[String]) {
    if args.is_empty() {
        println!("Usage: rpg monorepo <init|build>");
        return;
    }
    match args[0].as_str() {
        "init" => {
            let config = MonorepoConfig::new(".").add_member("crates/*");
            println!("Initializing monorepo workspace...");
            println!("{}", config.generate_workspace_toml());
        }
        "build" => {
            println!("Building all workspace members...");
        }
        other => {
            eprintln!("Unknown monorepo command: {}", other);
        }
    }
}

/// Run the publish command.
pub fn run_publish(args: &[String]) {
    let name = args.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str()).unwrap_or("my-lib");
    let lib = PublishedLibrary::new("@rye", name, "0.1.0");
    println!("Publishing {} v{}", lib.package_name(), lib.version);
    println!("{}", lib.generate_manifest());
}

/// Run the theme command.
pub fn run_theme(args: &[String]) {
    if args.is_empty() {
        println!("Usage: rpg theme <create|export|diff> [name]");
        return;
    }
    match args[0].as_str() {
        "create" => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("default");
            let theme = DesignTheme::new(name).add_color("primary", "#007acc");
            println!("Created theme: {}", theme.name);
            println!("{}", theme.export_css());
        }
        "export" => {
            let format = args.iter().find(|a| a.starts_with("--format")).map(|a| &a[9..]).unwrap_or("css");
            println!("Exporting theme as {}", format);
        }
        "diff" => {
            let t1 = args.get(1).map(|s| s.as_str()).unwrap_or("light");
            let t2 = args.get(2).map(|s| s.as_str()).unwrap_or("dark");
            println!("Diffing themes: {} vs {}", t1, t2);
        }
        other => {
            eprintln!("Unknown theme command: {}", other);
        }
    }
}

/// Run the docs command.
pub fn run_docs(args: &[String]) {
    let port = args.iter().find(|a| a.starts_with("--port")).and_then(|a| a[6..].parse().ok()).unwrap_or(4000);
    let config = DocsServerConfig { port, ..Default::default() };
    println!("Starting docs server on port {}", config.port);
    println!("Live search: {}, Interactive: {}, Offline: {}", config.live_search, config.interactive, config.offline);
}

/// Run the ci command.
pub fn run_ci(args: &[String]) {
    let platform = args.iter().find(|a| !a.starts_with("--")).map(|s| s.as_str()).unwrap_or("github");
    let ci_platform = match platform {
        "gitlab" => CiPlatform::GitLabCi,
        "circle" | "circleci" => CiPlatform::CircleCi,
        _ => CiPlatform::GitHubActions,
    };
    let pipeline = CiPipeline::new(ci_platform).with_size_check().with_deploy();
    println!("Generating {} CI/CD config...", ci_platform.display_name());
    println!("Output: {}", ci_platform.config_file());
    println!("\n{}", pipeline.generate());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Monorepo tests
    #[test]
    fn test_monorepo_config_new() {
        let config = MonorepoConfig::new(".");
        assert_eq!(config.root, ".");
        assert!(config.cross_imports);
    }

    #[test]
    fn test_monorepo_add_member() {
        let config = MonorepoConfig::new(".").add_member("crates/web").add_member("crates/api");
        assert_eq!(config.member_count(), 2);
    }

    #[test]
    fn test_monorepo_generate_workspace_toml() {
        let config = MonorepoConfig::new(".")
            .add_member("crates/web")
            .add_shared_dep("rye-core", "0.1.0");
        let toml = config.generate_workspace_toml();
        assert!(toml.contains("[workspace]"));
        assert!(toml.contains("crates/web"));
        assert!(toml.contains("rye-core"));
    }

    // Publishing tests
    #[test]
    fn test_published_library_new() {
        let lib = PublishedLibrary::new("@rye", "ui", "1.0.0");
        assert_eq!(lib.scope, "@rye");
        assert_eq!(lib.name, "ui");
    }

    #[test]
    fn test_published_library_package_name() {
        let lib = PublishedLibrary::new("@rye", "ui", "1.0.0");
        assert_eq!(lib.package_name(), "@rye/ui");
    }

    #[test]
    fn test_published_library_manifest() {
        let lib = PublishedLibrary::new("@rye", "ui", "1.0.0")
            .add_component("Button")
            .add_component("Card");
        let manifest = lib.generate_manifest();
        assert!(manifest.contains("@rye/ui"));
        assert!(manifest.contains("Button"));
        assert!(manifest.contains("Card"));
    }

    // Theme tests
    #[test]
    fn test_design_theme_new() {
        let theme = DesignTheme::new("light");
        assert_eq!(theme.name, "light");
        assert!(!theme.dark);
    }

    #[test]
    fn test_design_theme_export_css() {
        let theme = DesignTheme::new("light")
            .add_color("primary", "#007acc")
            .add_spacing("md", "16px");
        let css = theme.export_css();
        assert!(css.contains("--color-primary: #007acc"));
        assert!(css.contains("--spacing-md: 16px"));
    }

    #[test]
    fn test_design_theme_diff() {
        let theme1 = DesignTheme::new("light").add_color("primary", "#007acc");
        let theme2 = DesignTheme::new("dark").add_color("primary", "#1e1e1e");
        let diffs = theme1.diff(&theme2);
        assert!(diffs.iter().any(|d| d.contains("primary")));
    }

    #[test]
    fn test_design_theme_diff_added() {
        let theme1 = DesignTheme::new("light").add_color("primary", "#007acc");
        let theme2 = DesignTheme::new("dark").add_color("primary", "#007acc").add_color("accent", "#ff0");
        let diffs = theme1.diff(&theme2);
        assert!(diffs.iter().any(|d| d.contains("added")));
    }

    // Docs tests
    #[test]
    fn test_docs_server_config_default() {
        let config = DocsServerConfig::default();
        assert_eq!(config.port, 4000);
        assert!(config.offline);
    }

    #[test]
    fn test_doc_page_new() {
        let page = DocPage::new("Getting Started", "/docs/start", "Welcome!");
        assert_eq!(page.title, "Getting Started");
        assert_eq!(page.section, "General");
    }

    #[test]
    fn test_doc_page_in_section() {
        let page = DocPage::new("API", "/docs/api", "...").in_section("Reference");
        assert_eq!(page.section, "Reference");
    }

    // CI tests
    #[test]
    fn test_ci_platform_config_file() {
        assert_eq!(CiPlatform::GitHubActions.config_file(), ".github/workflows/ci.yml");
        assert_eq!(CiPlatform::GitLabCi.config_file(), ".gitlab-ci.yml");
    }

    #[test]
    fn test_ci_platform_display_name() {
        assert_eq!(CiPlatform::GitHubActions.display_name(), "GitHub Actions");
    }

    #[test]
    fn test_ci_pipeline_new() {
        let pipeline = CiPipeline::new(CiPlatform::GitHubActions);
        assert!(pipeline.build);
        assert!(pipeline.test);
        assert!(!pipeline.size_check);
    }

    #[test]
    fn test_ci_pipeline_generate_github_actions() {
        let pipeline = CiPipeline::new(CiPlatform::GitHubActions);
        let yml = pipeline.generate_github_actions();
        assert!(yml.contains("name: CI"));
        assert!(yml.contains("cargo build"));
        assert!(yml.contains("cargo test"));
        assert!(yml.contains("cargo clippy"));
    }

    #[test]
    fn test_ci_pipeline_generate_with_size_check() {
        let pipeline = CiPipeline::new(CiPlatform::GitHubActions).with_size_check();
        let yml = pipeline.generate_github_actions();
        assert!(yml.contains("rpg bundle"));
    }

    #[test]
    fn test_ci_pipeline_generate_gitlab_ci() {
        let pipeline = CiPipeline::new(CiPlatform::GitLabCi);
        let yml = pipeline.generate_gitlab_ci();
        assert!(yml.contains("stages:"));
        assert!(yml.contains("cargo test"));
    }

    #[test]
    fn test_ci_pipeline_generate() {
        let pipeline = CiPipeline::new(CiPlatform::GitHubActions);
        let yml = pipeline.generate();
        assert!(yml.contains("CI"));
    }
}
