//! CommandPalette — Cmd+K search palette (like Raycast/VS Code).

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub category: Option<String>,
}

impl CommandItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), icon: None, shortcut: None, category: None }
    }
    pub fn icon(mut self, i: impl Into<String>) -> Self { self.icon = Some(i.into()); self }
    pub fn shortcut(mut self, s: impl Into<String>) -> Self { self.shortcut = Some(s.into()); self }
    pub fn category(mut self, c: impl Into<String>) -> Self { self.category = Some(c.into()); self }
}

#[derive(Debug, Clone)]
pub struct CommandPaletteProps {
    pub open: bool,
    pub commands: Vec<CommandItem>,
    pub query: String,
    pub placeholder: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CommandPaletteProps {
    fn default() -> Self {
        Self { open: false, commands: Vec::new(), query: String::new(),
               placeholder: "Type a command...".to_string(), class: None, style: None }
    }
}

impl CommandPaletteProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn commands(mut self, c: Vec<CommandItem>) -> Self { self.commands = c; self }
    pub fn query(mut self, q: impl Into<String>) -> Self { self.query = q.into(); self }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = p.into(); self }
}

pub struct CommandPalette;

impl CommandPalette {
    pub fn render(props: CommandPaletteProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let backdrop_style = format!("position:fixed;inset:0;background:{};display:flex;align-items:flex-start;justify-content:center;padding-top:120px;z-index:{};", vars::OVERLAY, vars::Z_MODAL);

        let palette_style = format!(
            "width:560px;max-width:90vw;background:{};border-radius:var(--rye-radius-lg);\
             box-shadow:{};overflow:hidden;{}",
            vars::BG_ELEVATED, vars::SHADOW_XL, props.style.as_deref().unwrap_or(""),
        );

        let input_style = format!("width:100%;padding:16px 20px;border:none;border-bottom:1px solid {};font-size:var(--rye-font-size-lg);outline:none;box-sizing:border-box;", vars::BORDER);

        let mut children = vec![
            Template::new_element("input",
                vec![("type".to_string(), "text".to_string()),
                     ("style".to_string(), input_style.to_string()),
                     ("placeholder".to_string(), props.placeholder.clone()),
                     ("value".to_string(), props.query.clone()),
                     ("class".to_string(), "rye-cmd-palette-input".to_string())],
                Vec::new(), Vec::new()),
        ];

        let filtered: Vec<&CommandItem> = if props.query.is_empty() {
            props.commands.iter().collect()
        } else {
            props.commands.iter().filter(|c| c.label.to_lowercase().contains(&props.query.to_lowercase())).collect()
        };

        let list_style = "max-height:400px;overflow-y:auto;padding:4px;";
        let items: Vec<Template> = filtered.iter().map(|cmd| {
            let item_style = format!("padding:10px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;display:flex;align-items:center;gap:10px;border-radius:var(--rye-radius-md);", vars::TEXT);

            let mut item_children = Vec::new();
            if let Some(icon) = &cmd.icon {
                item_children.push(Template::new_element("span",
                    vec![("style".to_string(), "font-size:18px;".to_string())],
                    Vec::new(), vec![Template::text(icon)]));
            }
            item_children.push(Template::new_element("span",
                vec![("style".to_string(), "flex:1;".to_string())],
                Vec::new(), vec![Template::text(&cmd.label)]));
            if let Some(sc) = &cmd.shortcut {
                item_children.push(Template::new_element("kbd",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-xs);color:{};background:{};padding:2px 6px;border-radius:var(--rye-radius-sm);", vars::TEXT_MUTED, vars::BG_MUTED))],
                    Vec::new(), vec![Template::text(sc)]));
            }

            Template::new_element("div",
                vec![("style".to_string(), item_style.to_string()),
                     ("class".to_string(), "rye-cmd-palette-item".to_string()),
                     ("data-id".to_string(), cmd.id.clone())],
                Vec::new(), item_children)
        }).collect();

        children.push(Template::new_element("div",
            vec![("style".to_string(), list_style.to_string()),
                 ("class".to_string(), "rye-cmd-palette-list".to_string())],
            Vec::new(), items));

        let palette = Template::new_element("div",
            vec![("style".to_string(), palette_style),
                 ("class".to_string(), format!("rye-cmd-palette {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children);

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), backdrop_style.to_string()),
                 ("class".to_string(), "rye-cmd-palette-backdrop".to_string())],
            Vec::new(), vec![palette]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_item_new() {
        let c = CommandItem::new("save", "Save File");
        assert_eq!(c.id, "save");
        assert_eq!(c.label, "Save File");
    }

    #[test]
    fn test_command_item_builder() {
        let c = CommandItem::new("quit", "Quit").icon("⏻").shortcut("Ctrl+Q").category("App");
        assert_eq!(c.icon.as_deref(), Some("⏻"));
        assert_eq!(c.category.as_deref(), Some("App"));
    }

    #[test]
    fn test_command_palette_closed() {
        let el = CommandPalette::render(CommandPaletteProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_command_palette_open() {
        let el = CommandPalette::render(CommandPaletteProps::default()
            .open(true)
            .commands(vec![
                CommandItem::new("new", "New File").shortcut("Ctrl+N"),
                CommandItem::new("open", "Open File").shortcut("Ctrl+O"),
            ]));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_command_palette_filtered() {
        let el = CommandPalette::render(CommandPaletteProps::default()
            .open(true)
            .query("open")
            .commands(vec![
                CommandItem::new("new", "New File"),
                CommandItem::new("open", "Open File"),
            ]));
        assert!(matches!(el, Element::Template(_)));
    }
}
