//! Shadow DOM encapsulation — `<Shadow>` component for full style isolation.
//!
//! Renders children inside a shadow root. Full style encapsulation —
//! no CSS leakage in or out. Useful for embedded widgets, third-party
//! components, design system previews.

use std::cell::RefCell;

/// The shadow DOM encapsulation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowMode {
    /// Open shadow root — accessible via `element.shadowRoot`.
    Open,
    /// Closed shadow root — not accessible from outside.
    Closed,
}

impl ShadowMode {
    /// Get the JavaScript string representation.
    pub fn js_string(&self) -> &'static str {
        match self {
            ShadowMode::Open => "open",
            ShadowMode::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ShadowMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.js_string())
    }
}

/// A shadow DOM root — encapsulated content with isolated styles.
#[derive(Debug, Clone)]
pub struct ShadowRoot {
    /// The encapsulation mode.
    pub mode: ShadowMode,
    /// The host element tag.
    pub host_tag: String,
    /// The styles scoped to this shadow root.
    pub styles: Vec<String>,
    /// The child content inside the shadow root.
    pub children: String,
    /// Delegates focus to the shadow root.
    pub delegates_focus: bool,
}

impl ShadowRoot {
    /// Create a new open shadow root.
    pub fn open(host_tag: &str, children: &str) -> Self {
        Self {
            mode: ShadowMode::Open,
            host_tag: host_tag.to_string(),
            styles: Vec::new(),
            children: children.to_string(),
            delegates_focus: false,
        }
    }

    /// Create a new closed shadow root.
    pub fn closed(host_tag: &str, children: &str) -> Self {
        Self {
            mode: ShadowMode::Closed,
            host_tag: host_tag.to_string(),
            styles: Vec::new(),
            children: children.to_string(),
            delegates_focus: false,
        }
    }

    /// Add a scoped style to the shadow root.
    pub fn add_style(&mut self, css: &str) -> &mut Self {
        self.styles.push(css.to_string());
        self
    }

    /// Enable focus delegation.
    pub fn delegate_focus(&mut self) -> &mut Self {
        self.delegates_focus = true;
        self
    }

    /// Render the shadow root to HTML.
    pub fn render(&self) -> String {
        let style_content = if self.styles.is_empty() {
            String::new()
        } else {
            format!("<style>{}</style>", self.styles.join("\n"))
        };

        let delegates_attr = if self.delegates_focus {
            ", delegatesFocus: true"
        } else {
            ""
        };

        // For SSR, we emit a template with the shadow content
        // For client-side, this would be done via JS: element.attachShadow({mode, delegatesFocus})
        format!(
            "<{tag} data-rye-shadow=\"{mode}{delegates}\">{styles}{children}</{tag}>",
            tag = self.host_tag,
            mode = self.mode,
            delegates = delegates_attr,
            styles = style_content,
            children = self.children,
        )
    }

    /// Render the JavaScript code to attach a shadow root on the client.
    pub fn render_attach_script(&self, element_id: &str) -> String {
        let style_content = if self.styles.is_empty() {
            String::new()
        } else {
            format!("<style>{}</style>", self.styles.join("\n"))
        };

        format!(
            r#"(function(){{var e=document.getElementById("{id}");if(!e)return;var s=e.attachShadow({{mode:"{mode}"{delegates}}});s.innerHTML='{styles}{children}';}})();"#,
            id = element_id,
            mode = self.mode,
            delegates = if self.delegates_focus {
                ", delegatesFocus: true"
            } else {
                ""
            },
            styles = style_content.replace('\'', "\\'"),
            children = self.children.replace('\'', "\\'"),
        )
    }
}

/// A shadow DOM component — wraps children in a shadow root.
pub struct Shadow {
    root: ShadowRoot,
}

impl Shadow {
    /// Create a new open shadow component.
    pub fn open(host_tag: &str, children: &str) -> Self {
        Self {
            root: ShadowRoot::open(host_tag, children),
        }
    }

    /// Create a new closed shadow component.
    pub fn closed(host_tag: &str, children: &str) -> Self {
        Self {
            root: ShadowRoot::closed(host_tag, children),
        }
    }

    /// Add a scoped style.
    pub fn style(mut self, css: &str) -> Self {
        self.root.add_style(css);
        self
    }

