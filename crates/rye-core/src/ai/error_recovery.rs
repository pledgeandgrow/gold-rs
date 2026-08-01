//! AI-friendly error recovery suggestions — Goal 162.
//!
//! Extends error codes with step-by-step recovery instructions.
//! AI agents can follow these steps to fix errors automatically.

use crate::error_codes::{self, ErrorCode};

/// A recovery step in a fix sequence.
#[derive(Debug, Clone)]
pub struct RecoveryStep {
    /// Step number (1-indexed).
    pub step: usize,
    /// Action description.
    pub action: String,
    /// Code example for this step (if applicable).
    pub code_example: Option<String>,
    /// What to verify after this step.
    pub verify: String,
}

/// A complete recovery plan for an error code.
#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    /// Error code (e.g. "R802").
    pub error_code: String,
    /// Error message.
    pub error_message: String,
    /// Steps to fix the error.
    pub steps: Vec<RecoveryStep>,
    /// Common mistakes during recovery.
    pub common_mistakes: Vec<String>,
    /// Alternative approaches if the main fix doesn't apply.
    pub alternatives: Vec<String>,
}

impl RecoveryPlan {
    /// Format as human-readable text.
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Recovery Plan for {} — {}\n\n", self.error_code, self.error_message));
        out.push_str("Steps:\n");
        for step in &self.steps {
            out.push_str(&format!("  {}. {}\n", step.step, step.action));
            if let Some(code) = &step.code_example {
                for line in code.lines() {
                    out.push_str(&format!("     {}\n", line));
                }
            }
            out.push_str(&format!("     Verify: {}\n", step.verify));
        }

        if !self.common_mistakes.is_empty() {
            out.push_str("\nCommon mistakes:\n");
            for m in &self.common_mistakes {
                out.push_str(&format!("  - {}\n", m));
            }
        }

        if !self.alternatives.is_empty() {
            out.push_str("\nAlternatives:\n");
            for a in &self.alternatives {
                out.push_str(&format!("  - {}\n", a));
            }
        }

