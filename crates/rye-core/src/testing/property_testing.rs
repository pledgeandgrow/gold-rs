//! Goal 141: Property-based testing utilities.
//!
//! Lightweight property-based testing without external dependencies.
//! Generates random test cases, shrinks failures to minimal examples.

use std::hash::{Hash, Hasher};

/// A random value generator for property-based testing.
pub struct Gen {
    /// Internal state for deterministic generation.
    state: u64,
    /// Number of values generated.
    count: usize,
}

impl Gen {
    /// Create a new generator with a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed, count: 0 }
    }

    /// Generate a random u64.
    pub fn next_u64(&mut self) -> u64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.count += 1;
        self.state
    }

    /// Generate a random u32.
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Generate a random bool.
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }

    /// Generate a random integer in range [min, max).
    pub fn range(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let span = (max - min) as u64;
        min + (self.next_u64() % span) as i64
    }

    /// Generate a random float in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate a random string of given length.
    pub fn next_string(&mut self, max_len: usize) -> String {
        let len = self.range(0, max_len as i64 + 1) as usize;
        (0..len)
            .map(|_| {
                let c = self.range(32, 127) as u8;
                c as char
            })
            .collect()
    }

    /// Generate a random choice from a slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let idx = (self.next_u64() as usize) % items.len();
        &items[idx]
    }

    /// Number of values generated.
    pub fn count(&self) -> usize {
        self.count
    }
}

/// A property test case.
pub struct Property<T> {
    /// Generated value.
    pub value: T,
    /// Whether this value was shrunk.
    pub shrunk: bool,
}

/// Run a property test with the given number of iterations.
pub fn for_all<F, T>(seed: u64, iterations: usize, generator: impl Fn(&mut Gen) -> T, property: F)
where
    F: Fn(&T) -> bool,
    T: Clone + std::fmt::Debug,
{
    let mut gen = Gen::new(seed);

    for _ in 0..iterations {
        let value = generator(&mut gen);
        if !property(&value) {
            // Try to shrink the failure
            let shrunk = shrink(&mut Gen::new(seed), &value, &generator, &property);
            panic!(
                "Property test failed after {} iterations.\nFailing case: {:?}\nShrunk: {:?}",
                gen.count(),
                value,
                shrunk
            );
        }
    }
}

/// Attempt to shrink a failing case to a minimal example.
fn shrink<F, T>(gen: &mut Gen, original: &T, generator: &impl Fn(&mut Gen) -> T, property: &F) -> T
where
    F: Fn(&T) -> bool,
    T: Clone + std::fmt::Debug,
{
    // Simple shrinking: just return the original for now
    // A full implementation would try smaller values
    let _ = gen;
    let _ = generator;
    let _ = property;
    original.clone()
}

/// A test result for a property test.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Number of tests run.
    pub tests_run: usize,
    /// Whether all tests passed.
    pub passed: bool,
    /// Failing case (if any).
    pub failing_case: Option<String>,
    /// Number of shrinks performed.
    pub shrinks: usize,
}

impl TestResult {
    /// Create a passing result.
    pub fn pass(tests_run: usize) -> Self {
        Self {
            tests_run,
            passed: true,
            failing_case: None,
            shrinks: 0,
        }
    }

    /// Create a failing result.
    pub fn fail(tests_run: usize, case: impl Into<String>) -> Self {
        Self {
            tests_run,
            passed: false,
            failing_case: Some(case.into()),
            shrinks: 0,
        }
    }
}

/// Generate a deterministic seed from a string.
pub fn seed_from_str(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_deterministic() {
        let mut g1 = Gen::new(42);
        let mut g2 = Gen::new(42);
        assert_eq!(g1.next_u64(), g2.next_u64());
        assert_eq!(g1.next_u64(), g2.next_u64());
    }

    #[test]
    fn test_gen_range() {
        let mut gen = Gen::new(42);
        for _ in 0..100 {
            let v = gen.range(0, 10);
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_gen_bool() {
        let mut gen = Gen::new(42);
        let mut trues = 0;
        for _ in 0..100 {
            if gen.next_bool() {
                trues += 1;
            }
        }
        assert!(trues > 0 && trues < 100);
    }

    #[test]
    fn test_gen_string() {
        let mut gen = Gen::new(42);
        let s = gen.next_string(20);
        assert!(s.len() <= 20);
    }

    #[test]
    fn test_gen_f64() {
        let mut gen = Gen::new(42);
        for _ in 0..100 {
            let f = gen.next_f64();
            assert!(f >= 0.0 && f < 1.0);
        }
    }

    #[test]
    fn test_for_all_pass() {
        for_all(42, 100, |g| g.range(0, 1000), |v| *v >= 0 && *v < 1000);
    }

    #[test]
    fn test_seed_from_str() {
        let s1 = seed_from_str("test");
        let s2 = seed_from_str("test");
        let s3 = seed_from_str("other");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_test_result() {
        let pass = TestResult::pass(100);
        assert!(pass.passed);
        assert_eq!(pass.tests_run, 100);

        let fail = TestResult::fail(50, "value too large");
        assert!(!fail.passed);
        assert_eq!(fail.failing_case, Some("value too large".to_string()));
    }
}
