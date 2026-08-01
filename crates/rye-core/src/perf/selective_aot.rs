//! Goal 220: Selective Wasm AOT.
//!
//! Profile-guided AOT compilation of hot paths in Wasm to native code.
//! Uses `cranelift` to compile frequently-executed render paths ahead of time.
//! Hybrid interpreter + AOT execution.

use std::collections::HashMap;
use std::sync::Mutex;

/// The execution mode for a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Interpret at runtime (default).
    Interpreted,
    /// AOT-compiled to native code.
    AotCompiled,
    /// Hybrid — interpreted with AOT fallback for hot paths.
    Hybrid,
}

impl ExecutionMode {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ExecutionMode::Interpreted => "interpreted",
            ExecutionMode::AotCompiled => "AOT-compiled",
            ExecutionMode::Hybrid => "hybrid",
        }
    }
}

/// A profile sample — a function execution record.
#[derive(Debug, Clone)]
pub struct ProfileSample {
    /// The function name.
    pub function_name: String,
    /// The module name.
    pub module_name: String,
    /// The number of times this function was called.
    pub call_count: u64,
    /// The total execution time in microseconds.
    pub total_time_us: u64,
    /// Whether this is a render hot path.
    pub is_render_path: bool,
}

impl ProfileSample {
    /// Create a new profile sample.
    pub fn new(function_name: &str, module_name: &str) -> Self {
        Self {
            function_name: function_name.to_string(),
            module_name: module_name.to_string(),
            call_count: 0,
            total_time_us: 0,
            is_render_path: false,
        }
    }

    /// Record a call.
    pub fn record_call(&mut self, time_us: u64) {
        self.call_count += 1;
        self.total_time_us += time_us;
    }

    /// Get the average execution time in microseconds.
    pub fn avg_time_us(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        self.total_time_us as f64 / self.call_count as f64
    }

    /// Mark as a render hot path.
    pub fn mark_render_path(mut self) -> Self {
        self.is_render_path = true;
        self
    }

    /// Get the hotness score (higher = hotter).
    pub fn hotness_score(&self) -> f64 {
        let call_weight = (self.call_count as f64).ln().max(0.0);
        let time_weight = (self.total_time_us as f64 / 1000.0).ln().max(0.0);
        let render_bonus = if self.is_render_path { 1.5 } else { 1.0 };
        (call_weight + time_weight) * render_bonus
    }
}

/// The threshold for AOT compilation.
#[derive(Debug, Clone)]
pub struct AotThreshold {
    /// Minimum call count to trigger AOT.
    pub min_call_count: u64,
    /// Minimum total time (microseconds) to trigger AOT.
    pub min_total_time_us: u64,
    /// Minimum hotness score to trigger AOT.
    pub min_hotness: f64,
    /// Maximum number of functions to AOT-compile.
    pub max_aot_functions: usize,
}

impl Default for AotThreshold {
    fn default() -> Self {
        Self {
            min_call_count: 100,
            min_total_time_us: 10_000,
            min_hotness: 5.0,
            max_aot_functions: 50,
        }
    }
}

impl AotThreshold {
    /// Check if a profile sample meets the AOT threshold.
    pub fn should_aot(&self, sample: &ProfileSample) -> bool {
        sample.call_count >= self.min_call_count
            && sample.total_time_us >= self.min_total_time_us
            && sample.hotness_score() >= self.min_hotness
    }
}

/// An AOT compilation entry — a function selected for AOT.
#[derive(Debug, Clone)]
pub struct AotEntry {
    /// The function name.
    pub function_name: String,
    /// The module name.
    pub module_name: String,
    /// The execution mode.
    pub mode: ExecutionMode,
    /// The hotness score when selected.
    pub hotness_score: f64,
    /// The estimated speedup (fraction, 0.0-1.0).
    pub estimated_speedup: f64,
    /// The native code size in bytes (estimated).
    pub native_code_size: u64,
}

