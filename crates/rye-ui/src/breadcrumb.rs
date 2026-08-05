//! Breadcrumb — navigation trail.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub current: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), href: None, current: false }
    }
    pub fn href(mut self, h: impl Into<String>) -> Self { self.href = Some(h.into()); self }
    pub fn current(mut self) -> Self { self.current = true; self }
}

#[derive(Debug, Clone)]
pub struct BreadcrumbProps {
    pub items: Vec<BreadcrumbItem>,
    pub separator: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for BreadcrumbProps {
    fn default() -> Self {
        Self { items: Vec::new(), separator: "/".to_string(), class: None, style: None }
    }
}

impl BreadcrumbProps {
    pub fn items(mut self, i: Vec<BreadcrumbItem>) -> Self { self.items = i; self }
    pub fn separator(mut self, s: impl Into<String>) -> Self { self.separator = s.into(); self }
}

pub struct Breadcrumb;

impl Breadcrumb {
    pub fn render(props: BreadcrumbProps) -> Element {
        let mut children: Vec<Template> = Vec::new();
        for (i, item) in props.items.iter().enumerate() {
            let style = if item.current {
                format!("font-size:var(--rye-font-size-md);color:{};", vars::TEXT_MUTED)
            } else {
                format!("font-size:var(--rye-font-size-md);color:{};cursor:pointer;", vars::PRIMARY)
            };

            let mut attrs = vec![("style".to_string(), style), ("class".to_string(), "rye-breadcrumb-item".to_string())];
            if let Some(href) = &item.href {
                attrs.push(("href".to_string(), href.clone()));
            }
            if item.current {
                attrs.push(("aria-current".to_string(), "page".to_string()));
            }

            let tag = if item.href.is_some() && !item.current { "a" } else { "span" };
            children.push(Template::new_element(tag, attrs, Vec::new(), vec![Template::text(&item.label)]));

            if i < props.items.len() - 1 {
                children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);color:var(--rye-text-subtle);margin:0 8px;")),
                         ("class".to_string(), "rye-breadcrumb-separator".to_string())],
                    Vec::new(), vec![Template::text(&props.separator)]));
            }
        }

        Element::Template(Template::new_element("nav",
            vec![("class".to_string(), format!("rye-breadcrumb {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), props.style.as_deref().unwrap_or("").to_string())],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_item_new() {
        let i = BreadcrumbItem::new("Home");
        assert_eq!(i.label, "Home");
        assert!(!i.current);
    }

    #[test]
    fn test_breadcrumb_item_builder() {
        let i = BreadcrumbItem::new("Products").href("/products").current();
        assert_eq!(i.href.as_deref(), Some("/products"));
        assert!(i.current);
    }

    #[test]
    fn test_breadcrumb_render() {
        let el = Breadcrumb::render(BreadcrumbProps::default()
            .items(vec![
                BreadcrumbItem::new("Home").href("/"),
                BreadcrumbItem::new("Products").href("/products"),
                BreadcrumbItem::new("Details").current(),
            ]));
        assert!(matches!(el, Element::Template(_)));
    }
}
