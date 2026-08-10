//! ErrorBoundary — catch render errors, show fallback.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct ErrorFallback {
    pub title: String,
    pub description: String,
    pub show_details: bool,
    pub details: Option<String>,
}

impl Default for ErrorFallback {
    fn default() -> Self {
        Self {
            title: "Something went wrong".to_string(),
            description: "An unexpected error occurred while rendering this component.".to_string(),
            show_details: false,
            details: None,
        }
    }
}

impl ErrorFallback {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
    pub fn details(mut self, d: impl Into<String>) -> Self {
        self.details = Some(d.into());
        self.show_details = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ErrorBoundaryProps {
    pub has_error: bool,
    pub fallback: ErrorFallback,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ErrorBoundaryProps {
    fn default() -> Self {
        Self {
            has_error: false,
            fallback: ErrorFallback::default(),
            class: None,
            style: None,
        }
    }
}

impl ErrorBoundaryProps {
    pub fn has_error(mut self, e: bool) -> Self {
        self.has_error = e;
        self
    }
    pub fn fallback(mut self, f: ErrorFallback) -> Self {
        self.fallback = f;
        self
    }
}

pub struct ErrorBoundary;

impl ErrorBoundary {
    pub fn render(props: ErrorBoundaryProps) -> Element {
        if !props.has_error {
            return Element::None;
        }

        let style = format!(
            "padding:24px;border:1px solid {};border-radius:var(--rye-radius-lg);\
             background:{};text-align:center;{}",
            vars::DANGER,
            vars::BG,
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![
            Template::new_element("div",
                vec![("style".to_string(), "font-size:40px;margin-bottom:12px;".to_string())],
                Vec::new(), vec![Template::text("⚠")]),
            Template::new_element("h3",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);color:{};margin:0 0 8px 0;", vars::DANGER))],
                Vec::new(), vec![Template::text(&props.fallback.title)]),
            Template::new_element("p",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);color:{};margin:0 0 16px 0;", vars::DANGER))],
                Vec::new(), vec![Template::text(&props.fallback.description)]),
        ];

        if props.fallback.show_details {
            if let Some(details) = &props.fallback.details {
                children.push(Template::new_element("pre",
                    vec![("style".to_string(), format!("text-align:left;background:{};border:1px solid {};border-radius:var(--rye-radius-md);padding:12px;font-size:var(--rye-font-size-sm);color:{};overflow-x:auto;margin:0 0 16px 0;font-family:var(--rye-font-family-mono);", vars::BG, vars::DANGER, vars::DANGER))],
                    Vec::new(), vec![Template::text(details)]));
            }
        }

        children.push(Template::new_element("button",
            vec![("style".to_string(), format!("padding:8px 16px;border:none;border-radius:var(--rye-radius-md);background:{};color:{};font-size:var(--rye-font-size-md);cursor:pointer;font-family:var(--rye-font-family);", vars::DANGER, vars::DANGER_FG)),
                 ("class".to_string(), "rye-error-boundary-retry".to_string())],
            Vec::new(), vec![Template::text("Try again")]));

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!(
                        "rye-error-boundary {}",
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
    fn test_error_fallback_default() {
        let f = ErrorFallback::default();
        assert_eq!(f.title, "Something went wrong");
        assert!(!f.show_details);
    }

    #[test]
    fn test_error_fallback_builder() {
        let f = ErrorFallback::default()
            .title("Render failed")
            .description("Component crashed")
            .details("panic at line 42");
        assert_eq!(f.title, "Render failed");
        assert!(f.show_details);
        assert_eq!(f.details.as_deref(), Some("panic at line 42"));
    }

    #[test]
    fn test_error_boundary_no_error() {
        let el = ErrorBoundary::render(ErrorBoundaryProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_error_boundary_with_error() {
        let el = ErrorBoundary::render(ErrorBoundaryProps::default().has_error(true));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_error_boundary_with_details() {
        let el = ErrorBoundary::render(
            ErrorBoundaryProps::default()
                .has_error(true)
                .fallback(ErrorFallback::default().details("TypeError: cannot read property 'x'")),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
