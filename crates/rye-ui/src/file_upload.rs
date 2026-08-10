//! FileUpload — drag-and-drop file input.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct FileUploadProps {
    pub label: Option<String>,
    pub accept: Option<String>,
    pub multiple: bool,
    pub disabled: bool,
    pub hint: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for FileUploadProps {
    fn default() -> Self {
        Self {
            label: None,
            accept: None,
            multiple: false,
            disabled: false,
            hint: None,
            class: None,
            style: None,
        }
    }
}

impl FileUploadProps {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn accept(mut self, a: impl Into<String>) -> Self {
        self.accept = Some(a.into());
        self
    }
    pub fn multiple(mut self, m: bool) -> Self {
        self.multiple = m;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn hint(mut self, h: impl Into<String>) -> Self {
        self.hint = Some(h.into());
        self
    }
}

pub struct FileUpload;

impl FileUpload {
    pub fn render(props: FileUploadProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let drop_style = format!(
            "border:2px dashed {};border-radius:var(--rye-radius-lg);padding:32px 16px;text-align:center;\
             cursor:{};opacity:{};background:{};transition:var(--rye-transition-normal);{}",
            if props.disabled { vars::INPUT_BORDER } else { vars::TEXT_SUBTLE },
            if props.disabled { "not-allowed" } else { "pointer" },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { vars::BG_MUTED } else { vars::BG_SUBTLE },
            props.style.as_deref().unwrap_or(""),
        );

        let icon_style = format!(
            "font-size:32px;color:{};margin-bottom:8px;",
            vars::TEXT_SUBTLE
        );
        let text_style = format!(
            "font-size:var(--rye-font-size-md);color:{};",
            vars::TEXT_MUTED
        );

        let mut drop_children = vec![
            Template::new_element(
                "div",
                vec![("style".to_string(), icon_style.to_string())],
                Vec::new(),
                vec![Template::text("📁")],
            ),
            Template::new_element(
                "div",
                vec![("style".to_string(), text_style.to_string())],
                Vec::new(),
                vec![Template::text(if props.disabled {
                    "File upload disabled"
                } else if props.multiple {
                    "Click or drag files to upload"
                } else {
                    "Click or drag a file to upload"
                })],
            ),
        ];

        if let Some(hint) = &props.hint {
            drop_children.push(Template::new_element(
                "div",
                vec![(
                    "style".to_string(),
                    format!(
                        "font-size:var(--rye-font-size-sm);color:{};margin-top:4px;",
                        vars::TEXT_SUBTLE
                    ),
                )],
                Vec::new(),
                vec![Template::text(hint)],
            ));
        }

        // Hidden file input
        let mut input_attrs = vec![
            ("type".to_string(), "file".to_string()),
            (
                "style".to_string(),
                "position:absolute;opacity:0;width:0;height:0;".to_string(),
            ),
            (
                "class".to_string(),
                format!(
                    "rye-file-upload-input {}",
                    props.class.as_deref().unwrap_or("")
                ),
            ),
        ];
        if let Some(accept) = &props.accept {
            input_attrs.push(("accept".to_string(), accept.clone()));
        }
        if props.multiple {
            input_attrs.push(("multiple".to_string(), "true".to_string()));
        }
        if props.disabled {
            input_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        drop_children.push(Template::new_element(
            "input",
            input_attrs,
            Vec::new(),
            Vec::new(),
        ));

        children.push(Template::new_element(
            "div",
            vec![
                ("style".to_string(), drop_style),
                ("class".to_string(), "rye-file-upload".to_string()),
            ],
            Vec::new(),
            drop_children,
        ));

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-file-upload-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_upload_default() {
        let p = FileUploadProps::default();
        assert!(!p.multiple);
        assert!(!p.disabled);
    }

    #[test]
    fn test_file_upload_builder() {
        let p = FileUploadProps::default()
            .label("Upload CV")
            .accept(".pdf,.doc,.docx")
            .multiple(true)
            .hint("Max 5MB per file");
        assert_eq!(p.accept.as_deref(), Some(".pdf,.doc,.docx"));
        assert!(p.multiple);
        assert_eq!(p.hint.as_deref(), Some("Max 5MB per file"));
    }

    #[test]
    fn test_file_upload_render() {
        let el = FileUpload::render(FileUploadProps::default().label("Photo").accept("image/*"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_file_upload_render_disabled() {
        let el = FileUpload::render(FileUploadProps::default().disabled(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
