//! Query helpers — find elements in the test render tree.

use crate::test_renderer::{TestElement, TestNode, TestNodeKind};
use std::rc::Rc;
use std::cell::RefCell;

/// Get all text content from a render tree.
pub fn get_all_text(node: &TestNode) -> String {
    match &node.kind {
        TestNodeKind::Text(t) => t.borrow().content.clone(),
        TestNodeKind::Element(el) => el.borrow().text_content(),
        TestNodeKind::None => String::new(),
    }
}

/// Find elements by tag name.
pub fn get_by_tag(root: &Rc<RefCell<TestElement>>, tag: &str) -> Vec<Rc<RefCell<TestElement>>> {
    let mut results = Vec::new();
    find_by_tag(root, tag, &mut results);
    results
}

fn find_by_tag(
    el: &Rc<RefCell<TestElement>>,
    tag: &str,
    results: &mut Vec<Rc<RefCell<TestElement>>>,
) {
    if el.borrow().tag == tag {
        results.push(Rc::clone(el));
    }
    let children: Vec<TestNode> = el.borrow().children.clone();
    for child in &children {
        if let TestNodeKind::Element(child_el) = &child.kind {
            find_by_tag(child_el, tag, results);
        }
    }
}

/// Find elements by attribute value.
pub fn get_by_attribute(
    root: &Rc<RefCell<TestElement>>,
    name: &str,
    value: &str,
) -> Vec<Rc<RefCell<TestElement>>> {
    let mut results = Vec::new();
    find_by_attribute(root, name, value, &mut results);
    results
}

fn find_by_attribute(
    el: &Rc<RefCell<TestElement>>,
    name: &str,
    value: &str,
    results: &mut Vec<Rc<RefCell<TestElement>>>,
) {
    if el.borrow().attrs.iter().any(|(n, v)| n == name && v == value) {
        results.push(Rc::clone(el));
    }
    let children: Vec<TestNode> = el.borrow().children.clone();
    for child in &children {
        if let TestNodeKind::Element(child_el) = &child.kind {
            find_by_attribute(child_el, name, value, results);
        }
    }
}

/// Find elements by class name.
pub fn get_by_class(root: &Rc<RefCell<TestElement>>, class: &str) -> Vec<Rc<RefCell<TestElement>>> {
    let mut results = Vec::new();
    find_by_class(root, class, &mut results);
    results
}

fn find_by_class(
    el: &Rc<RefCell<TestElement>>,
    class: &str,
    results: &mut Vec<Rc<RefCell<TestElement>>>,
) {
    if el
        .borrow()
        .attrs
        .iter()
        .any(|(n, v)| n == "class" && v.split_whitespace().any(|c| c == class))
    {
        results.push(Rc::clone(el));
    }
    let children: Vec<TestNode> = el.borrow().children.clone();
    for child in &children {
        if let TestNodeKind::Element(child_el) = &child.kind {
            find_by_class(child_el, class, results);
        }
    }
}

/// Find elements by test_id attribute.
pub fn get_by_test_id(root: &Rc<RefCell<TestElement>>, test_id: &str) -> Vec<Rc<RefCell<TestElement>>> {
    get_by_attribute(root, "data-testid", test_id)
}

/// Find elements containing the given text.
pub fn get_by_text(root: &Rc<RefCell<TestElement>>, text: &str) -> Vec<Rc<RefCell<TestElement>>> {
    let mut results = Vec::new();
    find_by_text(root, text, &mut results);
    results
}

fn find_by_text(
    el: &Rc<RefCell<TestElement>>,
    text: &str,
    results: &mut Vec<Rc<RefCell<TestElement>>>,
) {
    if el.borrow().text_content().contains(text) {
        results.push(Rc::clone(el));
    }
    let children: Vec<TestNode> = el.borrow().children.clone();
    for child in &children {
        if let TestNodeKind::Element(child_el) = &child.kind {
            find_by_text(child_el, text, results);
        }
    }
}
