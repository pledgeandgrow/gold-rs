//! Radio and RadioGroup components.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct RadioProps {
    pub label: Option<String>,
    pub value: String,
    pub checked: bool,
    pub disabled: bool,
    pub name: String,
    pub class: Option<String>,
}

impl Default for RadioProps {
    fn default() -> Self {
        Self { label: None, value: String::new(), checked: false, disabled: false, name: String::new(), class: None }
    }
}

impl RadioProps {
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn checked(mut self, c: bool) -> Self { self.checked = c; self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn name(mut self, n: impl Into<String>) -> Self { self.name = n.into(); self }
}

pub struct Radio;

impl Radio {
    pub fn render(props: RadioProps) -> Element {
        let label_style = format!(
            "display:inline-flex;align-items:center;gap:6px;cursor:{};opacity:{};font-size:14px;",
            if props.disabled { "not-allowed" } else { "pointer" },
            if props.disabled { "0.6" } else { "1.0" },
        );

        let mut attrs = vec![
            ("type".to_string(), "radio".to_string()),
            ("value".to_string(), props.value.clone()),
            ("style".to_string(), format!("width:16px;height:16px;accent-color:{};", vars::PRIMARY)),
            ("class".to_string(), format!("rye-radio {}", props.class.as_deref().unwrap_or(""))),
        ];
        if !props.name.is_empty() {
            attrs.push(("name".to_string(), props.name.clone()));
        }
        if props.checked {
            attrs.push(("checked".to_string(), "true".to_string()));
        }
        if props.disabled {
            attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut children = vec![Template::new_element("input", attrs, Vec::new(), Vec::new())];
        if let Some(label) = &props.label {
            children.push(Template::text(label));
        }

        Element::Template(Template::new_element("label",
            vec![("style".to_string(), label_style)], Vec::new(), children))
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroupProps {
    pub name: String,
    pub options: Vec<(String, String)>, // (value, label)
    pub selected: Option<String>,
    pub disabled: bool,
    pub label: Option<String>,
    pub class: Option<String>,
}

impl Default for RadioGroupProps {
    fn default() -> Self {
        Self { name: String::new(), options: Vec::new(), selected: None, disabled: false, label: None, class: None }
    }
}

impl RadioGroupProps {
    pub fn name(mut self, n: impl Into<String>) -> Self { self.name = n.into(); self }
    pub fn options(mut self, opts: Vec<(String, String)>) -> Self { self.options = opts; self }
    pub fn selected(mut self, s: impl Into<String>) -> Self { self.selected = Some(s.into()); self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
}

pub struct RadioGroup;

impl RadioGroup {
    pub fn render(props: RadioGroupProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("div",
                vec![("style".to_string(), "font-size:14px;font-weight:500;margin-bottom:8px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let group_style = "display:flex;flex-direction:column;gap:8px;";
        let mut radios = Vec::new();
        for (value, label) in &props.options {
            let radio_props = RadioProps {
                label: Some(label.clone()),
                value: value.clone(),
                checked: props.selected.as_deref() == Some(value.as_str()),
                disabled: props.disabled,
                name: props.name.clone(),
                class: None,
            };
            let radio_el = Radio::render(radio_props);
            if let Element::Template(t) = radio_el {
                radios.push(t);
            }
        }
        children.push(Template::new_element("div",
            vec![("style".to_string(), group_style.to_string()),
                 ("class".to_string(), format!("rye-radio-group {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), radios));

        Element::Template(Template::new_element("div", Vec::new(), Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_default() {
        let p = RadioProps::default();
        assert!(!p.checked);
    }

    #[test]
    fn test_radio_builder() {
        let p = RadioProps::default().label("Option A").value("a").checked(true).name("group1");
        assert_eq!(p.label.as_deref(), Some("Option A"));
        assert_eq!(p.value, "a");
        assert!(p.checked);
    }

    #[test]
    fn test_radio_render() {
        let el = Radio::render(RadioProps::default().label("Yes").value("yes").checked(true));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_radio_group_render() {
        let el = RadioGroup::render(RadioGroupProps::default()
            .name("color")
            .options(vec![("red".into(), "Red".into()), ("blue".into(), "Blue".into())])
            .selected("blue")
            .label("Pick a color"));
        assert!(matches!(el, Element::Template(_)));
    }
}
