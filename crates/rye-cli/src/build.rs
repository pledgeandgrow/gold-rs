//! `rpg build` — production build for web, desktop, mobile, and SSR targets.
//!
//! ## Targets
//!
//! - **web** — `wasm-pack build --target web` (Wasm + JS glue for browsers)
//! - **desktop** — `cargo build` (native binary, wgpu + winit)
//! - **android** — `cargo build --target aarch64-linux-android` (JNI + wgpu)
//! - **ios** — `cargo build --target aarch64-apple-ios` (Obj-C + wgpu)
//! - **ssr** — `cargo build` (server-side rendering to HTML strings)
//!
//! ## Flags
//!
//! - `--release` — Optimized build (default for production)
//! - `--debug` — Debug build (no optimizations)
//! - `--out-dir <path>` — Output directory (web only, default: `pkg`)
//! - `--target <name>` — Build target (web, desktop, android, ios, ssr)
//! - `--features <list>` — Comma-separated Cargo features to enable
//! - `--watch` — Watch for changes and rebuild (delegates to dev server)

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build target — determines which toolchain and output format to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTarget {
    /// Web (Wasm via wasm-pack).
    Web,
    /// Desktop (native binary via cargo).
    Desktop,
    /// Android (shared library via cargo + NDK).
    Android,
    /// iOS (static library via cargo + Xcode toolchain).
    Ios,
    /// Server-side rendering (native binary via cargo).
    Ssr,
}

impl BuildTarget {
    /// Parse a target from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "web" | "wasm" => Some(Self::Web),
            "desktop" | "native" => Some(Self::Desktop),
            "android" => Some(Self::Android),
            "ios" => Some(Self::Ios),
            "ssr" | "server" => Some(Self::Ssr),
            _ => None,
        }
    }

    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Android => "android",
            Self::Ios => "iOS",
            Self::Ssr => "SSR",
        }
    }

    /// Whether this target uses wasm-pack.
    #[allow(dead_code)]
    pub fn uses_wasm_pack(&self) -> bool {
        matches!(self, Self::Web)
    }

    /// Whether this target uses cargo directly.
    #[allow(dead_code)]
    pub fn uses_cargo(&self) -> bool {
        !self.uses_wasm_pack()
    }

    /// Get the cargo target triple (if applicable).
    pub fn cargo_target_triple(&self) -> Option<&'static str> {
        match self {
            Self::Android => Some("aarch64-linux-android"),
            Self::Ios => Some("aarch64-apple-ios"),
            _ => None,
        }
    }
}

/// Build configuration — parsed from CLI args.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// The build target.
    pub target: BuildTarget,
    /// Whether to build in release mode.
    pub release: bool,
    /// Whether to build in debug mode (explicit).
    pub debug: bool,
    /// Output directory (web only).
    pub out_dir: String,
    /// Additional Cargo features to enable.
    pub features: Vec<String>,
    /// Whether to watch for changes.
    pub watch: bool,
    /// The project root directory.
    pub project_root: PathBuf,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: BuildTarget::Web,
            release: false,
            debug: false,
            out_dir: "pkg".to_string(),
            features: Vec::new(),
            watch: false,
            project_root: PathBuf::from("."),
        }
    }
}

impl BuildConfig {
    /// Parse build config from CLI args.
    ///
    /// Supported flags:
    /// - `--target <name>` — Build target (default: web)
    /// - `--release` — Release build
    /// - `--debug` — Debug build
    /// - `--out-dir <path>` — Output directory
    /// - `--features <list>` — Cargo features
    /// - `--watch` — Watch mode
    pub fn from_args(args: &[String]) -> Self {
        let mut config = Self::default();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "--release" | "-r" => {
                    config.release = true;
                }
                "--debug" | "-d" => {
                    config.debug = true;
                }
                "--watch" | "-w" => {
                    config.watch = true;
                }
                "--target" if i + 1 < args.len() => {
                    if let Some(target) = BuildTarget::from_str(&args[i + 1]) {
                        config.target = target;
                    }
                    i += 1;
                }
                s if s.starts_with("--target=") => {
                    if let Some(target) = BuildTarget::from_str(&s[9..]) {
                        config.target = target;
                    }
                }
                "--out-dir" if i + 1 < args.len() => {
                    config.out_dir = args[i + 1].clone();
                    i += 1;
                }
                s if s.starts_with("--out-dir=") => {
                    config.out_dir = s[10..].to_string();
                }
                "--features" if i + 1 < args.len() => {
                    config.features = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    i += 1;
                }
                s if s.starts_with("--features=") => {
                    config.features = s[11..].split(',').map(|s| s.trim().to_string()).collect();
                }
                _ => {}
            }
            i += 1;
        }

        // If neither --release nor --debug is specified, default to release
        // for production builds (i.e. `rpg build` without flags = release).
        if !config.release && !config.debug {
            config.release = true;
        }

        config
    }

    /// Whether this is a release build.
    pub fn is_release(&self) -> bool {
        self.release && !self.debug
    }
}

