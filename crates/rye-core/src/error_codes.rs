//! Error code database — Goals 151, 152, 158.
//!
//! Comprehensive error code registry for all rye error codes (R001–R899).
//! Used by `rpg explain` CLI to provide structured error lookups for humans and AI agents.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Category of an error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Parse errors (R001–R099)
    Parse,
    /// Validation errors (R100–R199)
    Validation,
    /// Type errors (R200–R299)
    Type,
    /// Reactivity errors (R300–R399)
    Reactivity,
    /// Renderer errors (R400–R499)
    Renderer,
    /// Router errors (R500–R599)
    Router,
    /// SSR errors (R600–R699)
    Ssr,
    /// CLI errors (R700–R799)
    Cli,
    /// AI-specific errors (R800–R899)
    Ai,
}

impl ErrorCategory {
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Parse => "Parse",
            Self::Validation => "Validation",
            Self::Type => "Type",
            Self::Reactivity => "Reactivity",
            Self::Renderer => "Renderer",
            Self::Router => "Router",
            Self::Ssr => "SSR",
            Self::Cli => "CLI",
            Self::Ai => "AI",
        }
    }

    /// Code range for this category.
    pub fn range(&self) -> (u16, u16) {
        match self {
            Self::Parse => (1, 99),
            Self::Validation => (100, 199),
            Self::Type => (200, 299),
            Self::Reactivity => (300, 399),
            Self::Renderer => (400, 499),
            Self::Router => (500, 599),
            Self::Ssr => (600, 699),
            Self::Cli => (700, 799),
            Self::Ai => (800, 899),
        }
    }
}

/// A single error code entry.
#[derive(Debug, Clone)]
pub struct ErrorCode {
    /// The error code string, e.g. "R001".
    pub code: &'static str,
    /// Category.
    pub category: ErrorCategory,
    /// Short human-readable message.
    pub message: &'static str,
    /// Common causes.
    pub common_causes: &'static [&'static str],
    /// Suggested fix.
    pub suggestion: &'static str,
    /// Correct usage example.
    pub correct_example: &'static str,
    /// Related error codes.
    pub related_errors: &'static [&'static str],
}

impl ErrorCode {
    /// Format as human-readable text (for `rpg explain R001`).
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("error[{}]: {}\n", self.code, self.message));
        out.push_str(&format!("  Category: {}\n\n", self.category.name()));

        if !self.common_causes.is_empty() {
            out.push_str("Common causes:\n");
            for cause in self.common_causes {
                out.push_str(&format!("  - {}\n", cause));
            }
            out.push('\n');
        }

        out.push_str(&format!("Suggestion: {}\n\n", self.suggestion));

        out.push_str("Correct example:\n");
        for line in self.correct_example.lines() {
            out.push_str(&format!("  {}\n", line));
        }

        if !self.related_errors.is_empty() {
            out.push_str(&format!("\nRelated errors: {}\n", self.related_errors.join(", ")));
        }

        out
    }

    /// Format as JSON (for `rpg explain R001 --json`).
    pub fn format_json(&self) -> String {
        let causes: Vec<String> = self.common_causes.iter().map(|s| format!("\"{}\"", json_escape(s))).collect();
        let related: Vec<String> = self.related_errors.iter().map(|s| format!("\"{}\"", s)).collect();

        format!(
            r#"{{"error_code":"{}","category":"{}","message":"{}","common_causes":[{}],"suggestion":"{}","correct_example":"{}","related_errors":[{}]}}"#,
            self.code,
            self.category.name(),
            json_escape(self.message),
            causes.join(","),
            json_escape(self.suggestion),
            json_escape(self.correct_example),
            related.join(",")
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

/// Get all error codes as a slice.
pub fn all_codes() -> &'static [ErrorCode] {
    &ERROR_CODES
}

/// Get the error code lookup map.
pub fn code_map() -> &'static HashMap<&'static str, &'static ErrorCode> {
    static MAP: OnceLock<HashMap<&'static str, &'static ErrorCode>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for code in ERROR_CODES.iter() {
            m.insert(code.code, code);
        }
        m
    })
}

/// Look up an error code by its string identifier (e.g. "R001").
pub fn lookup(code: &str) -> Option<&'static ErrorCode> {
    code_map().get(code).copied()
}

