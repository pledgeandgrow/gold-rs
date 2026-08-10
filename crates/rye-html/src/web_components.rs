//! Web Components interop — use custom elements from rye and vice versa.
//!
//! This module provides bidirectional interop between rye components and
//! Web Components (Custom Elements, Shadow DOM):
//!
//! - **Wrapping custom elements**: Use any Web Component in rye templates
//! - **Exporting rye components**: Expose rye components as custom elements
//! - **Shadow DOM**: Render rye components into Shadow DOM for style isolation
//! - **Slot mapping**: Map rye children to `<slot>` elements
//!
//! ## Usage
//!
//! ### Using a Web Component in rye
//!
//! ```ignore
//! use rye_html::web_components::CustomElement;
//!
//! template! {
//!     CustomElement::new("my-chart")
//!         .attr("data", "[1,2,3]")
//!         .child(template! { "Slot content" })
//! }
//! ```
//!
//! ### Exporting a rye component as a Web Component
//!
//! ```ignore
//! use rye_html::web_components::define_component;
//!
//! define_component("my-counter", || {
//!     template! { Counter { } }
//! });
//! ```

/// A custom element wrapper for use in rye templates.
///
/// Allows embedding any Web Component (custom element) within a rye
/// template with typed attributes and slot children.
pub struct CustomElement {
    /// The tag name of the custom element (must contain a hyphen).
    tag: String,
    /// Attributes to set on the element.
    attributes: Vec<(String, String)>,
    /// Whether to use Shadow DOM.
    shadow: bool,
    /// Children to render into the element's default slot.
    children_html: String,
}

impl CustomElement {
    /// Create a new custom element wrapper.
    ///
    /// The tag name must contain a hyphen (e.g. "my-chart", "x-button").
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
            shadow: false,
            children_html: String::new(),
        }
    }

    /// Set an attribute on the custom element.
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((name.into(), value.into()));
        self
    }

    /// Set multiple attributes from a slice of (name, value) pairs.
    pub fn attrs(mut self, attrs: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        for (name, value) in attrs {
            self.attributes
                .push((name.as_ref().to_string(), value.as_ref().to_string()));
        }
        self
    }

    /// Enable Shadow DOM for this element.
    pub fn with_shadow(mut self) -> Self {
        self.shadow = true;
        self
    }

    /// Set the children HTML (slot content).
    pub fn child(mut self, html: impl Into<String>) -> Self {
        self.children_html = html.into();
        self
    }

    /// Render the custom element to an HTML string (for SSR).
    pub fn to_html(&self) -> String {
        let attrs: String = self
            .attributes
            .iter()
            .map(|(name, value)| format!(" {}=\"{}\"", name, escape_attr(value)))
            .collect();

        if self.children_html.is_empty() {
            format!("<{}{}></{}>", self.tag, attrs, self.tag)
        } else {
            format!(
                "<{}{}>{}</{}>",
                self.tag, attrs, self.children_html, self.tag
            )
        }
    }

    /// Get the tag name.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Get the attributes.
    pub fn attributes(&self) -> &[(String, String)] {
        &self.attributes
    }

    /// Whether Shadow DOM is enabled.
    pub fn uses_shadow(&self) -> bool {
        self.shadow
    }
}

/// Definition of a rye component exported as a Web Component.
pub struct WebComponentDef {
    /// The custom element tag name.
    pub tag: String,
    /// Whether to use Shadow DOM.
    pub use_shadow: bool,
    /// Observed attributes (triggers `attributeChangedCallback`).
    pub observed_attributes: Vec<String>,
}

impl WebComponentDef {
    /// Create a new Web Component definition.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            use_shadow: true,
            observed_attributes: Vec::new(),
        }
    }

    /// Set whether to use Shadow DOM (default: true).
    pub fn with_shadow(mut self, shadow: bool) -> Self {
        self.use_shadow = shadow;
        self
    }

    /// Add an observed attribute.
    pub fn observe(mut self, attr: impl Into<String>) -> Self {
        self.observed_attributes.push(attr.into());
        self
    }

    /// Add multiple observed attributes.
    pub fn observe_many(mut self, attrs: &[&str]) -> Self {
        for attr in attrs {
            self.observed_attributes.push(attr.to_string());
        }
        self
    }
}