/// Result of a build.
#[derive(Debug)]
pub struct BuildResult {
    /// Whether the build succeeded.
    pub success: bool,
    /// The output directory or binary path.
    pub output_path: Option<PathBuf>,
    /// Build warnings/errors from stderr.
    pub warnings: String,
}

/// Run the build.
///
/// This is the main entry point called from `cmd_build` in main.rs.
pub fn run(args: &[String]) {
    let config = BuildConfig::from_args(args);

    println!("  rye build — target: {}", config.target.name());
    println!(
        "  mode:  {}",
        if config.is_release() {
            "release"
        } else {
            "debug"
        }
    );
    if !config.features.is_empty() {
        println!("  features: {}", config.features.join(", "));
    }
    println!();

    // Watch mode delegates to the dev server.
    if config.watch {
        println!("  Watch mode — starting dev server instead...");
        println!("  (Use `rpg dev` for the full dev server experience)");
        run_watch(&config);
        return;
    }

    let result = build(&config);
    if result.success {
        println!();
        println!("  Build succeeded!");
        if let Some(path) = &result.output_path {
            println!("  Output: {}", path.display());
        }
        if !result.warnings.is_empty() {
            println!();
            println!("  Warnings:");
            for line in result.warnings.lines().take(20) {
                println!("    {}", line);
            }
            let remaining = result.warnings.lines().count().saturating_sub(20);
            if remaining > 0 {
                println!("    ... and {} more", remaining);
            }
        }
    } else {
        eprintln!();
        eprintln!("  Build FAILED!");
        if !result.warnings.is_empty() {
            eprintln!();
            eprintln!("  Errors:");
            for line in result.warnings.lines() {
                eprintln!("    {}", line);
            }
        }
        std::process::exit(1);
    }
}

/// Execute the build based on the config.
fn build(config: &BuildConfig) -> BuildResult {
    // Detect the project root by looking for Cargo.toml.
    let project_root = find_project_root(&config.project_root);

    // Read the package name from Cargo.toml.
    let pkg_name = read_package_name(&project_root).unwrap_or_else(|| {
        eprintln!("  Warning: Could not read package name from Cargo.toml, using 'rye_app'");
        "rye_app".to_string()
    });

    match config.target {
        BuildTarget::Web => build_web(&project_root, &pkg_name, config),
        BuildTarget::Desktop => build_desktop(&project_root, &pkg_name, config),
        BuildTarget::Android => build_android(&project_root, &pkg_name, config),
        BuildTarget::Ios => build_ios(&project_root, &pkg_name, config),
        BuildTarget::Ssr => build_ssr(&project_root, &pkg_name, config),
    }
}

/// Build for web using `wasm-pack`.
fn build_web(project_root: &Path, pkg_name: &str, config: &BuildConfig) -> BuildResult {
    // Check if wasm-pack is installed.
    if !is_tool_installed("wasm-pack") {
        return BuildResult {
            success: false,
            output_path: None,
            warnings: "wasm-pack is not installed. Install it with: cargo install wasm-pack"
                .to_string(),
        };
    }

    let out_dir = &config.out_dir;

    let mut cmd = Command::new("wasm-pack");
    cmd.arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg(out_dir)
        .current_dir(project_root);

    if config.is_release() {
        cmd.arg("--release");
    } else {
        cmd.arg("--dev");
    }

    // Add features if specified.
    if !config.features.is_empty() {
        cmd.arg("--features").arg(config.features.join(","));
    }

    println!(
        "  Running: wasm-pack build --target web --out-dir {} {}",
        out_dir,
        if config.is_release() {
            "--release"
        } else {
            "--dev"
        }
    );

    let output = cmd.stdout(Stdio::inherit()).stderr(Stdio::piped()).output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            if result.status.success() {
                let output_path = project_root.join(out_dir);
                println!("  wasm-pack build completed for '{}'", pkg_name);
                BuildResult {
                    success: true,
                    output_path: Some(output_path),
                    warnings: stderr,
                }
            } else {
                BuildResult {
                    success: false,
                    output_path: None,
                    warnings: stderr,
                }
            }
        }
        Err(e) => BuildResult {
            success: false,
            output_path: None,
            warnings: format!("Failed to run wasm-pack: {}", e),
        },
    }
}