/// Search error codes by keyword. Returns matching codes.
pub fn search(keyword: &str) -> Vec<&'static ErrorCode> {
    let kw = keyword.to_lowercase();
    ERROR_CODES
        .iter()
        .filter(|c| {
            c.message.to_lowercase().contains(&kw)
                || c.suggestion.to_lowercase().contains(&kw)
                || c.code.to_lowercase().contains(&kw)
                || c.common_causes.iter().any(|cause| cause.to_lowercase().contains(&kw))
        })
        .collect()
}

/// List all error codes in a category.
pub fn list_category(category: ErrorCategory) -> Vec<&'static ErrorCode> {
    ERROR_CODES.iter().filter(|c| c.category == category).collect()
}

static ERROR_CODES: &[ErrorCode] = &[
    // ===== Parse Errors (R001–R099) =====
    ErrorCode {
        code: "R001",
        category: ErrorCategory::Parse,
        message: "Missing required prop '{prop}' for component '{name}'",
        common_causes: &[
            "Forgot to pass a required prop in the template",
            "Misspelled the prop name",
            "Changed the Props struct but didn't update all usages",
        ],
        suggestion: "Add the missing required prop. Check the component's Props struct for required fields (no #[prop(optional)] attribute).",
        correct_example: "Button { label: \"Submit\", disabled: true }",
        related_errors: &["R100", "R200"],
    },
    ErrorCode {
        code: "R002",
        category: ErrorCategory::Parse,
        message: "Unknown HTML tag '{tag}'",
        common_causes: &[
            "Typo in the tag name",
            "Used a custom component without importing it",
            "Tag name is not a valid HTML element",
        ],
        suggestion: "Check the spelling. If it's a custom component, make sure it's imported. Use Levenshtein suggestion if available.",
        correct_example: "button { \"Click me\" }",
        related_errors: &["R003"],
    },
    ErrorCode {
        code: "R003",
        category: ErrorCategory::Parse,
        message: "Invalid attribute '{name}' on element '{tag}'",
        common_causes: &[
            "Misspelled attribute name",
            "Attribute not valid for this HTML element",
            "Used a prop name instead of an HTML attribute",
        ],
        suggestion: "Check valid attributes for the HTML tag. Use the component's Props struct for component attributes.",
        correct_example: "div { class: \"container\", id: \"main\" }",
        related_errors: &["R002", "R100"],
    },
    ErrorCode {
        code: "R004",
        category: ErrorCategory::Parse,
        message: "Unexpected end of template, expected '}'",
        common_causes: &[
            "Missing closing brace in template",
            "Unclosed element block",
        ],
        suggestion: "Add the missing closing brace '}' to match the opening brace.",
        correct_example: "div { p { \"Hello\" } }",
        related_errors: &["R005"],
    },
    ErrorCode {
        code: "R005",
        category: ErrorCategory::Parse,
        message: "Expected ',' between attributes",
        common_causes: &[
            "Forgot comma between attribute pairs",
            "Mixed attribute and child syntax",
        ],
        suggestion: "Separate attributes with commas or newlines.",
        correct_example: "div { class: \"container\", id: \"main\" }",
        related_errors: &["R004"],
    },

    // ===== Validation Errors (R100–R199) =====
    ErrorCode {
        code: "R100",
        category: ErrorCategory::Validation,
        message: "Duplicate attribute '{name}' on element '{tag}'",
        common_causes: &[
            "Same attribute specified twice in the template",
            "Copy-paste error",
        ],
        suggestion: "Remove the duplicate attribute. Each attribute should appear only once per element.",
        correct_example: "div { class: \"container\" }",
        related_errors: &["R001", "R003"],
    },
    ErrorCode {
        code: "R101",
        category: ErrorCategory::Validation,
        message: "For loop requires a 'key:' attribute",
        common_causes: &[
            "Forgot to add key attribute in For component",
            "Used 'id' instead of 'key'",
        ],
        suggestion: "Add a key attribute to the For component for efficient list reconciliation.",
        correct_example: "For { items: list, key: |item| item.id, render: |item| ListItem { item } }",
        related_errors: &["R102"],
    },
    ErrorCode {
        code: "R102",
        category: ErrorCategory::Validation,
        message: "Invalid event name '{event}'",
        common_causes: &[
            "Misspelled event name",
            "Used a non-standard event",
        ],
        suggestion: "Use standard event names: click, input, change, submit, focus, blur, keydown, keyup, mouseover, mouseout.",
        correct_example: "button { onclick: move |_| do_something() }",
        related_errors: &["R103"],
    },
    ErrorCode {
        code: "R103",
        category: ErrorCategory::Validation,
        message: "Component '{name}' is not registered",
        common_causes: &[
            "Component not imported",
            "Component name doesn't match the function name",
            "Missing #[component] macro",
        ],
        suggestion: "Make sure the component function has #[component] attribute and is imported with 'use'.",
        correct_example: "#[component]\nfn MyComponent() { div { \"Hello\" } }",
        related_errors: &["R002"],
    },

    // ===== Type Errors (R200–R299) =====
    ErrorCode {
        code: "R200",
        category: ErrorCategory::Type,
        message: "Prop '{prop}' expects {expected}, got {actual}",
        common_causes: &[
            "Passed wrong type to a prop",
            "String vs &str mismatch",
            "Integer type mismatch (i32 vs u32)",
        ],
        suggestion: "Check the Props struct definition for the expected type and convert your value.",
        correct_example: "Icon { name: \"home\", size: 32 }",
        related_errors: &["R201", "R800"],
    },
    ErrorCode {
        code: "R201",
        category: ErrorCategory::Type,
        message: "Type '{T}' does not implement Clone",
        common_causes: &[
            "Used a non-Clone type in a template",
            "Type contains a non-Clone field (e.g. Rc, RefCell)",
        ],
        suggestion: "Wrap the type in Arc<T>, implement Clone, or use a Signal<T> instead.",
        correct_example: "Signal::new(my_data)",
        related_errors: &["R202"],
    },
    ErrorCode {
        code: "R202",
        category: ErrorCategory::Type,
        message: "Type '{T}' cannot be rendered as text",
        common_causes: &[
            "Type doesn't implement std::fmt::Display",
            "Tried to render a struct directly in template",
        ],
        suggestion: "Implement std::fmt::Display for the type or convert to a String before rendering.",
        correct_example: "format!(\"{}\", my_value)",
        related_errors: &["R201"],
    },

    // ===== Reactivity Errors (R300–R399) =====
    ErrorCode {
        code: "R300",
        category: ErrorCategory::Reactivity,
        message: "Signal written during render",
        common_causes: &[
            "Called .set() on a signal inside a render function",
            "Modified a signal in a Memo computation",
        ],
        suggestion: "Move signal writes to event handlers or effects. Never write to signals during render.",
        correct_example: "button { onclick: move |_| count.set(count.get() + 1) }",
        related_errors: &["R301", "R802"],
    },
    ErrorCode {
        code: "R301",
        category: ErrorCategory::Reactivity,
        message: "Memo depends on itself (circular dependency)",
        common_causes: &[
            "Memo reads its own value in its computation",
            "Two memos depend on each other",
        ],
        suggestion: "Break the cycle by restructuring the computation. Memos must form a DAG.",
        correct_example: "let a = Memo::new(move || count.get() * 2);\nlet b = Memo::new(move || a.get() + 1);",
        related_errors: &["R300"],
    },
    ErrorCode {
        code: "R302",
        category: ErrorCategory::Reactivity,
        message: "Effect cleanup not registered",
        common_causes: &[
            "Used on_cleanup() outside of an effect scope",
            "Effect was created outside component lifecycle",
        ],
        suggestion: "Ensure on_cleanup() is called inside use_effect() or a component scope.",
        correct_example: "use_effect(move || {\n    let handle = subscribe();\n    on_cleanup(move || handle.cancel());\n});",
        related_errors: &["R303"],
    },
    ErrorCode {
        code: "R303",
        category: ErrorCategory::Reactivity,
        message: "Resource not cancelled on unmount",
        common_causes: &[
            "Used use_resource without proper cleanup",
            "Async task leaked after component unmount",
        ],
        suggestion: "Use Resource<T> which auto-cancels on unmount. Don't spawn raw tokio tasks.",
        correct_example: "let data = use_resource(move || fetch_data());",
        related_errors: &["R302"],
    },

    // ===== Renderer Errors (R400–R499) =====
    ErrorCode {
        code: "R400",
        category: ErrorCategory::Renderer,
        message: "Renderer not initialized",
        common_causes: &[
            "Called render functions before setting up a renderer",
            "Missing renderer initialization in main()",
        ],
        suggestion: "Initialize the renderer before rendering components.",
        correct_example: "let mut renderer = DomRenderer::new();\nrenderer.render(root_component());",
        related_errors: &["R401"],
    },
    ErrorCode {
        code: "R401",
        category: ErrorCategory::Renderer,
        message: "Hydration mismatch: server and client output differ",
        common_causes: &[
            "Different data on server vs client during hydration",
            "Random or time-based values in render",
            "Browser-only APIs called during SSR",
        ],
        suggestion: "Ensure server and client produce identical HTML. Use Suspense for async data.",
        correct_example: "Suspense { fallback: || Loading {}, children: || AsyncComponent {} }",
        related_errors: &["R600"],
    },

    // ===== Router Errors (R500–R599) =====
    ErrorCode {
        code: "R500",
        category: ErrorCategory::Router,
        message: "No route matched the path '{path}'",
        common_causes: &[
            "Missing catch-all route",
            "Route path pattern is wrong",
            "Forgot to register the route",
        ],
        suggestion: "Add a catch-all fallback route or fix the route pattern.",
        correct_example: "Route::new(\"*\", NotFoundComponent)",
        related_errors: &["R501"],
    },
    ErrorCode {
        code: "R501",
        category: ErrorCategory::Router,
        message: "Route param '{param}' type mismatch",
        common_causes: &[
            "Expected integer but URL has non-numeric string",
            "Param type doesn't match the route definition",
        ],
        suggestion: "Use correct type annotations for route params or add validation.",
        correct_example: "Route::new(\"/users/:id\", UserComponent).param::<u32>(\"id\")",
        related_errors: &["R500"],
    },

    // ===== SSR Errors (R600–R699) =====
    ErrorCode {
        code: "R600",
        category: ErrorCategory::Ssr,
        message: "SSR serialization failed for type '{T}'",
        common_causes: &[
            "Type doesn't implement Serialize",
            "Type contains non-serializable fields",
        ],
        suggestion: "Add #[derive(Serialize)] to the type and ensure all fields are serializable.",
        correct_example: "#[derive(serde::Serialize)]\nstruct MyData { name: String }",
        related_errors: &["R601"],
    },
    ErrorCode {
        code: "R601",
        category: ErrorCategory::Ssr,
        message: "Streaming SSR chunk failed",
        common_causes: &[
            "Async resource panicked during streaming",
            "Network error during stream",
        ],
        suggestion: "Wrap async resources in ErrorBoundary and provide fallback UI.",
        correct_example: "ErrorBoundary { fallback: |err| ErrorUI { err }, children: || MyComponent {} }",
        related_errors: &["R600", "R401"],
    },

    // ===== CLI Errors (R700–R799) =====
    ErrorCode {
        code: "R700",
        category: ErrorCategory::Cli,
        message: "Project not found at '{path}'",
        common_causes: &[
            "Not in a rye project directory",
            "Missing Cargo.toml with rye dependencies",
        ],
        suggestion: "Run 'rpg new <name>' to create a new project, or navigate to the project root.",
        correct_example: "rpg new my-app --template web",
        related_errors: &["R701"],
    },
    ErrorCode {
        code: "R701",
        category: ErrorCategory::Cli,
        message: "Build failed for target '{target}'",
        common_causes: &[
            "Missing target toolchain (e.g. wasm32-unknown-unknown)",
            "Compilation errors in the project",
            "Missing dependencies",
        ],
        suggestion: "Install the required target with 'rustup target add {target}' and fix compilation errors.",
        correct_example: "rustup target add wasm32-unknown-unknown",
        related_errors: &["R700"],
    },

    // ===== AI-Specific Errors (R800–R899) — Goal 158 =====
    ErrorCode {
        code: "R800",
        category: ErrorCategory::Ai,
        message: "Wrong prop type — expected {expected}, got {actual}",
        common_causes: &[
            "AI generated code that passes &str where String is expected",
            "AI used a number literal where a Signal is expected",
            "AI didn't wrap values in Signal::new() when the prop expects Signal<T>",
        ],
        suggestion: "Convert the value to the expected type. Use .to_string() for &str -> String, Signal::new(value) for T -> Signal<T>.",
        correct_example: "// Wrong: MyComponent { count: 5 }\n// Right: MyComponent { count: Signal::new(5) }\n// Or if prop is String:\n// MyComponent { label: \"Hello\".to_string() }",
        related_errors: &["R200", "R801"],
    },
    ErrorCode {
        code: "R801",
        category: ErrorCategory::Ai,
        message: "Missing 'move' keyword in event handler closure",
        common_causes: &[
            "AI generated closure without 'move' keyword",
            "Closure captures signals but doesn't take ownership",
        ],
        suggestion: "Add 'move' before the closure in event handlers. Event handler closures must be 'move' to capture signal values.",
        correct_example: "button { onclick: move |_| count.set(count.get() + 1) }",
        related_errors: &["R802", "R803"],
    },
    ErrorCode {
        code: "R802",
        category: ErrorCategory::Ai,
        message: "Signal read without .get() — used Signal<T> where T is expected",
        common_causes: &[
            "AI used signal directly in template instead of calling .get()",
            "AI treated Signal like a regular variable",
        ],
        suggestion: "Call .get() on the signal to read its value. In templates, use {signal.get()} not {signal}.",
        correct_example: "// Wrong: p { \"Count: \" {count} }\n// Right: p { \"Count: \" {count.get()} }",
        related_errors: &["R803", "R300"],
    },
    ErrorCode {
        code: "R803",
        category: ErrorCategory::Ai,
        message: "Signal write without .set() — assigned to Signal directly",
        common_causes: &[
            "AI used assignment (=) instead of .set() to update a signal",
            "AI treated Signal like a mutable variable",
        ],
        suggestion: "Use .set() to update a signal's value. Never use direct assignment.",
        correct_example: "// Wrong: count = count + 1\n// Right: count.set(count.get() + 1)",
        related_errors: &["R802", "R300"],
    },
    ErrorCode {
        code: "R804",
        category: ErrorCategory::Ai,
        message: "Component name not capitalized — '{name}' should be PascalCase",
        common_causes: &[
            "AI used snake_case for a component function name",
            "AI used lowercase for a component in a template",
        ],
        suggestion: "Component names must be PascalCase (e.g. MyButton, not my_button). Rename the function and update template usages.",
        correct_example: "#[component]\nfn MyButton(props: MyButtonProps) { button { \"Click\" } }",
        related_errors: &["R805", "R103"],
    },
    ErrorCode {
        code: "R805",
        category: ErrorCategory::Ai,
        message: "Missing #[component] macro on function '{name}'",
        common_causes: &[
            "AI wrote a component function without the #[component] attribute",
            "AI forgot to add the macro when creating a new component",
        ],
        suggestion: "Add #[component] attribute above the function. This is required for all rye components.",
        correct_example: "#[component]\nfn Counter(props: CounterProps) {\n    div { \"Count: \" {props.count} }\n}",
        related_errors: &["R804", "R103"],
    },
    ErrorCode {
        code: "R806",
        category: ErrorCategory::Ai,
        message: "Used use_effect() for derived state — should use Memo",
        common_causes: &[
            "AI used use_effect to compute a value from signals",
            "AI manually updated a signal inside an effect instead of using Memo",
        ],
        suggestion: "Use Memo::new() for derived state. Effects are for side effects, not computations.",
        correct_example: "// Wrong: use_effect(move || { derived.set(a.get() + b.get()); })\n// Right: let derived = Memo::new(move || a.get() + b.get());",
        related_errors: &["R300", "R301"],
    },
    ErrorCode {
        code: "R807",
        category: ErrorCategory::Ai,
        message: "Unnecessary clone — value is already owned",
        common_causes: &[
            "AI added .clone() defensively without checking ownership",
            "AI cloned a Signal (Signals are already cheap to clone — they're Arc-based)",
        ],
        suggestion: "Remove unnecessary .clone() calls. Signals are Arc-based and cheap to clone, but other values may not need cloning.",
        correct_example: "// Wrong: let c = count.clone();\n// Right: let c = count; // Signal is already cheap to share",
        related_errors: &["R808"],
    },
    ErrorCode {
        code: "R808",
        category: ErrorCategory::Ai,
        message: "Prop drilling detected — use context instead",
        common_causes: &[
            "AI passed a prop through 3+ levels of components",
            "AI didn't know about rye's context system",
        ],
        suggestion: "Use provide_context() and use_context() instead of passing props through multiple layers.",
        correct_example: "// In parent:\nprovide_context(Theme::dark());\n// In any child:\nlet theme = use_context::<Theme>();",
        related_errors: &["R809"],
    },
    ErrorCode {
        code: "R809",
        category: ErrorCategory::Ai,
        message: "Used async block without Resource — will not auto-cancel",
        common_causes: &[
            "AI spawned async work directly instead of using use_resource",
            "AI used tokio::spawn in a component",
        ],
        suggestion: "Use use_resource() for async data. It auto-cancels on unmount and integrates with Suspense.",
        correct_example: "let data = use_resource(move || async { fetch_user(id).await });",
        related_errors: &["R303", "R806"],
    },
    ErrorCode {
        code: "R810",
        category: ErrorCategory::Ai,
        message: "Used template! outside of #[component] function",
        common_causes: &[
            "AI placed template! macro in a regular function",
            "AI tried to use template syntax in a non-component context",
        ],
        suggestion: "template! must be used inside a #[component] function. Wrap your function with #[component].",
        correct_example: "#[component]\nfn MyView() {\n    template! { div { \"Hello\" } }\n}",
        related_errors: &["R805", "R103"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_existing_code() {
        let code = lookup("R001");
        assert!(code.is_some());
        assert_eq!(code.unwrap().code, "R001");
        assert_eq!(code.unwrap().category, ErrorCategory::Parse);
    }

    #[test]
    fn test_lookup_nonexistent_code() {
        let code = lookup("R999");
        assert!(code.is_none());
    }

    #[test]
    fn test_lookup_ai_code() {
        let code = lookup("R800");
        assert!(code.is_some());
        assert_eq!(code.unwrap().category, ErrorCategory::Ai);
    }

    #[test]
    fn test_search() {
        let results = search("signal");
        assert!(!results.is_empty());
        // Should find R300, R301, R802, R803, R807
        assert!(results.iter().any(|c| c.code == "R802"));
    }

    #[test]
    fn test_list_category() {
        let ai_codes = list_category(ErrorCategory::Ai);
        assert!(!ai_codes.is_empty());
        assert!(ai_codes.iter().all(|c| c.category == ErrorCategory::Ai));
    }

    #[test]
    fn test_format_text() {
        let code = lookup("R001").unwrap();
        let text = code.format_text();
        assert!(text.contains("R001"));
        assert!(text.contains("Parse"));
        assert!(text.contains("Common causes"));
        assert!(text.contains("Suggestion"));
        assert!(text.contains("Correct example"));
    }

    #[test]
    fn test_format_json() {
        let code = lookup("R800").unwrap();
        let json = code.format_json();
        assert!(json.contains("\"error_code\":\"R800\""));
        assert!(json.contains("\"category\":\"AI\""));
        assert!(json.contains("\"common_causes\""));
        assert!(json.contains("\"related_errors\""));
    }

    #[test]
    fn test_all_categories_represented() {
        let categories = [
            ErrorCategory::Parse,
            ErrorCategory::Validation,
            ErrorCategory::Type,
            ErrorCategory::Reactivity,
            ErrorCategory::Renderer,
            ErrorCategory::Router,
            ErrorCategory::Ssr,
            ErrorCategory::Cli,
            ErrorCategory::Ai,
        ];
        for cat in &categories {
            let codes = list_category(*cat);
            assert!(!codes.is_empty(), "Category {:?} has no error codes", cat.name());
        }
    }

    #[test]
    fn test_json_escape() {
        assert_eq!(json_escape("hello\nworld"), "hello\\nworld");
        assert_eq!(json_escape("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(json_escape("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_error_category_range() {
        assert_eq!(ErrorCategory::Parse.range(), (1, 99));
        assert_eq!(ErrorCategory::Ai.range(), (800, 899));
    }
}
