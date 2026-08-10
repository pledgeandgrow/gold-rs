//! HoverCard — rich hover preview (like GitHub user cards).

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct HoverCardProps {
    pub trigger: String,
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub width: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for HoverCardProps {
    fn default() -> Self {
        Self {
            trigger: String::new(),
            title: String::new(),
            description: String::new(),
            image: None,
            width: "280px".to_string(),
            class: None,
            style: None,
        }
    }
}

impl HoverCardProps {
    pub fn trigger(mut self, t: impl Into<String>) -> Self {
        self.trigger = t.into();
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
    pub fn image(mut self, i: impl Into<String>) -> Self {
        self.image = Some(i.into());
        self
    }
    pub fn width(mut self, w: impl Into<String>) -> Self {
        self.width = w.into();
        self
    }
}

pub struct HoverCard;

impl HoverCard {
    pub fn render(props: HoverCardProps) -> Element {
        let container_style = "position:relative;display:inline-block;";

        let trigger_style = format!(
            "color:{};cursor:pointer;text-decoration:underline;font-size:var(--rye-font-size-md);",
            vars::PRIMARY
        );

        let card_style = format!(
            "position:absolute;bottom:100%;left:50%;transform:translateX(-50%);\
             margin-bottom:8px;width:{};background:{};border:1px solid {};\
             border-radius:var(--rye-radius-lg);box-shadow:{};padding:16px;\
             z-index:{};opacity:0;pointer-events:none;transition:var(--rye-transition-normal);{}",
            props.width,
            vars::BG_ELEVATED,
            vars::BORDER,
            vars::SHADOW_MD,
            vars::Z_DROPDOWN,
            props.style.as_deref().unwrap_or(""),
        );

        let mut card_children = Vec::new();

        if let Some(img) = &props.image {
            card_children.push(Template::new_element(
                "img",
                vec![
                    ("src".to_string(), img.clone()),
                    (
                        "style".to_string(),
                        "width:48px;height:48px;border-radius:50%;margin-bottom:8px;".to_string(),
                    ),
                ],
                Vec::new(),
                Vec::new(),
            ));
        }

        card_children.push(Template::new_element("div",
            vec![("style".to_string(), format!("font-size:var(--rye-font-size-lg);font-weight:var(--rye-font-weight-semibold);color:{};margin-bottom:4px;", vars::TEXT))],
            Vec::new(), vec![Template::text(&props.title)]));

        card_children.push(Template::new_element(
            "div",
            vec![(
                "style".to_string(),
                format!(
                    "font-size:var(--rye-font-size-sm);color:{};",
                    vars::TEXT_MUTED
                ),
            )],
            Vec::new(),
            vec![Template::text(&props.description)],
        ));

        let children = vec![
            Template::new_element(
                "span",
                vec![
                    ("style".to_string(), trigger_style.to_string()),
                    ("class".to_string(), "rye-hover-card-trigger".to_string()),
                ],
                Vec::new(),
                vec![Template::text(&props.trigger)],
            ),
            Template::new_element(
                "div",
                vec![
                    ("style".to_string(), card_style),
                    (
                        "class".to_string(),
                        format!("rye-hover-card {}", props.class.as_deref().unwrap_or("")),
                    ),
                ],
                Vec::new(),
                card_children,
            ),
        ];

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style.to_string()),
                ("class".to_string(), "rye-hover-card-wrapper".to_string()),
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
    fn test_hover_card_default() {
        let p = HoverCardProps::default();
        assert_eq!(p.width, "280px");
        assert!(p.image.is_none());
    }

    #[test]
    fn test_hover_card_builder() {
        let p = HoverCardProps::default()
            .trigger("@alice")
            .title("Alice Smith")
            .description("Software engineer at Acme")
            .image("/avatar.png");
        assert_eq!(p.trigger, "@alice");
        assert_eq!(p.title, "Alice Smith");
        assert!(p.image.is_some());
    }

    #[test]
    fn test_hover_card_render() {
        let el = HoverCard::render(
            HoverCardProps::default()
                .trigger("View profile")
                .title("John Doe")
                .description("Lead developer"),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