/// Build for desktop using `cargo build`.
fn build_desktop(project_root: &Path, pkg_name: &str, config: &BuildConfig) -> BuildResult {
    build_cargo(project_root, pkg_name, config, None, &["desktop"])
}

/// Build for Android using `cargo build` with the NDK target.
fn build_android(project_root: &Path, pkg_name: &str, config: &BuildConfig) -> BuildResult {
    let triple = BuildTarget::Android.cargo_target_triple().unwrap();

    // Check if the target is installed.
    if !is_rust_target_installed(triple) {
        return BuildResult {
            success: false,
            output_path: None,
            warnings: format!(
                "Rust target '{}' is not installed.\n  Install it with: rustup target add {}",
                triple, triple
            ),
        };
    }

    build_cargo(project_root, pkg_name, config, Some(triple), &["mobile"])
}

/// Build for iOS using `cargo build` with the iOS target.
fn build_ios(project_root: &Path, pkg_name: &str, config: &BuildConfig) -> BuildResult {
    let triple = BuildTarget::Ios.cargo_target_triple().unwrap();

    if !is_rust_target_installed(triple) {
        return BuildResult {
            success: false,
            output_path: None,
            warnings: format!(
                "Rust target '{}' is not installed.\n  Install it with: rustup target add {}",
                triple, triple
            ),
        };
    }

    build_cargo(project_root, pkg_name, config, Some(triple), &["mobile"])
}

/// Build for SSR using `cargo build`.
fn build_ssr(project_root: &Path, pkg_name: &str, config: &BuildConfig) -> BuildResult {
    build_cargo(project_root, pkg_name, config, None, &["ssr"])
}

/// Core `cargo build` invocation shared by desktop, mobile, and SSR targets.
fn build_cargo(
    project_root: &Path,
    pkg_name: &str,
    config: &BuildConfig,
    target_triple: Option<&str>,
    default_features: &[&str],
) -> BuildResult {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(project_root);

    if config.is_release() {
        cmd.arg("--release");
    }

    // Add target triple if specified.
    if let Some(triple) = target_triple {
        cmd.arg("--target").arg(triple);
    }

    // Add features: default features for the target + user-specified features.
    let mut all_features: Vec<String> = default_features.iter().map(|s| s.to_string()).collect();
    all_features.extend(config.features.clone());
    if !all_features.is_empty() {
        cmd.arg("--features").arg(all_features.join(","));
    }

    // Build the command display string.
    let mut display = format!("cargo build");
    if config.is_release() {
        display.push_str(" --release");
    }
    if let Some(triple) = target_triple {
        display.push_str(&format!(" --target {}", triple));
    }
    if !all_features.is_empty() {
        display.push_str(&format!(" --features {}", all_features.join(",")));
    }
    println!("  Running: {}", display);

    let output = cmd.stdout(Stdio::inherit()).stderr(Stdio::piped()).output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            if result.status.success() {
                // Determine the output binary path.
                let output_path =
                    determine_binary_path(project_root, pkg_name, config, target_triple);
                println!("  cargo build completed for '{}'", pkg_name);
                BuildResult {
                    success: true,
                    output_path,
                    warnings: stderr,
                }
            } else {
                BuildResult {
                    success: false,
                    output_path: None,
                    warnings: stderr,
                }
            }
        }
        Err(e) => BuildResult {
            success: false,
            output_path: None,
            warnings: format!("Failed to run cargo: {}", e),
        },
    }
}

