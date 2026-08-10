//! CircularProgress — circular progress ring.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct CircularProgressProps {
    pub value: Option<f64>, // 0.0-100.0, None = indeterminate
    pub size: u32,
    pub stroke_width: u32,
    pub color: String,
    pub track_color: String,
    pub label: Option<String>,
    pub show_percentage: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CircularProgressProps {
    fn default() -> Self {
        Self {
            value: None,
            size: 48,
            stroke_width: 4,
            color: vars::PRIMARY.to_string(),
            track_color: vars::BORDER.to_string(),
            label: None,
            show_percentage: false,
            class: None,
            style: None,
        }
    }
}

impl CircularProgressProps {
    pub fn value(mut self, v: f64) -> Self {
        self.value = Some(v);
        self
    }
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }
    pub fn size(mut self, s: u32) -> Self {
        self.size = s;
        self
    }
    pub fn stroke_width(mut self, w: u32) -> Self {
        self.stroke_width = w;
        self
    }
    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = c.into();
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn show_percentage(mut self, s: bool) -> Self {
        self.show_percentage = s;
        self
    }
}

pub struct CircularProgress;

impl CircularProgress {
    pub fn render(props: CircularProgressProps) -> Element {
        let radius = (props.size / 2).saturating_sub(props.stroke_width / 2);
        let circumference = 2.0 * std::f64::consts::PI * radius as f64;

        let (dash_offset, animation) = match props.value {
            Some(v) => {
                let clamped = v.clamp(0.0, 100.0);
                let offset = circumference * (1.0 - clamped / 100.0);
                (offset.to_string(), "none")
            }
            None => (
                circumference.to_string(),
                "rye-circular-spin 1s linear infinite",
            ),
        };

        let container_style = format!(
            "position:relative;width:{}px;height:{}px;display:inline-flex;align-items:center;justify-content:center;{}",
            props.size, props.size, props.style.as_deref().unwrap_or(""),
        );

        let svg_style = format!("animation:{};", animation);

        let mut children = vec![Template::new_element(
            "svg",
            vec![
                ("width".to_string(), props.size.to_string()),
                ("height".to_string(), props.size.to_string()),
                (
                    "viewBox".to_string(),
                    format!("0 0 {} {}", props.size, props.size),
                ),
                ("style".to_string(), svg_style),
                ("class".to_string(), "rye-circular-progress-svg".to_string()),
            ],
            Vec::new(),
            vec![
                Template::new_element(
                    "circle",
                    vec![
                        ("cx".to_string(), (props.size / 2).to_string()),
                        ("cy".to_string(), (props.size / 2).to_string()),
                        ("r".to_string(), radius.to_string()),
                        ("fill".to_string(), "none".to_string()),
                        ("stroke".to_string(), props.track_color.clone()),
                        ("stroke-width".to_string(), props.stroke_width.to_string()),
                    ],
                    Vec::new(),
                    Vec::new(),
                ),
                Template::new_element(
                    "circle",
                    vec![
                        ("cx".to_string(), (props.size / 2).to_string()),
                        ("cy".to_string(), (props.size / 2).to_string()),
                        ("r".to_string(), radius.to_string()),
                        ("fill".to_string(), "none".to_string()),
                        ("stroke".to_string(), props.color.clone()),
                        ("stroke-width".to_string(), props.stroke_width.to_string()),
                        ("stroke-dasharray".to_string(), circumference.to_string()),
                        ("stroke-dashoffset".to_string(), dash_offset),
                        ("stroke-linecap".to_string(), "round".to_string()),
                        (
                            "transform".to_string(),
                            format!("rotate(-90 {} {})", props.size / 2, props.size / 2),
                        ),
                    ],
                    Vec::new(),
                    Vec::new(),
                ),
            ],
        )];

        if props.show_percentage {
            if let Some(v) = props.value {
                children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("position:absolute;font-size:var(--rye-font-size-sm);font-weight:var(--rye-font-weight-semibold);color:{};", vars::TEXT))],
                    Vec::new(), vec![Template::text(&format!("{}%", v as u32))]));
            }
        } else if let Some(label) = &props.label {
            children.push(Template::new_element(
                "span",
                vec![(
                    "style".to_string(),
                    format!(
                        "position:absolute;font-size:var(--rye-font-size-xs);color:{};",
                        vars::TEXT_MUTED
                    ),
                )],
                Vec::new(),
                vec![Template::text(label)],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style),
                (
                    "class".to_string(),
                    format!(
                        "rye-circular-progress {}",
                        props.class.as_deref().unwrap_or("")
                    ),
                ),
            ],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_progress_default() {
        let p = CircularProgressProps::default();
        assert!(p.value.is_none());
        assert_eq!(p.size, 48);
    }

    #[test]
    fn test_circular_progress_builder() {
        let p = CircularProgressProps::default()
            .value(75.0)
            .size(64)
            .color("#16a34a")
            .show_percentage(true);
        assert_eq!(p.value, Some(75.0));
        assert!(p.show_percentage);
    }

    #[test]
    fn test_circular_progress_render_determinate() {
        let el = CircularProgress::render(
            CircularProgressProps::default()
                .value(60.0)
                .show_percentage(true),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_circular_progress_render_indeterminate() {
        let el = CircularProgress::render(CircularProgressProps::default().label("Loading"));
        assert!(matches!(el, Element::Template(_)));
    }
}
