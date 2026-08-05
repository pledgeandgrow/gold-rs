//! Container — max-width centered container.

use rye_core::Element;
use rye_core::template::Template;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSize {
    Small,
    Medium,
    Large,
    Full,
}

impl ContainerSize {
    pub fn max_width(&self) -> &'static str {
        match self {
            Self::Small => "640px",
            Self::Medium => "768px",
            Self::Large => "1024px",
            Self::Full => "100%",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerProps {
    pub size: ContainerSize,
    pub padding: String,
    pub centered: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self { size: ContainerSize::Medium, padding: "16px".to_string(),
               centered: true, class: None, style: None }
    }
}

impl ContainerProps {
    pub fn size(mut self, s: ContainerSize) -> Self { self.size = s; self }
    pub fn padding(mut self, p: impl Into<String>) -> Self { self.padding = p.into(); self }
    pub fn centered(mut self, c: bool) -> Self { self.centered = c; self }
}

pub struct Container;

impl Container {
    pub fn render(props: ContainerProps) -> Element {
        let mut parts = vec![
            format!("max-width:{}", props.size.max_width()),
            format!("padding:0 {}", props.padding),
        ];
        if props.centered {
            parts.push("margin-left:auto".to_string());
            parts.push("margin-right:auto".to_string());
        }
        if let Some(s) = &props.style { parts.push(s.clone()); }
        let style = parts.join(";");

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-container {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_size_max_width() {
        assert_eq!(ContainerSize::Small.max_width(), "640px");
        assert_eq!(ContainerSize::Medium.max_width(), "768px");
        assert_eq!(ContainerSize::Large.max_width(), "1024px");
        assert_eq!(ContainerSize::Full.max_width(), "100%");
    }

    #[test]
    fn test_container_default() {
        let p = ContainerProps::default();
        assert_eq!(p.size, ContainerSize::Medium);
        assert!(p.centered);
    }

    #[test]
    fn test_container_builder() {
        let p = ContainerProps::default().size(ContainerSize::Large).padding("24px").centered(false);
        assert_eq!(p.size, ContainerSize::Large);
        assert!(!p.centered);
    }

    #[test]
    fn test_container_render() {
        let el = Container::render(ContainerProps::default());
        assert!(matches!(el, Element::Template(_)));
    }
}
