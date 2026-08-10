//! List — ordered/unordered with items.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListVariant {
    Unordered,
    Ordered,
    Unstyled,
}

impl ListVariant {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Unordered | Self::Unstyled => "ul",
            Self::Ordered => "ol",
        }
    }
    pub fn style(&self) -> &'static str {
        match self {
            Self::Unordered => "padding-left:20px;list-style:disc;",
            Self::Ordered => "padding-left:20px;list-style:decimal;",
            Self::Unstyled => "list-style:none;padding:0;",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub text: String,
}

impl ListItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone)]
pub struct ListProps {
    pub items: Vec<ListItem>,
    pub variant: ListVariant,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ListProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            variant: ListVariant::Unordered,
            class: None,
            style: None,
        }
    }
}

impl ListProps {
    pub fn items(mut self, i: Vec<ListItem>) -> Self {
        self.items = i;
        self
    }
    pub fn variant(mut self, v: ListVariant) -> Self {
        self.variant = v;
        self
    }
}

pub struct List;

impl List {
    pub fn render(props: ListProps) -> Element {
        let items: Vec<Template> = props
            .items
            .iter()
            .map(|item| {
                Template::new_element(
                    "li",
                    vec![
                        (
                            "style".to_string(),
                            format!(
                                "margin-bottom:4px;font-size:var(--rye-font-size-md);color:{};",
                                vars::TEXT
                            ),
                        ),
                        ("class".to_string(), "rye-list-item".to_string()),
                    ],
                    Vec::new(),
                    vec![Template::text(&item.text)],
                )
            })
            .collect();

        let style = format!(
            "{}{}",
            props.variant.style(),
            props.style.as_deref().unwrap_or("")
        );

        Element::Template(Template::new_element(
            props.variant.tag(),
            vec![
                (
                    "class".to_string(),
                    format!("rye-list {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
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
    fn test_list_variant_tag() {
        assert_eq!(ListVariant::Unordered.tag(), "ul");
        assert_eq!(ListVariant::Ordered.tag(), "ol");
    }

    #[test]
    fn test_list_item_new() {
        let i = ListItem::new("Hello");
        assert_eq!(i.text, "Hello");
    }

    #[test]
    fn test_list_props_builder() {
        let p = ListProps::default()
            .items(vec![ListItem::new("A"), ListItem::new("B")])
            .variant(ListVariant::Ordered);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.variant, ListVariant::Ordered);
    }

    #[test]
    fn test_list_render() {
        let el = List::render(
            ListProps::default()
                .items(vec![ListItem::new("One"), ListItem::new("Two")])
                .variant(ListVariant::Unstyled),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