        out
    }

    /// Format as JSON for AI agent consumption.
    pub fn format_json(&self) -> String {
        let steps: Vec<String> = self
            .steps
            .iter()
            .map(|s| {
                let code = s
                    .code_example
                    .as_ref()
                    .map(|c| format!(",\"code_example\":\"{}\"", json_escape(c)))
                    .unwrap_or_default();
                format!(
                    r#"{{"step":{},"action":"{}","verify":"{}"{} }}"#,
                    s.step,
                    json_escape(&s.action),
                    json_escape(&s.verify),
                    code
                )
            })
            .collect();

        let mistakes: Vec<String> = self
            .common_mistakes
            .iter()
            .map(|m| format!("\"{}\"", json_escape(m)))
            .collect();

        let alternatives: Vec<String> = self
            .alternatives
            .iter()
            .map(|a| format!("\"{}\"", json_escape(a)))
            .collect();

        format!(
            r#"{{"error_code":"{}","error_message":"{}","steps":[{}],"common_mistakes":[{}],"alternatives":[{}]}}"#,
            json_escape(&self.error_code),
            json_escape(&self.error_message),
            steps.join(","),
            mistakes.join(","),
            alternatives.join(",")
        )
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Get a recovery plan for an error code.
pub fn get_recovery_plan(code: &str) -> Option<RecoveryPlan> {
    let error_code = error_codes::lookup(code)?;
    let plan = build_plan(error_code);
    Some(plan)
}

/// Build a recovery plan from an error code entry.
fn build_plan(ec: &ErrorCode) -> RecoveryPlan {
    match ec.code {
        "R800" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Check the Props struct definition to find the expected type".to_string(),
                    code_example: Some("#[derive(Props)]\nstruct ButtonProps {\n    label: String,  // expected type\n}".to_string()),
                    verify: "You know what type the prop expects".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Convert the value to the expected type".to_string(),
                    code_example: Some("// If prop expects String but you have &str:\nButton { label: \"Hello\".to_string() }\n\n// If prop expects Signal<T> but you have T:\nButton { count: Signal::new(5) }".to_string()),
                    verify: "The value type matches the prop type".to_string(),
                },
                RecoveryStep {
                    step: 3,
                    action: "Rebuild and verify the error is gone".to_string(),
                    code_example: None,
                    verify: "cargo build succeeds without R800".to_string(),
                },
            ],
            common_mistakes: vec![
                "Using &str where String is expected — add .to_string()".to_string(),
                "Forgetting Signal::new() wrapper when prop expects Signal<T>".to_string(),
                "Passing a number literal where a Signal is expected".to_string(),
            ],
            alternatives: vec![
                "If you control the Props struct, consider changing the prop type to be more ergonomic (e.g. impl Into<String>)".to_string(),
            ],
        },
        "R801" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Find the event handler closure in the template".to_string(),
                    code_example: Some("// Wrong:\nbutton { onclick: |_| count.set(count.get() + 1) }".to_string()),
                    verify: "You found the closure that's missing 'move'".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Add 'move' keyword before the closure".to_string(),
                    code_example: Some("// Right:\nbutton { onclick: move |_| count.set(count.get() + 1) }".to_string()),
                    verify: "The closure starts with 'move |'".to_string(),
                },
            ],
            common_mistakes: vec![
                "Adding 'move' to closures that don't capture anything (harmless but unnecessary)".to_string(),
            ],
            alternatives: vec![],
        },
        "R802" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Find where the Signal is used in the template without .get()".to_string(),
                    code_example: Some("// Wrong:\np { \"Count: \" {count} }".to_string()),
                    verify: "You found the Signal being used directly".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Add .get() to read the signal value".to_string(),
                    code_example: Some("// Right:\np { \"Count: \" {count.get()} }".to_string()),
                    verify: "Every Signal usage in the template has .get()".to_string(),
                },
                RecoveryStep {
                    step: 3,
                    action: "Check for other Signal usages in the same component".to_string(),
                    code_example: None,
                    verify: "All Signal reads use .get()".to_string(),
                },
            ],
            common_mistakes: vec![
                "Calling .get() on a Memo (Memos also have .get() — this is correct, not a mistake)".to_string(),
                "Using .get() inside a Memo closure is correct — don't remove it".to_string(),
            ],
            alternatives: vec![],
        },
        "R803" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Find the direct assignment to a Signal".to_string(),
                    code_example: Some("// Wrong:\ncount = count + 1".to_string()),
                    verify: "You found the assignment".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Replace assignment with .set() call".to_string(),
                    code_example: Some("// Right:\ncount.set(count.get() + 1)".to_string()),
                    verify: "The signal is updated via .set(), not =".to_string(),
                },
            ],
            common_mistakes: vec![
                "Using .set() inside a render function — this causes R300. Only set in event handlers or effects".to_string(),
            ],
            alternatives: vec![],
        },
        "R804" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Rename the component function to PascalCase".to_string(),
                    code_example: Some("// Wrong: fn my_button() { ... }\n// Right: fn MyButton() { ... }".to_string()),
                    verify: "The function name starts with an uppercase letter".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Update all template usages to use the new name".to_string(),
                    code_example: Some("// In templates:\nMyButton { label: \"OK\" }".to_string()),
                    verify: "All references use the PascalCase name".to_string(),
                },
            ],
            common_mistakes: vec![
                "Forgetting to update template usages after renaming".to_string(),
            ],
            alternatives: vec![],
        },
        "R805" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Add #[component] attribute above the function".to_string(),
                    code_example: Some("#[component]\nfn MyComponent(props: MyComponentProps) {\n    template! { div { \"Hello\" } }\n}".to_string()),
                    verify: "The function has #[component] directly above it".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Ensure the function returns an Element (via template!)".to_string(),
                    code_example: None,
                    verify: "The function body contains template! { ... }".to_string(),
                },
            ],
            common_mistakes: vec![
                "Putting #[component] on a non-function item".to_string(),
                "Having other attributes between #[component] and the function".to_string(),
            ],
            alternatives: vec![],
        },
        "R806" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Identify the effect that's computing a value".to_string(),
                    code_example: Some("// Wrong:\nuse_effect(move || {\n    derived.set(a.get() + b.get());\n});".to_string()),
                    verify: "You found an effect that sets a signal based on other signals".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Replace the effect with a Memo".to_string(),
                    code_example: Some("// Right:\nlet derived = Memo::new(move || a.get() + b.get());".to_string()),
                    verify: "The computation is a Memo, not an effect".to_string(),
                },
                RecoveryStep {
                    step: 3,
                    action: "Update all usages of the old signal to use the Memo".to_string(),
                    code_example: Some("// Use derived.get() just like a Signal".to_string()),
                    verify: "No references to the old signal remain".to_string(),
                },
            ],
            common_mistakes: vec![
                "Using Memo for side effects — Memo is for computation only".to_string(),
            ],
            alternatives: vec![
                "If you need to trigger a side effect when derived value changes, use use_effect with the Memo as a dependency".to_string(),
            ],
        },
        "R808" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Identify the prop being passed through multiple layers".to_string(),
                    code_example: Some("// Wrong: Parent -> Middle -> Child -> GrandChild all receive 'theme'".to_string()),
                    verify: "You found a prop passed through 3+ component layers".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Use provide_context in the top-level component".to_string(),
                    code_example: Some("// In parent:\nprovide_context(theme.clone());".to_string()),
                    verify: "The value is provided at the top level".to_string(),
                },
                RecoveryStep {
                    step: 3,
                    action: "Use use_context in the leaf component that needs the value".to_string(),
                    code_example: Some("// In grandchild:\nlet theme = use_context::<Theme>();".to_string()),
                    verify: "The value is accessed via use_context, not props".to_string(),
                },
                RecoveryStep {
                    step: 4,
                    action: "Remove the prop from intermediate components".to_string(),
                    code_example: None,
                    verify: "Intermediate components no longer receive the prop".to_string(),
                },
            ],
            common_mistakes: vec![
                "Providing context inside a component that might unmount — context is lost on unmount".to_string(),
                "Using use_context before provide_context has run (context is available to children only)".to_string(),
            ],
            alternatives: vec![],
        },
        "R809" => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: "Find the raw async call (tokio::spawn or async block)".to_string(),
                    code_example: Some("// Wrong:\ntokio::spawn(async { fetch_data().await });".to_string()),
                    verify: "You found async work not using use_resource".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Replace with use_resource".to_string(),
                    code_example: Some("// Right:\nlet data = use_resource(move || async { fetch_data().await });".to_string()),
                    verify: "The async work is wrapped in use_resource".to_string(),
                },
                RecoveryStep {
                    step: 3,
                    action: "Use Suspense to handle the loading state".to_string(),
                    code_example: Some("Suspense {\n    fallback: || Loading {},\n    children: || DataView { data: data.get() },\n}".to_string()),
                    verify: "Loading state is handled with Suspense".to_string(),
                },
            ],
            common_mistakes: vec![
                "Forgetting to handle the error case — use ErrorBoundary around Suspense".to_string(),
            ],
            alternatives: vec![],
        },
        _ => RecoveryPlan {
            error_code: ec.code.to_string(),
            error_message: ec.message.to_string(),
            steps: vec![
                RecoveryStep {
                    step: 1,
                    action: format!("Read the error suggestion: {}", ec.suggestion),
                    code_example: Some(ec.correct_example.to_string()),
                    verify: "The suggested fix has been applied".to_string(),
                },
                RecoveryStep {
                    step: 2,
                    action: "Rebuild and verify the error is resolved".to_string(),
                    code_example: None,
                    verify: "cargo build succeeds".to_string(),
                },
            ],
            common_mistakes: ec.common_causes.iter().map(|c| c.to_string()).collect(),
            alternatives: vec![],
        },
    }
}

