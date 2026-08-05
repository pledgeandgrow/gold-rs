//! Input component — text input with label, error, placeholder.

use rye_core::Element;
use rye_core::template::{Template, ReactiveValue, SharedEventHandler, shared_event_handler};
use crate::theme::{Size, vars};

/// Props for the Input component.
pub struct InputProps {
    pub placeholder: String,
    pub value: ReactiveValue<String>,
    pub label: Option<String>,
    pub error: Option<String>,
    pub hint: Option<String>,
    pub disabled: bool,
    pub size: Size,
    pub input_type: InputType,
    pub class: Option<String>,
    pub style: Option<String>,
    /// Optional input event handler (fires on value change).
    pub on_input: Option<SharedEventHandler>,
}

/// HTML input type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Search,
    Tel,
    Url,
}

impl Default for InputType {
    fn default() -> Self { Self::Text }
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text", Self::Password => "password", Self::Email => "email",
            Self::Number => "number", Self::Search => "search", Self::Tel => "tel", Self::Url => "url",
        }
    }
}

impl Default for InputProps {
    fn default() -> Self {
        Self {
            placeholder: String::new(), value: ReactiveValue::Static(String::new()), label: None,
            error: None, hint: None, disabled: false, size: Size::Medium,
            input_type: InputType::Text, class: None, style: None,
            on_input: None,
        }
    }
}

impl InputProps {
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = p.into(); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = ReactiveValue::Static(v.into()); self }

    /// Set the value as a reactive signal.
    pub fn value_reactive(mut self, signal: rye_signals::Signal<String>) -> Self { self.value = ReactiveValue::Reactive(signal); self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn error(mut self, e: impl Into<String>) -> Self { self.error = Some(e.into()); self }
    pub fn hint(mut self, h: impl Into<String>) -> Self { self.hint = Some(h.into()); self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn input_type(mut self, t: InputType) -> Self { self.input_type = t; self }

    /// Set an input event handler (fires on value change).
    pub fn on_input<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&dyn std::any::Any) + 'static,
    {
        self.on_input = Some(shared_event_handler(handler));
        self
    }
}

/// Input component.
pub struct Input;

impl Input {
    pub fn render(props: InputProps) -> Element {
        let border_color = if props.error.is_some() { vars::DANGER } else { vars::INPUT_BORDER };
        let style = format!(
            "width:100%;padding:{};font-size:{};border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};color:var(--rye-text);opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;",
            props.size.padding(), props.size.font_size(), border_color,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "text" },
        );

        let mut children = Vec::new();

        if let Some(label) = &props.label {
            let label_style = "display:block;margin-bottom:4px;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);color:var(--rye-text);";
            children.push(Template::new_element("label",
                vec![("style".to_string(), label_style.to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let mut input_attrs = vec![
            ("type".to_string(), props.input_type.as_str().to_string()),
            ("style".to_string(), if let Some(extra) = &props.style { format!("{}{}", style, extra) } else { style }),
            ("class".to_string(), format!("rye-input {}", props.class.as_deref().unwrap_or(""))),
        ];
        let mut input_events = Vec::new();
        if let Some(handler) = props.on_input {
            input_events.push(("input".to_string(), handler));
        }

        if props.value.is_reactive() {
            input_attrs.push(("value".to_string(), props.value.get()));
            let reactive_attrs = vec![
                ("value".to_string(), props.value.to_reactive_fn()),
            ];
            if props.disabled {
                input_attrs.push(("disabled".to_string(), "true".to_string()));
            }
            if !props.placeholder.is_empty() {
                input_attrs.push(("placeholder".to_string(), props.placeholder.clone()));
            }
            children.push(Template::new_element_reactive("input", input_attrs, reactive_attrs, input_events, Vec::new()));
        } else {
            let val = props.value.get();
            if !val.is_empty() {
                input_attrs.push(("value".to_string(), val));
            }
            if !props.placeholder.is_empty() {
                input_attrs.push(("placeholder".to_string(), props.placeholder.clone()));
            }
            if props.disabled {
                input_attrs.push(("disabled".to_string(), "true".to_string()));
            }
            children.push(Template::new_element("input", input_attrs, input_events, Vec::new()));
        }

        if let Some(error) = &props.error {
            children.push(Template::new_element("span",
                vec![("style".to_string(), "display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:var(--rye-danger);".to_string())],
                Vec::new(), vec![Template::text(error)]));
        } else if let Some(hint) = &props.hint {
            children.push(Template::new_element("span",
                vec![("style".to_string(), "display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:var(--rye-text-muted);".to_string())],
                Vec::new(), vec![Template::text(hint)]));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-input-wrapper".to_string())],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_default() {
        let props = InputProps::default();
        assert_eq!(props.input_type, InputType::Text);
        assert!(!props.disabled);
    }

    #[test]
    fn test_input_builder() {
        let props = InputProps::default()
            .placeholder("Enter name")
            .label("Name")
            .error("Required")
            .input_type(InputType::Email);
        assert_eq!(props.placeholder, "Enter name");
        assert_eq!(props.label.as_deref(), Some("Name"));
        assert_eq!(props.error.as_deref(), Some("Required"));
        assert_eq!(props.input_type, InputType::Email);
    }

    #[test]
    fn test_input_render() {
        let el = Input::render(InputProps::default().label("Email").placeholder("you@example.com"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_input_type_as_str() {
        assert_eq!(InputType::Password.as_str(), "password");
        assert_eq!(InputType::Number.as_str(), "number");
    }

    #[test]
    fn test_input_reactive_value() {
        use rye_signals::Signal;
        let value = Signal::new("hello".to_string());
        let props = InputProps::default()
            .value_reactive(value.clone());
        assert!(props.value.is_reactive());
        assert_eq!(props.value.get(), "hello");

        value.set("world".to_string());
        assert_eq!(props.value.get(), "world");
    }

    #[test]
    fn test_input_on_input() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fired = Rc::new(Cell::new(false));
        let fired_clone = fired.clone();
        let props = InputProps::default()
            .placeholder("Type here")
            .on_input(move |_| fired_clone.set(true));
        assert!(props.on_input.is_some());

        // Simulate input event
        if let Some(handler) = &props.on_input {
            handler.borrow_mut()(&0usize);
        }
        assert!(fired.get());
    }
}
