//! Render — server-side rendering to string.

use rye_core::renderer::{EventHandler, Renderer};
use std::cell::RefCell;
use std::rc::Rc;

/// SSR renderer — produces HTML strings with hydration markers.
pub struct SsrRenderer {
    /// Counter for unique hydration IDs.
    next_id: RefCell<usize>,
}

/// SSR node — either an element or text, stored as a string.
#[derive(Clone, Debug)]
pub struct SsrNode {
    html: String,
}

/// SSR element — an HTML element string.
#[derive(Clone, Debug)]
pub struct SsrElement {
    html: String,
}

/// SSR text — a text node string.
#[derive(Clone, Debug)]
pub struct SsrText {
    content: String,
}

impl SsrRenderer {
    /// Create a new SSR renderer.
    pub fn new() -> Self {
        Self {
            next_id: RefCell::new(0),
        }
    }

    fn next_hydration_id(&self) -> String {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() = id + 1;
        format!("r{}", id)
    }
}

impl Default for SsrRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SsrRenderer {
    type Node = SsrNode;
    type Text = SsrText;
    type Element = SsrElement;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        let hydration_id = self.next_hydration_id();
        SsrElement {
            html: format!("<{} data-rye-id=\"{}\">", tag, hydration_id),
        }
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        SsrText {
            content: html_escape(content),
        }
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        // SSR text is immutable in this simple impl; in a real impl we'd use Rc<RefCell>
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        // In a real impl, we'd modify the element's attribute map
        // For now, SSR elements are built at creation time
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        // Same as above
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        // In a real impl, we'd insert into a children vector
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        // Same as above
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        // Same as above
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        // Same as above
    }

    fn set_event_listener(&mut self, _el: &Self::Element, _event: &str, _handler: EventHandler) {
        // SSR doesn't attach event listeners — they're attached during hydration
    }

    fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {
        // No-op in SSR
    }

    fn root(&self) -> Self::Element {
        SsrElement {
            html: String::new(),
        }
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        SsrNode {
            html: text.content.clone(),
        }
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        SsrNode {
            html: el.html.clone(),
        }
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render an SSR element tree to an HTML string with hydration markers.
///
/// Walks the `Element` tree and produces HTML with `data-rye-id` attributes
/// on each element node. These markers allow the client-side hydration to
/// match server-rendered DOM nodes to the component tree.
///
/// # Example
/// ```ignore
/// use rye_core::Element;
/// use rye_core::Template;
///
/// let el = Element::Template(Template::new_element(
///     "div",
///     vec![("class".to_string(), "container".to_string())],
///     Vec::new(),
///     vec![Template::text("Hello, world!")],
/// ));
///
/// let html = render_to_string(&el);
/// assert!(html.contains("<div"));
/// assert!(html.contains("Hello, world!"));
/// ```
pub fn render_to_string(root: &rye_core::Element) -> String {
    let mut output = String::new();
    let mut id_counter = 0usize;
    render_element_to_html(root, &mut output, &mut id_counter);
    output
}

/// Render an Element tree to a complete HTML document string.
///
/// Wraps the body content in a proper `<!DOCTYPE html>` document structure
/// with `<head>` containing the provided CSS and a `<body>` containing the
/// rendered element tree with hydration markers.
///
/// # Example
/// ```ignore
/// use rye_ssr::render_to_html_document;
/// use rye_ui::{ThemeProvider, ThemeProviderProps};
///
/// let css = ThemeProvider::css_only(&ThemeProviderProps::light());
/// let body = my_app::render_app();
/// let html = render_to_html_document(&body, &css, "My App");
/// ```
pub fn render_to_html_document(root: &rye_core::Element, css: &str, title: &str) -> String {
    let body = render_to_string(root);
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n\
         <meta charset=\"utf-8\"/>\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\n\
         <title>{title}</title>\n\
         <style id=\"rye-theme\">\n{css}\n</style>\n\
         </head>\n<body>\n{body}\n</body>\n</html>\n",
        title = html_escape(title),
        css = css,
        body = body,
    )
}

/// Render an Element to HTML string.
fn render_element_to_html(element: &rye_core::Element, output: &mut String, id_counter: &mut usize) {
    match element {
        rye_core::Element::None => {}
        rye_core::Element::Template(template) => {
            for node in &template.nodes {
                render_template_node_to_html(node, output, id_counter);
            }
        }
        rye_core::Element::Fragment(elements) => {
            for el in elements {
                render_element_to_html(el, output, id_counter);
            }
        }
        rye_core::Element::Component(_) => {
            // Components are opaque — in a full impl we'd recursively render
            output.push_str("<!--rye-component-->");
        }
    }
}

