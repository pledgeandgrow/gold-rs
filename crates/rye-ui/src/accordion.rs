//! Accordion — collapsible sections.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct AccordionItem {
    pub title: String,
    pub id: String,
    pub open: bool,
}

impl AccordionItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self { id: id.into(), title: title.into(), open: false }
    }
    pub fn open(mut self) -> Self { self.open = true; self }
}

#[derive(Debug, Clone)]
pub struct AccordionProps {
    pub items: Vec<AccordionItem>,
    pub multiple: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for AccordionProps {
    fn default() -> Self {
        Self { items: Vec::new(), multiple: false, class: None, style: None }
    }
}

impl AccordionProps {
    pub fn items(mut self, i: Vec<AccordionItem>) -> Self { self.items = i; self }
    pub fn multiple(mut self, m: bool) -> Self { self.multiple = m; self }
}

pub struct Accordion;

impl Accordion {
    pub fn render(props: AccordionProps) -> Element {
        let sections: Vec<Template> = props.items.iter().map(|item| {
            let header_style = format!(
                "padding:12px 16px;background:{};border:1px solid {};cursor:pointer;\
                 display:flex;justify-content:space-between;align-items:center;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);",
                if item.open { vars::BG_MUTED } else { vars::BG }, vars::BORDER,
            );
            let icon = if item.open { "▼" } else { "▶" };

            let mut children = vec![
                Template::new_element("div",
                    vec![("style".to_string(), header_style),
                         ("class".to_string(), "rye-accordion-header".to_string())],
                    Vec::new(), vec![
                        Template::text(&item.title),
                        Template::new_element("span",
                            vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};", vars::TEXT_MUTED))],
                            Vec::new(), vec![Template::text(icon)]),
                    ]),
            ];

            if item.open {
                children.push(Template::new_element("div",
                    vec![("style".to_string(), format!("padding:12px 16px;border:1px solid {};border-top:none;font-size:var(--rye-font-size-md);color:{};", vars::BORDER, vars::TEXT)),
                         ("class".to_string(), "rye-accordion-content".to_string())],
                    Vec::new(), Vec::new()));
            }

            Template::new_element("div",
                vec![("class".to_string(), "rye-accordion-item".to_string())],
                Vec::new(), children)
        }).collect();

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-accordion {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), props.style.as_deref().unwrap_or("").to_string())],
            Vec::new(), sections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accordion_item_new() {
        let i = AccordionItem::new("s1", "Section 1");
        assert_eq!(i.id, "s1");
        assert!(!i.open);
    }

    #[test]
    fn test_accordion_item_open() {
        let i = AccordionItem::new("s1", "Section 1").open();
        assert!(i.open);
    }

    #[test]
    fn test_accordion_props_builder() {
        let p = AccordionProps::default()
            .items(vec![AccordionItem::new("a", "A"), AccordionItem::new("b", "B").open()])
            .multiple(true);
        assert_eq!(p.items.len(), 2);
        assert!(p.items[1].open);
        assert!(p.multiple);
    }

    #[test]
    fn test_accordion_render() {
        let el = Accordion::render(AccordionProps::default()
            .items(vec![AccordionItem::new("1", "One").open()]));
        assert!(matches!(el, Element::Template(_)));
    }
}