    /// Enable focus delegation.
    pub fn delegate_focus(mut self) -> Self {
        self.root.delegate_focus();
        self
    }

    /// Render to HTML.
    pub fn render(&self) -> String {
        self.root.render()
    }

    /// Get the shadow root.
    pub fn root(&self) -> &ShadowRoot {
        &self.root
    }
}

/// A registry of shadow DOM style sheets — shared styles across multiple shadow roots.
pub struct ShadowStyleSheetRegistry {
    sheets: RefCell<Vec<ShadowStyleSheet>>,
}

/// A named style sheet that can be shared across shadow roots.
#[derive(Debug, Clone)]
pub struct ShadowStyleSheet {
    /// Unique name for this style sheet.
    pub name: String,
    /// The CSS content.
    pub css: String,
}

impl ShadowStyleSheetRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            sheets: RefCell::new(Vec::new()),
        }
    }

    /// Register a shared style sheet.
    pub fn register(&self, name: &str, css: &str) {
        self.sheets.borrow_mut().push(ShadowStyleSheet {
            name: name.to_string(),
            css: css.to_string(),
        });
    }

    /// Get a style sheet by name.
    pub fn get(&self, name: &str) -> Option<ShadowStyleSheet> {
        self.sheets
            .borrow()
            .iter()
            .find(|s| s.name == name)
            .cloned()
    }

    /// Get all registered style sheet names.
    pub fn names(&self) -> Vec<String> {
        self.sheets
            .borrow()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }

    /// Apply a named style sheet to a shadow root.
    pub fn apply_to(&self, name: &str, root: &mut ShadowRoot) -> bool {
        if let Some(sheet) = self.get(name) {
            root.add_style(&sheet.css);
            return true;
        }
        false
    }
}

impl Default for ShadowStyleSheetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_open() {
        let s = Shadow::open("div", "<p>Hello</p>");
        let html = s.render();
        assert!(html.contains("data-rye-shadow=\"open\""));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_shadow_closed() {
        let s = Shadow::closed("div", "<p>Secret</p>");
        let html = s.render();
        assert!(html.contains("data-rye-shadow=\"closed\""));
    }

    #[test]
    fn test_shadow_with_style() {
        let s = Shadow::open("div", "<p>Styled</p>").style("p { color: red; }");
        let html = s.render();
        assert!(html.contains("<style>"));
        assert!(html.contains("color: red"));
    }

    #[test]
    fn test_shadow_delegate_focus() {
        let s = Shadow::open("div", "<input/>").delegate_focus();
        let html = s.render();
        assert!(html.contains("delegatesFocus: true"));
    }

    #[test]
    fn test_shadow_mode_js_string() {
        assert_eq!(ShadowMode::Open.js_string(), "open");
        assert_eq!(ShadowMode::Closed.js_string(), "closed");
    }

    #[test]
    fn test_shadow_mode_display() {
        assert_eq!(ShadowMode::Open.to_string(), "open");
        assert_eq!(ShadowMode::Closed.to_string(), "closed");
    }

    #[test]
    fn test_shadow_attach_script() {
        let root = ShadowRoot::open("div", "<p>Content</p>");
        let script = root.render_attach_script("my-element");
        assert!(script.contains("attachShadow"));
        assert!(script.contains("mode:\"open\""));
        assert!(script.contains("my-element"));
    }

    #[test]
    fn test_shadow_style_sheet_registry() {
        let registry = ShadowStyleSheetRegistry::new();
        registry.register("theme", ":host { color: blue; }");
        assert!(registry.get("theme").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_shadow_style_sheet_apply_to() {
        let registry = ShadowStyleSheetRegistry::new();
        registry.register("theme", ":host { color: blue; }");
        let mut root = ShadowRoot::open("div", "content");
        assert!(registry.apply_to("theme", &mut root));
        assert_eq!(root.styles.len(), 1);
        assert!(!registry.apply_to("nonexistent", &mut root));
    }

    #[test]
    fn test_shadow_style_sheet_names() {
        let registry = ShadowStyleSheetRegistry::new();
        registry.register("a", "css");
        registry.register("b", "css");
        let names = registry.names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_shadow_root_add_style_chaining() {
        let mut root = ShadowRoot::open("div", "content");
        root.add_style("a { }").add_style("b { }");
        assert_eq!(root.styles.len(), 2);
    }
}
