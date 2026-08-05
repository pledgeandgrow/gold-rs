//! EmptyState — placeholder for empty data.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct EmptyStateProps {
    pub icon: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub action_label: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for EmptyStateProps {
    fn default() -> Self {
        Self { icon: None, title: String::new(), description: None,
               action_label: None, class: None, style: None }
    }
}

impl EmptyStateProps {
    pub fn icon(mut self, i: impl Into<String>) -> Self { self.icon = Some(i.into()); self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn description(mut self, d: impl Into<String>) -> Self { self.description = Some(d.into()); self }
    pub fn action(mut self, label: impl Into<String>) -> Self { self.action_label = Some(label.into()); self }
}

pub struct EmptyState;

impl EmptyState {
    pub fn render(props: EmptyStateProps) -> Element {
        let style = format!(
            "display:flex;flex-direction:column;align-items:center;justify-content:center;\
             padding:48px 24px;text-align:center;{}",
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        if let Some(icon) = &props.icon {
            children.push(Template::new_element("div",
                vec![("style".to_string(), "font-size:48px;margin-bottom:16px;opacity:0.5;".to_string())],
                Vec::new(), vec![Template::text(icon)]));
        }

        children.push(Template::new_element("h3",
            vec![("style".to_string(), format!("font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);color:{};margin:0 0 8px 0;", vars::TEXT))],
            Vec::new(), vec![Template::text(&props.title)]));

        if let Some(desc) = &props.description {
            children.push(Template::new_element("p",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);color:{};margin:0 0 24px 0;max-width:400px;", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(desc)]));
        }

        if let Some(label) = &props.action_label {
            children.push(Template::new_element("button",
                vec![("style".to_string(), format!("padding:10px 20px;border:none;border-radius:var(--rye-radius-md);background:{};color:{};font-size:var(--rye-font-size-md);cursor:pointer;font-family:var(--rye-font-family);", vars::PRIMARY, vars::PRIMARY_FG)),
                     ("class".to_string(), "rye-empty-state-action".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-empty-state {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state_default() {
        let p = EmptyStateProps::default();
        assert!(p.icon.is_none());
        assert!(p.action_label.is_none());
    }

    #[test]
    fn test_empty_state_builder() {
        let p = EmptyStateProps::default()
            .icon("📭")
            .title("No data found")
            .description("Upload a file to get started")
            .action("Upload now");
        assert_eq!(p.icon.as_deref(), Some("📭"));
        assert_eq!(p.action_label.as_deref(), Some("Upload now"));
    }

    #[test]
    fn test_empty_state_render() {
        let el = EmptyState::render(EmptyStateProps::default()
            .icon("📋").title("No tasks").description("Create your first task").action("New Task"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_empty_state_render_minimal() {
        let el = EmptyState::render(EmptyStateProps::default().title("Nothing here"));
        assert!(matches!(el, Element::Template(_)));
    }
}
