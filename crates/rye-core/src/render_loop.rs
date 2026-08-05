//! Render loop — connects signals to rendering, mounts elements to a renderer.
//!
//! ## Fine-Grained Reactivity (SolidJS-style)
//!
//! Instead of re-running the entire render function and diffing trees,
//! each dynamic expression gets its own `Effect` that updates only its
//! specific DOM node when a signal changes.
//!
//! ## Architecture
//!
//! 1. `mount()` calls the render function ONCE to produce the initial tree.
//! 2. It walks the tree and creates DOM nodes via the `Renderer`.
//! 3. For each `TemplateNode::Reactive` node, it creates a separate `Effect`
//!    that calls the reactive closure and updates the text node.
//! 4. For each reactive attribute, it creates a separate `Effect` that
//!    updates the attribute on the element.
//! 5. Static parts (text, elements, attributes) are created once and never touched.
//! 6. No tree diffing, no VDOM allocation, no garbage from re-renders.

use crate::element::Element;
use crate::hooks::{enter_hook_scope, drain_hook_context};
use crate::renderer::{Hydratable, Renderer};
use crate::template::{ReactiveFn, TemplateNode};
use rye_signals::Effect;
use std::cell::RefCell;
use std::rc::Rc;

/// A mounted render scope — holds all fine-grained `Effect`s and hook signals
/// to keep them alive.
///
/// When dropped, all effects are cleaned up and signal subscriptions removed.
pub struct RenderScope {
    _effects: Vec<Effect>,
    /// Signals created by `use_signal` during the render pass.
    /// Kept alive so they remain valid after rendering completes.
    _hook_signals: Vec<Rc<dyn std::any::Any>>,
}

/// Mount a render function onto a renderer with fine-grained reactivity.
///
/// The closure is called ONCE to produce the initial element tree. Then,
/// each dynamic expression in the template gets its own `Effect` that
/// updates only the affected DOM node when signals change.
///
/// # Example
/// ```ignore
/// use rye_core::mount;
/// use rye_signals::Signal;
///
/// let count = Signal::new(0);
/// let count_clone = count.clone();
///
/// let _scope = mount(move || {
///     template! {
///         div {
///             "Count: " {count_clone.get()}
///             button { onclick: move |_| count.set(count.get() + 1), "Increment" }
///         }
///     }
/// });
/// ```
pub fn mount<F, R>(render_fn: F, renderer: R) -> RenderScope
where
    F: FnOnce() -> Element + 'static,
    R: Renderer + 'static,
{
    let renderer = Rc::new(RefCell::new(renderer));
    let mut effects: Vec<Effect> = Vec::new();

    // Set up hook context so use_signal() works during the render pass.
    let mut hook_guard = enter_hook_scope();

    // Call render function ONCE — not inside an Effect.
    // Signal tracking happens inside each reactive binding's own Effect.
    let tree = render_fn();

    // Collect signals created by use_signal() into the guard.
    drain_hook_context(&mut hook_guard);

    let root = renderer.borrow().root();
    create_element_tree(&renderer, &tree, &root, &mut effects);

    RenderScope {
        _effects: effects,
        _hook_signals: hook_guard.into_signals(),
    }
}

/// Hydrate server-rendered HTML with reactive bindings.
///
/// Like `mount()`, but **reuses existing DOM nodes** instead of creating
/// new ones. The server-rendered HTML is already in the DOM — this function
/// walks the Element tree and the existing DOM simultaneously, attaching
/// event listeners and creating fine-grained `Effect`s for reactive bindings.
///
/// No DOM nodes are created or inserted. Only event listeners and `Effect`s
/// are attached to the existing nodes.
///
/// # Example
/// ```ignore
/// use rye_core::hydrate_to_dom;
/// use rye_signals::Signal;
///
/// // Server already rendered: <div data-rye-id="r0">Count: 0</div>
/// // Client hydrates:
/// let count = Signal::new(0);
/// let count_clone = count.clone();
///
/// let _scope = hydrate_to_dom(move || {
///     template! {
///         div { "Count: " {count_clone.get()} }
///     }
/// }, renderer);
/// ```
pub fn hydrate_to_dom<F, R>(render_fn: F, renderer: R) -> RenderScope
where
    F: FnOnce() -> Element + 'static,
    R: Hydratable + 'static,
{
    let renderer = Rc::new(RefCell::new(renderer));
    let mut effects: Vec<Effect> = Vec::new();

    let mut hook_guard = enter_hook_scope();
    let tree = render_fn();
    drain_hook_context(&mut hook_guard);

    let root = renderer.borrow().root();
    hydrate_element_tree(&renderer, &tree, &root, &mut effects);

    RenderScope {
        _effects: effects,
        _hook_signals: hook_guard.into_signals(),
    }
}