/// Generate the JavaScript boilerplate to register a rye component
/// as a custom element.
///
/// This script defines a custom element class that:
/// 1. Creates a Shadow DOM root (if enabled)
/// 2. Mounts the rye Wasm app into the shadow root
/// 3. Maps attributes to component props
/// 4. Cleans up on disconnect
pub fn define_component_script(def: &WebComponentDef) -> String {
    let observed = def
        .observed_attributes
        .iter()
        .map(|a| format!("'{}'", a))
        .collect::<Vec<_>>()
        .join(", ");

    let shadow_line = if def.use_shadow {
        "this._shadow = this.attachShadow({mode: 'open'});"
    } else {
        "this._shadow = this;"
    };

    format!(
        r#"<script>
(function() {{
  class {class_name} extends HTMLElement {{
    constructor() {{
      super();
      {shadow_line}
      this._rye_root = null;
      this._props = {{}};
    }}

    static get observedAttributes() {{
      return [{observed}];
    }}

    connectedCallback() {{
      // Mount rye app into shadow root
      if (window.__rye_mount) {{
        this._rye_root = window.__rye_mount(this._shadow, this._props);
      }}
    }}

    disconnectedCallback() {{
      if (this._rye_root && window.__rye_unmount) {{
        window.__rye_unmount(this._rye_root);
        this._rye_root = null;
      }}
    }}

    attributeChangedCallback(name, oldVal, newVal) {{
      this._props[name] = newVal;
      if (this._rye_root && window.__rye_update_props) {{
        window.__rye_update_props(this._rye_root, this._props);
      }}
    }}
  }}

  customElements.define('{tag}', {class_name});
}})();
</script>"#,
        class_name = tag_to_class_name(&def.tag),
        shadow_line = shadow_line,
        observed = observed,
        tag = def.tag,
    )
}

/// Convert a custom element tag name to a valid JS class name.
///
/// e.g. "my-counter" → "MyCounter", "x-button" → "XButton"
fn tag_to_class_name(tag: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in tag.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Escape a string for use in an HTML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_element_basic() {
        let el = CustomElement::new("my-chart");
        let html = el.to_html();
        assert_eq!(html, "<my-chart></my-chart>");
    }

    #[test]
    fn test_custom_element_with_attrs() {
        let el = CustomElement::new("my-chart")
            .attr("width", "400")
            .attr("height", "300");
        let html = el.to_html();
        assert!(html.contains(r#"width="400""#));
        assert!(html.contains(r#"height="300""#));
    }

    #[test]
    fn test_custom_element_with_children() {
        let el = CustomElement::new("my-card").child("<p>Hello</p>");
        let html = el.to_html();
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("<my-card>"));
        assert!(html.contains("</my-card>"));
    }

    #[test]
    fn test_custom_element_self_closing() {
        let el = CustomElement::new("x-icon");
        let html = el.to_html();
        assert_eq!(html, "<x-icon></x-icon>");
    }

    #[test]
    fn test_custom_element_shadow() {
        let el = CustomElement::new("my-widget").with_shadow();
        assert!(el.uses_shadow());
    }

    #[test]
    fn test_custom_element_attrs_slice() {
        let el = CustomElement::new("my-chart").attrs(&[("data", "[1,2,3]"), ("type", "bar")]);
        let html = el.to_html();
        assert!(html.contains(r#"data="[1,2,3]""#));
        assert!(html.contains(r#"type="bar""#));
    }

    #[test]
    fn test_custom_element_attr_escaping() {
        let el = CustomElement::new("x-test").attr("value", "a\"b<c>");
        let html = el.to_html();
        assert!(html.contains("&quot;"));
        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
    }

    #[test]
    fn test_web_component_def() {
        let def = WebComponentDef::new("my-counter")
            .with_shadow(true)
            .observe("count")
            .observe_many(&["label", "color"]);

        assert_eq!(def.tag, "my-counter");
        assert!(def.use_shadow);
        assert_eq!(def.observed_attributes, vec!["count", "label", "color"]);
    }

    #[test]
    fn test_define_component_script() {
        let def = WebComponentDef::new("my-counter").observe("count");

        let script = define_component_script(&def);
        assert!(script.contains("customElements.define('my-counter'"));
        assert!(script.contains("MyCounter"));
        assert!(script.contains("attachShadow"));
        assert!(script.contains("'count'"));
        assert!(script.contains("connectedCallback"));
        assert!(script.contains("disconnectedCallback"));
        assert!(script.contains("attributeChangedCallback"));
    }

    #[test]
    fn test_define_component_script_no_shadow() {
        let def = WebComponentDef::new("x-widget").with_shadow(false);
        let script = define_component_script(&def);
        assert!(!script.contains("attachShadow"));
    }

    #[test]
    fn test_tag_to_class_name() {
        assert_eq!(tag_to_class_name("my-counter"), "MyCounter");
        assert_eq!(tag_to_class_name("x-button"), "XButton");
        assert_eq!(tag_to_class_name("my-very-long-name"), "MyVeryLongName");
    }
}