/// Render a TemplateNode to HTML.
fn render_template_node_to_html(
    node: &rye_core::TemplateNode,
    output: &mut String,
    id_counter: &mut usize,
) {
    match node {
        rye_core::TemplateNode::Text(text) => {
            output.push_str(&html_escape(text));
        }
        rye_core::TemplateNode::Dynamic(value) => {
            // Try to extract the value as a string
            if let Some(s) = value.downcast_ref::<String>() {
                output.push_str(&html_escape(s));
            } else if let Some(s) = value.downcast_ref::<&str>() {
                output.push_str(&html_escape(s));
            } else if let Some(n) = value.downcast_ref::<i32>() {
                output.push_str(&n.to_string());
            } else if let Some(n) = value.downcast_ref::<u32>() {
                output.push_str(&n.to_string());
            } else if let Some(n) = value.downcast_ref::<i64>() {
                output.push_str(&n.to_string());
            } else if let Some(n) = value.downcast_ref::<u64>() {
                output.push_str(&n.to_string());
            } else if let Some(n) = value.downcast_ref::<f64>() {
                output.push_str(&n.to_string());
            } else if let Some(b) = value.downcast_ref::<bool>() {
                output.push_str(&b.to_string());
            }
        }
        rye_core::TemplateNode::Reactive(compute) => {
            // For SSR, evaluate the reactive closure once
            let value = compute();
            output.push_str(&html_escape(&value));
        }
        rye_core::TemplateNode::Element {
            tag,
            attrs,
            reactive_attrs,
            events,
            children,
        } => {
            let hydration_id = format!("r{}", id_counter);
            *id_counter += 1;

            // Void elements that don't have closing tags
            let is_void = matches!(
                tag.as_str(),
                "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "source" | "track" | "wbr"
            );

            output.push('<');
            output.push_str(tag);
            output.push_str(&format!(" data-rye-id=\"{}\"", hydration_id));

            for (name, value) in attrs {
                output.push(' ');
                output.push_str(name);
                output.push_str("=\"");
                output.push_str(&html_escape(value));
                output.push('"');
            }

            // Reactive attributes — evaluated once for SSR
            for (name, compute) in reactive_attrs {
                let value = compute();
                output.push(' ');
                output.push_str(name);
                output.push_str("=\"");
                output.push_str(&html_escape(&value));
                output.push('"');
            }

            // Event handlers are marked with data attributes for hydration
            for (event, _) in events {
                output.push_str(&format!(" data-rye-event=\"{}\"", event));
            }

            if is_void {
                output.push_str(" />");
                return;
            }

            output.push('>');

            for child in children {
                for node in &child.nodes {
                    render_template_node_to_html(node, output, id_counter);
                }
            }

            output.push_str("</");
            output.push_str(tag);
            output.push('>');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rye_core::{Element, Template, TemplateNode};

    #[test]
    fn test_render_to_string_text() {
        let el = Element::Template(Template::text("Hello, world!"));
        let html = render_to_string(&el);
        assert_eq!(html, "Hello, world!");
    }

    #[test]
    fn test_render_to_string_element() {
        let el = Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "container".to_string())],
            Vec::new(),
            vec![Template::text("Hello")],
        ));
        let html = render_to_string(&el);
        assert!(html.contains("<div"));
        assert!(html.contains("class=\"container\""));
        assert!(html.contains("data-rye-id=\"r0\""));
        assert!(html.contains("Hello"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_render_to_string_nested() {
        let child = Template::new_element(
            "span",
            vec![],
            Vec::new(),
            vec![Template::text("inner")],
        );
        let el = Element::Template(Template::new_element(
            "div",
            vec![],
            Vec::new(),
            vec![child],
        ));
        let html = render_to_string(&el);
        assert!(html.contains("<div"));
        assert!(html.contains("<span"));
        assert!(html.contains("inner"));
        assert!(html.contains("</span>"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_render_to_string_void_element() {
        let el = Element::Template(Template::new_element(
            "input",
            vec![("type".to_string(), "text".to_string())],
            Vec::new(),
            Vec::new(),
        ));
        let html = render_to_string(&el);
        assert!(html.contains("<input"));
        assert!(html.contains("type=\"text\""));
        assert!(html.contains("/>"));
        assert!(!html.contains("</input>"));
    }

    #[test]
    fn test_render_to_string_html_escape() {
        let el = Element::Template(Template::text("<script>alert('xss')</script>"));
        let html = render_to_string(&el);
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_to_string_fragment() {
        let el = Element::Fragment(vec![
            Element::Template(Template::text("A")),
            Element::Template(Template::text("B")),
        ]);
        let html = render_to_string(&el);
        assert_eq!(html, "AB");
    }

    #[test]
    fn test_render_to_string_none() {
        let el = Element::None;
        let html = render_to_string(&el);
        assert_eq!(html, "");
    }

    #[test]
    fn test_render_to_string_with_events() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let handler: rye_core::renderer::EventHandler = Box::new(|_| {});
        let shared: rye_core::template::SharedEventHandler =
            Rc::new(RefCell::new(handler));
        let el = Element::Template(Template::new_element(
            "button",
            Vec::new(),
            vec![("click".to_string(), shared)],
            vec![Template::text("Click")],
        ));
        let html = render_to_string(&el);
        assert!(html.contains("data-rye-event=\"click\""));
        assert!(html.contains("Click"));
    }

    #[test]
    fn test_render_to_string_dynamic_string() {
        let el = Element::Template(Template::new(vec![TemplateNode::Dynamic(
            Box::new("dynamic value".to_string()),
        )]));
        let html = render_to_string(&el);
        assert!(html.contains("dynamic value"));
    }

    #[test]
    fn test_render_to_string_dynamic_number() {
        let el = Element::Template(Template::new(vec![TemplateNode::Dynamic(
            Box::new(42i32),
        )]));
        let html = render_to_string(&el);
        assert!(html.contains("42"));
    }

    #[test]
    fn test_hydration_ids_increment() {
        let child = Template::new_element("span", Vec::new(), Vec::new(), vec![Template::text("inner")]);
        let el = Element::Template(Template::new_element(
            "div",
            Vec::new(),
            Vec::new(),
            vec![child],
        ));
        let html = render_to_string(&el);
        assert!(html.contains("data-rye-id=\"r0\""));
        assert!(html.contains("data-rye-id=\"r1\""));
    }

    #[test]
    fn test_render_to_html_document() {
        let el = Element::Template(Template::new(vec![TemplateNode::Text("Hello".to_string())]));
        let html = render_to_html_document(&el, ":root { --rye-primary: #2563eb; }", "Test App");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<title>Test App</title>"));
        assert!(html.contains("<style id=\"rye-theme\">"));
        assert!(html.contains("--rye-primary: #2563eb"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("</html>"));
    }
}