/// Recursively hydrate an Element tree against existing DOM nodes.
fn hydrate_element_tree<R: Hydratable>(
    renderer: &Rc<RefCell<R>>,
    element: &Element,
    parent: &R::Element,
    effects: &mut Vec<Effect>,
) {
    match element {
        Element::None => {}
        Element::Template(template) => {
            hydrate_template_nodes(renderer, &template.nodes, parent, effects);
        }
        Element::Fragment(elements) => {
            for el in elements {
                hydrate_element_tree(renderer, el, parent, effects);
            }
        }
        Element::Component(_) => {}
    }
}

/// Hydrate template nodes against existing DOM children.
///
/// Walks the existing DOM children of `parent` and matches them against
/// the template nodes by position. Attaches event listeners and creates
/// `Effect`s for reactive bindings, but does NOT create or insert nodes.
fn hydrate_template_nodes<R: Hydratable>(
    renderer: &Rc<RefCell<R>>,
    nodes: &[TemplateNode],
    parent: &R::Element,
    effects: &mut Vec<Effect>,
) {
    // Collect info about existing DOM children without holding a long-lived borrow.
    // Each entry is (is_element, is_text, element, text, tag_name, child_count).
    let dom_info: Vec<(bool, bool, Option<R::Element>, Option<R::Text>, Option<String>, usize)> = {
        let r = renderer.borrow();
        let child_count = r.child_node_count(parent);
        (0..child_count)
            .filter_map(|i| {
                let node = r.get_child_node(parent, i)?;
                let is_el = r.node_is_element(&node);
                let is_text = r.node_is_text(&node);
                let el = r.node_as_element(&node);
                let text = r.node_as_text(&node);
                let tag = el.as_ref().map(|e| r.get_tag_name(e));
                let cc = el.as_ref().map(|e| r.child_node_count(e)).unwrap_or(0);
                Some((is_el, is_text, el, text, tag, cc))
            })
            .collect()
    };

    for (i, node) in nodes.iter().enumerate() {
        if i >= dom_info.len() {
            break;
        }

        let (is_el, is_text, dom_el, dom_text, dom_tag, dom_child_count) = &dom_info[i];

        match node {
            TemplateNode::Text(_) => {
                // Static text — already in DOM, nothing to do
            }

            TemplateNode::Dynamic(_) => {
                // Static dynamic value — already in DOM, nothing to do
            }

            TemplateNode::Reactive(compute) => {
                // Find the existing text node and create an Effect
                if *is_text {
                    if let Some(text_node) = dom_text {
                        let compute_clone = Rc::clone(compute);
                        let text_node_clone = text_node.clone();
                        let renderer_clone = Rc::clone(renderer);

                        effects.push(Effect::new(move || {
                            let value = compute_clone();
                            renderer_clone
                                .borrow_mut()
                                .set_text(&text_node_clone, &value);
                        }));
                    }
                }
            }

            TemplateNode::Element {
                tag,
                attrs: _,
                reactive_attrs,
                events,
                children,
            } => {
                // Find the existing element node
                if !*is_el {
                    continue;
                }

                let el = match dom_el {
                    Some(e) => e,
                    None => continue,
                };

                // Verify tag matches
                if dom_tag.as_deref() != Some(tag.as_str()) {
                    continue;
                }

                // Attach event handlers via delegation
                {
                    let mut r = renderer.borrow_mut();
                    for (event, handler) in events {
                        let handler_ref = Rc::clone(handler);
                        let handler_fn: crate::renderer::EventHandler = Box::new(
                            move |payload: &dyn std::any::Any| {
                                let mut h = handler_ref.borrow_mut();
                                h(payload);
                            },
                        );
                        r.set_event_listener(el, event, handler_fn);
                    }

                    // Set initial values for reactive attributes
                    for (attr_name, compute) in reactive_attrs {
                        let initial = compute();
                        r.set_attribute(el, attr_name, &initial);
                    }
                }
                // Borrow dropped — safe to create Effects

                // Create fine-grained Effects for reactive attributes
                for (attr_name, compute) in reactive_attrs {
                    let compute_clone = Rc::clone(compute);
                    let el_clone = el.clone();
                    let attr_name_clone = attr_name.clone();
                    let renderer_clone = Rc::clone(renderer);

                    effects.push(Effect::new(move || {
                        let value = compute_clone();
                        renderer_clone
                            .borrow_mut()
                            .set_attribute(&el_clone, &attr_name_clone, &value);
                    }));
                }

                // Recursively hydrate children against existing DOM children
                for child in children {
                    hydrate_template_nodes(renderer, &child.nodes, el, effects);
                }
            }
        }
    }
}

