//! Custom render hooks — plugins that intercept and modify render output.
//!
//! `use_render_hook()` lets plugins intercept and modify the render output
//! of any component. Useful for adding wrapper elements, injecting analytics
//! attributes, modifying class names globally.

use std::cell::RefCell;
use std::rc::Rc;

/// A render hook — intercepts and modifies the output of a component render.
pub type RenderHook = Rc<dyn Fn(&RenderContext) -> RenderHookResult>;

/// The result of a render hook — either modifies the output or passes through.
#[derive(Debug, Clone)]
pub enum RenderHookResult {
    /// Pass through — don't modify the output.
    Pass,
    /// Replace the output with new content.
    Replace(String),
    /// Wrap the output in additional HTML.
    Wrap { before: String, after: String },
    /// Modify attributes on the root element.
    ModifyAttrs(Vec<(String, String)>),
}

/// Context provided to render hooks — information about the component being rendered.
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// The component name (e.g. "Counter", "Header").
    pub component_name: String,
    /// The tag of the root element being rendered.
    pub tag: String,
    /// The current attributes on the root element.
    pub attrs: Vec<(String, String)>,
    /// The rendered content (HTML string).
    pub content: String,
}

impl RenderContext {
    /// Create a new render context.
    pub fn new(component_name: &str, tag: &str, content: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            tag: tag.to_string(),
            attrs: Vec::new(),
            content: content.to_string(),
        }
    }

    /// Get an attribute value.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// The render hook registry — stores hooks and applies them during rendering.
pub struct RenderHookRegistry {
    hooks: RefCell<Vec<RenderHook>>,
}

impl RenderHookRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            hooks: RefCell::new(Vec::new()),
        }
    }

    /// Register a render hook. Hooks are called in registration order.
    pub fn register<F: Fn(&RenderContext) -> RenderHookResult + 'static>(&self, hook: F) {
        self.hooks.borrow_mut().push(Rc::new(hook));
    }

    /// Apply all hooks to a render context and return the final output.
    pub fn apply(&self, mut ctx: RenderContext) -> String {
        let hooks = self.hooks.borrow().clone();
        let mut output = ctx.content.clone();

        for hook in &hooks {
            ctx.content = output.clone();
            match hook(&ctx) {
                RenderHookResult::Pass => {}
                RenderHookResult::Replace(new_content) => {
                    output = new_content;
                }
                RenderHookResult::Wrap { before, after } => {
                    output = format!("{}{}{}", before, output, after);
                }
                RenderHookResult::ModifyAttrs(new_attrs) => {
                    for (name, value) in &new_attrs {
                        ctx.attrs.push((name.clone(), value.clone()));
                    }
                    // Re-render with new attrs (simplified — just append as data-attrs)
                    let attr_str: String = new_attrs
                        .iter()
                        .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                        .collect();
                    // Inject attrs into the opening tag
                    if let Some(close_pos) = output.find('>') {
                        output.insert_str(close_pos, &attr_str);
                    }
                }
            }
        }

        output
    }

    /// Get the number of registered hooks.
    pub fn hook_count(&self) -> usize {
        self.hooks.borrow().len()
    }

    /// Clear all hooks.
    pub fn clear(&self) {
        self.hooks.borrow_mut().clear();
    }
}

impl Default for RenderHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global render hook registry.
thread_local! {
    static GLOBAL_REGISTRY: RefCell<Option<Rc<RenderHookRegistry>>> = const { RefCell::new(None) };
}

/// Initialize the global render hook registry.
pub fn init_global_registry() {
    GLOBAL_REGISTRY.with(|r| {
        *r.borrow_mut() = Some(Rc::new(RenderHookRegistry::new()));
    });
}

/// Get the global render hook registry, if initialized.
pub fn global_registry() -> Option<Rc<RenderHookRegistry>> {
    GLOBAL_REGISTRY.with(|r| r.borrow().clone())
}

/// Register a hook on the global registry.
pub fn use_render_hook<F: Fn(&RenderContext) -> RenderHookResult + 'static>(hook: F) {
    if let Some(registry) = global_registry() {
        registry.register(hook);
    } else {
        init_global_registry();
        if let Some(registry) = global_registry() {
            registry.register(hook);
        }
    }
}

/// Apply global hooks to a render context.
pub fn apply_hooks(ctx: RenderContext) -> String {
    if let Some(registry) = global_registry() {
        registry.apply(ctx)
    } else {
        ctx.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_pass() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| RenderHookResult::Pass);
        let ctx = RenderContext::new("Counter", "div", "<div>Hello</div>");
        assert_eq!(registry.apply(ctx), "<div>Hello</div>");
    }

    #[test]
    fn test_registry_replace() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| RenderHookResult::Replace("<span>Replaced</span>".to_string()));
        let ctx = RenderContext::new("Counter", "div", "<div>Hello</div>");
        assert_eq!(registry.apply(ctx), "<span>Replaced</span>");
    }

    #[test]
    fn test_registry_wrap() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| RenderHookResult::Wrap {
            before: "<wrapper>".to_string(),
            after: "</wrapper>".to_string(),
        });
        let ctx = RenderContext::new("Counter", "div", "<div>Hello</div>");
        assert_eq!(registry.apply(ctx), "<wrapper><div>Hello</div></wrapper>");
    }

    #[test]
    fn test_registry_modify_attrs() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| {
            RenderHookResult::ModifyAttrs(vec![("data-analytics".to_string(), "true".to_string())])
        });
        let ctx = RenderContext::new("Counter", "div", "<div>Hello</div>");
        let result = registry.apply(ctx);
        assert!(result.contains("data-analytics=\"true\""));
    }

    #[test]
    fn test_registry_multiple_hooks() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| RenderHookResult::Wrap {
            before: "<a>".to_string(),
            after: "</a>".to_string(),
        });
        registry.register(|_| RenderHookResult::Wrap {
            before: "<b>".to_string(),
            after: "</b>".to_string(),
        });
        let ctx = RenderContext::new("Test", "div", "content");
        assert_eq!(registry.apply(ctx), "<b><a>content</a></b>");
    }

    #[test]
    fn test_registry_clear() {
        let registry = RenderHookRegistry::new();
        registry.register(|_| RenderHookResult::Pass);
        assert_eq!(registry.hook_count(), 1);
        registry.clear();
        assert_eq!(registry.hook_count(), 0);
    }

    #[test]
    fn test_render_context_get_attr() {
        let mut ctx = RenderContext::new("Test", "div", "content");
        ctx.attrs.push(("class".to_string(), "btn".to_string()));
        assert_eq!(ctx.get_attr("class"), Some("btn"));
        assert_eq!(ctx.get_attr("id"), None);
    }

    #[test]
    fn test_global_registry() {
        init_global_registry();
        use_render_hook(|_| RenderHookResult::Wrap {
            before: "<global>".to_string(),
            after: "</global>".to_string(),
        });
        let ctx = RenderContext::new("Test", "div", "hello");
        assert_eq!(apply_hooks(ctx), "<global>hello</global>");
    }
}
