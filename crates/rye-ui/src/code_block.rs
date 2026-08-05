//! CodeBlock — syntax-highlighted code display.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Json,
    Html,
    Css,
    Bash,
    Plain,
}

impl CodeLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust", Self::TypeScript => "typescript", Self::JavaScript => "javascript",
            Self::Python => "python", Self::Go => "go", Self::Json => "json",
            Self::Html => "html", Self::Css => "css", Self::Bash => "bash", Self::Plain => "text",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlockProps {
    pub code: String,
    pub language: CodeLanguage,
    pub show_line_numbers: bool,
    pub show_copy_button: bool,
    pub title: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CodeBlockProps {
    fn default() -> Self {
        Self { code: String::new(), language: CodeLanguage::Plain,
               show_line_numbers: true, show_copy_button: true,
               title: None, class: None, style: None }
    }
}

impl CodeBlockProps {
    pub fn code(mut self, c: impl Into<String>) -> Self { self.code = c.into(); self }
    pub fn language(mut self, l: CodeLanguage) -> Self { self.language = l; self }
    pub fn line_numbers(mut self, s: bool) -> Self { self.show_line_numbers = s; self }
    pub fn copy_button(mut self, s: bool) -> Self { self.show_copy_button = s; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
}

pub struct CodeBlock;

impl CodeBlock {
    pub fn render(props: CodeBlockProps) -> Element {
        let container_style = format!(
            "background:{};border-radius:var(--rye-radius-lg);overflow:hidden;font-family:var(--rye-font-mono);{}",
            vars::CODE_BG, props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        // Header bar
        let mut header_children = Vec::new();
        header_children.push(Template::new_element("span",
            vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};text-transform:uppercase;font-weight:var(--rye-font-weight-semibold);", vars::TEXT_SUBTLE))],
            Vec::new(), vec![Template::text(props.language.as_str())]));

        if let Some(title) = &props.title {
            header_children.push(Template::new_element("span",
                vec![("style".to_string(), format!("font-size:13px;color:{};margin-left:12px;", vars::BORDER_STRONG))],
                Vec::new(), vec![Template::text(title)]));
        }

        if props.show_copy_button {
            header_children.push(Template::new_element("button",
                vec![("style".to_string(), format!("margin-left:auto;padding:4px 10px;border:1px solid {};border-radius:var(--rye-radius-sm);background:transparent;color:{};cursor:pointer;font-size:var(--rye-font-size-sm);", vars::CODE_BORDER, vars::TEXT_SUBTLE)),
                     ("class".to_string(), "rye-code-block-copy".to_string())],
                Vec::new(), vec![Template::text("Copy")]));
        }

        children.push(Template::new_element("div",
            vec![("style".to_string(), format!("display:flex;align-items:center;padding:10px 16px;background:{};border-bottom:1px solid {};", vars::CODE_HEADER_BG, vars::CODE_BORDER)),
                 ("class".to_string(), "rye-code-block-header".to_string())],
            Vec::new(), header_children));

        // Code content
        let lines: Vec<&str> = props.code.lines().collect();
        let mut code_children = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let mut line_children = Vec::new();

            if props.show_line_numbers {
                line_children.push(Template::new_element("span",
                    vec![("style".to_string(), format!("color:{};user-select:none;display:inline-block;width:32px;text-align:right;margin-right:16px;", vars::CODE_LINE_NUMBER))],
                    Vec::new(), vec![Template::text(&(i + 1).to_string())]));
            }

            line_children.push(Template::new_element("span",
                vec![("style".to_string(), format!("color:{};", vars::CODE_TEXT))],
                Vec::new(), vec![Template::text(if line.is_empty() { " " } else { line })]));

            code_children.push(Template::new_element("div",
                vec![("style".to_string(), "display:flex;min-height:20px;".to_string())],
                Vec::new(), line_children));
        }

        if code_children.is_empty() {
            code_children.push(Template::new_element("div",
                vec![("style".to_string(), format!("color:{};", vars::CODE_LINE_NUMBER))],
                Vec::new(), vec![Template::text(" ")]));
        }

        children.push(Template::new_element("div",
            vec![("style".to_string(), "padding:16px;overflow-x:auto;font-size:13px;line-height:1.6;".to_string()),
                 ("class".to_string(), "rye-code-block-content".to_string())],
            Vec::new(), code_children));

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), container_style),
                 ("class".to_string(), format!("rye-code-block {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_language_as_str() {
        assert_eq!(CodeLanguage::Rust.as_str(), "rust");
        assert_eq!(CodeLanguage::Python.as_str(), "python");
        assert_eq!(CodeLanguage::Json.as_str(), "json");
    }

    #[test]
    fn test_code_block_default() {
        let p = CodeBlockProps::default();
        assert_eq!(p.language, CodeLanguage::Plain);
        assert!(p.show_line_numbers);
        assert!(p.show_copy_button);
    }

    #[test]
    fn test_code_block_builder() {
        let p = CodeBlockProps::default()
            .code("fn main() {}")
            .language(CodeLanguage::Rust)
            .title("main.rs")
            .line_numbers(false);
        assert_eq!(p.code, "fn main() {}");
        assert_eq!(p.language, CodeLanguage::Rust);
        assert!(!p.show_line_numbers);
    }

    #[test]
    fn test_code_block_render() {
        let el = CodeBlock::render(CodeBlockProps::default()
            .code("let x = 42;\nprintln!(\"{}\", x);")
            .language(CodeLanguage::Rust)
            .title("example.rs"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_code_block_render_empty() {
        let el = CodeBlock::render(CodeBlockProps::default().language(CodeLanguage::Bash));
        assert!(matches!(el, Element::Template(_)));
    }
}