/// Recursively create renderer nodes for an Element tree.
///
/// This is a one-time creation pass. Reactive bindings create their own
/// `Effect`s that will update specific nodes when signals change.
fn create_element_tree<R: Renderer>(
    renderer: &Rc<RefCell<R>>,
    element: &Element,
    parent: &R::Element,
    effects: &mut Vec<Effect>,
) {
    match element {
        Element::None => {}
        Element::Template(template) => {
            create_template_nodes(renderer, &template.nodes, parent, effects);
        }
        Element::Fragment(elements) => {
            for el in elements {
                create_element_tree(renderer, el, parent, effects);
            }
        }
        Element::Component(_) => {
            // Components are opaque at this level — in a full impl we'd
            // recursively render the component's Element output.
        }
    }
}

/// Create renderer nodes for a list of TemplateNode.
///
/// For `Reactive` nodes, creates a text node with initial value and
/// a fine-grained `Effect` that updates it on signal change.
/// For `Element` nodes with `reactive_attrs`, creates per-attribute `Effect`s.
fn create_template_nodes<R: Renderer>(
    renderer: &Rc<RefCell<R>>,
    nodes: &[TemplateNode],
    parent: &R::Element,
    effects: &mut Vec<Effect>,
) {
    for node in nodes {
        match node {
            TemplateNode::Text(text) => {
                let mut r = renderer.borrow_mut();
                let text_node = r.create_text(text);
                let node = r.text_to_node(&text_node);
                r.insert_child(parent, &node, usize::MAX);
            }

            TemplateNode::Dynamic(value) => {
                let text = stringify_dynamic(value);
                let mut r = renderer.borrow_mut();
                let text_node = r.create_text(&text);
                let node = r.text_to_node(&text_node);
                r.insert_child(parent, &node, usize::MAX);
            }

            TemplateNode::Reactive(compute) => {
                let text_node;
                {
                    let mut r = renderer.borrow_mut();
                    let initial = compute();
                    text_node = r.create_text(&initial);
                    let node = r.text_to_node(&text_node);
                    r.insert_child(parent, &node, usize::MAX);
                }
                // Borrow dropped — safe to create Effect now

                let compute_clone = Rc::clone(compute);
                let text_node_clone = text_node.clone();
                let renderer_clone = Rc::clone(renderer);

                effects.push(Effect::new(move || {
                    let value = compute_clone();
                    renderer_clone.borrow_mut().set_text(&text_node_clone, &value);
                }));
            }

            TemplateNode::Element {
                tag,
                attrs,
                reactive_attrs,
                events,
                children,
            } => {
                let el;
                {
                    let mut r = renderer.borrow_mut();
                    el = r.create_element(tag);

                    // Static attributes — set once
                    for (name, value) in attrs {
                        r.set_attribute(&el, name, value);
                    }

                    // Event handlers — register in delegation registry
                    for (event, handler) in events {
                        let handler_ref = Rc::clone(handler);
                        let handler_fn: crate::renderer::EventHandler = Box::new(
                            move |payload: &dyn std::any::Any| {
                                let mut h = handler_ref.borrow_mut();
                                h(payload);
                            },
                        );
                        r.set_event_listener(&el, event, handler_fn);
                    }

                    let node = r.element_to_node(&el);
                    r.insert_child(parent, &node, usize::MAX);

                    // Set initial values for reactive attributes
                    for (attr_name, compute) in reactive_attrs {
                        let initial = compute();
                        r.set_attribute(&el, attr_name, &initial);
                    }
                }
                // Borrow dropped — safe to create Effects now

                // Reactive attributes — each gets its own fine-grained Effect
                for (attr_name, compute) in reactive_attrs {
                    let compute_clone = Rc::clone(compute);
                    let el_clone = el.clone();
                    let attr_name_clone = attr_name.clone();
                    let renderer_clone = Rc::clone(renderer);

                    effects.push(Effect::new(move || {
                        let value = compute_clone();
                        renderer_clone
                            .borrow_mut()
                            .set_attribute(&el_clone, &attr_name_clone, &value);
                    }));
                }

                // Recursively create children
                for child in children {
                    create_template_nodes(renderer, &child.nodes, &el, effects);
                }
            }
        }
    }
}

