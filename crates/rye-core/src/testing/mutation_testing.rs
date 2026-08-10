//! Goal 144: Mutation testing support.
//!
//! Mutation testing utilities — inject small code changes (mutants) and
//! verify that existing tests catch them. Helps measure test effectiveness.

use std::collections::HashMap;

/// A mutation operator.
#[derive(Debug, Clone, PartialEq)]
pub enum Mutant {
    /// Replace a binary operator.
    BinaryOp {
        original: String,
        replacement: String,
    },
    /// Replace a comparison operator.
    ComparisonOp {
        original: String,
        replacement: String,
    },
    /// Replace a boolean literal.
    BooleanLiteral { from: bool, to: bool },
    /// Replace an integer literal.
    IntLiteral { from: i64, to: i64 },
    /// Remove a statement.
    RemoveStatement,
    /// Negate a condition.
    NegateCondition,
    /// Remove a function call argument.
    RemoveArgument,
}

impl Mutant {
    /// Get a description of the mutation.
    pub fn description(&self) -> String {
        match self {
            Mutant::BinaryOp {
                original,
                replacement,
            } => {
                format!("Replace {} with {}", original, replacement)
            }
            Mutant::ComparisonOp {
                original,
                replacement,
            } => {
                format!("Replace {} with {}", original, replacement)
            }
            Mutant::BooleanLiteral { from, to } => {
                format!("Replace {} with {}", from, to)
            }
            Mutant::IntLiteral { from, to } => {
                format!("Replace {} with {}", from, to)
            }
            Mutant::RemoveStatement => "Remove statement".to_string(),
            Mutant::NegateCondition => "Negate condition".to_string(),
            Mutant::RemoveArgument => "Remove function argument".to_string(),
        }
    }
}

/// A mutation test result.
#[derive(Debug, Clone)]
pub struct MutationResult {
    /// The mutant that was applied.
    pub mutant: Mutant,
    /// Location in the source (file:line).
    pub location: String,
    /// Whether the mutant was killed (caught by tests).
    pub killed: bool,
    /// Test that killed the mutant (if killed).
    pub killed_by: Option<String>,
}

/// Mutation test summary.
#[derive(Debug, Clone)]
pub struct MutationSummary {
    /// Total mutants generated.
    pub total: usize,
    /// Mutants killed by tests.
    pub killed: usize,
    /// Mutants that survived (not caught by tests).
    pub survived: usize,
    /// Mutants that caused a timeout.
    pub timeout: usize,
    /// Mutation score (killed / total).
    pub score: f64,
}

impl MutationSummary {
    /// Create a summary from results.
    pub fn from_results(results: &[MutationResult]) -> Self {
        let total = results.len();
        let killed = results.iter().filter(|r| r.killed).count();
        let survived = total - killed;

        Self {
            total,
            killed,
            survived,
            timeout: 0,
            score: if total > 0 {
                killed as f64 / total as f64
            } else {
                0.0
            },
        }
    }

    /// Generate a report string.
    pub fn report(&self) -> String {
        format!(
            "Mutation Testing Summary:\n  Total: {}\n  Killed: {}\n  Survived: {}\n  Score: {:.1}%",
            self.total,
            self.killed,
            self.survived,
            self.score * 100.0
        )
    }
}

/// Generate common binary operator mutations.
pub fn binary_op_mutations(op: &str) -> Vec<Mutant> {
    let replacements = match op {
        "+" => vec!["-", "*"],
        "-" => vec!["+", "*"],
        "*" => vec!["+", "/"],
        "/" => vec!["*", "%"],
        "%" => vec!["/", "*"],
        _ => return Vec::new(),
    };

    replacements
        .iter()
        .map(|r| Mutant::BinaryOp {
            original: op.to_string(),
            replacement: r.to_string(),
        })
        .collect()
}

/// Generate common comparison operator mutations.
pub fn comparison_op_mutations(op: &str) -> Vec<Mutant> {
    let replacements = match op {
        "==" => vec!["!=", ">=", "<="],
        "!=" => vec!["==", ">=", "<="],
        ">" => vec!["<", ">=", "<="],
        "<" => vec![">", ">=", "<="],
        ">=" => vec!["<", ">", "<="],
        "<=" => vec![">", "<", ">="],
        _ => return Vec::new(),
    };

    replacements
        .iter()
        .map(|r| Mutant::ComparisonOp {
            original: op.to_string(),
            replacement: r.to_string(),
        })
        .collect()
}

/// Generate boolean literal mutations.
pub fn boolean_mutations(value: bool) -> Vec<Mutant> {
    vec![Mutant::BooleanLiteral {
        from: value,
        to: !value,
    }]
}

