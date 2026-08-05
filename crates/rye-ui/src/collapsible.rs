//! Collapsible — show/hide content with animation.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct CollapsibleProps {
    pub title: String,
    pub open: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CollapsibleProps {
    fn default() -> Self { Self { title: String::new(), open: false, class: None, style: None } }
}

impl CollapsibleProps {
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
}

pub struct Collapsible;

impl Collapsible {
    pub fn render(props: CollapsibleProps) -> Element {
        let style = format!(
            "border:1px solid {};border-radius:var(--rye-radius-md);overflow:hidden;{}",
            vars::BORDER, props.style.as_deref().unwrap_or(""),
        );

        let arrow = if props.open { "▼" } else { "▶" };

        let header = Template::new_element("div",
            vec![("style".to_string(), format!("display:flex;align-items:center;justify-content:space-between;padding:12px 16px;cursor:pointer;background:{};font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);", vars::BG_SUBTLE)),
                 ("class".to_string(), "rye-collapsible-header".to_string())],
            Vec::new(), vec![
                Template::text(&props.title),
                Template::new_element("span",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};transition:var(--rye-transition-normal);", vars::TEXT_MUTED))],
                    Vec::new(), vec![Template::text(arrow)]),
            ]);

        let mut children = vec![header];

        if props.open {
            children.push(Template::new_element("div",
                vec![("style".to_string(), format!("padding:16px;font-size:var(--rye-font-size-md);color:{};", vars::TEXT)),
                     ("class".to_string(), "rye-collapsible-content".to_string())],
                Vec::new(), Vec::new()));
        }

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-collapsible {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapsible_default() {
        let p = CollapsibleProps::default();
        assert!(!p.open);
    }

    #[test]
    fn test_collapsible_builder() {
        let p = CollapsibleProps::default().title("Details").open(true);
        assert_eq!(p.title, "Details");
        assert!(p.open);
    }

    #[test]
    fn test_collapsible_render_closed() {
        let el = Collapsible::render(CollapsibleProps::default().title("More info"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_collapsible_render_open() {
        let el = Collapsible::render(CollapsibleProps::default().title("More info").open(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
