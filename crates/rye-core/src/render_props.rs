//! Render delegation / render props — components that accept a render function as a prop.
//!
//! `#[prop(render)]` attribute marks a prop as a render function.
//! Complements existing slot system. Gives AI and developers more composition patterns.

use std::rc::Rc;

/// A render prop — a function that returns renderable content.
///
/// Instead of passing data to a child component, you pass a function
/// that the child calls to get its content. This enables the "render prop"
/// pattern popularized by React.
///
/// # Example
/// ```
/// use rye_core::render_props::RenderProp;
///
/// let data = vec![1, 2, 3];
/// let render_item: RenderProp<i32> = RenderProp::new(|item| {
///     format!("<li>{}</li>", item)
/// });
///
/// let html: String = data.iter().map(|item| render_item.render(item)).collect();
/// assert!(html.contains("<li>1</li>"));
/// ```
pub struct RenderProp<T: 'static> {
    render_fn: Rc<dyn Fn(&T) -> String>,
}

impl<T: 'static> RenderProp<T> {
    /// Create a new render prop from a function.
    pub fn new<F: Fn(&T) -> String + 'static>(render_fn: F) -> Self {
        Self {
            render_fn: Rc::new(render_fn),
        }
    }

    /// Render the content for the given value.
    pub fn render(&self, value: &T) -> String {
        (self.render_fn)(value)
    }

    /// Render for multiple values, joining the results.
    pub fn render_all(&self, values: &[T]) -> String {
        values.iter().map(|v| self.render(v)).collect::<Vec<_>>().join("")
    }
}

impl<T: 'static> Clone for RenderProp<T> {
    fn clone(&self) -> Self {
        Self {
            render_fn: Rc::clone(&self.render_fn),
        }
    }
}

/// A render prop that receives an index along with the value.
pub struct IndexedRenderProp<T: 'static> {
    render_fn: Rc<dyn Fn(usize, &T) -> String>,
}

impl<T: 'static> IndexedRenderProp<T> {
    /// Create a new indexed render prop.
    pub fn new<F: Fn(usize, &T) -> String + 'static>(render_fn: F) -> Self {
        Self {
            render_fn: Rc::new(render_fn),
        }
    }

    /// Render the content for the given index and value.
    pub fn render(&self, index: usize, value: &T) -> String {
        (self.render_fn)(index, value)
    }

    /// Render for multiple values with their indices.
    pub fn render_all(&self, values: &[T]) -> String {
        values
            .iter()
            .enumerate()
            .map(|(i, v)| self.render(i, v))
            .collect::<Vec<_>>()
            .join("")
    }
}

impl<T: 'static> Clone for IndexedRenderProp<T> {
    fn clone(&self) -> Self {
        Self {
            render_fn: Rc::clone(&self.render_fn),
        }
    }
}

/// A render prop that can render nothing (optional content).
pub struct OptionRenderProp<T: 'static> {
    render_fn: Rc<dyn Fn(&T) -> Option<String>>,
}

impl<T: 'static> OptionRenderProp<T> {
    /// Create a new option render prop.
    pub fn new<F: Fn(&T) -> Option<String> + 'static>(render_fn: F) -> Self {
        Self {
            render_fn: Rc::new(render_fn),
        }
    }

    /// Render the content, or None if the prop returns None.
    pub fn render(&self, value: &T) -> Option<String> {
        (self.render_fn)(value)
    }
}

impl<T: 'static> Clone for OptionRenderProp<T> {
    fn clone(&self) -> Self {
        Self {
            render_fn: Rc::clone(&self.render_fn),
        }
    }
}

/// A component that delegates rendering to a render prop.
pub struct RenderPropComponent<T: 'static> {
    prop: RenderProp<T>,
}

impl<T: 'static> RenderPropComponent<T> {
    /// Create a new render prop component.
    pub fn new(prop: RenderProp<T>) -> Self {
        Self { prop }
    }

    /// Render the component with a value.
    pub fn render(&self, value: &T) -> String {
        self.prop.render(value)
    }
}

impl<T: 'static> Clone for RenderPropComponent<T> {
    fn clone(&self) -> Self {
        Self {
            prop: self.prop.clone(),
        }
    }
}

/// A switch render prop — renders different content based on a condition.
pub struct SwitchRenderProp<T: 'static> {
    cases: Vec<(Rc<dyn Fn(&T) -> bool>, Rc<dyn Fn(&T) -> String>)>,
    default: Option<Rc<dyn Fn(&T) -> String>>,
}

