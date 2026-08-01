//! Event simulation — fire events on test elements.

use crate::test_renderer::{TestElement, TestRenderer};
use std::cell::RefCell;
use std::rc::Rc;

/// Simulate a click event on an element.
pub fn fire_click(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "click", &());
}

/// Simulate an input event with the given value.
pub fn fire_input(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>, value: &str) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "input", &value.to_string());
}

/// Simulate a keydown event.
pub fn fire_keydown(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>, key: &str) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "keydown", &key.to_string());
}

/// Simulate a keyup event.
pub fn fire_keyup(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>, key: &str) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "keyup", &key.to_string());
}

/// Simulate a focus event.
pub fn fire_focus(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "focus", &());
}

/// Simulate a blur event.
pub fn fire_blur(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "blur", &());
}

/// Simulate a submit event on a form.
pub fn fire_submit(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, "submit", &());
}

/// Simulate a custom event.
pub fn fire_event(renderer: &mut TestRenderer, el: &Rc<RefCell<TestElement>>, event: &str, payload: &dyn std::any::Any) {
    let addr = Rc::as_ptr(el) as usize;
    renderer.fire_event(addr, event, payload);
}
