//! AI prompt templates — Goal 159.
//!
//! Provides ready-made prompt templates for common rye patterns.
//! AI agents can use these as starting points instead of writing from scratch.

use std::collections::HashMap;
use std::sync::OnceLock;

/// A prompt template for generating rye code.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// Template identifier (e.g. "component", "form", "list").
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Category (e.g. "component", "pattern", "page").
    pub category: &'static str,
    /// Description of what this template generates.
    pub description: &'static str,
    /// The prompt text with {placeholders}.
    pub prompt: &'static str,
    /// Placeholders that need to be filled.
    pub placeholders: &'static [&'static str],
    /// Example filled prompt.
    pub example: &'static str,
}

impl PromptTemplate {
    /// Fill in the template placeholders.
    pub fn fill(&self, values: &HashMap<&str, String>) -> String {
        let mut result = self.prompt.to_string();
        for &key in self.placeholders {
            if let Some(val) = values.get(key) {
                result = result.replace(&format!("{{{}}}", key), val);
            }
        }
        result
    }
}

/// Get all prompt templates.
pub fn all_templates() -> &'static [PromptTemplate] {
    &TEMPLATES
}

/// Get template by ID.
pub fn get_template(id: &str) -> Option<&'static PromptTemplate> {
    TEMPLATES.iter().find(|t| t.id == id)
}

/// List templates by category.
pub fn templates_by_category(category: &str) -> Vec<&'static PromptTemplate> {
    TEMPLATES.iter().filter(|t| t.category == category).collect()
}

/// Get all categories.
pub fn categories() -> Vec<&'static str> {
    let mut cats: Vec<&'static str> = TEMPLATES.iter().map(|t| t.category).collect();
    cats.sort();
    cats.dedup();
    cats
}

