//! Progress — progress bar (determinate/indeterminate).

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct ProgressProps {
    pub value: Option<f64>, // 0.0-100.0, None = indeterminate
    pub color: String,
    pub track_color: String,
    pub height: String,
    pub label: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ProgressProps {
    fn default() -> Self {
        Self {
            value: None,
            color: vars::PRIMARY.to_string(),
            track_color: vars::BORDER.to_string(),
            height: "8px".to_string(),
            label: None,
            class: None,
            style: None,
        }
    }
}

impl ProgressProps {
    pub fn value(mut self, v: f64) -> Self {
        self.value = Some(v);
        self
    }
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }
    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = c.into();
        self
    }
    pub fn height(mut self, h: impl Into<String>) -> Self {
        self.height = h.into();
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
}

pub struct Progress;

impl Progress {
    pub fn render(props: ProgressProps) -> Element {
        let track_style = format!(
            "width:100%;height:{};background:{};border-radius:{};overflow:hidden;{}",
            props.height,
            props.track_color,
            props.height,
            props.style.as_deref().unwrap_or(""),
        );

        let bar_style = match props.value {
            Some(v) => {
                let clamped = v.clamp(0.0, 100.0);
                format!("width:{}%;height:100%;background:{};border-radius:inherit;transition:width 0.3s;",
                    clamped, props.color)
            }
            None => format!("width:40%;height:100%;background:{};border-radius:inherit;animation:rye-progress-indeterminate 1.5s ease-in-out infinite;",
                props.color),
        };

        let bar = Template::new_element(
            "div",
            vec![
                ("class".to_string(), "rye-progress-bar".to_string()),
                ("style".to_string(), bar_style),
            ],
            Vec::new(),
            Vec::new(),
        );

        let mut children = vec![Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!(
                        "rye-progress-track {}",
                        props.class.as_deref().unwrap_or("")
                    ),
                ),
                ("style".to_string(), track_style),
            ],
            Vec::new(),
            vec![bar],
        )];

        if let Some(label) = &props.label {
            children.push(Template::new_element(
                "div",
                vec![(
                    "style".to_string(),
                    format!(
                        "font-size:var(--rye-font-size-sm);color:{};margin-top:4px;",
                        vars::TEXT_MUTED
                    ),
                )],
                Vec::new(),
                vec![Template::text(label)],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-progress".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_default() {
        let p = ProgressProps::default();
        assert!(p.value.is_none()); // indeterminate by default
    }

    #[test]
    fn test_progress_builder() {
        let p = ProgressProps::default()
            .value(75.0)
            .color("#16a34a")
            .label("Uploading...");
        assert_eq!(p.value, Some(75.0));
        assert_eq!(p.label.as_deref(), Some("Uploading..."));
    }

    #[test]
    fn test_progress_render_determinate() {
        let el = Progress::render(ProgressProps::default().value(50.0));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_progress_render_indeterminate() {
        let el = Progress::render(ProgressProps::default());
        assert!(matches!(el, Element::Template(_)));
    }
}
