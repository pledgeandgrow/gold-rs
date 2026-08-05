//! Button component — variants, sizes, loading, disabled, icon support.

use rye_core::Element;
use rye_core::template::{Template, TemplateNode, ReactiveValue, SharedEventHandler, shared_event_handler};
use crate::theme::{Size, Variant};

/// Props for the Button component.
pub struct ButtonProps {
    /// Button label text (static or reactive).
    pub label: ReactiveValue<String>,
    /// Visual variant (primary, secondary, ghost, destructive, outline).
    pub variant: Variant,
    /// Button size.
    pub size: Size,
    /// Whether the button is disabled.
    pub disabled: bool,
    /// Whether to show a loading spinner.
    pub loading: bool,
    /// Optional icon (emoji or text symbol).
    pub icon: Option<String>,
    /// Additional CSS class.
    pub class: Option<String>,
    /// Additional inline styles.
    pub style: Option<String>,
    /// Button type attribute.
    pub button_type: ButtonType,
    /// Optional click event handler.
    pub on_click: Option<SharedEventHandler>,
}

/// HTML button type attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    Button,
    Submit,
    Reset,
}

impl Default for ButtonType {
    fn default() -> Self {
        Self::Button
    }
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            label: ReactiveValue::Static(String::new()),
            variant: Variant::Primary,
            size: Size::Medium,
            disabled: false,
            loading: false,
            icon: None,
            class: None,
            style: None,
            button_type: ButtonType::Button,
            on_click: None,
        }
    }
}

impl ButtonProps {
    /// Set the label (static).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = ReactiveValue::Static(label.into());
        self
    }

    /// Set the label as a reactive signal.
    pub fn label_reactive(mut self, signal: rye_signals::Signal<String>) -> Self {
        self.label = ReactiveValue::Reactive(signal);
        self
    }

    /// Set the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set loading state.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set an icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set a click event handler.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&dyn std::any::Any) + 'static,
    {
        self.on_click = Some(shared_event_handler(handler));
        self
    }
}

/// Button component.
pub struct Button;

impl Button {
    /// Render a button to an Element.
    pub fn render(props: ButtonProps) -> Element {
        let mut style = format!(
            "display:inline-flex;align-items:center;gap:6px;padding:{};font-size:{};\
             border:1px solid {};border-radius:var(--rye-radius-md);background:{};color:{};\
             cursor:{};opacity:{};font-family:var(--rye-font-family);transition:var(--rye-transition-fast);",
            props.size.padding(),
            props.size.font_size(),
            props.variant.border(),
            props.variant.background(),
            props.variant.color(),
            if props.disabled || props.loading { "not-allowed" } else { "pointer" },
            if props.disabled { "0.5" } else { "1.0" },
        );

        if let Some(extra) = &props.style {
            style.push_str(extra);
        }

        let class = format!("rye-btn rye-btn-{} {}", props.variant.as_str(), props.class.as_deref().unwrap_or(""));

        let type_str = match props.button_type {
            ButtonType::Button => "button",
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
        };

        let mut children = Vec::new();

        if props.loading {
            children.push(Template::text("⏳"));
        } else if let Some(icon) = &props.icon {
            children.push(Template::text(icon));
        }

        if props.label.is_reactive() {
            children.push(Template::new(vec![TemplateNode::Reactive(props.label.to_reactive_fn())]));
        } else {
            let label = props.label.get();
            if !label.is_empty() {
                children.push(Template::text(&label));
            }
        }

        let mut attrs = vec![
            ("class".to_string(), class),
            ("style".to_string(), style),
            ("type".to_string(), type_str.to_string()),
        ];

        if props.disabled || props.loading {
            attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut events = Vec::new();
        if let Some(handler) = props.on_click {
            events.push(("click".to_string(), handler));
        }

        let template = Template::new_element("button", attrs, events, children);
        Element::Template(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_default() {
        let props = ButtonProps::default();
        assert_eq!(props.variant, Variant::Primary);
        assert_eq!(props.size, Size::Medium);
        assert!(!props.disabled);
        assert!(!props.loading);
    }

    #[test]
    fn test_button_builder() {
        let props = ButtonProps::default()
            .label("Click me")
            .variant(Variant::Destructive)
            .size(Size::Large)
            .disabled(true);
        assert_eq!(props.label.get(), "Click me");
        assert_eq!(props.variant, Variant::Destructive);
        assert_eq!(props.size, Size::Large);
        assert!(props.disabled);
    }

    #[test]
    fn test_button_render() {
        let el = Button::render(ButtonProps::default().label("Submit"));
        match el {
            Element::Template(t) => {
                assert_eq!(t.nodes.len(), 1);
            }
            _ => panic!("Expected Template"),
        }
    }

    #[test]
    fn test_button_loading() {
        let props = ButtonProps::default().label("Save").loading(true);
        assert!(props.loading);
        let _el = Button::render(props);
    }

    #[test]
    fn test_button_icon() {
        let props = ButtonProps::default().label("Delete").icon("🗑");
        assert_eq!(props.icon.as_deref(), Some("🗑"));
    }

    #[test]
    fn test_button_type_submit() {
        let props = ButtonProps {
            button_type: ButtonType::Submit,
            ..Default::default()
        };
        assert_eq!(props.button_type, ButtonType::Submit);
    }

    #[test]
    fn test_button_reactive_label() {
        use rye_signals::Signal;
        let label = Signal::new("Submit".to_string());
        let props = ButtonProps::default()
            .label_reactive(label.clone());
        assert!(props.label.is_reactive());
        assert_eq!(props.label.get(), "Submit");

        // Signal change should be reflected
        label.set("Saved".to_string());
        assert_eq!(props.label.get(), "Saved");
    }

    #[test]
    fn test_button_on_click() {
        use std::cell::Cell;
        use std::rc::Rc;
        let clicked = Rc::new(Cell::new(false));
        let clicked_clone = clicked.clone();
        let props = ButtonProps::default()
            .label("Click me")
            .on_click(move |_| clicked_clone.set(true));
        assert!(props.on_click.is_some());

        // Simulate click
        if let Some(handler) = &props.on_click {
            handler.borrow_mut()(&0usize);
        }
        assert!(clicked.get());
    }
}
