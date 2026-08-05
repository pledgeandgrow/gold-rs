//! DataTable — sortable, filterable, paginated table.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
    None,
}

impl SortDirection {
    pub fn icon(&self) -> &'static str {
        match self { Self::Asc => "↑", Self::Desc => "↓", Self::None => "↕" }
    }
    pub fn toggle(&self) -> Self {
        match self { Self::None => Self::Asc, Self::Asc => Self::Desc, Self::Desc => Self::Asc }
    }
}

#[derive(Debug, Clone)]
pub struct DataColumn {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub width: Option<String>,
}

impl DataColumn {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self { key: key.into(), label: label.into(), sortable: false, width: None }
    }
    pub fn sortable(mut self) -> Self { self.sortable = true; self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = Some(w.into()); self }
}

#[derive(Debug, Clone)]
pub struct DataRow {
    pub cells: Vec<String>,
}

impl DataRow {
    pub fn new(cells: Vec<String>) -> Self { Self { cells } }
}

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub enabled: bool,
    pub placeholder: String,
}

impl Default for FilterConfig {
    fn default() -> Self { Self { enabled: false, placeholder: "Filter...".to_string() } }
}

#[derive(Debug, Clone)]
pub struct PaginationConfig {
    pub enabled: bool,
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
}

impl Default for PaginationConfig {
    fn default() -> Self { Self { enabled: false, page: 1, per_page: 10, total: 0 } }
}

#[derive(Debug, Clone)]
pub struct DataTableProps {
    pub columns: Vec<DataColumn>,
    pub rows: Vec<DataRow>,
    pub sort_column: Option<String>,
    pub sort_direction: SortDirection,
    pub filter: FilterConfig,
    pub pagination: PaginationConfig,
    pub striped: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DataTableProps {
    fn default() -> Self {
        Self { columns: Vec::new(), rows: Vec::new(), sort_column: None,
               sort_direction: SortDirection::None, filter: FilterConfig::default(),
               pagination: PaginationConfig::default(), striped: true, class: None, style: None }
    }
}

impl DataTableProps {
    pub fn columns(mut self, c: Vec<DataColumn>) -> Self { self.columns = c; self }
    pub fn rows(mut self, r: Vec<DataRow>) -> Self { self.rows = r; self }
    pub fn sort(mut self, col: impl Into<String>, dir: SortDirection) -> Self {
        self.sort_column = Some(col.into()); self.sort_direction = dir; self
    }
    pub fn filter(mut self, f: FilterConfig) -> Self { self.filter = f; self }
    pub fn pagination(mut self, p: PaginationConfig) -> Self { self.pagination = p; self }
    pub fn striped(mut self, s: bool) -> Self { self.striped = s; self }
}

pub struct DataTable;

impl DataTable {
    pub fn render(props: DataTableProps) -> Element {
        let mut children = Vec::new();

        // Filter bar
        if props.filter.enabled {
            children.push(Template::new_element("div",
                vec![("style".to_string(), "padding:8px 0;display:flex;gap:8px;".to_string()),
                     ("class".to_string(), "rye-data-table-filter".to_string())],
                Vec::new(), vec![
                    Template::new_element("input",
                        vec![("type".to_string(), "text".to_string()),
                             ("placeholder".to_string(), props.filter.placeholder.clone()),
                             ("style".to_string(), format!("padding:6px 12px;border:1px solid {};border-radius:var(--rye-radius-md);font-size:var(--rye-font-size-md);flex:1;", vars::INPUT_BORDER))],
                        Vec::new(), Vec::new()),
                ]));
        }

        // Table
        let table_style = format!("width:100%;border-collapse:collapse;font-size:var(--rye-font-size-md);{}", props.style.as_deref().unwrap_or(""));

        let headers: Vec<Template> = props.columns.iter().map(|col| {
            let mut style = format!("padding:10px 12px;text-align:left;font-weight:var(--rye-font-weight-semibold);background:{};border-bottom:2px solid {};", vars::BG_SUBTLE, vars::BORDER);
            if let Some(w) = &col.width { style.push_str(&format!("width:{};", w)); }

            let mut h_children = vec![Template::text(&col.label)];
            if col.sortable {
                let icon = if props.sort_column.as_deref() == Some(col.key.as_str()) {
                    props.sort_direction.icon()
                } else {
                    SortDirection::None.icon()
                };
                h_children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("margin-left:4px;color:{};font-size:var(--rye-font-size-xs);", vars::TEXT_SUBTLE))],
                    Vec::new(), vec![Template::text(icon)]));
            }