impl<T: 'static> SwitchRenderProp<T> {
    /// Create a new switch render prop.
    pub fn new() -> Self {
        Self {
            cases: Vec::new(),
            default: None,
        }
    }

    /// Add a case to the switch.
    pub fn case<F, R>(mut self, condition: F, render: R) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        R: Fn(&T) -> String + 'static,
    {
        self.cases.push((Rc::new(condition), Rc::new(render)));
        self
    }

    /// Set the default case.
    pub fn default<F: Fn(&T) -> String + 'static>(mut self, render: F) -> Self {
        self.default = Some(Rc::new(render));
        self
    }

    /// Render the appropriate case for the given value.
    pub fn render(&self, value: &T) -> String {
        for (condition, render) in &self.cases {
            if condition(value) {
                return render(value);
            }
        }
        if let Some(default) = &self.default {
            return default(value);
        }
        String::new()
    }
}

impl<T: 'static> Default for SwitchRenderProp<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Clone for SwitchRenderProp<T> {
    fn clone(&self) -> Self {
        Self {
            cases: self.cases.clone(),
            default: self.default.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_prop_basic() {
        let prop: RenderProp<i32> = RenderProp::new(|v| format!("<span>{}</span>", v));
        assert_eq!(prop.render(&42), "<span>42</span>");
    }

    #[test]
    fn test_render_prop_all() {
        let prop: RenderProp<i32> = RenderProp::new(|v| format!("<li>{}</li>", v));
        let result = prop.render_all(&[1, 2, 3]);
        assert_eq!(result, "<li>1</li><li>2</li><li>3</li>");
    }

    #[test]
    fn test_render_prop_clone() {
        let prop: RenderProp<i32> = RenderProp::new(|v| format!("{}", v));
        let prop2 = prop.clone();
        assert_eq!(prop.render(&5), prop2.render(&5));
    }

    #[test]
    fn test_indexed_render_prop() {
        let prop: IndexedRenderProp<String> = IndexedRenderProp::new(|i, v| {
            format!("<div data-index=\"{}\">{}</div>", i, v)
        });
        let result = prop.render_all(&["a".to_string(), "b".to_string()]);
        assert!(result.contains("data-index=\"0\""));
        assert!(result.contains("data-index=\"1\""));
    }

    #[test]
    fn test_option_render_prop_some() {
        let prop: OptionRenderProp<i32> = OptionRenderProp::new(|v| {
            if *v > 0 {
                Some(format!("Positive: {}", v))
            } else {
                None
            }
        });
        assert_eq!(prop.render(&5), Some("Positive: 5".to_string()));
        assert_eq!(prop.render(&-1), None);
    }

    #[test]
    fn test_render_prop_component() {
        let prop = RenderProp::new(|v: &String| format!("<p>{}</p>", v));
        let component = RenderPropComponent::new(prop);
        assert_eq!(component.render(&"hello".to_string()), "<p>hello</p>");
    }

    #[test]
    fn test_switch_render_prop() {
        let switch: SwitchRenderProp<i32> = SwitchRenderProp::new()
            .case(|v| *v > 10, |v| format!("Big: {}", v))
            .case(|v| *v > 0, |v| format!("Small: {}", v))
            .default(|v| format!("Zero or negative: {}", v));

        assert_eq!(switch.render(&20), "Big: 20");
        assert_eq!(switch.render(&5), "Small: 5");
        assert_eq!(switch.render(&0), "Zero or negative: 0");
    }

    #[test]
    fn test_switch_render_prop_no_default() {
        let switch: SwitchRenderProp<i32> = SwitchRenderProp::new()
            .case(|v| *v > 0, |v| format!("Positive: {}", v));

        assert_eq!(switch.render(&5), "Positive: 5");
        assert_eq!(switch.render(&-1), "");
    }

    #[test]
    fn test_switch_render_prop_clone() {
        let switch: SwitchRenderProp<i32> = SwitchRenderProp::new()
            .case(|v| *v > 0, |v| format!("Pos: {}", v));
        let switch2 = switch.clone();
        assert_eq!(switch.render(&5), switch2.render(&5));
    }

    #[test]
    fn test_render_prop_with_struct() {
        struct User {
            name: String,
            age: i32,
        }
        let prop: RenderProp<User> = RenderProp::new(|u: &User| {
            format!("<div>{} ({})</div>", u.name, u.age)
        });
        let user = User { name: "Alice".to_string(), age: 30 };
        assert_eq!(prop.render(&user), "<div>Alice (30)</div>");
    }
}