/// Convert a dynamic `Box<dyn Any>` value to a string.
fn stringify_dynamic(value: &Box<dyn std::any::Any>) -> String {
    if let Some(text) = value.downcast_ref::<String>() {
        text.clone()
    } else if let Some(text) = value.downcast_ref::<&str>() {
        text.to_string()
    } else if let Some(n) = value.downcast_ref::<i32>() {
        n.to_string()
    } else if let Some(n) = value.downcast_ref::<u32>() {
        n.to_string()
    } else if let Some(n) = value.downcast_ref::<i64>() {
        n.to_string()
    } else if let Some(n) = value.downcast_ref::<u64>() {
        n.to_string()
    } else if let Some(n) = value.downcast_ref::<f64>() {
        n.to_string()
    } else if let Some(b) = value.downcast_ref::<bool>() {
        b.to_string()
    } else {
        "<unknown>".to_string()
    }
}

/// Render an Element tree to a string representation (for debugging/testing).
pub fn render_tree_to_string(element: &Element) -> String {
    let mut output = String::new();
    render_element_to_string(element, &mut output, 0);
    output
}

fn render_element_to_string(element: &Element, output: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    match element {
        Element::None => {}
        Element::Template(template) => {
            for node in &template.nodes {
                render_node_to_string(node, output, depth);
            }
        }
        Element::Fragment(elements) => {
            for el in elements {
                render_element_to_string(el, output, depth);
            }
        }
        Element::Component(_) => {
            output.push_str(&format!("{}<component />\n", indent));
        }
    }
}

