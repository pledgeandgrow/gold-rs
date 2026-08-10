//! Tooltip — hover tooltip with positions.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl TooltipPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TooltipProps {
    pub content: String,
    pub position: TooltipPosition,
    pub delay_ms: u64,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TooltipProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            position: TooltipPosition::Top,
            delay_ms: 200,
            class: None,
            style: None,
        }
    }
}

impl TooltipProps {
    pub fn content(mut self, c: impl Into<String>) -> Self {
        self.content = c.into();
        self
    }
    pub fn position(mut self, p: TooltipPosition) -> Self {
        self.position = p;
        self
    }
    pub fn delay(mut self, d: u64) -> Self {
        self.delay_ms = d;
        self
    }
}

pub struct Tooltip;

impl Tooltip {
    pub fn render(props: TooltipProps) -> Element {
        let style = format!(
            "position:relative;display:inline-block;{}",
            props.style.as_deref().unwrap_or(""),
        );

        let tooltip_style = format!(
            "position:absolute;bottom:100%;left:50%;transform:translateX(-50%);margin-bottom:6px;\
             padding:4px 8px;background:{};color:{};font-size:var(--rye-font-size-sm);border-radius:var(--rye-radius-sm);\
             white-space:nowrap;pointer-events:none;z-index:{};opacity:0;transition:var(--rye-transition-normal);",
            vars::TEXT, vars::BG, vars::Z_TOOLTIP,
        );

        let children = vec![Template::new_element(
            "span",
            vec![
                ("class".to_string(), "rye-tooltip-content".to_string()),
                ("style".to_string(), tooltip_style),
                (
                    "data-position".to_string(),
                    props.position.as_str().to_string(),
                ),
            ],
            Vec::new(),
            vec![Template::text(&props.content)],
        )];

        Element::Template(Template::new_element(
            "span",
            vec![
                (
                    "class".to_string(),
                    format!("rye-tooltip {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
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
    fn test_tooltip_position_as_str() {
        assert_eq!(TooltipPosition::Top.as_str(), "top");
        assert_eq!(TooltipPosition::Bottom.as_str(), "bottom");
    }

    #[test]
    fn test_tooltip_default() {
        let p = TooltipProps::default();
        assert_eq!(p.position, TooltipPosition::Top);
        assert_eq!(p.delay_ms, 200);
    }

    #[test]
    fn test_tooltip_builder() {
        let p = TooltipProps::default()
            .content("Click me")
            .position(TooltipPosition::Right)
            .delay(500);
        assert_eq!(p.content, "Click me");
        assert_eq!(p.position, TooltipPosition::Right);
    }

    #[test]
    fn test_tooltip_render() {
        let el = Tooltip::render(TooltipProps::default().content("Help"));
        assert!(matches!(el, Element::Template(_)));
    }
}
