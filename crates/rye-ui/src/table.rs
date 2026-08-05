//! Table — sortable columns, rows, headers.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    pub width: Option<String>,
    pub sortable: bool,
}

impl TableColumn {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self { key: key.into(), label: label.into(), width: None, sortable: false }
    }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = Some(w.into()); self }
    pub fn sortable(mut self) -> Self { self.sortable = true; self }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

impl TableRow {
    pub fn new(cells: Vec<String>) -> Self { Self { cells } }
}

#[derive(Debug, Clone)]
pub struct TableProps {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub striped: bool,
    pub bordered: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TableProps {
    fn default() -> Self {
        Self { columns: Vec::new(), rows: Vec::new(), striped: false,
               bordered: true, class: None, style: None }
    }
}

impl TableProps {
    pub fn columns(mut self, c: Vec<TableColumn>) -> Self { self.columns = c; self }
    pub fn rows(mut self, r: Vec<TableRow>) -> Self { self.rows = r; self }
    pub fn striped(mut self, s: bool) -> Self { self.striped = s; self }
    pub fn bordered(mut self, b: bool) -> Self { self.bordered = b; self }
}

pub struct Table;

impl Table {
    pub fn render(props: TableProps) -> Element {
        let border = if props.bordered { format!("border:1px solid {};", vars::BORDER) } else { String::new() };
        let style = format!("width:100%;border-collapse:collapse;font-size:var(--rye-font-size-md);{}{}", border, props.style.as_deref().unwrap_or(""));

        // Header
        let headers: Vec<Template> = props.columns.iter().map(|col| {
            let mut style = format!("padding:10px 12px;text-align:left;font-weight:var(--rye-font-weight-semibold);background:{};border-bottom:1px solid {};", vars::BG_SUBTLE, vars::BORDER);
            if let Some(w) = &col.width {
                style.push_str(&format!("width:{};", w));
            }
            let mut children = vec![Template::text(&col.label)];
            if col.sortable {
                children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("margin-left:4px;color:{};font-size:var(--rye-font-size-xs);", vars::TEXT_SUBTLE))],
                    Vec::new(), vec![Template::text("↕")]));
            }
            Template::new_element("th",
                vec![("style".to_string(), style), ("class".to_string(), "rye-table-th".to_string())],
                Vec::new(), children)
        }).collect();

        let thead = Template::new_element("thead", Vec::new(), Vec::new(), vec![
            Template::new_element("tr", Vec::new(), Vec::new(), headers),
        ]);

        // Body
        let body_rows: Vec<Template> = props.rows.iter().enumerate().map(|(i, row)| {
            let bg = if props.striped && i % 2 == 1 { vars::BG_SUBTLE } else { vars::BG };
            let cells: Vec<Template> = row.cells.iter().map(|cell| {
                Template::new_element("td",
                    vec![("style".to_string(), format!("padding:10px 12px;border-bottom:1px solid {};background:{};", vars::BORDER, bg)),
                         ("class".to_string(), "rye-table-td".to_string())],
                    Vec::new(), vec![Template::text(cell)])
            }).collect();
            Template::new_element("tr",
                vec![("class".to_string(), "rye-table-tr".to_string())],
                Vec::new(), cells)
        }).collect();

        let tbody = Template::new_element("tbody", Vec::new(), Vec::new(), body_rows);

        Element::Template(Template::new_element("table",
            vec![("class".to_string(), format!("rye-table {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), vec![thead, tbody]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_column_new() {
        let c = TableColumn::new("name", "Name");
        assert_eq!(c.key, "name");
        assert!(!c.sortable);
    }

    #[test]
    fn test_table_column_builder() {
        let c = TableColumn::new("age", "Age").width("80px").sortable();
        assert_eq!(c.width.as_deref(), Some("80px"));
        assert!(c.sortable);
    }

    #[test]
    fn test_table_row_new() {
        let r = TableRow::new(vec!["Alice".into(), "30".into()]);
        assert_eq!(r.cells.len(), 2);
    }

    #[test]
    fn test_table_props_builder() {
        let p = TableProps::default()
            .columns(vec![TableColumn::new("a", "A"), TableColumn::new("b", "B")])
            .rows(vec![TableRow::new(vec!["1".into(), "2".into()])])
            .striped(true);
        assert_eq!(p.columns.len(), 2);
        assert_eq!(p.rows.len(), 1);
        assert!(p.striped);
    }

    #[test]
    fn test_table_render() {
        let el = Table::render(TableProps::default()
            .columns(vec![TableColumn::new("name", "Name")])
            .rows(vec![TableRow::new(vec!["Alice".into()])]));
        assert!(matches!(el, Element::Template(_)));
    }
}