/// Watch mode — rebuild on file change.
fn run_watch(config: &BuildConfig) {
    let project_root = find_project_root(&config.project_root);
    let _pkg_name = read_package_name(&project_root).unwrap_or_else(|| "rye_app".to_string());

    println!("  Watching for changes... (Ctrl+C to stop)");
    println!();

    // Simple polling watcher — check for file modifications every 1 second.
    let watch_dir = project_root.join("src");
    let mut last_modified = get_latest_modified(&watch_dir);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let current = get_latest_modified(&watch_dir);
        if current != last_modified {
            last_modified = current;
            println!();
            println!("  Change detected, rebuilding...");
            let result = build(config);
            if result.success {
                println!("  Rebuild succeeded!");
            } else {
                eprintln!("  Rebuild failed!");
                for line in result.warnings.lines() {
                    eprintln!("    {}", line);
                }
            }
            println!();
            println!("  Watching for changes... (Ctrl+C to stop)");
        }
    }
}

/// Find the project root by searching for Cargo.toml.
fn find_project_root(start: &Path) -> PathBuf {
    let mut current = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    loop {
        if current.join("Cargo.toml").exists() {
            return current;
        }
        if !current.pop() {
            // Reached the root without finding Cargo.toml.
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

/// Read the package name from Cargo.toml.
fn read_package_name(project_root: &Path) -> Option<String> {
    let cargo_toml = project_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).ok()?;

    // Simple parse: look for `name = "..."` in the [package] section.
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_package = false;
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            if let Some(name) = trimmed.split('=').nth(1) {
                return Some(name.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Determine the output binary path for a cargo build.
fn determine_binary_path(
    project_root: &Path,
    pkg_name: &str,
    config: &BuildConfig,
    target_triple: Option<&str>,
) -> Option<PathBuf> {
    let profile = if config.is_release() {
        "release"
    } else {
        "debug"
    };

    let target_dir = if let Some(triple) = target_triple {
        project_root.join("target").join(triple).join(profile)
    } else {
        project_root.join("target").join(profile)
    };

    // On Windows, binaries have .exe extension.
    let exe_name = if cfg!(windows) {
        format!("{}.exe", pkg_name.replace('-', "_"))
    } else {
        pkg_name.replace('-', "_")
    };

    let bin_path = target_dir.join(&exe_name);
    if bin_path.exists() {
        Some(bin_path)
    } else {
        // For library builds (e.g. mobile), the output is a .so/.a/.dylib.
        let lib_name = if cfg!(windows) {
            format!("{}.dll", pkg_name.replace('-', "_"))
        } else if target_triple
            .map(|t| t.contains("android"))
            .unwrap_or(false)
        {
            format!("lib{}.so", pkg_name.replace('-', "_"))
        } else if target_triple
            .map(|t| t.contains("apple-ios"))
            .unwrap_or(false)
        {
            format!("lib{}.a", pkg_name.replace('-', "_"))
        } else {
            format!("lib{}.so", pkg_name.replace('-', "_"))
        };
        let lib_path = target_dir.join(&lib_name);
        if lib_path.exists() {
            Some(lib_path)
        } else {
            Some(target_dir)
        }
    }
}

/// Check if a command-line tool is installed.
fn is_tool_installed(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Check if a Rust target is installed.
fn is_rust_target_installed(target: &str) -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.lines().any(|line| line.trim() == target)
        })
        .unwrap_or(false)
}

/// Get the latest modified timestamp of any file in a directory (recursive).
fn get_latest_modified(dir: &Path) -> Option<std::time::SystemTime> {
    if !dir.exists() {
        return None;
    }
    let mut latest: Option<std::time::SystemTime> = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(t) = get_latest_modified(&path) {
                    latest = Some(latest.map_or(t, |l| l.max(t)));
                }
            } else if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    latest = Some(latest.map_or(modified, |l| l.max(modified)));
                }
            }
        }
    }
    latest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_from_str() {
        assert_eq!(BuildTarget::from_str("web"), Some(BuildTarget::Web));
        assert_eq!(BuildTarget::from_str("wasm"), Some(BuildTarget::Web));
        assert_eq!(BuildTarget::from_str("desktop"), Some(BuildTarget::Desktop));
        assert_eq!(BuildTarget::from_str("native"), Some(BuildTarget::Desktop));
        assert_eq!(BuildTarget::from_str("android"), Some(BuildTarget::Android));
        assert_eq!(BuildTarget::from_str("ios"), Some(BuildTarget::Ios));
        assert_eq!(BuildTarget::from_str("ssr"), Some(BuildTarget::Ssr));
        assert_eq!(BuildTarget::from_str("server"), Some(BuildTarget::Ssr));
        assert_eq!(BuildTarget::from_str("unknown"), None);
    }

    #[test]
    fn test_build_target_name() {
        assert_eq!(BuildTarget::Web.name(), "web");
        assert_eq!(BuildTarget::Desktop.name(), "desktop");
        assert_eq!(BuildTarget::Android.name(), "android");
        assert_eq!(BuildTarget::Ios.name(), "iOS");
        assert_eq!(BuildTarget::Ssr.name(), "SSR");
    }

    #[test]
    fn test_build_target_tools() {
        assert!(BuildTarget::Web.uses_wasm_pack());
        assert!(!BuildTarget::Web.uses_cargo());
        assert!(!BuildTarget::Desktop.uses_wasm_pack());
        assert!(BuildTarget::Desktop.uses_cargo());
    }

    #[test]
    fn test_build_target_triple() {
        assert_eq!(
            BuildTarget::Android.cargo_target_triple(),
            Some("aarch64-linux-android")
        );
        assert_eq!(
            BuildTarget::Ios.cargo_target_triple(),
            Some("aarch64-apple-ios")
        );
        assert_eq!(BuildTarget::Web.cargo_target_triple(), None);
        assert_eq!(BuildTarget::Desktop.cargo_target_triple(), None);
    }

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.target, BuildTarget::Web);
        assert!(!config.release);
        assert!(!config.debug);
        assert_eq!(config.out_dir, "pkg");
    }

    #[test]
    fn test_build_config_from_args_target() {
        let args = vec!["--target".to_string(), "desktop".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.target, BuildTarget::Desktop);
    }

    #[test]
    fn test_build_config_from_args_target_equals() {
        let args = vec!["--target=android".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.target, BuildTarget::Android);
    }

    #[test]
    fn test_build_config_from_args_release() {
        let args = vec!["--release".to_string()];
        let config = BuildConfig::from_args(&args);
        assert!(config.release);
        assert!(config.is_release());
    }

    #[test]
    fn test_build_config_from_args_debug() {
        let args = vec!["--debug".to_string()];
        let config = BuildConfig::from_args(&args);
        assert!(config.debug);
        assert!(!config.is_release());
    }

    #[test]
    fn test_build_config_default_is_release() {
        // No flags = release (production default).
        let config = BuildConfig::from_args(&[]);
        assert!(config.is_release());
    }

    #[test]
    fn test_build_config_from_args_out_dir() {
        let args = vec!["--out-dir".to_string(), "dist".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.out_dir, "dist");
    }

    #[test]
    fn test_build_config_from_args_out_dir_equals() {
        let args = vec!["--out-dir=build".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.out_dir, "build");
    }

    #[test]
    fn test_build_config_from_args_features() {
        let args = vec!["--features".to_string(), "foo,bar".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.features, vec!["foo", "bar"]);
    }

    #[test]
    fn test_build_config_from_args_features_equals() {
        let args = vec!["--features=foo,bar,baz".to_string()];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.features, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_build_config_from_args_watch() {
        let args = vec!["--watch".to_string()];
        let config = BuildConfig::from_args(&args);
        assert!(config.watch);
    }

    #[test]
    fn test_build_config_from_args_combined() {
        let args = vec![
            "--target".to_string(),
            "desktop".to_string(),
            "--release".to_string(),
            "--features".to_string(),
            "foo,bar".to_string(),
        ];
        let config = BuildConfig::from_args(&args);
        assert_eq!(config.target, BuildTarget::Desktop);
        assert!(config.is_release());
        assert_eq!(config.features, vec!["foo", "bar"]);
    }

    #[test]
    fn test_read_package_name() {
        // Test with the workspace Cargo.toml.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let name = read_package_name(&root);
        assert_eq!(name.as_deref(), Some("rye-cli"));
    }

    #[test]
    fn test_read_package_name_not_found() {
        let name = read_package_name(Path::new("/nonexistent/path"));
        assert!(name.is_none());
    }

    #[test]
    fn test_find_project_root() {
        // The test directory should find the workspace root.
        let root = find_project_root(Path::new("."));
        assert!(root.join("Cargo.toml").exists());
    }
}
