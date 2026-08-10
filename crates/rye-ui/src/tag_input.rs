//! TagInput — multi-value tag input (type + enter to add).

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct TagInputProps {
    pub tags: Vec<String>,
    pub placeholder: String,
    pub label: Option<String>,
    pub disabled: bool,
    pub max_tags: Option<usize>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TagInputProps {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            placeholder: "Add tag...".to_string(),
            label: None,
            disabled: false,
            max_tags: None,
            class: None,
            style: None,
        }
    }
}

impl TagInputProps {
    pub fn tags(mut self, t: Vec<String>) -> Self {
        self.tags = t;
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
    pub fn max_tags(mut self, m: usize) -> Self {
        self.max_tags = Some(m);
        self
    }
}

pub struct TagInput;

impl TagInput {
    pub fn render(props: TagInputProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let at_max = props
            .max_tags
            .map(|m| props.tags.len() >= m)
            .unwrap_or(false);
        let input_disabled = props.disabled || at_max;

        let container_style = format!(
            "display:flex;flex-wrap:wrap;gap:6px;padding:8px;min-height:40px;\
             border:1px solid {};border-radius:var(--rye-radius-md);background:{};cursor:{};\
             align-items:center;{}",
            if input_disabled {
                vars::BORDER
            } else {
                vars::INPUT_BORDER
            },
            if input_disabled {
                vars::BG_MUTED
            } else {
                vars::INPUT_BG
            },
            if input_disabled {
                "not-allowed"
            } else {
                "text"
            },
            props.style.as_deref().unwrap_or(""),
        );

        let mut container_children: Vec<Template> = props.tags.iter().map(|tag| {
            let tag_style = format!(
                "display:inline-flex;align-items:center;gap:4px;padding:4px 10px;\
                 font-size:var(--rye-font-size-sm);border-radius:9999px;background:{};color:{};",
                if input_disabled { vars::BG_MUTED } else { vars::PRIMARY },
                if input_disabled { vars::TEXT_SUBTLE } else { vars::PRIMARY_FG },
            );

            let mut tag_children = vec![Template::text(tag)];
            if !input_disabled {
                tag_children.push(Template::new_element("button",
                    vec![("style".to_string(), "border:none;background:none;color:inherit;cursor:pointer;font-size:var(--rye-font-size-md);padding:0;line-height:1;".to_string()),
                         ("aria-label".to_string(), format!("Remove {}", tag)),
                         ("class".to_string(), "rye-tag-input-remove".to_string())],
                    Vec::new(), vec![Template::text("×")]));
            }

            Template::new_element("span",
                vec![("style".to_string(), tag_style), ("class".to_string(), "rye-tag-input-tag".to_string())],
                Vec::new(), tag_children)
        }).collect();

        // Input field
        let input_style = "flex:1;min-width:80px;border:none;outline:none;font-size:var(--rye-font-size-md);background:transparent;font-family:var(--rye-font-family);";
        let mut input_attrs = vec![
            ("type".to_string(), "text".to_string()),
            ("style".to_string(), input_style.to_string()),
            ("placeholder".to_string(), props.placeholder.clone()),
            ("class".to_string(), "rye-tag-input-field".to_string()),
        ];
        if input_disabled {
            input_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        container_children.push(Template::new_element(
            "input",
            input_attrs,
            Vec::new(),
            Vec::new(),
        ));

        children.push(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style),
                (
                    "class".to_string(),
                    format!("rye-tag-input {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            container_children,
        ));

        if let Some(max) = props.max_tags {
            children.push(Template::new_element(
                "span",
                vec![(
                    "style".to_string(),
                    format!(
                        "font-size:var(--rye-font-size-sm);color:{};margin-top:4px;",
                        vars::TEXT_MUTED
                    ),
                )],
                Vec::new(),
                vec![Template::text(&format!("{}/{}", props.tags.len(), max))],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-tag-input-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_input_default() {
        let p = TagInputProps::default();
        assert!(p.tags.is_empty());
        assert!(!p.disabled);
    }

    #[test]
    fn test_tag_input_builder() {
        let p = TagInputProps::default()
            .tags(vec!["rust".into(), "ui".into()])
            .placeholder("Add skill...")
            .label("Skills")
            .max_tags(5);
        assert_eq!(p.tags.len(), 2);
        assert_eq!(p.max_tags, Some(5));
    }

    #[test]
    fn test_tag_input_render_empty() {
        let el = TagInput::render(TagInputProps::default().label("Tags"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_tag_input_render_with_tags() {
        let el = TagInput::render(TagInputProps::default().tags(vec![
            "a".into(),
            "b".into(),
            "c".into(),
        ]));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_tag_input_render_max() {
        let el = TagInput::render(
            TagInputProps::default()
                .tags(vec!["a".into(), "b".into()])
                .max_tags(2),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