            Template::new_element("th",
                vec![("style".to_string(), style), ("class".to_string(), "rye-data-table-th".to_string())],
                Vec::new(), h_children)
        }).collect();

        let thead = Template::new_element("thead", Vec::new(), Vec::new(),
            vec![Template::new_element("tr", Vec::new(), Vec::new(), headers)]);

        let body_rows: Vec<Template> = props.rows.iter().enumerate().map(|(i, row)| {
            let bg = if props.striped && i % 2 == 1 { vars::BG_SUBTLE } else { vars::BG };
            let cells: Vec<Template> = row.cells.iter().map(|cell| {
                Template::new_element("td",
                    vec![("style".to_string(), format!("padding:10px 12px;border-bottom:1px solid {};background:{};", vars::BORDER, bg))],
                    Vec::new(), vec![Template::text(cell)])
            }).collect();
            Template::new_element("tr", Vec::new(), Vec::new(), cells)
        }).collect();

        let tbody = Template::new_element("tbody", Vec::new(), Vec::new(), body_rows);

        children.push(Template::new_element("table",
            vec![("style".to_string(), table_style),
                 ("class".to_string(), format!("rye-data-table {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), vec![thead, tbody]));

        // Pagination
        if props.pagination.enabled && props.pagination.total > 0 {
            let total_pages = (props.pagination.total + props.pagination.per_page - 1) / props.pagination.per_page;
            let pag_style = format!("display:flex;align-items:center;justify-content:space-between;padding:8px 0;font-size:var(--rye-font-size-sm);color:{};", vars::TEXT_MUTED);
            let pag_children = vec![
                Template::new_element("span", Vec::new(), Vec::new(),
                    vec![Template::text(&format!("Page {} of {} ({} items)", props.pagination.page, total_pages, props.pagination.total))]),
                Template::new_element("div",
                    vec![("style".to_string(), "display:flex;gap:4px;".to_string())],
                    Vec::new(), vec![
                        Template::new_element("button",
                            vec![("style".to_string(), format!("padding:4px 10px;border:1px solid {};border-radius:var(--rye-radius-sm);background:{};cursor:pointer;font-size:var(--rye-font-size-sm);", vars::INPUT_BORDER, vars::BG))],
                            Vec::new(), vec![Template::text("‹ Prev")]),
                        Template::new_element("button",
                            vec![("style".to_string(), format!("padding:4px 10px;border:1px solid {};border-radius:var(--rye-radius-sm);background:{};cursor:pointer;font-size:var(--rye-font-size-sm);", vars::INPUT_BORDER, vars::BG))],
                            Vec::new(), vec![Template::text("Next ›")]),
                    ]),
            ];
            children.push(Template::new_element("div",
                vec![("style".to_string(), pag_style.to_string()),
                     ("class".to_string(), "rye-data-table-pagination".to_string())],
                Vec::new(), pag_children));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-data-table-wrapper".to_string())],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction_toggle() {
        assert_eq!(SortDirection::None.toggle(), SortDirection::Asc);
        assert_eq!(SortDirection::Asc.toggle(), SortDirection::Desc);
        assert_eq!(SortDirection::Desc.toggle(), SortDirection::Asc);
    }

    #[test]
    fn test_sort_direction_icon() {
        assert_eq!(SortDirection::Asc.icon(), "↑");
        assert_eq!(SortDirection::Desc.icon(), "↓");
    }

    #[test]
    fn test_data_column_builder() {
        let c = DataColumn::new("name", "Name").sortable().width("200px");
        assert!(c.sortable);
        assert_eq!(c.width.as_deref(), Some("200px"));
    }

    #[test]
    fn test_data_table_props_builder() {
        let p = DataTableProps::default()
            .columns(vec![DataColumn::new("id", "ID").sortable(), DataColumn::new("name", "Name")])
            .rows(vec![DataRow::new(vec!["1".into(), "Alice".into()])])
            .sort("name", SortDirection::Asc)
            .filter(FilterConfig { enabled: true, placeholder: "Search...".to_string() })
            .pagination(PaginationConfig { enabled: true, page: 1, per_page: 5, total: 23 });
        assert_eq!(p.columns.len(), 2);
        assert_eq!(p.sort_direction, SortDirection::Asc);
        assert!(p.filter.enabled);
        assert!(p.pagination.enabled);
    }

    #[test]
    fn test_data_table_render() {
        let el = DataTable::render(DataTableProps::default()
            .columns(vec![DataColumn::new("a", "A")])
            .rows(vec![DataRow::new(vec!["1".into()])]));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_data_table_render_with_pagination() {
        let el = DataTable::render(DataTableProps::default()
            .columns(vec![DataColumn::new("a", "A")])
            .rows(vec![DataRow::new(vec!["1".into()])])
            .pagination(PaginationConfig { enabled: true, page: 2, per_page: 10, total: 50 }));
        assert!(matches!(el, Element::Template(_)));
    }
}
