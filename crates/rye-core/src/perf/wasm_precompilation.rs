//! Goal 219: Wasm precompilation.
//!
//! Pre-compile Wasm to native code during `rpg build` using `wizer` or similar.
//! Ship pre-initialized Wasm that starts faster.

use std::collections::HashMap;
use std::sync::Mutex;

/// The precompilation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecompilationStrategy {
    /// Use `wizer` to pre-initialize Wasm state.
    Wizer,
    /// Use `wasmer` to pre-compile to native object code.
    WasmerAot,
    /// Use `wasmtime` cranelift to pre-compile.
    WasmtimeCranelift,
    /// No precompilation — use streaming compilation at runtime.
    None,
}

impl PrecompilationStrategy {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            PrecompilationStrategy::Wizer => "wizer",
            PrecompilationStrategy::WasmerAot => "wasmer-aot",
            PrecompilationStrategy::WasmtimeCranelift => "wasmtime-cranelift",
            PrecompilationStrategy::None => "none",
        }
    }

    /// Estimate the cold-start time reduction (as a fraction, 0.0-1.0).
    pub fn estimated_speedup(&self) -> f64 {
        match self {
            PrecompilationStrategy::Wizer => 0.40,
            PrecompilationStrategy::WasmerAot => 0.50,
            PrecompilationStrategy::WasmtimeCranelift => 0.45,
            PrecompilationStrategy::None => 0.0,
        }
    }
}

/// Configuration for Wasm precompilation.
#[derive(Debug, Clone)]
pub struct PrecompilationConfig {
    /// The precompilation strategy.
    pub strategy: PrecompilationStrategy,
    /// Whether to keep the original Wasm as a fallback.
    pub keep_fallback: bool,
    /// Whether to generate a precompilation report.
    pub generate_report: bool,
    /// The initialization function to run (for wizer).
    pub init_function: String,
    /// Whether to enable SIMD during precompilation.
    pub enable_simd: bool,
    /// Whether to enable threading during precompilation.
    pub enable_threading: bool,
}

impl Default for PrecompilationConfig {
    fn default() -> Self {
        Self {
            strategy: PrecompilationStrategy::Wizer,
            keep_fallback: true,
            generate_report: true,
            init_function: "rye_init".to_string(),
            enable_simd: true,
            enable_threading: false,
        }
    }
}

impl PrecompilationConfig {
    /// Create a config with a specific strategy.
    pub fn with_strategy(strategy: PrecompilationStrategy) -> Self {
        Self {
            strategy,
            ..Default::default()
        }
    }

    /// Disable precompilation.
    pub fn disabled() -> Self {
        Self {
            strategy: PrecompilationStrategy::None,
            keep_fallback: false,
            generate_report: false,
            ..Default::default()
        }
    }
}

/// A precompilation result.
#[derive(Debug, Clone)]
pub struct PrecompilationResult {
    /// The output Wasm file path.
    pub output_path: String,
    /// The original Wasm size in bytes.
    pub original_size: u64,
    /// The precompiled Wasm size in bytes.
    pub precompiled_size: u64,
    /// The strategy used.
    pub strategy: PrecompilationStrategy,
    /// Whether precompilation succeeded.
    pub success: bool,
    /// Error message (if failed).
    pub error: Option<String>,
    /// The estimated cold-start time reduction (ms).
    pub estimated_speedup_ms: u64,
}

impl PrecompilationResult {
    /// Create a successful result.
    pub fn success(
        output_path: &str,
        original_size: u64,
        precompiled_size: u64,
        strategy: PrecompilationStrategy,
    ) -> Self {
        let speedup_ms = (strategy.estimated_speedup() * 100.0) as u64;
        Self {
            output_path: output_path.to_string(),
            original_size,
            precompiled_size,
            strategy,
            success: true,
            error: None,
            estimated_speedup_ms: speedup_ms,
        }
    }

    /// Create a failed result.
    pub fn failure(error: &str, strategy: PrecompilationStrategy) -> Self {
        Self {
            output_path: String::new(),
            original_size: 0,
            precompiled_size: 0,
            strategy,
            success: false,
            error: Some(error.to_string()),
            estimated_speedup_ms: 0,
        }
    }

    /// Get the size change (positive = larger, negative = smaller).
    pub fn size_change(&self) -> i64 {
        self.precompiled_size as i64 - self.original_size as i64
    }

    /// Get the size change as a percentage.
    pub fn size_change_percent(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        (self.precompiled_size as f64 - self.original_size as f64) / self.original_size as f64
            * 100.0
    }
}

/// A precompilation report.
#[derive(Debug, Clone)]
pub struct PrecompilationReport {
    /// The results for each module.
    pub results: Vec<PrecompilationResult>,
    /// The total original size.
    pub total_original_size: u64,
    /// The total precompiled size.
    pub total_precompiled_size: u64,
    /// The total estimated speedup (ms).
    pub total_speedup_ms: u64,
    /// The config used.
    pub config: PrecompilationConfig,
}

