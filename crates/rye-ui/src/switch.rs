//! Switch (toggle) component.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct SwitchProps {
    pub label: Option<String>,
    pub checked: bool,
    pub disabled: bool,
    pub size: SwitchSize,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchSize {
    Small,
    Medium,
    Large,
}

impl SwitchSize {
    fn dimensions(&self) -> (u32, u32, u32) {
        match self {
            Self::Small => (28, 16, 12),
            Self::Medium => (40, 22, 18),
            Self::Large => (52, 28, 24),
        }
    }
}

impl Default for SwitchProps {
    fn default() -> Self {
        Self { label: None, checked: false, disabled: false, size: SwitchSize::Medium, class: None, style: None }
    }
}

impl SwitchProps {
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn checked(mut self, c: bool) -> Self { self.checked = c; self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn size(mut self, s: SwitchSize) -> Self { self.size = s; self }
}

pub struct Switch;

impl Switch {
    pub fn render(props: SwitchProps) -> Element {
        let (w, h, knob) = props.size.dimensions();
        let bg = if props.checked { vars::PRIMARY } else { vars::INPUT_BORDER };
        let knob_offset = if props.checked { w - knob - 2 } else { 2 };

        let track_style = format!(
            "width:{}px;height:{}px;border-radius:{}px;background:{};opacity:{};\
             cursor:{};position:relative;transition:var(--rye-transition-normal);display:inline-block;",
            w, h, h / 2, bg,
            if props.disabled { "0.5" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "pointer" },
        );

        let knob_style = format!(
            "width:{}px;height:{}px;border-radius:50%;background:{};\
             position:absolute;top:{}px;left:{}px;transition:left 0.2s;box-shadow:{};",
            knob, knob, vars::BG, (h - knob) / 2, knob_offset, vars::SHADOW_SM,
        );

        let track_children = vec![Template::new_element("span",
            vec![("style".to_string(), knob_style)], Vec::new(), Vec::new())];

        let mut input_attrs = vec![
            ("type".to_string(), "checkbox".to_string()),
            ("style".to_string(), "position:absolute;opacity:0;width:0;height:0;".to_string()),
            ("class".to_string(), format!("rye-switch-input {}", props.class.as_deref().unwrap_or(""))),
        ];
        if props.checked {
            input_attrs.push(("checked".to_string(), "true".to_string()));
        }
        if props.disabled {
            input_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut label_children = vec![
            Template::new_element("span",
                vec![("style".to_string(), track_style), ("class".to_string(), "rye-switch-track".to_string())],
                Vec::new(), track_children),
            Template::new_element("input", input_attrs, Vec::new(), Vec::new()),
        ];

        if let Some(label) = &props.label {
            label_children.push(Template::text(label));
        }

        let label_style = format!(
            "display:inline-flex;align-items:center;gap:8px;cursor:{};font-size:14px;",
            if props.disabled { "not-allowed" } else { "pointer" },
        );

        let final_style = if let Some(extra) = &props.style { format!("{}{}", label_style, extra) } else { label_style };
        Element::Template(Template::new_element("label",
            vec![("style".to_string(), final_style), ("class".to_string(), "rye-switch".to_string())],
            Vec::new(), label_children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_default() {
        let p = SwitchProps::default();
        assert!(!p.checked);
        assert_eq!(p.size, SwitchSize::Medium);
    }

    #[test]
    fn test_switch_builder() {
        let p = SwitchProps::default().label("Dark mode").checked(true).size(SwitchSize::Large);
        assert_eq!(p.label.as_deref(), Some("Dark mode"));
        assert!(p.checked);
        assert_eq!(p.size, SwitchSize::Large);
    }

    #[test]
    fn test_switch_render() {
        let el = Switch::render(SwitchProps::default().checked(true));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_switch_size_dimensions() {
        let (w, h, _) = SwitchSize::Small.dimensions();
        assert_eq!(w, 28);
        assert_eq!(h, 16);
        let (w, h, _) = SwitchSize::Large.dimensions();
        assert_eq!(w, 52);
        assert_eq!(h, 28);
    }
}
