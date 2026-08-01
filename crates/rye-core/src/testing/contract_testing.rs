//! Goal 145: Contract testing for server actions.
//!
//! Verify that server actions and their client stubs agree on the contract
//! (argument types, return types, error types). Catch breaking changes early.

use std::collections::HashMap;

/// A server action contract.
#[derive(Debug, Clone)]
pub struct ActionContract {
    /// Action name.
    pub name: String,
    /// Argument types (in order).
    pub args: Vec<ContractType>,
    /// Return type.
    pub return_type: ContractType,
    /// Error type (if Result).
    pub error_type: Option<ContractType>,
    /// Whether the action is async.
    pub is_async: bool,
    /// HTTP method (POST by default).
    pub method: String,
    /// Whether the action requires authentication.
    pub requires_auth: bool,
}

/// A type in the contract system.
#[derive(Debug, Clone, PartialEq)]
pub enum ContractType {
    /// String type.
    String,
    /// Integer type.
    Int,
    /// Float type.
    Float,
    /// Boolean type.
    Bool,
    /// Unit type (no value).
    Unit,
    /// Vec of a type.
    Vec(Box<ContractType>),
    /// Option of a type.
    Option(Box<ContractType>),
    /// Custom named type.
    Custom(String),
}

impl ContractType {
    /// Get the type name as a string.
    pub fn type_name(&self) -> String {
        match self {
            ContractType::String => "String".to_string(),
            ContractType::Int => "i64".to_string(),
            ContractType::Float => "f64".to_string(),
            ContractType::Bool => "bool".to_string(),
            ContractType::Unit => "()".to_string(),
            ContractType::Vec(inner) => format!("Vec<{}>", inner.type_name()),
            ContractType::Option(inner) => format!("Option<{}>", inner.type_name()),
            ContractType::Custom(name) => name.clone(),
        }
    }

    /// Whether this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(self, ContractType::String | ContractType::Int | ContractType::Float | ContractType::Bool | ContractType::Unit)
    }
}

/// A contract violation.
#[derive(Debug, Clone)]
pub struct ContractViolation {
    /// Action name.
    pub action: String,
    /// Violation type.
    pub violation: ViolationKind,
    /// Description.
    pub description: String,
}

/// Kind of contract violation.
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationKind {
    /// Argument count mismatch.
    ArgCountMismatch,
    /// Argument type mismatch.
    ArgTypeMismatch,
    /// Return type mismatch.
    ReturnTypeMismatch,
    /// Error type mismatch.
    ErrorTypeMismatch,
    /// Method mismatch.
    MethodMismatch,
    /// Auth requirement mismatch.
    AuthMismatch,
    /// Missing action on one side.
    MissingAction,
}

/// Contract test result.
#[derive(Debug, Clone)]
pub struct ContractTestResult {
    /// Whether the contract test passed.
    pub passed: bool,
    /// Violations found.
    pub violations: Vec<ContractViolation>,
    /// Actions tested.
    pub actions_tested: usize,
}

impl ContractTestResult {
    /// Whether there are any violations.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Generate a summary.
    pub fn summary(&self) -> String {
        if self.passed {
            format!("Contract test PASSED: {} actions verified, 0 violations", self.actions_tested)
        } else {
            format!(
                "Contract test FAILED: {} actions tested, {} violations",
                self.actions_tested, self.violations.len()
            )
        }
    }
}

/// Compare two action contracts for compatibility.
pub fn compare_contracts(server: &ActionContract, client: &ActionContract) -> Vec<ContractViolation> {
    let mut violations = Vec::new();

    // Check argument count
    if server.args.len() != client.args.len() {
        violations.push(ContractViolation {
            action: server.name.clone(),
            violation: ViolationKind::ArgCountMismatch,
            description: format!(
                "Server expects {} args, client sends {}",
                server.args.len(), client.args.len()
            ),
        });
    }

    // Check argument types
    for (i, (s_arg, c_arg)) in server.args.iter().zip(client.args.iter()).enumerate() {
        if s_arg != c_arg {
            violations.push(ContractViolation {
                action: server.name.clone(),
                violation: ViolationKind::ArgTypeMismatch,
                description: format!(
                    "Arg {}: server expects {}, client sends {}",
                    i, s_arg.type_name(), c_arg.type_name()
                ),
            });
        }
    }

    // Check return type
    if server.return_type != client.return_type {
        violations.push(ContractViolation {
            action: server.name.clone(),
            violation: ViolationKind::ReturnTypeMismatch,
            description: format!(
                "Server returns {}, client expects {}",
                server.return_type.type_name(), client.return_type.type_name()
            ),
        });
    }

    // Check error type
    if server.error_type != client.error_type {
        violations.push(ContractViolation {
            action: server.name.clone(),
            violation: ViolationKind::ErrorTypeMismatch,
            description: format!(
                "Server errors with {:?}, client expects {:?}",
                server.error_type, client.error_type
            ),
        });
    }

    // Check method
    if server.method != client.method {
        violations.push(ContractViolation {
            action: server.name.clone(),
            violation: ViolationKind::MethodMismatch,
            description: format!(
                "Server uses {}, client uses {}",
                server.method, client.method
            ),
        });
    }

    // Check auth requirement
    if server.requires_auth != client.requires_auth {
        violations.push(ContractViolation {
            action: server.name.clone(),
            violation: ViolationKind::AuthMismatch,
            description: format!(
                "Server requires_auth={}, client requires_auth={}",
                server.requires_auth, client.requires_auth
            ),
        });
    }

    violations
}

