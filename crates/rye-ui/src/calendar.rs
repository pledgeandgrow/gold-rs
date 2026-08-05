//! Calendar — full month calendar grid.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct CalendarDay {
    pub day: u32,
    pub in_month: bool,
    pub selected: bool,
    pub today: bool,
    pub disabled: bool,
}

impl CalendarDay {
    pub fn new(day: u32, in_month: bool) -> Self {
        Self { day, in_month, selected: false, today: false, disabled: false }
    }
    pub fn selected(mut self) -> Self { self.selected = true; self }
    pub fn today(mut self) -> Self { self.today = true; self }
    pub fn disabled(mut self) -> Self { self.disabled = true; self }
}

#[derive(Debug, Clone)]
pub struct CalendarProps {
    pub year: u32,
    pub month: u32, // 1-12
    pub days: Vec<CalendarDay>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CalendarProps {
    fn default() -> Self {
        Self { year: 2025, month: 1, days: Vec::new(), class: None, style: None }
    }
}

impl CalendarProps {
    pub fn year(mut self, y: u32) -> Self { self.year = y; self }
    pub fn month(mut self, m: u32) -> Self { self.month = m; self }
    pub fn days(mut self, d: Vec<CalendarDay>) -> Self { self.days = d; self }
}

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub struct Calendar;

impl Calendar {
    pub fn render(props: CalendarProps) -> Element {
        let style = format!(
            "width:320px;background:{};border-radius:var(--rye-radius-lg);border:1px solid {};padding:12px;{}",
            vars::BG, vars::BORDER, props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        // Header
        let month_name = MONTH_NAMES.get((props.month - 1) as usize).unwrap_or(&"Unknown");
        children.push(Template::new_element("div",
            vec![("style".to_string(), "display:flex;align-items:center;justify-content:space-between;margin-bottom:12px;".to_string()),
                 ("class".to_string(), "rye-calendar-header".to_string())],
            Vec::new(), vec![
                Template::new_element("button",
                    vec![("style".to_string(), format!("border:none;background:none;cursor:pointer;font-size:var(--rye-font-size-xl);color:{};padding:4px 8px;", vars::TEXT_MUTED))],
                    Vec::new(), vec![Template::text("‹")]),
                Template::new_element("span",
                    vec![("style".to_string(), format!("font-size:var(--rye-font-size-lg);font-weight:var(--rye-font-weight-semibold);color:{};", vars::TEXT))],
                    Vec::new(), vec![Template::text(&format!("{} {}", month_name, props.year))]),
                Template::new_element("button",
                    vec![("style".to_string(), format!("border:none;background:none;cursor:pointer;font-size:var(--rye-font-size-xl);color:{};padding:4px 8px;", vars::TEXT_MUTED))],
                    Vec::new(), vec![Template::text("›")]),
            ]));

        // Weekday header
        let weekday_cells: Vec<Template> = WEEKDAYS.iter().map(|wd| {
            Template::new_element("div",
                vec![("style".to_string(), format!("text-align:center;font-size:var(--rye-font-size-sm);font-weight:var(--rye-font-weight-semibold);color:{};padding:8px 0;", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(*wd)])
        }).collect();

        children.push(Template::new_element("div",
            vec![("style".to_string(), "display:grid;grid-template-columns:repeat(7,1fr);".to_string())],
            Vec::new(), weekday_cells));

        // Days grid
        let day_cells: Vec<Template> = props.days.iter().map(|d| {
            let bg = if d.selected { vars::PRIMARY } else if d.today { "color-mix(in srgb, var(--rye-primary) 12%, transparent)" } else { "transparent" };
            let color = if d.selected { vars::BG } else if !d.in_month { vars::BORDER_STRONG } else if d.disabled { vars::TEXT_SUBTLE } else { vars::TEXT };
            let cursor = if d.disabled { "not-allowed" } else { "pointer" };

            let style = format!(
                "text-align:center;font-size:var(--rye-font-size-md);padding:8px 0;border-radius:var(--rye-radius-md);\
                 background:{};color:{};cursor:{};",
                bg, color, cursor,
            );

            Template::new_element("div",
                vec![("style".to_string(), style),
                     ("class".to_string(), "rye-calendar-day".to_string())],
                Vec::new(), vec![Template::text(&d.day.to_string())])
        }).collect();

        children.push(Template::new_element("div",
            vec![("style".to_string(), "display:grid;grid-template-columns:repeat(7,1fr);gap:2px;".to_string()),
                 ("class".to_string(), "rye-calendar-grid".to_string())],
            Vec::new(), day_cells));

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-calendar {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_day_new() {
        let d = CalendarDay::new(15, true);
        assert_eq!(d.day, 15);
        assert!(d.in_month);
        assert!(!d.selected);
    }

    #[test]
    fn test_calendar_day_builder() {
        let d = CalendarDay::new(1, true).selected().today();
        assert!(d.selected);
        assert!(d.today);
    }

    #[test]
    fn test_calendar_props_builder() {
        let p = CalendarProps::default().year(2025).month(6).days(vec![
            CalendarDay::new(1, true),
            CalendarDay::new(15, true).selected(),
        ]);
        assert_eq!(p.year, 2025);
        assert_eq!(p.month, 6);
        assert_eq!(p.days.len(), 2);
    }

    #[test]
    fn test_calendar_render() {
        let days: Vec<CalendarDay> = (1..31).map(|d| CalendarDay::new(d, true)).collect();
        let el = Calendar::render(CalendarProps::default().year(2025).month(3).days(days));
        assert!(matches!(el, Element::Template(_)));
    }
}
