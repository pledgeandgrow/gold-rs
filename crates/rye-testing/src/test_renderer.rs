//! Test renderer — in-memory renderer for unit testing without a browser.

use rye_core::renderer::{BatchRenderer, EventHandler, Renderer};
use std::cell::RefCell;
use std::rc::Rc;

/// In-memory test renderer — no browser or WASM needed.
pub struct TestRenderer {
    root: Rc<RefCell<TestElement>>,
    /// Event handlers stored by (element address, event name).
    handlers: Rc<RefCell<Vec<(usize, String, EventHandler)>>>,
    /// Batch state.
    batching: bool,
}

/// A test node — in-memory representation of a DOM node.
#[derive(Clone, Debug)]
pub struct TestNode {
    /// The node content.
    pub kind: TestNodeKind,
}

/// The kind of test node.
#[derive(Clone, Debug)]
pub enum TestNodeKind {
    /// An element node.
    Element(Rc<RefCell<TestElement>>),
    /// A text node.
    Text(Rc<RefCell<TestText>>),
    /// Empty.
    None,
}

impl Default for TestNodeKind {
    fn default() -> Self {
        TestNodeKind::None
    }
}

/// A test element — in-memory representation of a DOM element.
#[derive(Debug, Default)]
pub struct TestElement {
    /// The tag name.
    pub tag: String,
    /// The attributes.
    pub attrs: Vec<(String, String)>,
    /// The children.
    pub children: Vec<TestNode>,
}

/// A test text node.
#[derive(Debug, Default)]
pub struct TestText {
    /// The text content.
    pub content: String,
}

impl TestElement {
    /// Get an attribute by name.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Get all text content from this element and its descendants.
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        for child in &self.children {
            match &child.kind {
                TestNodeKind::Text(t) => {
                    result.push_str(&t.borrow().content);
                }
                TestNodeKind::Element(el) => {
                    result.push_str(&el.borrow().text_content());
                }
                TestNodeKind::None => {}
            }
        }
        result
    }
}

impl Clone for TestElement {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
            children: self.children.clone(),
        }
    }
}

impl Clone for TestText {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
        }
    }
}

impl TestRenderer {
    /// Create a new test renderer.
    pub fn new() -> Self {
        Self {
            root: Rc::new(RefCell::new(TestElement {
                tag: "root".to_string(),
                ..Default::default()
            })),
            handlers: Rc::new(RefCell::new(Vec::new())),
            batching: false,
        }
    }

    /// Get the root element (cloned snapshot).
    pub fn root_element(&self) -> TestElement {
        self.root.borrow().clone()
    }

    /// Fire an event on an element by calling its registered handler.
    pub fn fire_event(&mut self, el_addr: usize, event: &str, payload: &dyn std::any::Any) {
        let mut handlers = self.handlers.borrow_mut();
        for (addr, evt, handler) in handlers.iter_mut() {
            if *addr == el_addr && evt == event {
                handler(payload);
            }
        }
    }
}

impl Default for TestRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn element_addr(el: &Rc<RefCell<TestElement>>) -> usize {
    Rc::as_ptr(el) as usize
}

impl Renderer for TestRenderer {
    type Node = TestNode;
    type Text = Rc<RefCell<TestText>>;
    type Element = Rc<RefCell<TestElement>>;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        Rc::new(RefCell::new(TestElement {
            tag: tag.to_string(),
            ..Default::default()
        }))
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        Rc::new(RefCell::new(TestText {
            content: content.to_string(),
        }))
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        node.borrow_mut().content = content.to_string();
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        let mut el = el.borrow_mut();
        if let Some(attr) = el.attrs.iter_mut().find(|(n, _)| n == name) {
            attr.1 = value.to_string();
        } else {
            el.attrs.push((name.to_string(), value.to_string()));
        }
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        let mut el = el.borrow_mut();
        el.attrs.retain(|(n, _)| n != name);
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        let mut parent = parent.borrow_mut();
        let idx = index.min(parent.children.len());
        parent.children.insert(idx, child.clone());
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        let mut parent = parent.borrow_mut();
        if index < parent.children.len() {
            parent.children.remove(index);
        }
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        let mut parent = parent.borrow_mut();
        if index < parent.children.len() {
            parent.children[index] = new.clone();
        }
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        let mut parent = parent.borrow_mut();
        if from < parent.children.len() && to <= parent.children.len() {
            let child = parent.children.remove(from);
            let to = to.min(parent.children.len());
            parent.children.insert(to, child);
        }
    }

    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler) {
        let addr = element_addr(el);
        // Remove existing handler for this element+event
        self.handlers
            .borrow_mut()
            .retain(|(a, e, _)| *a != addr || e != event);
        self.handlers
            .borrow_mut()
            .push((addr, event.to_string(), handler));
    }

    fn remove_event_listener(&mut self, el: &Self::Element, event: &str) {
        let addr = element_addr(el);
        self.handlers
            .borrow_mut()
            .retain(|(a, e, _)| *a != addr || e != event);
    }

    fn root(&self) -> Self::Element {
        Rc::clone(&self.root)
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        TestNode {
            kind: TestNodeKind::Text(Rc::clone(text)),
        }
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        TestNode {
            kind: TestNodeKind::Element(Rc::clone(el)),
        }
    }
}

impl BatchRenderer for TestRenderer {
    fn begin_batch(&mut self) {
        self.batching = true;
    }

    fn flush_batch(&mut self) {
        self.batching = false;
        // Test renderer applies operations immediately, so flush is a no-op.
    }
}