/// Format all templates as JSON for AI discovery.
pub fn format_all_json() -> String {
    let entries: Vec<String> = TEMPLATES
        .iter()
        .map(|t| {
            let placeholders: Vec<String> = t.placeholders.iter().map(|p| format!("\"{}\"", p)).collect();
            format!(
                r#"{{"id":"{}","name":"{}","category":"{}","description":"{}","placeholders":[{}]}}"#,
                t.id, t.name, t.category, t.description, placeholders.join(",")
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

static TEMPLATES: &[PromptTemplate] = &[
    PromptTemplate {
        id: "component",
        name: "Basic Component",
        category: "component",
        description: "Generate a rye component with props and event handling",
        prompt: "Create a rye component called {name} with the following props: {props}. The component should {description}. Use the #[component] macro and template! syntax. Include event handlers for {events}.",
        placeholders: &["name", "props", "description", "events"],
        example: "Create a rye component called Button with the following props: label: String, disabled: bool. The component should render a clickable button. Use the #[component] macro and template! syntax. Include event handlers for click.",
    },
    PromptTemplate {
        id: "form",
        name: "Form Component",
        category: "pattern",
        description: "Generate a form with validation and submission",
        prompt: "Create a rye form component called {name} with fields: {fields}. The form should validate {validation_rules} and submit to {endpoint}. Use Signal for form state, use_resource for submission, and ErrorBoundary for error handling.",
        placeholders: &["name", "fields", "validation_rules", "endpoint"],
        example: "Create a rye form component called LoginForm with fields: email: String, password: String. The form should validate email format and password length >= 8 and submit to /api/login. Use Signal for form state, use_resource for submission, and ErrorBoundary for error handling.",
    },
    PromptTemplate {
        id: "list",
        name: "Data List with Filtering",
        category: "pattern",
        description: "Generate a filtered, sortable data list",
        prompt: "Create a rye data list component called {name} that displays {item_type} items. Include: a Signal for the search query, a Memo for filtered items, a For loop with key for rendering, and sorting by {sort_field}. The data source is {data_source}.",
        placeholders: &["name", "item_type", "sort_field", "data_source"],
        example: "Create a rye data list component called UserList that displays User items. Include: a Signal for the search query, a Memo for filtered items, a For loop with key for rendering, and sorting by name. The data source is use_resource(fetch_users).",
    },
    PromptTemplate {
        id: "page",
        name: "Page Component",
        category: "page",
        description: "Generate a page with route, data loading, and SEO",
        prompt: "Create a rye page component called {name} at route {route}. The page should load data from {data_source}, display {content_description}, and include Suspense for loading state. Use #[component] and provide_context for page metadata.",
        placeholders: &["name", "route", "data_source", "content_description"],
        example: "Create a rye page component called UserProfile at route /users/:id. The page should load data from use_resource(fetch_user(id)), display user profile information, and include Suspense for loading state. Use #[component] and provide_context for page metadata.",
    },
    PromptTemplate {
        id: "store",
        name: "Signal Store",
        category: "pattern",
        description: "Generate a signal-based store for state management",
        prompt: "Create a rye signal store called {name} with state fields: {fields}. Include: Signal for each field, getter and setter methods, and a Memo for {derived_field}. The store should be shared via provide_context.",
        placeholders: &["name", "fields", "derived_field"],
        example: "Create a rye signal store called CartStore with state fields: items: Vec<Item>, total: f64. Include: Signal for each field, getter and setter methods, and a Memo for item_count. The store should be shared via provide_context.",
    },
    PromptTemplate {
        id: "action",
        name: "Server Action",
        category: "pattern",
        description: "Generate a type-safe server action",
        prompt: "Create a rye server action called {name} that takes parameters {params} and returns {return_type}. The action should {description}. Use #[server] macro and handle errors with ServerError. Include input validation.",
        placeholders: &["name", "params", "return_type", "description"],
        example: "Create a rye server action called CreateUser that takes parameters name: String, email: String and returns Result<User, ServerError>. The action should insert a new user into the database. Use #[server] macro and handle errors with ServerError. Include input validation.",
    },
    PromptTemplate {
        id: "island",
        name: "Island Component",
        category: "component",
        description: "Generate a client-only hydrated island component",
        prompt: "Create a rye island component called {name} that {description}. The island should be hydrated client-side only using #[rye::island]. Include {features} and ensure it works with SSR fallback.",
        placeholders: &["name", "description", "features"],
        example: "Create a rye island component called InteractiveChart that renders a real-time data visualization. The island should be hydrated client-side only using #[rye::island]. Include canvas rendering and mouse interaction and ensure it works with SSR fallback.",
    },
    PromptTemplate {
        id: "crud",
        name: "CRUD Operations",
        category: "pattern",
        description: "Generate complete CRUD UI with server actions",
        prompt: "Create a rye CRUD interface for {entity} with: a list view with search, a detail view, a create form, an edit form, and a delete confirmation. Use server actions for {create_action}, {read_action}, {update_action}, {delete_action}. Include loading states with Suspense and error handling with ErrorBoundary.",
        placeholders: &["entity", "create_action", "read_action", "update_action", "delete_action"],
        example: "Create a rye CRUD interface for Task with: a list view with search, a detail view, a create form, an edit form, and a delete confirmation. Use server actions for create_task, get_tasks, update_task, delete_task. Include loading states with Suspense and error handling with ErrorBoundary.",
    },
    PromptTemplate {
        id: "modal",
        name: "Modal/Dialog Component",
        category: "component",
        description: "Generate a modal dialog with backdrop and focus trap",
        prompt: "Create a rye modal component called {name} that {description}. Include: a Signal for open/close state, backdrop click to close, escape key handler, focus trap, and transition animation. Use provide_context for modal state.",
        placeholders: &["name", "description"],
        example: "Create a rye modal component called ConfirmDialog that displays a confirmation message with OK/Cancel buttons. Include: a Signal for open/close state, backdrop click to close, escape key handler, focus trap, and transition animation. Use provide_context for modal state.",
    },
    PromptTemplate {
        id: "auth",
        name: "Authentication Flow",
        category: "pattern",
        description: "Generate login/logout/register flow with session management",
        prompt: "Create a rye authentication flow with: a LoginForm component, a RegisterForm component, a use_auth() hook that returns user state, server actions for {login_action} and {logout_action}, and route guards for protected pages. Use provide_context for the auth store.",
        placeholders: &["login_action", "logout_action"],
        example: "Create a rye authentication flow with: a LoginForm component, a RegisterForm component, a use_auth() hook that returns user state, server actions for login and logout, and route guards for protected pages. Use provide_context for the auth store.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        let t = get_template("component");
        assert!(t.is_some());
        assert_eq!(t.unwrap().id, "component");
    }

    #[test]
    fn test_get_template_nonexistent() {
        let t = get_template("nonexistent");
        assert!(t.is_none());
    }

    #[test]
    fn test_templates_by_category() {
        let components = templates_by_category("component");
        assert!(components.len() >= 3); // component, island, modal
        assert!(components.iter().all(|t| t.category == "component"));
    }

    #[test]
    fn test_categories() {
        let cats = categories();
        assert!(cats.contains(&"component"));
        assert!(cats.contains(&"pattern"));
        assert!(cats.contains(&"page"));
    }

    #[test]
    fn test_fill_template() {
        let t = get_template("component").unwrap();
        let mut values = HashMap::new();
        values.insert("name", "Button".to_string());
        values.insert("props", "label: String".to_string());
        values.insert("description", "render a button".to_string());
        values.insert("events", "click".to_string());
        let filled = t.fill(&values);
        assert!(filled.contains("Button"));
        assert!(filled.contains("label: String"));
        assert!(filled.contains("render a button"));
        assert!(filled.contains("click"));
    }

    #[test]
    fn test_format_all_json() {
        let json = format_all_json();
        assert!(json.starts_with("["));
        assert!(json.ends_with("]"));
        assert!(json.contains("\"id\":\"component\""));
    }

    #[test]
    fn test_all_templates_nonempty() {
        assert!(all_templates().len() >= 10);
    }
}