/// Get recovery plans for multiple error codes at once.
pub fn get_recovery_plans(codes: &[&str]) -> Vec<RecoveryPlan> {
    codes
        .iter()
        .filter_map(|c| get_recovery_plan(c))
        .collect()
}

/// Format recovery plans as a JSON array.
pub fn format_plans_json(plans: &[RecoveryPlan]) -> String {
    let entries: Vec<String> = plans.iter().map(|p| p.format_json()).collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recovery_plan_r802() {
        let plan = get_recovery_plan("R802");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.error_code, "R802");
        assert!(!plan.steps.is_empty());
        assert!(plan.steps.iter().any(|s| s.action.contains(".get()")));
    }

    #[test]
    fn test_get_recovery_plan_unknown() {
        let plan = get_recovery_plan("R999");
        assert!(plan.is_none());
    }

    #[test]
    fn test_recovery_plan_format_text() {
        let plan = get_recovery_plan("R801").unwrap();
        let text = plan.format_text();
        assert!(text.contains("R801"));
        assert!(text.contains("Steps:"));
        assert!(text.contains("move"));
    }

    #[test]
    fn test_recovery_plan_format_json() {
        let plan = get_recovery_plan("R800").unwrap();
        let json = plan.format_json();
        assert!(json.contains("\"error_code\":\"R800\""));
        assert!(json.contains("\"steps\""));
        assert!(json.contains("\"common_mistakes\""));
    }

    #[test]
    fn test_get_recovery_plans_multiple() {
        let plans = get_recovery_plans(&["R801", "R802", "R803"]);
        assert_eq!(plans.len(), 3);
    }

    #[test]
    fn test_recovery_plan_has_code_examples() {
        let plan = get_recovery_plan("R802").unwrap();
        assert!(plan.steps.iter().any(|s| s.code_example.is_some()));
    }

    #[test]
    fn test_recovery_plan_common_mistakes() {
        let plan = get_recovery_plan("R803").unwrap();
        assert!(!plan.common_mistakes.is_empty());
    }

    #[test]
    fn test_default_recovery_plan() {
        let plan = get_recovery_plan("R001").unwrap();
        assert!(!plan.steps.is_empty());
        assert_eq!(plan.steps[0].step, 1);
    }

    #[test]
    fn test_format_plans_json() {
        let plans = get_recovery_plans(&["R801", "R802"]);
        let json = format_plans_json(&plans);
        assert!(json.starts_with("["));
        assert!(json.ends_with("]"));
        assert!(json.contains("R801"));
        assert!(json.contains("R802"));
    }
}
