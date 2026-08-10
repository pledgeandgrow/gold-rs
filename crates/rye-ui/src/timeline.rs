//! Timeline — vertical event timeline.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineVariant {
    Default,
    Success,
    Warning,
    Error,
    Info,
}

impl TimelineVariant {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Default => vars::TEXT_MUTED,
            Self::Success => vars::SUCCESS,
            Self::Warning => vars::WARNING,
            Self::Error => vars::DANGER,
            Self::Info => vars::INFO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineItem {
    pub title: String,
    pub description: Option<String>,
    pub timestamp: Option<String>,
    pub variant: TimelineVariant,
}

impl TimelineItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            timestamp: None,
            variant: TimelineVariant::Default,
        }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn timestamp(mut self, t: impl Into<String>) -> Self {
        self.timestamp = Some(t.into());
        self
    }
    pub fn variant(mut self, v: TimelineVariant) -> Self {
        self.variant = v;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TimelineProps {
    pub items: Vec<TimelineItem>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TimelineProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            class: None,
            style: None,
        }
    }
}

impl TimelineProps {
    pub fn items(mut self, i: Vec<TimelineItem>) -> Self {
        self.items = i;
        self
    }
}

pub struct Timeline;

impl Timeline {
    pub fn render(props: TimelineProps) -> Element {
        let items: Vec<Template> = props.items.iter().map(|item| {
            let dot_style = format!(
                "width:12px;height:12px;border-radius:50%;background:{};flex-shrink:0;margin-top:4px;",
                item.variant.color(),
            );

            let mut content_children = vec![
                Template::new_element("div",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-semibold);color:{};", vars::TEXT))],
                    Vec::new(), vec![Template::text(&item.title)]),
            ];

            if let Some(desc) = &item.description {
                content_children.push(Template::new_element("div",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};margin-top:2px;", vars::TEXT_MUTED))],
                    Vec::new(), vec![Template::text(desc)]));
            }

            if let Some(ts) = &item.timestamp {
                content_children.push(Template::new_element("div",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};margin-top:4px;", vars::TEXT_SUBTLE))],
                    Vec::new(), vec![Template::text(ts)]));
            }

            let row_style = "display:flex;gap:12px;padding-bottom:20px;position:relative;";

            let line_style = format!("position:absolute;left:5px;top:16px;bottom:0;width:2px;background:{};", vars::BORDER);

            Template::new_element("div",
                vec![("style".to_string(), row_style.to_string()),
                     ("class".to_string(), "rye-timeline-item".to_string())],
                Vec::new(), vec![
                    Template::new_element("div",
                        vec![("style".to_string(), dot_style), ("class".to_string(), "rye-timeline-dot".to_string())],
                        Vec::new(), Vec::new()),
                    Template::new_element("div",
                        vec![("style".to_string(), line_style.to_string()), ("class".to_string(), "rye-timeline-line".to_string())],
                        Vec::new(), Vec::new()),
                    Template::new_element("div",
                        vec![("style".to_string(), "flex:1;".to_string())],
                        Vec::new(), content_children),
                ])
        }).collect();

        let style = format!("padding:8px 0;{}", props.style.as_deref().unwrap_or(""));

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!("rye-timeline {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            items,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_variant_color() {
        assert_eq!(TimelineVariant::Success.color(), vars::SUCCESS);
        assert_eq!(TimelineVariant::Error.color(), vars::DANGER);
    }

    #[test]
    fn test_timeline_item_new() {
        let i = TimelineItem::new("Deployed");
        assert_eq!(i.title, "Deployed");
        assert_eq!(i.variant, TimelineVariant::Default);
    }

    #[test]
    fn test_timeline_item_builder() {
        let i = TimelineItem::new("Build failed")
            .description("Compilation error in main.rs")
            .timestamp("2 min ago")
            .variant(TimelineVariant::Error);
        assert_eq!(
            i.description.as_deref(),
            Some("Compilation error in main.rs")
        );
        assert_eq!(i.variant, TimelineVariant::Error);
    }

    #[test]
    fn test_timeline_render() {
        let el = Timeline::render(TimelineProps::default().items(vec![
                TimelineItem::new("Created").timestamp("1h ago").variant(TimelineVariant::Info),
                TimelineItem::new("Updated").timestamp("30m ago").variant(TimelineVariant::Success),
            ]));
        assert!(matches!(el, Element::Template(_)));
    }
}
