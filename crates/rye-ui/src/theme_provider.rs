//! ThemeProvider — injects CSS custom properties into the DOM.
//!
//! The `ThemeProvider` renders a `<style>` tag containing all design tokens
//! as CSS custom properties, plus an optional `data-theme` attribute on a
//! wrapper element. This enables:
//!
//! - **Runtime theme switching** — change `data-theme` on the wrapper, all
//!   components update instantly via CSS variables (no re-render needed).
//! - **System preference detection** — `prefers-color-scheme: dark` media
//!   query is included by default.
//! - **Custom themes** — pass a custom [`DesignTokens`] to override any token.
//! - **SSR-safe** — the `<style>` tag is static HTML, no JavaScript required.
//!
//! ## Usage
//!
//! ```ignore
//! use rye_ui::{ThemeProvider, ThemeProviderProps, DesignTokens};
//!
//! // Light theme (default)
//! let provider = ThemeProvider::render(ThemeProviderProps::default());
//!
//! // Dark theme
//! let provider = ThemeProvider::render(ThemeProviderProps::dark());
//!
//! // Custom theme with both light + dark (auto-switching)
//! let provider = ThemeProvider::render(ThemeProviderProps::auto());
//!
//! // Custom tokens
//! let provider = ThemeProvider::render(ThemeProviderProps::light()
//!     .tokens(DesignTokens::light().primary("#7c3aed")));
//! ```

use crate::tokens::DesignTokens;
use rye_core::template::Template;
use rye_core::Element;

/// Which theme mode to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Always light.
    Light,
    /// Always dark.
    Dark,
    /// Include both light and dark CSS; switch via `prefers-color-scheme`.
    Auto,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Light
    }
}

/// Props for the [`ThemeProvider`] component.
#[derive(Debug, Clone)]
pub struct ThemeProviderProps {
    /// Which theme mode to render.
    pub mode: ThemeMode,
    /// Custom design tokens. If `None`, uses the default for the mode.
    pub tokens: Option<DesignTokens>,
    /// Custom dark tokens for `Auto` mode. If `None`, uses `DesignTokens::dark()`.
    pub dark_tokens: Option<DesignTokens>,
    /// Whether to set `data-theme` attribute on the wrapper div.
    pub set_data_attr: bool,
    /// Additional class on the wrapper div.
    pub class: Option<String>,
    /// Additional inline style on the wrapper div.
    pub style: Option<String>,
}

impl Default for ThemeProviderProps {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
            tokens: None,
            dark_tokens: None,
            set_data_attr: true,
            class: None,
            style: None,
        }
    }
}

impl ThemeProviderProps {
    /// Light theme.
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            ..Default::default()
        }
    }
    /// Dark theme.
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            ..Default::default()
        }
    }
    /// Auto-switching (light + dark via media query).
    pub fn auto() -> Self {
        Self {
            mode: ThemeMode::Auto,
            ..Default::default()
        }
    }

    pub fn mode(mut self, m: ThemeMode) -> Self {
        self.mode = m;
        self
    }
    pub fn tokens(mut self, t: DesignTokens) -> Self {
        self.tokens = Some(t);
        self
    }
    pub fn dark_tokens(mut self, t: DesignTokens) -> Self {
        self.dark_tokens = Some(t);
        self
    }
    pub fn class(mut self, c: impl Into<String>) -> Self {
        self.class = Some(c.into());
        self
    }
}

/// ThemeProvider component — injects CSS variables and wraps children.
pub struct ThemeProvider;

impl ThemeProvider {
    pub fn render(props: ThemeProviderProps) -> Element {
        let css = generate_css(&props);

        // <style> tag with all CSS variables
        let style_tag = Template::new_element(
            "style",
            vec![("id".to_string(), "rye-theme".to_string())],
            Vec::new(),
            vec![Template::text(&css)],
        );

        // Wrapper div with data-theme attribute
        let (data_attr, wrapper_style) = match props.mode {
            ThemeMode::Light => (
                Some(("data-theme".to_string(), "light".to_string())),
                format!("font-family:var(--rye-font-family);color:var(--rye-text);background:var(--rye-bg);{}", props.style.as_deref().unwrap_or("")),
            ),
            ThemeMode::Dark => (
                Some(("data-theme".to_string(), "dark".to_string())),
                format!("font-family:var(--rye-font-family);color:var(--rye-text);background:var(--rye-bg);{}", props.style.as_deref().unwrap_or("")),
            ),
            ThemeMode::Auto => (
                None, // Auto mode relies on media query, no data-theme needed
                format!("font-family:var(--rye-font-family);color:var(--rye-text);background:var(--rye-bg);{}", props.style.as_deref().unwrap_or("")),
            ),
        };

        let mut wrapper_attrs = vec![
            ("style".to_string(), wrapper_style),
            (
                "class".to_string(),
                format!(
                    "rye-theme-provider {}",
                    props.class.as_deref().unwrap_or("")
                ),
            ),
        ];
        if let Some((k, v)) = data_attr {
            wrapper_attrs.push((k, v));
        }

        let wrapper = Template::new_element("div", wrapper_attrs, Vec::new(), Vec::new());

        Element::Template(Template::new_element(
            "div",
            Vec::new(),
            Vec::new(),
            vec![style_tag, wrapper],
        ))
    }

    /// Generate only the CSS string (useful for SSR or head injection).
    pub fn css_only(props: &ThemeProviderProps) -> String {
        generate_css(props)
    }
}

