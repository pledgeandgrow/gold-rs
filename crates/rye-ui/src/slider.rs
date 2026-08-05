//! Slider — range input with min/max/step.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct SliderProps {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
    pub label: Option<String>,
    pub disabled: bool,
    pub show_value: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self { min: 0.0, max: 100.0, step: 1.0, value: 50.0,
               label: None, disabled: false, show_value: false, class: None, style: None }
    }
}

impl SliderProps {
    pub fn min(mut self, m: f64) -> Self { self.min = m; self }
    pub fn max(mut self, m: f64) -> Self { self.max = m; self }
    pub fn step(mut self, s: f64) -> Self { self.step = s; self }
    pub fn value(mut self, v: f64) -> Self { self.value = v; self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn show_value(mut self, s: bool) -> Self { self.show_value = s; self }
}

pub struct Slider;

impl Slider {
    pub fn render(props: SliderProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            let label_style = if props.show_value {
                format!("display:flex;justify-content:space-between;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;")
            } else {
                "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string()
            };

            let mut label_children = vec![Template::text(label)];
            if props.show_value {
                label_children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("color:{};font-weight:var(--rye-font-weight-normal);", vars::TEXT_MUTED)), ("class".to_string(), "rye-slider-value".to_string())],
                    Vec::new(), vec![Template::text(&format!("{}", props.value))]));
            }

            children.push(Template::new_element("div",
                vec![("style".to_string(), label_style), ("class".to_string(), "rye-slider-label".to_string())],
                Vec::new(), label_children));
        }

        let slider_style = format!(
            "width:100%;accent-color:{};cursor:{};opacity:{};{}",
            vars::PRIMARY,
            if props.disabled { "not-allowed" } else { "pointer" },
            if props.disabled { "0.6" } else { "1.0" },
            props.style.as_deref().unwrap_or(""),
        );

        let mut attrs = vec![
            ("type".to_string(), "range".to_string()),
            ("style".to_string(), slider_style),
            ("class".to_string(), format!("rye-slider {}", props.class.as_deref().unwrap_or(""))),
            ("min".to_string(), props.min.to_string()),
            ("max".to_string(), props.max.to_string()),
            ("step".to_string(), props.step.to_string()),
            ("value".to_string(), props.value.to_string()),
        ];
        if props.disabled {
            attrs.push(("disabled".to_string(), "true".to_string()));
        }

        children.push(Template::new_element("input", attrs, Vec::new(), Vec::new()));

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-slider-wrapper".to_string())],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_default() {
        let p = SliderProps::default();
        assert_eq!(p.min, 0.0);
        assert_eq!(p.max, 100.0);
        assert_eq!(p.value, 50.0);
    }

    #[test]
    fn test_slider_builder() {
        let p = SliderProps::default().min(0.0).max(10.0).step(0.5).value(3.5).label("Volume").show_value(true);
        assert_eq!(p.step, 0.5);
        assert_eq!(p.value, 3.5);
        assert!(p.show_value);
    }

    #[test]
    fn test_slider_render() {
        let el = Slider::render(SliderProps::default().label("Brightness").show_value(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
