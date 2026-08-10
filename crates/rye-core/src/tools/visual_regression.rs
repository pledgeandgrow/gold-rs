//! Goal 135: Visual regression testing.
//!
//! Snapshot-based visual regression testing. Compare rendered output against
//! baseline snapshots. Integrates with `rye test --visual`.

use std::collections::HashMap;
use std::path::PathBuf;

/// Visual regression test configuration.
#[derive(Debug, Clone)]
pub struct VisualRegressionConfig {
    /// Directory for baseline snapshots.
    pub baseline_dir: PathBuf,
    /// Directory for current screenshots.
    pub current_dir: PathBuf,
    /// Directory for diff images.
    pub diff_dir: PathBuf,
    /// Threshold for pixel difference (0.0 to 1.0).
    pub threshold: f32,
    /// Whether to update baselines automatically.
    pub update_baselines: bool,
}

impl Default for VisualRegressionConfig {
    fn default() -> Self {
        Self {
            baseline_dir: PathBuf::from(".rye/visual/baseline"),
            current_dir: PathBuf::from(".rye/visual/current"),
            diff_dir: PathBuf::from(".rye/visual/diff"),
            threshold: 0.1,
            update_baselines: false,
        }
    }
}

/// A visual regression test result.
#[derive(Debug, Clone)]
pub struct VisualTestResult {
    /// Test name.
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Number of pixels that differ.
    pub diff_pixels: usize,
    /// Total pixels compared.
    pub total_pixels: usize,
    /// Diff ratio (0.0 to 1.0).
    pub diff_ratio: f32,
    /// Path to the diff image (if failed).
    pub diff_image: Option<PathBuf>,
}

impl VisualTestResult {
    /// Whether the diff exceeds the threshold.
    pub fn exceeds_threshold(&self, threshold: f32) -> bool {
        self.diff_ratio > threshold
    }
}

/// Compare two images pixel by pixel.
///
/// Returns the number of differing pixels and the total pixels.
pub fn compare_images(
    baseline: &[u8],
    current: &[u8],
    width: usize,
    height: usize,
) -> (usize, usize) {
    let total = width * height;
    if baseline.len() < total * 4 || current.len() < total * 4 {
        return (0, total);
    }

    let mut diff_pixels = 0;
    for i in 0..total {
        let offset = i * 4;
        let br = baseline[offset];
        let bg = baseline[offset + 1];
        let bb = baseline[offset + 2];
        let ba = baseline[offset + 3];

        let cr = current[offset];
        let cg = current[offset + 1];
        let cb = current[offset + 2];
        let ca = current[offset + 3];

        // Simple per-channel comparison with tolerance
        let dr = if br >= cr { br - cr } else { cr - br };
        let dg = if bg >= cg { bg - cg } else { cg - bg };
        let db = if bb >= cb { bb - cb } else { cb - bb };
        let da = if ba >= ca { ba - ca } else { ca - ba };

        if dr > 10 || dg > 10 || db > 10 || da > 10 {
            diff_pixels += 1;
        }
    }

    (diff_pixels, total)
}

/// Generate a diff image highlighting differences.
pub fn generate_diff_image(
    baseline: &[u8],
    current: &[u8],
    width: usize,
    height: usize,
) -> Vec<u8> {
    let total = width * height;
    let mut diff = vec![0u8; total * 4];

    for i in 0..total {
        let offset = i * 4;
        if offset + 3 >= baseline.len() || offset + 3 >= current.len() {
            break;
        }

        let is_diff = {
            let dr = if baseline[offset] >= current[offset] {
                baseline[offset] - current[offset]
            } else {
                current[offset] - baseline[offset]
            };
            let dg = if baseline[offset + 1] >= current[offset + 1] {
                baseline[offset + 1] - current[offset + 1]
            } else {
                current[offset + 1] - baseline[offset + 1]
            };
            let db = if baseline[offset + 2] >= current[offset + 2] {
                baseline[offset + 2] - current[offset + 2]
            } else {
                current[offset + 2] - baseline[offset + 2]
            };
            dr > 10 || dg > 10 || db > 10
        };

        if is_diff {
            // Highlight differences in red
            diff[offset] = 255;
            diff[offset + 1] = 0;
            diff[offset + 2] = 0;
            diff[offset + 3] = 255;
        } else {
            // Keep original (dimmed)
            diff[offset] = current[offset] / 2;
            diff[offset + 1] = current[offset + 1] / 2;
            diff[offset + 2] = current[offset + 2] / 2;
            diff[offset + 3] = 255;
        }
    }

    diff
}

