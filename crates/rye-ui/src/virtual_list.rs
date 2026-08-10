//! VirtualList — virtualized list for 10k+ items.

use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct VirtualItem {
    pub id: String,
    pub content: String,
}

impl VirtualItem {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListProps {
    pub items: Vec<VirtualItem>,
    pub item_height: u32,
    pub visible_height: u32,
    pub scroll_offset: u32,
    pub overscan: u32,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for VirtualListProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            item_height: 40,
            visible_height: 400,
            scroll_offset: 0,
            overscan: 3,
            class: None,
            style: None,
        }
    }
}

impl VirtualListProps {
    pub fn items(mut self, i: Vec<VirtualItem>) -> Self {
        self.items = i;
        self
    }
    pub fn item_height(mut self, h: u32) -> Self {
        self.item_height = h;
        self
    }
    pub fn visible_height(mut self, h: u32) -> Self {
        self.visible_height = h;
        self
    }
    pub fn scroll_offset(mut self, o: u32) -> Self {
        self.scroll_offset = o;
        self
    }
    pub fn overscan(mut self, o: u32) -> Self {
        self.overscan = o;
        self
    }
}

pub struct VirtualList;

impl VirtualList {
    pub fn render(props: VirtualListProps) -> Element {
        let total_height = props.items.len() as u32 * props.item_height;

        let start_idx =
            (props.scroll_offset / props.item_height).saturating_sub(props.overscan) as usize;
        let visible_count =
            (props.visible_height / props.item_height + 2 * props.overscan) as usize;
        let end_idx = (start_idx + visible_count).min(props.items.len());

        let visible_items: Vec<Template> = (start_idx..end_idx)
            .map(|i| {
                let item = &props.items[i];
                let top = i as u32 * props.item_height;
                let item_style = format!(
                    "position:absolute;top:{}px;left:0;right:0;height:{}px;\
                 display:flex;align-items:center;padding:0 16px;font-size:14px;\
                 color:#334155;box-sizing:border-box;",
                    top, props.item_height,
                );
                Template::new_element(
                    "div",
                    vec![
                        ("style".to_string(), item_style),
                        ("class".to_string(), "rye-virtual-list-item".to_string()),
                        ("data-id".to_string(), item.id.clone()),
                    ],
                    Vec::new(),
                    vec![Template::text(&item.content)],
                )
            })
            .collect();

        let inner_style = format!("position:relative;height:{}px;", total_height);

        let container_style = format!(
            "height:{}px;overflow-y:auto;position:relative;{}",
            props.visible_height,
            props.style.as_deref().unwrap_or(""),
        );

        let inner = Template::new_element(
            "div",
            vec![
                ("style".to_string(), inner_style),
                ("class".to_string(), "rye-virtual-list-inner".to_string()),
            ],
            Vec::new(),
            visible_items,
        );

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style),
                (
                    "class".to_string(),
                    format!("rye-virtual-list {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            vec![inner],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_item_new() {
        let i = VirtualItem::new("1", "First item");
        assert_eq!(i.id, "1");
        assert_eq!(i.content, "First item");
    }

    #[test]
    fn test_virtual_list_default() {
        let p = VirtualListProps::default();
        assert_eq!(p.item_height, 40);
        assert_eq!(p.visible_height, 400);
    }

    #[test]
    fn test_virtual_list_builder() {
        let items: Vec<VirtualItem> = (0..100)
            .map(|i| VirtualItem::new(i.to_string(), format!("Item {}", i)))
            .collect();
        let p = VirtualListProps::default()
            .items(items)
            .item_height(50)
            .visible_height(500)
            .scroll_offset(200);
        assert_eq!(p.items.len(), 100);
        assert_eq!(p.item_height, 50);
    }

    #[test]
    fn test_virtual_list_render_empty() {
        let el = VirtualList::render(VirtualListProps::default());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_virtual_list_render_with_items() {
        let items: Vec<VirtualItem> = (0..1000)
            .map(|i| VirtualItem::new(i.to_string(), format!("Row {}", i)))
            .collect();
        let el = VirtualList::render(VirtualListProps::default().items(items).scroll_offset(400));
        assert!(matches!(el, Element::Template(_)));
    }
}
