//! Stat / KPI Card — metric with label, value, trend indicator.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatTrend {
    Up,
    Down,
    Neutral,
}

impl StatTrend {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Up => "↑",
            Self::Down => "↓",
            Self::Neutral => "→",
        }
    }
    pub fn color(&self) -> &'static str {
        match self {
            Self::Up => vars::SUCCESS,
            Self::Down => vars::DANGER,
            Self::Neutral => vars::TEXT_MUTED,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatProps {
    pub label: String,
    pub value: String,
    pub trend: StatTrend,
    pub trend_value: Option<String>,
    pub icon: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for StatProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: String::new(),
            trend: StatTrend::Neutral,
            trend_value: None,
            icon: None,
            class: None,
            style: None,
        }
    }
}

impl StatProps {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn trend(mut self, t: StatTrend) -> Self {
        self.trend = t;
        self
    }
    pub fn trend_value(mut self, tv: impl Into<String>) -> Self {
        self.trend_value = Some(tv.into());
        self
    }
    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(i.into());
        self
    }
}

pub struct Stat;

impl Stat {
    pub fn render(props: StatProps) -> Element {
        let style =
            format!(
            "padding:20px;background:{};border:1px solid {};border-radius:var(--rye-radius-lg);\
             box-shadow:{};{}",
            vars::BG, vars::BORDER, vars::SHADOW_SM,
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        // Top row: label + icon
        let mut top_children = vec![
            Template::new_element("span",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};font-weight:var(--rye-font-weight-medium);", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(&props.label)]),
        ];

        if let Some(icon) = &props.icon {
            top_children.push(Template::new_element(
                "span",
                vec![("style".to_string(), "font-size:20px;".to_string())],
                Vec::new(),
                vec![Template::text(icon)],
            ));
        }

        children.push(Template::new_element(
            "div",
            vec![(
                "style".to_string(),
                "display:flex;align-items:center;justify-content:space-between;margin-bottom:12px;"
                    .to_string(),
            )],
            Vec::new(),
            top_children,
        ));

        // Value
        children.push(Template::new_element("div",
            vec![("style".to_string(), format!("font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-bold);color:{};margin-bottom:8px;", vars::TEXT))],
            Vec::new(), vec![Template::text(&props.value)]));

        // Trend
        if let Some(tv) = &props.trend_value {
            let trend_style = format!("display:inline-flex;align-items:center;gap:4px;font-size:var(--rye-font-size-sm);color:{};font-weight:var(--rye-font-weight-medium);", props.trend.color());
            children.push(Template::new_element(
                "div",
                vec![("style".to_string(), trend_style)],
                Vec::new(),
                vec![
                    Template::new_element(
                        "span",
                        vec![("style".to_string(), "font-size:16px;".to_string())],
                        Vec::new(),
                        vec![Template::text(props.trend.icon())],
                    ),
                    Template::text(tv),
                ],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!("rye-stat {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_trend() {
        assert_eq!(StatTrend::Up.icon(), "↑");
        assert_eq!(StatTrend::Down.color(), vars::DANGER);
    }

    #[test]
    fn test_stat_default() {
        let p = StatProps::default();
        assert_eq!(p.trend, StatTrend::Neutral);
        assert!(p.trend_value.is_none());
    }

    #[test]
    fn test_stat_builder() {
        let p = StatProps::default()
            .label("Revenue")
            .value("$42,580")
            .trend(StatTrend::Up)
            .trend_value("+12.5%")
            .icon("💰");
        assert_eq!(p.label, "Revenue");
        assert_eq!(p.trend, StatTrend::Up);
    }

    #[test]
    fn test_stat_render() {
        let el = Stat::render(
            StatProps::default()
                .label("Users")
                .value("1,234")
                .trend(StatTrend::Up)
                .trend_value("+8%"),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_stat_render_no_trend() {
        let el = Stat::render(StatProps::default().label("Status").value("Active"));
        assert!(matches!(el, Element::Template(_)));
    }
}