fn generate_css(props: &ThemeProviderProps) -> String {
    match props.mode {
        ThemeMode::Light => {
            let tokens = props.tokens.clone().unwrap_or_else(DesignTokens::light);
            tokens.to_css()
        }
        ThemeMode::Dark => {
            let tokens = props.tokens.clone().unwrap_or_else(DesignTokens::dark);
            tokens.to_css()
        }
        ThemeMode::Auto => {
            let light = props.tokens.clone().unwrap_or_else(DesignTokens::light);
            let dark = props.dark_tokens.clone().unwrap_or_else(DesignTokens::dark);

            // :root gets light, then @media prefers-color-scheme: dark overrides
            let mut css = light.to_css();
            css.push_str("\n@media (prefers-color-scheme: dark) {\n");
            css.push_str("  :root {\n");
            write_dark_overrides(&mut css, &dark);
            css.push_str("  }\n");
            css.push_str("}\n");

            // Also support [data-theme="dark"] manual override
            css.push_str("\n[data-theme=\"dark\"] {\n");
            write_dark_overrides(&mut css, &dark);
            css.push_str("}\n");

            css
        }
    }
}

fn write_dark_overrides(css: &mut String, dark: &DesignTokens) {
    use std::fmt::Write;
    let c = &dark.colors;
    let _ = writeln!(css, "    --rye-primary: {};", c.primary);
    let _ = writeln!(css, "    --rye-primary-hover: {};", c.primary_hover);
    let _ = writeln!(css, "    --rye-primary-active: {};", c.primary_active);
    let _ = writeln!(css, "    --rye-secondary: {};", c.secondary);
    let _ = writeln!(css, "    --rye-secondary-hover: {};", c.secondary_hover);
    let _ = writeln!(css, "    --rye-success: {};", c.success);
    let _ = writeln!(css, "    --rye-warning: {};", c.warning);
    let _ = writeln!(css, "    --rye-danger: {};", c.danger);
    let _ = writeln!(css, "    --rye-info: {};", c.info);
    let _ = writeln!(css, "    --rye-bg: {};", c.bg);
    let _ = writeln!(css, "    --rye-bg-subtle: {};", c.bg_subtle);
    let _ = writeln!(css, "    --rye-bg-muted: {};", c.bg_muted);
    let _ = writeln!(css, "    --rye-bg-elevated: {};", c.bg_elevated);
    let _ = writeln!(css, "    --rye-text: {};", c.text);
    let _ = writeln!(css, "    --rye-text-muted: {};", c.text_muted);
    let _ = writeln!(css, "    --rye-text-subtle: {};", c.text_subtle);
    let _ = writeln!(css, "    --rye-border: {};", c.border);
    let _ = writeln!(css, "    --rye-border-strong: {};", c.border_strong);
    let _ = writeln!(css, "    --rye-border-subtle: {};", c.border_subtle);
    let _ = writeln!(css, "    --rye-input-bg: {};", c.input_bg);
    let _ = writeln!(css, "    --rye-input-border: {};", c.input_border);
    let _ = writeln!(
        css,
        "    --rye-input-border-focus: {};",
        c.input_border_focus
    );
    let _ = writeln!(css, "    --rye-overlay: {};", c.overlay);
    let _ = writeln!(css, "    --rye-ring: {};", c.ring);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default() {
        assert_eq!(ThemeMode::default(), ThemeMode::Light);
    }

    #[test]
    fn test_props_light() {
        let p = ThemeProviderProps::light();
        assert_eq!(p.mode, ThemeMode::Light);
    }

    #[test]
    fn test_props_dark() {
        let p = ThemeProviderProps::dark();
        assert_eq!(p.mode, ThemeMode::Dark);
    }

    #[test]
    fn test_props_auto() {
        let p = ThemeProviderProps::auto();
        assert_eq!(p.mode, ThemeMode::Auto);
    }

    #[test]
    fn test_props_custom_tokens() {
        let p = ThemeProviderProps::light().tokens(DesignTokens::light().primary("#ff0000"));
        assert_eq!(p.tokens.as_ref().unwrap().colors.primary, "#ff0000");
    }

    #[test]
    fn test_render_light() {
        let el = ThemeProvider::render(ThemeProviderProps::light());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_render_dark() {
        let el = ThemeProvider::render(ThemeProviderProps::dark());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_render_auto() {
        let el = ThemeProvider::render(ThemeProviderProps::auto());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_css_only_light() {
        let css = ThemeProvider::css_only(&ThemeProviderProps::light());
        assert!(css.contains(":root"));
        assert!(css.contains("--rye-primary: #2563eb;"));
    }

    #[test]
    fn test_css_only_dark() {
        let css = ThemeProvider::css_only(&ThemeProviderProps::dark());
        assert!(css.contains("--rye-bg: #0f172a;"));
    }

    #[test]
    fn test_css_only_auto_has_media_query() {
        let css = ThemeProvider::css_only(&ThemeProviderProps::auto());
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        assert!(css.contains("[data-theme=\"dark\"]"));
    }

    #[test]
    fn test_css_only_auto_has_both_themes() {
        let css = ThemeProvider::css_only(&ThemeProviderProps::auto());
        // Light values in :root
        assert!(css.contains("--rye-bg: #ffffff;"));
        // Dark values in media query
        assert!(css.contains("--rye-bg: #0f172a;"));
    }

    #[test]
    fn test_css_only_custom_tokens() {
        let p = ThemeProviderProps::light().tokens(DesignTokens::light().primary("#7c3aed"));
        let css = ThemeProvider::css_only(&p);
        assert!(css.contains("--rye-primary: #7c3aed;"));
    }
}
