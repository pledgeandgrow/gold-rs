//! Flex — flexbox container.

use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
            Self::RowReverse => "row-reverse",
            Self::ColumnReverse => "column-reverse",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl FlexWrap {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoWrap => "nowrap",
            Self::Wrap => "wrap",
            Self::WrapReverse => "wrap-reverse",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl JustifyContent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FlexStart => "flex-start",
            Self::FlexEnd => "flex-end",
            Self::Center => "center",
            Self::SpaceBetween => "space-between",
            Self::SpaceAround => "space-around",
            Self::SpaceEvenly => "space-evenly",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl AlignItems {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stretch => "stretch",
            Self::FlexStart => "flex-start",
            Self::FlexEnd => "flex-end",
            Self::Center => "center",
            Self::Baseline => "baseline",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlexProps {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify: JustifyContent,
    pub align: AlignItems,
    pub gap: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for FlexProps {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify: JustifyContent::FlexStart,
            align: AlignItems::Stretch,
            gap: None,
            class: None,
            style: None,
        }
    }
}

impl FlexProps {
    pub fn direction(mut self, d: FlexDirection) -> Self {
        self.direction = d;
        self
    }
    pub fn wrap(mut self, w: FlexWrap) -> Self {
        self.wrap = w;
        self
    }
    pub fn justify(mut self, j: JustifyContent) -> Self {
        self.justify = j;
        self
    }
    pub fn align(mut self, a: AlignItems) -> Self {
        self.align = a;
        self
    }
    pub fn gap(mut self, g: impl Into<String>) -> Self {
        self.gap = Some(g.into());
        self
    }
}

pub struct Flex;

impl Flex {
    pub fn render(props: FlexProps) -> Element {
        let mut parts = vec![format!(
            "display:flex;flex-direction:{};flex-wrap:{};justify-content:{};align-items:{}",
            props.direction.as_str(),
            props.wrap.as_str(),
            props.justify.as_str(),
            props.align.as_str()
        )];
        if let Some(g) = &props.gap {
            parts.push(format!("gap:{}", g));
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
                    format!("rye-flex {}", props.class.as_deref().unwrap_or("")),
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
    fn test_flex_default() {
        let p = FlexProps::default();
        assert_eq!(p.direction, FlexDirection::Row);
        assert_eq!(p.justify, JustifyContent::FlexStart);
    }

    #[test]
    fn test_flex_builder() {
        let p = FlexProps::default()
            .direction(FlexDirection::Column)
            .justify(JustifyContent::Center)
            .align(AlignItems::Center)
            .gap("16px");
        assert_eq!(p.direction, FlexDirection::Column);
        assert_eq!(p.gap.as_deref(), Some("16px"));
    }

    #[test]
    fn test_flex_direction_as_str() {
        assert_eq!(FlexDirection::Row.as_str(), "row");
        assert_eq!(FlexDirection::Column.as_str(), "column");
        assert_eq!(FlexDirection::RowReverse.as_str(), "row-reverse");
    }

    #[test]
    fn test_flex_justify_as_str() {
        assert_eq!(JustifyContent::SpaceBetween.as_str(), "space-between");
        assert_eq!(JustifyContent::Center.as_str(), "center");
    }

    #[test]
    fn test_flex_render() {
        let el = Flex::render(
            FlexProps::default()
                .direction(FlexDirection::Column)
                .gap("8px"),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