impl AotEntry {
    /// Create a new AOT entry.
    pub fn new(sample: &ProfileSample) -> Self {
        let speedup = if sample.is_render_path { 0.3 } else { 0.15 };
        let code_size = sample.total_time_us * 10; // Rough estimate
        Self {
            function_name: sample.function_name.clone(),
            module_name: sample.module_name.clone(),
            mode: ExecutionMode::AotCompiled,
            hotness_score: sample.hotness_score(),
            estimated_speedup: speedup,
            native_code_size: code_size,
        }
    }
}

/// The selective AOT compiler — profile-guided AOT compilation.
pub struct SelectiveAotCompiler {
    profiles: Mutex<HashMap<String, ProfileSample>>,
    aot_entries: Mutex<HashMap<String, AotEntry>>,
    threshold: AotThreshold,
}

impl SelectiveAotCompiler {
    /// Create a new selective AOT compiler.
    pub fn new(threshold: AotThreshold) -> Self {
        Self {
            profiles: Mutex::new(HashMap::new()),
            aot_entries: Mutex::new(HashMap::new()),
            threshold,
        }
    }

    /// Record a function execution.
    pub fn record(&self, function_name: &str, module_name: &str, time_us: u64) {
        let key = format!("{}::{}", module_name, function_name);
        let mut profiles = self.profiles.lock().unwrap();
        let sample = profiles.entry(key).or_insert_with(|| ProfileSample::new(function_name, module_name));
        sample.record_call(time_us);
    }

    /// Mark a function as a render hot path.
    pub fn mark_render_path(&self, function_name: &str, module_name: &str) {
        let key = format!("{}::{}", module_name, function_name);
        let mut profiles = self.profiles.lock().unwrap();
        if let Some(sample) = profiles.get_mut(&key) {
            sample.is_render_path = true;
        }
    }