/// Test suite for visual regression.
pub struct VisualTestSuite {
    /// Configuration.
    pub config: VisualRegressionConfig,
    /// Test results.
    pub results: Vec<VisualTestResult>,
}

impl VisualTestSuite {
    /// Create a new visual test suite.
    pub fn new(config: VisualRegressionConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Record a test result.
    pub fn record(&mut self, result: VisualTestResult) {
        self.results.push(result);
    }

    /// Whether all tests passed.
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    /// Number of failed tests.
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    /// Generate a summary report.
    pub fn summary(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let mut report = format!(
            "=== Visual Regression Test Summary ===\n{} tests, {} passed, {} failed\n\n",
            total, passed, failed
        );

        for result in &self.results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            report.push_str(&format!(
                "  [{}] {} ({:.2}% diff)\n",
                status,
                result.name,
                result.diff_ratio * 100.0
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_regression_config() {
        let config = VisualRegressionConfig::default();
        assert_eq!(config.threshold, 0.1);
        assert!(!config.update_baselines);
    }

    #[test]
    fn test_compare_images_identical() {
        let img = vec![100u8; 400]; // 10x10 RGBA
        let (diff, total) = compare_images(&img, &img, 10, 10);
        assert_eq!(diff, 0);
        assert_eq!(total, 100);
    }

    #[test]
    fn test_compare_images_different() {
        let baseline = vec![100u8; 400];
        let current = vec![200u8; 400]; // All pixels differ significantly
        let (diff, total) = compare_images(&baseline, &current, 10, 10);
        assert_eq!(diff, 100);
        assert_eq!(total, 100);
    }

    #[test]
    fn test_compare_images_small_diff() {
        let baseline = vec![100u8; 400];
        let mut current = baseline.clone();
        // Small difference within tolerance (10)
        current[0] = 105;
        let (diff, _) = compare_images(&baseline, &current, 10, 10);
        assert_eq!(diff, 0);
    }

    #[test]
    fn test_generate_diff_image() {
        let baseline = vec![100u8; 400];
        let current = vec![200u8; 400];
        let diff = generate_diff_image(&baseline, &current, 10, 10);
        // First pixel should be red (diff)
        assert_eq!(diff[0], 255);
        assert_eq!(diff[1], 0);
        assert_eq!(diff[2], 0);
    }

    #[test]
    fn test_visual_test_result_threshold() {
        let result = VisualTestResult {
            name: "test1".to_string(),
            passed: false,
            diff_pixels: 50,
            total_pixels: 100,
            diff_ratio: 0.5,
            diff_image: None,
        };
        assert!(result.exceeds_threshold(0.1));
        assert!(!result.exceeds_threshold(0.6));
    }

    #[test]
    fn test_visual_test_suite() {
        let config = VisualRegressionConfig::default();
        let mut suite = VisualTestSuite::new(config);

        suite.record(VisualTestResult {
            name: "test1".to_string(),
            passed: true,
            diff_pixels: 0,
            total_pixels: 100,
            diff_ratio: 0.0,
            diff_image: None,
        });
        suite.record(VisualTestResult {
            name: "test2".to_string(),
            passed: false,
            diff_pixels: 50,
            total_pixels: 100,
            diff_ratio: 0.5,
            diff_image: Some(PathBuf::from("diff/test2.png")),
        });

        assert!(!suite.all_passed());
        assert_eq!(suite.failed_count(), 1);

        let summary = suite.summary();
        assert!(summary.contains("2 tests"));
        assert!(summary.contains("1 passed"));
        assert!(summary.contains("1 failed"));
        assert!(summary.contains("test2"));
    }
}