/// Generate integer literal mutations.
pub fn int_literal_mutations(value: i64) -> Vec<Mutant> {
    let mut mutants = Vec::new();
    mutants.push(Mutant::IntLiteral { from: value, to: 0 });
    mutants.push(Mutant::IntLiteral {
        from: value,
        to: value + 1,
    });
    if value != 0 {
        mutants.push(Mutant::IntLiteral {
            from: value,
            to: value - 1,
        });
    }
    mutants
}

/// Mutation test configuration.
#[derive(Debug, Clone)]
pub struct MutationConfig {
    /// Maximum number of mutants to generate.
    pub max_mutants: usize,
    /// Timeout per mutant in milliseconds.
    pub timeout_ms: u32,
    /// Whether to generate boolean mutations.
    pub bool_mutations: bool,
    /// Whether to generate integer mutations.
    pub int_mutations: bool,
    /// Whether to generate operator mutations.
    pub operator_mutations: bool,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            max_mutants: 100,
            timeout_ms: 5000,
            bool_mutations: true,
            int_mutations: true,
            operator_mutations: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_mutations() {
        let mutants = binary_op_mutations("+");
        assert_eq!(mutants.len(), 2);
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::BinaryOp { replacement, .. } if replacement == "-")));
    }

    #[test]
    fn test_binary_op_mutations_unknown() {
        let mutants = binary_op_mutations("^");
        assert_eq!(mutants.len(), 0);
    }

    #[test]
    fn test_comparison_op_mutations() {
        let mutants = comparison_op_mutations("==");
        assert_eq!(mutants.len(), 3);
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::ComparisonOp { replacement, .. } if replacement == "!=")));
    }

    #[test]
    fn test_boolean_mutations() {
        let mutants = boolean_mutations(true);
        assert_eq!(mutants.len(), 1);
        assert!(matches!(
            mutants[0],
            Mutant::BooleanLiteral {
                from: true,
                to: false
            }
        ));
    }

    #[test]
    fn test_int_literal_mutations() {
        let mutants = int_literal_mutations(42);
        assert_eq!(mutants.len(), 3);
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::IntLiteral { to: 0, .. })));
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::IntLiteral { to: 43, .. })));
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::IntLiteral { to: 41, .. })));
    }

    #[test]
    fn test_int_literal_mutations_zero() {
        let mutants = int_literal_mutations(0);
        assert_eq!(mutants.len(), 2); // 0 and 1, no -1 since value == 0
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::IntLiteral { to: 0, .. })));
        assert!(mutants
            .iter()
            .any(|m| matches!(m, Mutant::IntLiteral { to: 1, .. })));
    }

    #[test]
    fn test_mutation_summary() {
        let results = vec![
            MutationResult {
                mutant: Mutant::RemoveStatement,
                location: "file.rs:10".to_string(),
                killed: true,
                killed_by: Some("test_basic".to_string()),
            },
            MutationResult {
                mutant: Mutant::NegateCondition,
                location: "file.rs:20".to_string(),
                killed: false,
                killed_by: None,
            },
            MutationResult {
                mutant: Mutant::BooleanLiteral {
                    from: true,
                    to: false,
                },
                location: "file.rs:30".to_string(),
                killed: true,
                killed_by: Some("test_edge".to_string()),
            },
        ];
        let summary = MutationSummary::from_results(&results);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.killed, 2);
        assert_eq!(summary.survived, 1);
        assert!((summary.score - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_mutation_summary_empty() {
        let summary = MutationSummary::from_results(&[]);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.score, 0.0);
    }

    #[test]
    fn test_mutation_summary_report() {
        let results = vec![
            MutationResult {
                mutant: Mutant::RemoveStatement,
                location: "x".to_string(),
                killed: true,
                killed_by: None,
            },
            MutationResult {
                mutant: Mutant::RemoveStatement,
                location: "y".to_string(),
                killed: false,
                killed_by: None,
            },
        ];
        let summary = MutationSummary::from_results(&results);
        let report = summary.report();
        assert!(report.contains("Total: 2"));
        assert!(report.contains("Killed: 1"));
        assert!(report.contains("Survived: 1"));
        assert!(report.contains("50.0%"));
    }

    #[test]
    fn test_mutant_description() {
        let m = Mutant::BinaryOp {
            original: "+".to_string(),
            replacement: "-".to_string(),
        };
        assert!(m.description().contains("Replace"));
        assert!(m.description().contains("+"));
        assert!(m.description().contains("-"));
    }

    #[test]
    fn test_mutation_config_default() {
        let config = MutationConfig::default();
        assert_eq!(config.max_mutants, 100);
        assert!(config.bool_mutations);
        assert!(config.int_mutations);
        assert!(config.operator_mutations);
    }
}