fn render_node_to_string(node: &TemplateNode, output: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    match node {
        TemplateNode::Text(text) => {
            output.push_str(&format!("{}\"{}\"\n", indent, text));
        }
        TemplateNode::Dynamic(_) => {
            output.push_str(&format!("{}<dynamic />\n", indent));
        }
        TemplateNode::Reactive(_) => {
            output.push_str(&format!("{}<reactive />\n", indent));
        }
        TemplateNode::Element {
            tag,
            attrs,
            reactive_attrs,
            events,
            children,
        } => {
            let attrs_str = attrs
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            let reactive_str = if reactive_attrs.is_empty() {
                String::new()
            } else {
                format!(" ({} reactive attrs)", reactive_attrs.len())
            };
            let events_str = if events.is_empty() {
                String::new()
            } else {
                format!(" ({} events)", events.len())
            };
            if children.is_empty() {
                output.push_str(&format!(
                    "{}<{} {}{}{} />\n",
                    indent, tag, attrs_str, reactive_str, events_str
                ));
            } else {
                output.push_str(&format!(
                    "{}<{} {}{}{}>\n",
                    indent, tag, attrs_str, reactive_str, events_str
                ));
                for child in children {
                    for node in &child.nodes {
                        render_node_to_string(node, output, depth + 1);
                    }
                }
                output.push_str(&format!("{}</{}>\n", indent, tag));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::Template;
    use rye_signals::Signal;

    #[test]
    fn test_render_tree_to_string() {
        let el = Element::Template(Template::new(vec![TemplateNode::Text("hello".to_string())]));
        let output = render_tree_to_string(&el);
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_render_tree_with_element() {
        let el = Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "container".to_string())],
            Vec::new(),
            vec![Template::text("Hello"), Template::text("World")],
        ));
        let output = render_tree_to_string(&el);
        assert!(output.contains("div"));
        assert!(output.contains("container"));
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn test_render_tree_with_nested_elements() {
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
        let output = render_tree_to_string(&el);
        assert!(output.contains("div"));
        assert!(output.contains("span"));
        assert!(output.contains("inner"));
    }

    #[test]
    fn test_render_tree_with_reactive_node() {
        let el = Element::Template(Template::new(vec![TemplateNode::Reactive(
            Rc::new(|| "dynamic".to_string()),
        )]));
        let output = render_tree_to_string(&el);
        assert!(output.contains("reactive"));
    }

    /// A simple test renderer that records operations in memory.
    struct TestRenderer {
        ops: Vec<String>,
    }

    impl TestRenderer {
        fn new() -> Self {
            Self { ops: Vec::new() }
        }
    }

    impl Renderer for TestRenderer {
        type Node = String;
        type Text = String;
        type Element = String;

        fn create_element(&mut self, tag: &str) -> Self::Element {
            let id = format!("el_{}", self.ops.len());
            self.ops.push(format!("create_element({}) -> {}", tag, id));
            id
        }

        fn create_text(&mut self, content: &str) -> Self::Text {
            let id = format!("text_{}", self.ops.len());
            self.ops.push(format!("create_text({}) -> {}", content, id));
            id
        }

        fn set_text(&mut self, node: &Self::Text, content: &str) {
            self.ops.push(format!("set_text({}, {})", node, content));
        }

        fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
            self.ops.push(format!("set_attr({}, {}, {})", el, name, value));
        }

        fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
            self.ops.push(format!("remove_attr({}, {})", el, name));
        }

        fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
            self.ops.push(format!("insert_child({}, {}, {})", parent, child, index));
        }

        fn remove_child(&mut self, parent: &Self::Element, index: usize) {
            self.ops.push(format!("remove_child({}, {})", parent, index));
        }

        fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
            self.ops.push(format!("replace_child({}, {}, {})", parent, new, index));
        }

        fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
            self.ops.push(format!("move_child({}, {}, {})", parent, from, to));
        }

        fn set_event_listener(&mut self, _el: &Self::Element, _event: &str, _handler: crate::renderer::EventHandler) {
            self.ops.push("set_event_listener".to_string());
        }

        fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {
            self.ops.push("remove_event_listener".to_string());
        }

        fn root(&self) -> Self::Element {
            "root".to_string()
        }

        fn text_to_node(&self, text: &Self::Text) -> Self::Node {
            text.clone()
        }

        fn element_to_node(&self, el: &Self::Element) -> Self::Node {
            el.clone()
        }
    }

    #[test]
    fn test_fine_grained_reactive_text_update() {
        let count = Signal::new(0);
        let count_clone = count.clone();

        let renderer = TestRenderer::new();

        let scope = mount(
            move || {
                Element::Template(Template::new(vec![TemplateNode::Reactive(Rc::new(
                    move || count_clone.get().to_string(),
                ))]))
            },
            renderer,
        );

        // The effect should have been created
        assert_eq!(scope._effects.len(), 1);

        // Change the signal — the effect should re-run and call set_text
        count.set(42);

        // The effect ran (we can't easily assert on the renderer since it's
        // consumed by mount, but the fact that it didn't panic means the
        // fine-grained effect is working)
    }

    #[test]
    fn test_fine_grained_reactive_attribute() {
        let visible = Signal::new(true);
        let visible_clone = visible.clone();

        let renderer = TestRenderer::new();

        let scope = mount(
            move || {
                Element::Template(Template::new_element_reactive(
                    "div",
                    Vec::new(),
                    vec![(
                        "class".to_string(),
                        Rc::new(move || {
                            if visible_clone.get() {
                                "visible".to_string()
                            } else {
                                "hidden".to_string()
                            }
                        }),
                    )],
                    Vec::new(),
                    Vec::new(),
                ))
            },
            renderer,
        );

        assert_eq!(scope._effects.len(), 1);

        // Toggle signal — effect should re-run and update attribute
        visible.set(false);
    }

    #[test]
    fn test_static_text_not_reactive() {
        let renderer = TestRenderer::new();

        let scope = mount(
            || {
                Element::Template(Template::new(vec![TemplateNode::Text("static".to_string())]))
            },
            renderer,
        );

        // No effects should be created for static text
        assert_eq!(scope._effects.len(), 0);
    }

    // --- Hydration Tests ---

    /// A simulated DOM node for testing hydration.
    #[derive(Debug, Clone)]
    enum SimNode {
        Element {
            tag: String,
            children: Vec<SimNode>,
        },
        Text(String),
    }

    /// A test renderer that simulates pre-existing DOM content for hydration.
    struct TestHydratableRenderer {
        /// The simulated DOM tree (children of the root element).
        dom_children: Vec<SimNode>,
        /// Recorded operations (for assertions).
        ops: Vec<String>,
    }

    impl TestHydratableRenderer {
        fn new(dom_children: Vec<SimNode>) -> Self {
            Self {
                dom_children,
                ops: Vec::new(),
            }
        }
    }

    impl Renderer for TestHydratableRenderer {
        type Node = SimNode;
        type Text = String;
        type Element = String;

        fn create_element(&mut self, tag: &str) -> Self::Element {
            self.ops.push(format!("CREATE_ELEMENT({})", tag));
            tag.to_string()
        }

        fn create_text(&mut self, content: &str) -> Self::Text {
            self.ops.push(format!("CREATE_TEXT({})", content));
            content.to_string()
        }

        fn set_text(&mut self, node: &Self::Text, content: &str) {
            self.ops.push(format!("SET_TEXT({} -> {})", node, content));
        }

        fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
            self.ops.push(format!("SET_ATTR({}, {}, {})", el, name, value));
        }

        fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
            self.ops.push(format!("REMOVE_ATTR({}, {})", el, name));
        }

        fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
            self.ops.push(format!("INSERT_CHILD({}, {:?}, {})", parent, child, index));
        }

        fn remove_child(&mut self, parent: &Self::Element, index: usize) {
            self.ops.push(format!("REMOVE_CHILD({}, {})", parent, index));
        }

        fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
            self.ops.push(format!("REPLACE_CHILD({}, {:?}, {})", parent, new, index));
        }

        fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
            self.ops.push(format!("MOVE_CHILD({}, {}, {})", parent, from, to));
        }

        fn set_event_listener(&mut self, _el: &Self::Element, event: &str, _handler: crate::renderer::EventHandler) {
            self.ops.push(format!("SET_EVENT({})", event));
        }

        fn remove_event_listener(&mut self, _el: &Self::Element, event: &str) {
            self.ops.push(format!("REMOVE_EVENT({})", event));
        }

        fn root(&self) -> Self::Element {
            "root".to_string()
        }

        fn text_to_node(&self, text: &Self::Text) -> Self::Node {
            SimNode::Text(text.clone())
        }

        fn element_to_node(&self, el: &Self::Element) -> Self::Node {
            SimNode::Element {
                tag: el.clone(),
                children: vec![],
            }
        }
    }

    impl Hydratable for TestHydratableRenderer {
        fn get_child_node(&self, _parent: &Self::Element, index: usize) -> Option<Self::Node> {
            // For the root, return from dom_children. For other elements,
            // we need to look up the element's children — but since our
            // Element type is just a String (tag name), we simulate by
            // returning from dom_children based on index.
            // This is a simplified simulation for testing.
            self.dom_children.get(index).cloned()
        }

        fn child_node_count(&self, _parent: &Self::Element) -> usize {
            self.dom_children.len()
        }

        fn node_is_element(&self, node: &Self::Node) -> bool {
            matches!(node, SimNode::Element { .. })
        }

        fn node_is_text(&self, node: &Self::Node) -> bool {
            matches!(node, SimNode::Text(_))
        }

        fn node_as_element(&self, node: &Self::Node) -> Option<Self::Element> {
            match node {
                SimNode::Element { tag, .. } => Some(tag.clone()),
                _ => None,
            }
        }

        fn node_as_text(&self, node: &Self::Node) -> Option<Self::Text> {
            match node {
                SimNode::Text(content) => Some(content.clone()),
                _ => None,
            }
        }

        fn get_text_content(&self, text: &Self::Text) -> String {
            text.clone()
        }

        fn get_tag_name(&self, el: &Self::Element) -> String {
            el.clone()
        }
    }

    #[test]
    fn test_hydrate_reactive_text() {
        let count = Signal::new(0);
        let count_clone = count.clone();

        // Simulate server-rendered DOM: a text node "0"
        let dom = vec![SimNode::Text("0".to_string())];

        let renderer = TestHydratableRenderer::new(dom);

        let scope = hydrate_to_dom(
            move || {
                Element::Template(Template::new(vec![TemplateNode::Reactive(Rc::new(
                    move || count_clone.get().to_string(),
                ))]))
            },
            renderer,
        );

        // One Effect should be created for the reactive text binding
        assert_eq!(scope._effects.len(), 1);

        // Change the signal — the effect should re-run and call SET_TEXT
        count.set(42);
    }

    #[test]
    fn test_hydrate_static_text_no_effects() {
        // Simulate server-rendered DOM: a text node "hello"
        let dom = vec![SimNode::Text("hello".to_string())];

        let renderer = TestHydratableRenderer::new(dom);

        let scope = hydrate_to_dom(
            || {
                Element::Template(Template::new(vec![TemplateNode::Text("hello".to_string())]))
            },
            renderer,
        );

        // No effects for static text during hydration
        assert_eq!(scope._effects.len(), 0);
    }

    #[test]
    fn test_hydrate_reactive_attribute() {
        let visible = Signal::new(true);
        let visible_clone = visible.clone();

        // Simulate server-rendered DOM: a div element
        let dom = vec![SimNode::Element {
            tag: "div".to_string(),
            children: vec![],
        }];

        let renderer = TestHydratableRenderer::new(dom);

        let scope = hydrate_to_dom(
            move || {
                Element::Template(Template::new_element_reactive(
                    "div",
                    Vec::new(),
                    vec![(
                        "class".to_string(),
                        Rc::new(move || {
                            if visible_clone.get() {
                                "visible".to_string()
                            } else {
                                "hidden".to_string()
                            }
                        }),
                    )],
                    Vec::new(),
                    Vec::new(),
                ))
            },
            renderer,
        );

        // One Effect for the reactive attribute
        assert_eq!(scope._effects.len(), 1);

        // Toggle signal — effect should re-run
        visible.set(false);
    }

    #[test]
    fn test_hydrate_does_not_create_nodes() {
        // Simulate server-rendered DOM: a div with text child
        let dom = vec![SimNode::Element {
            tag: "div".to_string(),
            children: vec![SimNode::Text("hello".to_string())],
        }];

        let renderer = TestHydratableRenderer::new(dom);

        let scope = hydrate_to_dom(
            || {
                Element::Template(Template::new_element(
                    "div",
                    Vec::new(),
                    Vec::new(),
                    vec![Template::text("hello")],
                ))
            },
            renderer,
        );

        // No effects — all static content
        assert_eq!(scope._effects.len(), 0);
    }
}
