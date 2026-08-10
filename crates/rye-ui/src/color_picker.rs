//! ColorPicker — color swatch picker.

use crate::theme::vars;
use rye_core::template::{ReactiveValue, Template, TemplateNode};
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct ColorPickerProps {
    pub value: ReactiveValue<String>,
    pub swatches: Vec<String>,
    pub label: Option<String>,
    pub show_input: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ColorPickerProps {
    fn default() -> Self {
        Self {
            value: ReactiveValue::Static("#2563eb".to_string()),
            swatches: vec![
                "#2563eb".into(),
                "#16a34a".into(),
                "#dc2626".into(),
                "#d97706".into(),
                "#0891b2".into(),
                "#7c3aed".into(),
                "#db2777".into(),
                "#000000".into(),
                "#ffffff".into(),
                "#64748b".into(),
                "#94a3b8".into(),
                "#e2e8f0".into(),
            ],
            label: None,
            show_input: true,
            class: None,
            style: None,
        }
    }
}

impl ColorPickerProps {
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = ReactiveValue::Static(v.into());
        self
    }

    /// Set the value as a reactive signal.
    pub fn value_reactive(mut self, signal: rye_signals::Signal<String>) -> Self {
        self.value = ReactiveValue::Reactive(signal);
        self
    }
    pub fn swatches(mut self, s: Vec<String>) -> Self {
        self.swatches = s;
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn show_input(mut self, s: bool) -> Self {
        self.show_input = s;
        self
    }
}

pub struct ColorPicker;

impl ColorPicker {
    pub fn render(props: ColorPickerProps) -> Element {
        let current_value = props.value.get();

        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:8px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        // Swatches grid
        let swatch_cells: Vec<Template> = props.swatches.iter().map(|color| {
            let is_selected = *color == current_value;
            let border = if is_selected { format!("2px solid {}", vars::TEXT) } else { format!("1px solid {}", vars::BORDER) };
            let style = format!(
                "width:28px;height:28px;border-radius:var(--rye-radius-md);background:{};border:{};cursor:pointer;",
                color, border,
            );
            Template::new_element("button",
                vec![("style".to_string(), style),
                     ("class".to_string(), "rye-color-picker-swatch".to_string()),
                     ("data-color".to_string(), color.clone())],
                Vec::new(), Vec::new())
        }).collect();

        children.push(Template::new_element(
            "div",
            vec![
                (
                    "style".to_string(),
                    "display:grid;grid-template-columns:repeat(6,28px);gap:8px;margin-bottom:12px;"
                        .to_string(),
                ),
                ("class".to_string(), "rye-color-picker-swatches".to_string()),
            ],
            Vec::new(),
            swatch_cells,
        ));

        // Native color input
        if props.show_input {
            let input_style = format!(
                "width:60px;height:36px;border:1px solid {};border-radius:var(--rye-radius-md);cursor:pointer;padding:2px;{}",
                vars::INPUT_BORDER, props.style.as_deref().unwrap_or(""),
            );

            let (input_el, span_children) = if props.value.is_reactive() {
                // Reactive: use reactive_attrs for value, Reactive node for display
                let input = Template::new_element_reactive(
                    "input",
                    vec![
                        ("type".to_string(), "color".to_string()),
                        ("style".to_string(), input_style),
                        ("class".to_string(), "rye-color-picker-input".to_string()),
                    ],
                    vec![("value".to_string(), props.value.to_reactive_fn())],
                    Vec::new(),
                    Vec::new(),
                );
                let span =
                    Template::new(vec![TemplateNode::Reactive(props.value.to_reactive_fn())]);
                (input, vec![span])
            } else {
                let input = Template::new_element(
                    "input",
                    vec![
                        ("type".to_string(), "color".to_string()),
                        ("value".to_string(), current_value.clone()),
                        ("style".to_string(), input_style),
                        ("class".to_string(), "rye-color-picker-input".to_string()),
                    ],
                    Vec::new(),
                    Vec::new(),
                );
                let span = Template::text(&current_value);
                (input, vec![span])
            };

            children.push(Template::new_element(
                "div",
                vec![(
                    "style".to_string(),
                    "display:flex;align-items:center;gap:8px;".to_string(),
                )],
                Vec::new(),
                vec![input_el, span_children.into_iter().next().unwrap()],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![(
                "class".to_string(),
                format!("rye-color-picker {}", props.class.as_deref().unwrap_or("")),
            )],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_picker_default() {
        let p = ColorPickerProps::default();
        assert_eq!(p.value.get(), "#2563eb");
        assert_eq!(p.swatches.len(), 12);
        assert!(p.show_input);
    }

    #[test]
    fn test_color_picker_builder() {
        let p = ColorPickerProps::default()
            .value("#dc2626")
            .swatches(vec!["#ff0000".into(), "#00ff00".into()])
            .label("Brand color")
            .show_input(false);
        assert_eq!(p.value.get(), "#dc2626");
        assert_eq!(p.swatches.len(), 2);
        assert!(!p.show_input);
    }

    #[test]
    fn test_color_picker_render() {
        let el = ColorPicker::render(ColorPickerProps::default().label("Pick a color"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_color_picker_reactive_value() {
        use rye_signals::Signal;
        let value = Signal::new("#2563eb".to_string());
        let props = ColorPickerProps::default().value_reactive(value.clone());
        assert!(props.value.is_reactive());
        assert_eq!(props.value.get(), "#2563eb");

        value.set("#dc2626".to_string());
        assert_eq!(props.value.get(), "#dc2626");
    }
}