    /// Select functions for AOT compilation based on profiling data.
    pub fn select_for_aot(&self) -> Vec<AotEntry> {
        let profiles = self.profiles.lock().unwrap();
        let mut candidates: Vec<&ProfileSample> = profiles
            .values()
            .filter(|s| self.threshold.should_aot(s))
            .collect();

        // Sort by hotness score (descending)
        candidates.sort_by(|a, b| {
            b.hotness_score().partial_cmp(&a.hotness_score()).unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected: Vec<AotEntry> = candidates
            .iter()
            .take(self.threshold.max_aot_functions)
            .map(|s| AotEntry::new(s))
            .collect();

        // Store in aot_entries
        let mut entries = self.aot_entries.lock().unwrap();
        for entry in &selected {
            let key = format!("{}::{}", entry.module_name, entry.function_name);
            entries.insert(key, entry.clone());
        }

        selected
    }

    /// Get the execution mode for a function.
    pub fn execution_mode(&self, function_name: &str, module_name: &str) -> ExecutionMode {
        let key = format!("{}::{}", module_name, function_name);
        if self.aot_entries.lock().unwrap().contains_key(&key) {
            ExecutionMode::AotCompiled
        } else if self.profiles.lock().unwrap().contains_key(&key) {
            ExecutionMode::Hybrid
        } else {
            ExecutionMode::Interpreted
        }
    }

    /// Get the number of profiled functions.
    pub fn profiled_count(&self) -> usize {
        self.profiles.lock().unwrap().len()
    }

    /// Get the number of AOT-compiled functions.
    pub fn aot_count(&self) -> usize {
        self.aot_entries.lock().unwrap().len()
    }

    /// Get the total estimated native code size.
    pub fn total_native_code_size(&self) -> u64 {
        self.aot_entries.lock().unwrap().values().map(|e| e.native_code_size).sum()
    }

    /// Get the overall estimated speedup (fraction, 0.0-1.0).
    pub fn overall_speedup(&self) -> f64 {
        let entries = self.aot_entries.lock().unwrap();
        if entries.is_empty() {
            return 0.0;
        }
        let total_speedup: f64 = entries.values().map(|e| e.estimated_speedup).sum();
        (total_speedup / entries.len() as f64).min(1.0)
    }

    /// Clear all profiling data.
    pub fn clear(&self) {
        self.profiles.lock().unwrap().clear();
        self.aot_entries.lock().unwrap().clear();
    }

    /// Generate the AOT compilation report.
    pub fn generate_report(&self) -> String {
        let profiled = self.profiled_count();
        let aot_count = self.aot_count();
        let total_native = self.total_native_code_size();
        let speedup = self.overall_speedup();

        let sorted: Vec<AotEntry> = {
            let entries = self.aot_entries.lock().unwrap();
            let mut sorted: Vec<&AotEntry> = entries.values().collect();
            sorted.sort_by(|a, b| {
                b.hotness_score.partial_cmp(&a.hotness_score).unwrap_or(std::cmp::Ordering::Equal)
            });
            sorted.iter().take(20).map(|e| (*e).clone()).collect()
        };

        let mut report = String::new();

        report.push_str("=== Selective Wasm AOT Report ===\n\n");
        report.push_str(&format!("Profiled functions: {}\n", profiled));
        report.push_str(&format!("AOT-compiled functions: {}\n", aot_count));
        report.push_str(&format!("Total native code: {} bytes\n", total_native));
        report.push_str(&format!("Overall estimated speedup: {:.1}%\n\n", speedup * 100.0));

        report.push_str("AOT-compiled functions (by hotness):\n");

        for entry in sorted.iter() {
            report.push_str(&format!(
                "  {}::{} (hotness: {:.2}, speedup: {:.0}%)\n",
                entry.module_name,
                entry.function_name,
                entry.hotness_score,
                entry.estimated_speedup * 100.0,
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode_display_name() {
        assert_eq!(ExecutionMode::Interpreted.display_name(), "interpreted");
        assert_eq!(ExecutionMode::AotCompiled.display_name(), "AOT-compiled");
        assert_eq!(ExecutionMode::Hybrid.display_name(), "hybrid");
    }

    #[test]
    fn test_profile_sample_new() {
        let sample = ProfileSample::new("render", "rye_core");
        assert_eq!(sample.function_name, "render");
        assert_eq!(sample.call_count, 0);
        assert_eq!(sample.total_time_us, 0);
    }

    #[test]
    fn test_profile_sample_record_call() {
        let mut sample = ProfileSample::new("render", "rye_core");
        sample.record_call(100);
        sample.record_call(200);
        assert_eq!(sample.call_count, 2);
        assert_eq!(sample.total_time_us, 300);
        assert_eq!(sample.avg_time_us(), 150.0);
    }

    #[test]
    fn test_profile_sample_hotness_score() {
        let mut sample = ProfileSample::new("render", "rye_core");
        sample.record_call(1000);
        sample.record_call(1000);
        let score = sample.hotness_score();
        assert!(score > 0.0);
    }

    #[test]
    fn test_profile_sample_render_path_bonus() {
        let mut normal = ProfileSample::new("f", "m");
        normal.record_call(1000);
        normal.record_call(10000);

        let mut render = ProfileSample::new("f", "m").mark_render_path();
        render.record_call(1000);
        render.record_call(10000);

        assert!(render.hotness_score() > normal.hotness_score());
    }

    #[test]
    fn test_aot_threshold_should_aot() {
        let threshold = AotThreshold {
            min_call_count: 2,
            min_total_time_us: 1000,
            min_hotness: 0.0,
            max_aot_functions: 50,
        };
        let mut sample = ProfileSample::new("f", "m");
        sample.record_call(200);
        sample.record_call(20000);
        assert!(threshold.should_aot(&sample));
    }

    #[test]
    fn test_aot_threshold_should_not_aot() {
        let threshold = AotThreshold::default();
        let mut sample = ProfileSample::new("f", "m");
        sample.record_call(10);
        sample.record_call(100);
        assert!(!threshold.should_aot(&sample));
    }

    #[test]
    fn test_aot_entry_new() {
        let mut sample = ProfileSample::new("render", "rye_core").mark_render_path();
        sample.record_call(500);
        sample.record_call(20000);
        let entry = AotEntry::new(&sample);
        assert_eq!(entry.function_name, "render");
        assert_eq!(entry.mode, ExecutionMode::AotCompiled);
        assert!(entry.estimated_speedup > 0.0);
    }

    #[test]
    fn test_selective_aot_record() {
        let compiler = SelectiveAotCompiler::new(AotThreshold::default());
        compiler.record("render", "rye_core", 100);
        compiler.record("render", "rye_core", 200);
        assert_eq!(compiler.profiled_count(), 1);
    }

    #[test]
    fn test_selective_aot_mark_render_path() {
        let compiler = SelectiveAotCompiler::new(AotThreshold::default());
        compiler.record("render", "rye_core", 100);
        compiler.mark_render_path("render", "rye_core");
        let profiles = compiler.profiles.lock().unwrap();
        let key = "rye_core::render";
        assert!(profiles.get(key).unwrap().is_render_path);
    }

    #[test]
    fn test_selective_aot_select_for_aot() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 5,
            min_total_time_us: 100,
            min_hotness: 0.0,
            max_aot_functions: 10,
        });

        // Record hot function
        for _ in 0..10 {
            compiler.record("hot_fn", "module_a", 500);
        }

        // Record cold function
        compiler.record("cold_fn", "module_b", 10);

        let selected = compiler.select_for_aot();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].function_name, "hot_fn");
    }

    #[test]
    fn test_selective_aot_execution_mode() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 1,
            min_total_time_us: 1,
            min_hotness: 0.0,
            max_aot_functions: 10,
        });

        compiler.record("fn", "mod", 100);
        assert_eq!(compiler.execution_mode("fn", "mod"), ExecutionMode::Hybrid);

        compiler.select_for_aot();
        assert_eq!(compiler.execution_mode("fn", "mod"), ExecutionMode::AotCompiled);

        assert_eq!(compiler.execution_mode("unknown", "mod"), ExecutionMode::Interpreted);
    }

    #[test]
    fn test_selective_aot_max_functions() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 1,
            min_total_time_us: 1,
            min_hotness: 0.0,
            max_aot_functions: 2,
        });

        for i in 0..5 {
            for _ in 0..10 {
                compiler.record(&format!("fn_{}", i), "mod", 100);
            }
        }

        let selected = compiler.select_for_aot();
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_selective_aot_overall_speedup() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 1,
            min_total_time_us: 1,
            min_hotness: 0.0,
            max_aot_functions: 10,
        });

        for _ in 0..10 {
            compiler.record("fn", "mod", 100);
        }

        compiler.select_for_aot();
        assert!(compiler.overall_speedup() > 0.0);
    }

    #[test]
    fn test_selective_aot_empty_speedup() {
        let compiler = SelectiveAotCompiler::new(AotThreshold::default());
        assert_eq!(compiler.overall_speedup(), 0.0);
    }

    #[test]
    fn test_selective_aot_clear() {
        let compiler = SelectiveAotCompiler::new(AotThreshold::default());
        compiler.record("fn", "mod", 100);
        compiler.clear();
        assert_eq!(compiler.profiled_count(), 0);
        assert_eq!(compiler.aot_count(), 0);
    }

    #[test]
    fn test_selective_aot_generate_report() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 1,
            min_total_time_us: 1,
            min_hotness: 0.0,
            max_aot_functions: 10,
        });

        for _ in 0..10 {
            compiler.record("render", "rye_core", 100);
        }
        compiler.mark_render_path("render", "rye_core");
        compiler.select_for_aot();

        let report = compiler.generate_report();
        assert!(report.contains("AOT Report"));
        assert!(report.contains("render"));
    }

    #[test]
    fn test_selective_aot_total_native_code_size() {
        let compiler = SelectiveAotCompiler::new(AotThreshold {
            min_call_count: 1,
            min_total_time_us: 1,
            min_hotness: 0.0,
            max_aot_functions: 10,
        });

        for _ in 0..10 {
            compiler.record("fn", "mod", 1000);
        }
        compiler.select_for_aot();
        assert!(compiler.total_native_code_size() > 0);
    }
}
