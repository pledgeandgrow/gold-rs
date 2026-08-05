//! Rating — star rating input.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct RatingProps {
    pub value: u32,
    pub max: u32,
    pub readonly: bool,
    pub size: String,
    pub color: String,
    pub allow_half: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for RatingProps {
    fn default() -> Self {
        Self { value: 0, max: 5, readonly: false, size: "24px".to_string(),
               color: vars::WARNING.to_string(), allow_half: false, class: None, style: None }
    }
}

impl RatingProps {
    pub fn value(mut self, v: u32) -> Self { self.value = v; self }
    pub fn max(mut self, m: u32) -> Self { self.max = m; self }
    pub fn readonly(mut self, r: bool) -> Self { self.readonly = r; self }
    pub fn size(mut self, s: impl Into<String>) -> Self { self.size = s.into(); self }
    pub fn color(mut self, c: impl Into<String>) -> Self { self.color = c.into(); self }
    pub fn allow_half(mut self, a: bool) -> Self { self.allow_half = a; self }
}

pub struct Rating;

impl Rating {
    pub fn render(props: RatingProps) -> Element {
        let cursor = if props.readonly { "default" } else { "pointer" };
        let style = format!(
            "display:inline-flex;align-items:center;gap:2px;cursor:{};{}",
            cursor, props.style.as_deref().unwrap_or(""),
        );

        let stars: Vec<Template> = (0..props.max).map(|i| {
            let filled = i < props.value;
            let star_color = if filled { props.color.clone() } else { vars::BORDER.to_string() };
            let star_style = format!(
                "font-size:{};color:{};line-height:1;",
                props.size, star_color,
            );
            Template::new_element("span",
                vec![("style".to_string(), star_style),
                     ("class".to_string(), "rye-rating-star".to_string()),
                     ("data-value".to_string(), (i + 1).to_string())],
                Vec::new(), vec![Template::text("★")])
        }).collect();

        let mut children = stars;

        // Show value text
        if props.value > 0 {
            children.push(Template::new_element("span",
                vec![("style".to_string(), format!("margin-left:8px;font-size:var(--rye-font-size-md);color:{};", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(&format!("{}/{}", props.value, props.max))]));
        }

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-rating {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rating_default() {
        let p = RatingProps::default();
        assert_eq!(p.value, 0);
        assert_eq!(p.max, 5);
        assert!(!p.readonly);
    }

    #[test]
    fn test_rating_builder() {
        let p = RatingProps::default().value(4).max(10).readonly(true).size("32px").color("#dc2626");
        assert_eq!(p.value, 4);
        assert_eq!(p.max, 10);
        assert!(p.readonly);
    }

    #[test]
    fn test_rating_render_empty() {
        let el = Rating::render(RatingProps::default());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_rating_render_filled() {
        let el = Rating::render(RatingProps::default().value(3).max(5));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_rating_render_readonly() {
        let el = Rating::render(RatingProps::default().value(5).readonly(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
