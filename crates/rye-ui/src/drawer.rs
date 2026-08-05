//! Drawer — slide-in side panel (left/right).

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerSide {
    Left,
    Right,
}

impl DrawerSide {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Left => "left", Self::Right => "right" }
    }
}

#[derive(Debug, Clone)]
pub struct DrawerProps {
    pub open: bool,
    pub side: DrawerSide,
    pub title: Option<String>,
    pub width: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DrawerProps {
    fn default() -> Self {
        Self { open: false, side: DrawerSide::Right, title: None,
               width: "400px".to_string(), class: None, style: None }
    }
}

impl DrawerProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn side(mut self, s: DrawerSide) -> Self { self.side = s; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = w.into(); self }
}

pub struct Drawer;

impl Drawer {
    pub fn render(props: DrawerProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let backdrop_style = format!("position:fixed;inset:0;background:{};z-index:{};", vars::OVERLAY, vars::Z_OVERLAY);

        let (side_pos, transform) = match props.side {
            DrawerSide::Left => ("left:0;top:0;bottom:0;", ""),
            DrawerSide::Right => ("right:0;top:0;bottom:0;", ""),
        };

        let drawer_style = format!(
            "position:fixed;{}width:{};max-width:90vw;background:{};\
             box-shadow:{};z-index:{};\
             display:flex;flex-direction:column;overflow:hidden;{}{}",
            side_pos, props.width, vars::BG_ELEVATED, vars::SHADOW_LG, vars::Z_MODAL, transform, props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        if let Some(title) = &props.title {
            children.push(Template::new_element("div",
                vec![("style".to_string(), format!("display:flex;align-items:center;justify-content:space-between;padding:16px 20px;border-bottom:1px solid {};flex-shrink:0;", vars::BORDER)),
                     ("class".to_string(), "rye-drawer-header".to_string())],
                Vec::new(), vec![
                    Template::new_element("h2",
                        vec![("style".to_string(), "font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);margin:0;".to_string())],
                        Vec::new(), vec![Template::text(title)]),
                    Template::new_element("button",
                        vec![("style".to_string(), format!("border:none;background:none;font-size:24px;cursor:pointer;color:{};padding:0;", vars::TEXT_MUTED)),
                             ("aria-label".to_string(), "Close".to_string())],
                        Vec::new(), vec![Template::text("×")]),
                ]));
        }

        children.push(Template::new_element("div",
            vec![("style".to_string(), "padding:20px;overflow-y:auto;flex:1;".to_string()),
                 ("class".to_string(), "rye-drawer-body".to_string())],
            Vec::new(), Vec::new()));

        let drawer = Template::new_element("div",
            vec![("style".to_string(), drawer_style),
                 ("class".to_string(), format!("rye-drawer rye-drawer-{} {}", props.side.as_str(), props.class.as_deref().unwrap_or("")))],
            Vec::new(), children);

        let backdrop = Template::new_element("div",
            vec![("style".to_string(), backdrop_style.to_string()),
                 ("class".to_string(), "rye-drawer-backdrop".to_string())],
            Vec::new(), Vec::new());

        Element::Template(Template::new_element("div", Vec::new(), Vec::new(), vec![backdrop, drawer]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_side_as_str() {
        assert_eq!(DrawerSide::Left.as_str(), "left");
        assert_eq!(DrawerSide::Right.as_str(), "right");
    }

    #[test]
    fn test_drawer_default() {
        let p = DrawerProps::default();
        assert!(!p.open);
        assert_eq!(p.side, DrawerSide::Right);
    }

    #[test]
    fn test_drawer_builder() {
        let p = DrawerProps::default().open(true).side(DrawerSide::Left).title("Filters").width("500px");
        assert!(p.open);
        assert_eq!(p.side, DrawerSide::Left);
    }

    #[test]
    fn test_drawer_closed() {
        let el = Drawer::render(DrawerProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_drawer_open_left() {
        let el = Drawer::render(DrawerProps::default().open(true).side(DrawerSide::Left).title("Menu"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_drawer_open_right() {
        let el = Drawer::render(DrawerProps::default().open(true).title("Cart"));
        assert!(matches!(el, Element::Template(_)));
    }
}
