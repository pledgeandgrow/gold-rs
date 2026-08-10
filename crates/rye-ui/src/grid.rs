//! Grid — CSS grid container.

use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct GridProps {
    pub columns: String,
    pub rows: Option<String>,
    pub gap: Option<String>,
    pub column_gap: Option<String>,
    pub row_gap: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for GridProps {
    fn default() -> Self {
        Self {
            columns: "1fr".to_string(),
            rows: None,
            gap: None,
            column_gap: None,
            row_gap: None,
            class: None,
            style: None,
        }
    }
}

impl GridProps {
    pub fn columns(mut self, c: impl Into<String>) -> Self {
        self.columns = c.into();
        self
    }
    pub fn rows(mut self, r: impl Into<String>) -> Self {
        self.rows = Some(r.into());
        self
    }
    pub fn gap(mut self, g: impl Into<String>) -> Self {
        self.gap = Some(g.into());
        self
    }
    pub fn column_gap(mut self, g: impl Into<String>) -> Self {
        self.column_gap = Some(g.into());
        self
    }
    pub fn row_gap(mut self, g: impl Into<String>) -> Self {
        self.row_gap = Some(g.into());
        self
    }
}

pub struct Grid;

impl Grid {
    pub fn render(props: GridProps) -> Element {
        let mut parts = vec![format!(
            "display:grid;grid-template-columns:{}",
            props.columns
        )];
        if let Some(r) = &props.rows {
            parts.push(format!("grid-template-rows:{}", r));
        }
        if let Some(g) = &props.gap {
            parts.push(format!("gap:{}", g));
        }
        if let Some(cg) = &props.column_gap {
            parts.push(format!("column-gap:{}", cg));
        }
        if let Some(rg) = &props.row_gap {
            parts.push(format!("row-gap:{}", rg));
        }
        if let Some(s) = &props.style {
            parts.push(s.clone());
        }
        let style = parts.join(";");

        Element::Template(Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!("rye-grid {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
            ],
            Vec::new(),
            Vec::new(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_default() {
        let p = GridProps::default();
        assert_eq!(p.columns, "1fr");
    }

    #[test]
    fn test_grid_builder() {
        let p = GridProps::default()
            .columns("repeat(3, 1fr)")
            .gap("16px")
            .rows("auto auto");
        assert_eq!(p.columns, "repeat(3, 1fr)");
        assert_eq!(p.gap.as_deref(), Some("16px"));
        assert_eq!(p.rows.as_deref(), Some("auto auto"));
    }

    #[test]
    fn test_grid_render() {
        let el = Grid::render(GridProps::default().columns("1fr 1fr 1fr").gap("12px"));
        assert!(matches!(el, Element::Template(_)));
    }
}
