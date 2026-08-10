//! Skeleton — loading placeholder with shimmer.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonShape {
    Text,
    Circle,
    Rect,
}

#[derive(Debug, Clone)]
pub struct SkeletonProps {
    pub shape: SkeletonShape,
    pub width: String,
    pub height: String,
    pub count: usize,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for SkeletonProps {
    fn default() -> Self {
        Self {
            shape: SkeletonShape::Text,
            width: "100%".to_string(),
            height: "14px".to_string(),
            count: 1,
            class: None,
            style: None,
        }
    }
}

impl SkeletonProps {
    pub fn shape(mut self, s: SkeletonShape) -> Self {
        self.shape = s;
        self
    }
    pub fn width(mut self, w: impl Into<String>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<String>) -> Self {
        self.height = h.into();
        self
    }
    pub fn count(mut self, c: usize) -> Self {
        self.count = c;
        self
    }
}

pub struct Skeleton;

impl Skeleton {
    pub fn render(props: SkeletonProps) -> Element {
        let radius = match props.shape {
            SkeletonShape::Text => "4px",
            SkeletonShape::Circle => "50%",
            SkeletonShape::Rect => "0px",
        };

        let item_style = format!(
            "width:{};height:{};border-radius:{};background:{};\
             animation:rye-skeleton-shimmer 1.5s ease-in-out infinite;{}",
            props.width,
            props.height,
            radius,
            vars::BG_MUTED,
            props.style.as_deref().unwrap_or(""),
        );

        let items: Vec<Template> = (0..props.count)
            .map(|_| {
                Template::new_element(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            format!("rye-skeleton {}", props.class.as_deref().unwrap_or("")),
                        ),
                        ("style".to_string(), item_style.clone()),
                    ],
                    Vec::new(),
                    Vec::new(),
                )
            })
            .collect();

        if items.len() == 1 {
            Element::Template(items.into_iter().next().unwrap())
        } else {
            Element::Template(Template::new_element(
                "div",
                vec![(
                    "style".to_string(),
                    "display:flex;flex-direction:column;gap:8px;".to_string(),
                )],
                Vec::new(),
                items,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_default() {
        let p = SkeletonProps::default();
        assert_eq!(p.shape, SkeletonShape::Text);
        assert_eq!(p.count, 1);
    }

    #[test]
    fn test_skeleton_builder() {
        let p = SkeletonProps::default()
            .shape(SkeletonShape::Circle)
            .width("48px")
            .height("48px")
            .count(3);
        assert_eq!(p.shape, SkeletonShape::Circle);
        assert_eq!(p.count, 3);
    }

    #[test]
    fn test_skeleton_render_single() {
        let el = Skeleton::render(SkeletonProps::default());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_skeleton_render_multiple() {
        let el = Skeleton::render(SkeletonProps::default().count(5));
        assert!(matches!(el, Element::Template(_)));
    }
}