impl PrecompilationReport {
    /// Create a new report.
    pub fn new(config: PrecompilationConfig) -> Self {
        Self {
            results: Vec::new(),
            total_original_size: 0,
            total_precompiled_size: 0,
            total_speedup_ms: 0,
            config,
        }
    }

    /// Add a result to the report.
    pub fn add_result(&mut self, result: PrecompilationResult) {
        if result.success {
            self.total_original_size += result.original_size;
            self.total_precompiled_size += result.precompiled_size;
            self.total_speedup_ms += result.estimated_speedup_ms;
        }
        self.results.push(result);
    }

    /// Get the number of successful precompilations.
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    /// Get the number of failed precompilations.
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }

    /// Get the total size change.
    pub fn total_size_change(&self) -> i64 {
        self.total_precompiled_size as i64 - self.total_original_size as i64
    }

    /// Generate a text report.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str("=== Wasm Precompilation Report ===\n\n");
        text.push_str(&format!(
            "Strategy: {}\n",
            self.config.strategy.display_name()
        ));
        text.push_str(&format!(
            "Modules: {} total, {} succeeded, {} failed\n\n",
            self.results.len(),
            self.success_count(),
            self.failure_count()
        ));

        for result in &self.results {
            if result.success {
                text.push_str(&format!(
                    "  ✓ {} ({}KB → {}KB, {:.1}% change, ~{}ms faster)\n",
                    result.output_path,
                    result.original_size / 1024,
                    result.precompiled_size / 1024,
                    result.size_change_percent(),
                    result.estimated_speedup_ms,
                ));
            } else if let Some(error) = &result.error {
                text.push_str(&format!(
                    "  ✗ {} — {}\n",
                    result.strategy.display_name(),
                    error
                ));
            }
        }

        text.push_str(&format!(
            "\nTotal: {}KB → {}KB ({}ms estimated speedup)\n",
            self.total_original_size / 1024,
            self.total_precompiled_size / 1024,
            self.total_speedup_ms
        ));

        text
    }
}

/// The Wasm precompiler — manages precompilation of Wasm modules.
pub struct WasmPrecompiler {
    config: PrecompilationConfig,
    reports: Mutex<Vec<PrecompilationReport>>,
}

impl WasmPrecompiler {
    /// Create a new precompiler.
    pub fn new(config: PrecompilationConfig) -> Self {
        Self {
            config,
            reports: Mutex::new(Vec::new()),
        }
    }

    /// Get the config.
    pub fn config(&self) -> &PrecompilationConfig {
        &self.config
    }

    /// Simulate precompiling a Wasm module.
    pub fn precompile(&self, wasm_path: &str, original_size: u64) -> PrecompilationResult {
        if self.config.strategy == PrecompilationStrategy::None {
            return PrecompilationResult::failure("Precompilation disabled", self.config.strategy);
        }

        // Simulate: precompiled Wasm is slightly larger but starts faster
        let precompiled_size = (original_size as f64 * 1.05) as u64;
        let output_path = wasm_path.replace(".wasm", ".precompiled.wasm");

        PrecompilationResult::success(
            &output_path,
            original_size,
            precompiled_size,
            self.config.strategy,
        )
    }

    /// Precompile multiple modules and generate a report.
    pub fn precompile_all(&self, modules: &[(&str, u64)]) -> PrecompilationReport {
        let mut report = PrecompilationReport::new(self.config.clone());

        for (path, size) in modules {
            let result = self.precompile(path, *size);
            report.add_result(result);
        }

        self.reports.lock().unwrap().push(report.clone());
        report
    }

    /// Get all reports.
    pub fn reports(&self) -> Vec<PrecompilationReport> {
        self.reports.lock().unwrap().clone()
    }

