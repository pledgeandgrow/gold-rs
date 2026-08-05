//! Stack — vertical or horizontal stack with spacing.

use rye_core::Element;
use rye_core::template::Template;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection { Vertical, Horizontal }

#[derive(Debug, Clone)]
pub struct StackProps {
    pub direction: StackDirection,
    pub spacing: String,
    pub align: Option<String>,
    pub justify: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for StackProps {
    fn default() -> Self {
        Self { direction: StackDirection::Vertical, spacing: "8px".to_string(),
               align: None, justify: None, class: None, style: None }
    }
}

impl StackProps {
    pub fn direction(mut self, d: StackDirection) -> Self { self.direction = d; self }
    pub fn spacing(mut self, s: impl Into<String>) -> Self { self.spacing = s.into(); self }
    pub fn align(mut self, a: impl Into<String>) -> Self { self.align = Some(a.into()); self }
    pub fn justify(mut self, j: impl Into<String>) -> Self { self.justify = Some(j.into()); self }
}

pub struct Stack;

impl Stack {
    pub fn render(props: StackProps) -> Element {
        let (dir, gap_prop) = match props.direction {
            StackDirection::Vertical => ("flex-direction:column", "row-gap"),
            StackDirection::Horizontal => ("flex-direction:row", "column-gap"),
        };
        let mut parts = vec![format!("display:flex;{};{}:{}", dir, gap_prop, props.spacing)];
        if let Some(a) = &props.align { parts.push(format!("align-items:{}", a)); }
        if let Some(j) = &props.justify { parts.push(format!("justify-content:{}", j)); }
        if let Some(s) = &props.style { parts.push(s.clone()); }
        let style = parts.join(";");

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-stack {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_default() {
        let p = StackProps::default();
        assert_eq!(p.direction, StackDirection::Vertical);
        assert_eq!(p.spacing, "8px");
    }

    #[test]
    fn test_stack_builder() {
        let p = StackProps::default().direction(StackDirection::Horizontal).spacing("24px").align("center");
        assert_eq!(p.direction, StackDirection::Horizontal);
        assert_eq!(p.spacing, "24px");
        assert_eq!(p.align.as_deref(), Some("center"));
    }

    #[test]
    fn test_stack_render() {
        let el = Stack::render(StackProps::default().spacing("16px"));
        assert!(matches!(el, Element::Template(_)));
    }
}
