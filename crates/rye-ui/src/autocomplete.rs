//! AutoComplete / Combobox — searchable select with free text.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct AutoCompleteOption {
    pub value: String,
    pub label: String,
}

impl AutoCompleteOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutoCompleteProps {
    pub options: Vec<AutoCompleteOption>,
    pub value: String,
    pub query: String,
    pub open: bool,
    pub placeholder: String,
    pub label: Option<String>,
    pub disabled: bool,
    pub allow_free_text: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for AutoCompleteProps {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            value: String::new(),
            query: String::new(),
            open: false,
            placeholder: "Search...".to_string(),
            label: None,
            disabled: false,
            allow_free_text: false,
            class: None,
            style: None,
        }
    }
}

impl AutoCompleteProps {
    pub fn options(mut self, o: Vec<AutoCompleteOption>) -> Self {
        self.options = o;
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.query = q.into();
        self
    }
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn allow_free_text(mut self, a: bool) -> Self {
        self.allow_free_text = a;
        self
    }
}

pub struct AutoComplete;

impl AutoComplete {
    pub fn render(props: AutoCompleteProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let container_style = "position:relative;";

        let input_style = format!(
            "width:100%;padding:8px 16px;font-size:var(--rye-font-size-md);border:1px solid {};\
             border-radius:var(--rye-radius-md);background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);\
             box-sizing:border-box;{}",
            vars::INPUT_BORDER,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "text" },
            props.style.as_deref().unwrap_or(""),
        );

        let display_value = if props.open {
            props.query.clone()
        } else {
            props.value.clone()
        };

        let mut input_attrs = vec![
            ("type".to_string(), "text".to_string()),
            ("style".to_string(), input_style),
            ("placeholder".to_string(), props.placeholder.clone()),
            (
                "class".to_string(),
                format!(
                    "rye-autocomplete-input {}",
                    props.class.as_deref().unwrap_or("")
                ),
            ),
        ];
        if !display_value.is_empty() {
            input_attrs.push(("value".to_string(), display_value));
        }
        if props.disabled {
            input_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut container_children = vec![Template::new_element(
            "input",
            input_attrs,
            Vec::new(),
            Vec::new(),
        )];

        // Dropdown
        if props.open && !props.disabled {
            let filtered: Vec<&AutoCompleteOption> = if props.query.is_empty() {
                props.options.iter().collect()
            } else {
                props
                    .options
                    .iter()
                    .filter(|o| o.label.to_lowercase().contains(&props.query.to_lowercase()))
                    .collect()
            };

            let dropdown_style = format!("position:absolute;top:100%;left:0;right:0;margin-top:4px;max-height:240px;overflow-y:auto;background:{};border:1px solid {};border-radius:var(--rye-radius-md);box-shadow:{};z-index:{};padding:4px;", vars::BG_ELEVATED, vars::BORDER, vars::SHADOW_MD, vars::Z_DROPDOWN);

            let items: Vec<Template> = if filtered.is_empty() {
                if props.allow_free_text && !props.query.is_empty() {
                    vec![Template::new_element("div",
                        vec![("style".to_string(), format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;border-radius:var(--rye-radius-sm);", vars::TEXT_MUTED))],
                        Vec::new(), vec![Template::text(&format!("Press Enter to add \"{}\"", props.query))])]
                } else {
                    vec![Template::new_element(
                        "div",
                        vec![(
                            "style".to_string(),
                            format!(
                                "padding:8px 12px;font-size:var(--rye-font-size-md);color:{};",
                                vars::TEXT_SUBTLE
                            ),
                        )],
                        Vec::new(),
                        vec![Template::text("No results found")],
                    )]
                }
            } else {
                filtered.iter().map(|opt| {
                    let is_selected = opt.label == props.value;
                    let style = if is_selected {
                        format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;border-radius:var(--rye-radius-sm);background:color-mix(in srgb, {} 12%, transparent);", vars::TEXT, vars::PRIMARY)
                    } else {
                        format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;border-radius:var(--rye-radius-sm);", vars::TEXT)
                    };
                    Template::new_element("div",
                        vec![("style".to_string(), style.to_string()),
                             ("class".to_string(), "rye-autocomplete-option".to_string()),
                             ("data-value".to_string(), opt.value.clone())],
                        Vec::new(), vec![Template::text(&opt.label)])
                }).collect()
            };

            container_children.push(Template::new_element(
                "div",
                vec![
                    ("style".to_string(), dropdown_style.to_string()),
                    ("class".to_string(), "rye-autocomplete-dropdown".to_string()),
                ],
                Vec::new(),
                items,
            ));
        }

        children.push(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style.to_string()),
                ("class".to_string(), "rye-autocomplete".to_string()),
            ],
            Vec::new(),
            container_children,
        ));

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-autocomplete-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_option_new() {
        let o = AutoCompleteOption::new("us", "United States");
        assert_eq!(o.value, "us");
        assert_eq!(o.label, "United States");
    }

    #[test]
    fn test_autocomplete_default() {
        let p = AutoCompleteProps::default();
        assert!(!p.open);
        assert!(!p.allow_free_text);
    }

    #[test]
    fn test_autocomplete_builder() {
        let p = AutoCompleteProps::default()
            .options(vec![AutoCompleteOption::new("fr", "France")])
            .value("France")
            .open(true)
            .allow_free_text(true)
            .label("Country");
        assert_eq!(p.options.len(), 1);
        assert!(p.allow_free_text);
    }

    #[test]
    fn test_autocomplete_render_closed() {
        let el = AutoComplete::render(
            AutoCompleteProps::default()
                .options(vec![AutoCompleteOption::new("a", "Apple")])
                .value("Apple"),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_autocomplete_render_open() {
        let el = AutoComplete::render(
            AutoCompleteProps::default()
                .options(vec![
                    AutoCompleteOption::new("apple", "Apple"),
                    AutoCompleteOption::new("banana", "Banana"),
                ])
                .query("ap")
                .open(true),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_autocomplete_render_no_results() {
        let el = AutoComplete::render(
            AutoCompleteProps::default()
                .options(vec![AutoCompleteOption::new("a", "Apple")])
                .query("xyz")
                .open(true),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