/// Run contract tests between server and client contract registries.
pub fn test_contracts(
    server_contracts: &HashMap<String, ActionContract>,
    client_contracts: &HashMap<String, ActionContract>,
) -> ContractTestResult {
    let mut all_violations = Vec::new();
    let mut actions_tested = 0;

    // Check all server actions have matching client contracts
    for (name, server_contract) in server_contracts {
        actions_tested += 1;
        if let Some(client_contract) = client_contracts.get(name) {
            let violations = compare_contracts(server_contract, client_contract);
            all_violations.extend(violations);
        } else {
            all_violations.push(ContractViolation {
                action: name.clone(),
                violation: ViolationKind::MissingAction,
                description: "Action exists on server but not on client".to_string(),
            });
        }
    }

    // Check for client actions missing on server
    for name in client_contracts.keys() {
        if !server_contracts.contains_key(name) {
            all_violations.push(ContractViolation {
                action: name.clone(),
                violation: ViolationKind::MissingAction,
                description: "Action exists on client but not on server".to_string(),
            });
        }
    }

    let passed = all_violations.is_empty();

    ContractTestResult {
        passed,
        violations: all_violations,
        actions_tested,
    }
}

/// Helper to create a simple action contract.
pub fn action(
    name: impl Into<String>,
    args: Vec<ContractType>,
    return_type: ContractType,
) -> ActionContract {
    ActionContract {
        name: name.into(),
        args,
        return_type,
        error_type: None,
        is_async: true,
        method: "POST".to_string(),
        requires_auth: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_type_names() {
        assert_eq!(ContractType::String.type_name(), "String");
        assert_eq!(ContractType::Int.type_name(), "i64");
        assert_eq!(ContractType::Vec(Box::new(ContractType::String)).type_name(), "Vec<String>");
        assert_eq!(ContractType::Option(Box::new(ContractType::Int)).type_name(), "Option<i64>");
    }

    #[test]
    fn test_contract_type_is_primitive() {
        assert!(ContractType::String.is_primitive());
        assert!(ContractType::Bool.is_primitive());
        assert!(!ContractType::Vec(Box::new(ContractType::String)).is_primitive());
    }

    #[test]
    fn test_compare_contracts_match() {
        let server = action("get_user", vec![ContractType::String], ContractType::String);
        let client = action("get_user", vec![ContractType::String], ContractType::String);
        let violations = compare_contracts(&server, &client);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_compare_contracts_arg_mismatch() {
        let server = action("get_user", vec![ContractType::String], ContractType::String);
        let client = action("get_user", vec![ContractType::Int], ContractType::String);
        let violations = compare_contracts(&server, &client);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation, ViolationKind::ArgTypeMismatch);
    }

    #[test]
    fn test_compare_contracts_arg_count() {
        let server = action("get_user", vec![ContractType::String], ContractType::String);
        let client = action("get_user", vec![], ContractType::String);
        let violations = compare_contracts(&server, &client);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation, ViolationKind::ArgCountMismatch);
    }

    #[test]
    fn test_compare_contracts_return_mismatch() {
        let server = action("get_user", vec![ContractType::String], ContractType::String);
        let client = action("get_user", vec![ContractType::String], ContractType::Int);
        let violations = compare_contracts(&server, &client);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation, ViolationKind::ReturnTypeMismatch);
    }

    #[test]
    fn test_test_contracts_pass() {
        let mut server = HashMap::new();
        server.insert("action1".to_string(), action("action1", vec![ContractType::String], ContractType::Bool));
        let mut client = HashMap::new();
        client.insert("action1".to_string(), action("action1", vec![ContractType::String], ContractType::Bool));

        let result = test_contracts(&server, &client);
        assert!(result.passed);
        assert_eq!(result.actions_tested, 1);
    }

    #[test]
    fn test_test_contracts_missing_on_client() {
        let mut server = HashMap::new();
        server.insert("action1".to_string(), action("action1", vec![], ContractType::Unit));
        let client = HashMap::new();

        let result = test_contracts(&server, &client);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].violation, ViolationKind::MissingAction);
    }

    #[test]
    fn test_test_contracts_missing_on_server() {
        let server = HashMap::new();
        let mut client = HashMap::new();
        client.insert("action1".to_string(), action("action1", vec![], ContractType::Unit));

        let result = test_contracts(&server, &client);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.violation == ViolationKind::MissingAction));
    }

    #[test]
    fn test_contract_summary_pass() {
        let result = ContractTestResult {
            passed: true,
            violations: vec![],
            actions_tested: 5,
        };
        let summary = result.summary();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("5 actions"));
    }

    #[test]
    fn test_contract_summary_fail() {
        let result = ContractTestResult {
            passed: false,
            violations: vec![ContractViolation {
                action: "test".to_string(),
                violation: ViolationKind::ArgTypeMismatch,
                description: "mismatch".to_string(),
            }],
            actions_tested: 3,
        };
        let summary = result.summary();
        assert!(summary.contains("FAILED"));
        assert!(summary.contains("1 violations"));
    }
}