    /// Generate the build script addition for precompilation.
    pub fn generate_build_script(&self) -> String {
        match self.config.strategy {
            PrecompilationStrategy::Wizer => {
                format!(
                    "# Precompilation step (wizer)\n\
                     # Run initialization function ahead of time\n\
                     wizer {} --init-fn {} --allow-wasi --inherit-stdio\n",
                    "rye_app.wasm", self.config.init_function
                )
            }
            PrecompilationStrategy::WasmerAot => "# Precompilation step (wasmer)\n\
                 wasmer create exe rye_app.wasm --output rye_app.native\n"
                .to_string(),
            PrecompilationStrategy::WasmtimeCranelift => "# Precompilation step (wasmtime)\n\
                 wasmtime compile rye_app.wasm --output rye_app.cwasm\n"
                .to_string(),
            PrecompilationStrategy::None => {
                "# No precompilation — using streaming compilation\n".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precompilation_strategy_display_name() {
        assert_eq!(PrecompilationStrategy::Wizer.display_name(), "wizer");
        assert_eq!(PrecompilationStrategy::None.display_name(), "none");
    }

    #[test]
    fn test_precompilation_strategy_estimated_speedup() {
        assert_eq!(PrecompilationStrategy::Wizer.estimated_speedup(), 0.40);
        assert_eq!(PrecompilationStrategy::None.estimated_speedup(), 0.0);
        assert!(
            PrecompilationStrategy::WasmerAot.estimated_speedup()
                > PrecompilationStrategy::Wizer.estimated_speedup()
        );
    }

    #[test]
    fn test_precompilation_config_default() {
        let config = PrecompilationConfig::default();
        assert_eq!(config.strategy, PrecompilationStrategy::Wizer);
        assert!(config.keep_fallback);
        assert!(config.generate_report);
    }

    #[test]
    fn test_precompilation_config_disabled() {
        let config = PrecompilationConfig::disabled();
        assert_eq!(config.strategy, PrecompilationStrategy::None);
        assert!(!config.keep_fallback);
    }

    #[test]
    fn test_precompilation_result_success() {
        let result = PrecompilationResult::success(
            "out.wasm",
            100000,
            105000,
            PrecompilationStrategy::Wizer,
        );
        assert!(result.success);
        assert_eq!(result.original_size, 100000);
        assert_eq!(result.precompiled_size, 105000);
        assert!(result.estimated_speedup_ms > 0);
    }

    #[test]
    fn test_precompilation_result_failure() {
        let result = PrecompilationResult::failure("error", PrecompilationStrategy::Wizer);
        assert!(!result.success);
        assert_eq!(result.error, Some("error".to_string()));
        assert_eq!(result.estimated_speedup_ms, 0);
    }

    #[test]
    fn test_precompilation_result_size_change() {
        let result = PrecompilationResult::success(
            "out.wasm",
            100000,
            105000,
            PrecompilationStrategy::Wizer,
        );
        assert_eq!(result.size_change(), 5000);
        assert!((result.size_change_percent() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_precompilation_report_new() {
        let report = PrecompilationReport::new(PrecompilationConfig::default());
        assert_eq!(report.success_count(), 0);
        assert_eq!(report.failure_count(), 0);
    }

    #[test]
    fn test_precompilation_report_add_result() {
        let mut report = PrecompilationReport::new(PrecompilationConfig::default());
        report.add_result(PrecompilationResult::success(
            "a.wasm",
            100,
            105,
            PrecompilationStrategy::Wizer,
        ));
        assert_eq!(report.success_count(), 1);
        assert_eq!(report.total_original_size, 100);
        assert_eq!(report.total_precompiled_size, 105);
    }

    #[test]
    fn test_precompilation_report_failure() {
        let mut report = PrecompilationReport::new(PrecompilationConfig::default());
        report.add_result(PrecompilationResult::failure(
            "err",
            PrecompilationStrategy::Wizer,
        ));
        assert_eq!(report.failure_count(), 1);
        assert_eq!(report.success_count(), 0);
    }

    #[test]
    fn test_precompilation_report_to_text() {
        let mut report = PrecompilationReport::new(PrecompilationConfig::default());
        report.add_result(PrecompilationResult::success(
            "a.wasm",
            1024,
            1075,
            PrecompilationStrategy::Wizer,
        ));
        let text = report.to_text();
        assert!(text.contains("Precompilation Report"));
        assert!(text.contains("wizer"));
        assert!(text.contains("succeeded"));
    }

    #[test]
    fn test_wasm_precompiler_precompile() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::default());
        let result = precompiler.precompile("app.wasm", 100000);
        assert!(result.success);
        assert!(result.output_path.contains("precompiled"));
    }

    #[test]
    fn test_wasm_precompiler_precompile_disabled() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::disabled());
        let result = precompiler.precompile("app.wasm", 100000);
        assert!(!result.success);
    }

    #[test]
    fn test_wasm_precompiler_precompile_all() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::default());
        let report = precompiler.precompile_all(&[("a.wasm", 1000), ("b.wasm", 2000)]);
        assert_eq!(report.success_count(), 2);
        assert_eq!(report.total_original_size, 3000);
    }

    #[test]
    fn test_wasm_precompiler_reports() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::default());
        precompiler.precompile_all(&[("a.wasm", 1000)]);
        precompiler.precompile_all(&[("b.wasm", 2000)]);
        assert_eq!(precompiler.reports().len(), 2);
    }

    #[test]
    fn test_wasm_precompiler_build_script_wizer() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::default());
        let script = precompiler.generate_build_script();
        assert!(script.contains("wizer"));
        assert!(script.contains("rye_init"));
    }

    #[test]
    fn test_wasm_precompiler_build_script_none() {
        let precompiler = WasmPrecompiler::new(PrecompilationConfig::disabled());
        let script = precompiler.generate_build_script();
        assert!(script.contains("No precompilation"));
    }
}
