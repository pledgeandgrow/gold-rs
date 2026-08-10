//! Design tokens — CSS custom properties for production-grade theming.
//!
//! This module defines the complete set of design tokens used across all
//! `rye-ui` components. Tokens are exposed as CSS custom properties (variables)
//! which enables runtime theme switching without re-rendering components.
//!
//! ## Token categories
//!
//! | Category | Examples |
//! |---|---|
//! | **Colors** | `--rye-primary`, `--rye-bg`, `--rye-text`, `--rye-border` |
//! | **Typography** | `--rye-font-family`, `--rye-font-size-sm`, `--rye-line-height` |
//! | **Spacing** | `--rye-space-1` through `--rye-space-20` |
//! | **Border** | `--rye-radius-sm`, `--rye-radius-md`, `--rye-radius-lg` |
//! | **Shadow** | `--rye-shadow-sm`, `--rye-shadow-md`, `--rye-shadow-lg` |
//! | **Z-index** | `--rye-z-dropdown`, `--rye-z-modal`, `--rye-z-toast` |
//! | **Transition** | `--rye-transition-fast`, `--rye-transition-normal` |
//!
//! ## Usage
//!
//! ### Generating CSS
//!
//! ```ignore
//! use rye_ui::DesignTokens;
//!
//! // Use the default light theme
//! let tokens = DesignTokens::light();
//!
//! // Use the dark theme
//! let tokens = DesignTokens::dark();
//!
//! // Custom theme — override specific tokens
//! let tokens = DesignTokens::light()
//!     .primary("#7c3aed")
//!     .radius("12px");
//!
//! // Render the CSS variables into a <style> tag
//! let css = tokens.to_css();
//! ```
//!
//! ### Referencing variables in components
//!
//! Use the [`vars`] module constants or the [`v`]/[`vf`] helpers to reference
//! CSS variables in inline style strings:
//!
//! ```ignore
//! use rye_ui::vars;
//!
//! // Direct constant references
//! let style = format!("color:{};background:{};", vars::TEXT, vars::BG);
//!
//! // Helper functions
//! use rye_ui::v;
//! let style = format!("border:1px solid {};", v("border"));
//! ```
//!
//! ## Available `vars` constants
//!
//! | Group | Constants |
//! |---|---|
//! | **Colors** | `PRIMARY`, `PRIMARY_HOVER`, `PRIMARY_FG`, `SECONDARY`, `SECONDARY_HOVER`, `SECONDARY_FG`, `SUCCESS`, `SUCCESS_FG`, `WARNING`, `WARNING_FG`, `DANGER`, `DANGER_FG`, `DANGER_BG`, `INFO`, `INFO_FG` |
//! | **Backgrounds** | `BG`, `BG_SUBTLE`, `BG_MUTED`, `BG_ELEVATED` |
//! | **Text** | `TEXT`, `TEXT_MUTED`, `TEXT_SUBTLE` |
//! | **Borders** | `BORDER`, `BORDER_STRONG`, `BORDER_SUBTLE` |
//! | **Inputs** | `INPUT_BG`, `INPUT_BORDER`, `INPUT_BORDER_FOCUS` |
//! | **Overlay & Ring** | `OVERLAY`, `RING` |
//! | **Code** | `CODE_BG`, `CODE_HEADER_BG`, `CODE_BORDER`, `CODE_TEXT`, `CODE_LINE_NUMBER` |
//! | **Typography** | `FONT_FAMILY`, `FONT_FAMILY_MONO`, `FONT_SIZE_XS`–`FONT_SIZE_XL`, `FONT_WEIGHT_NORMAL`–`FONT_WEIGHT_BOLD`, `LINE_HEIGHT` |
//! | **Spacing** | `SPACE_1`–`SPACE_8` |
//! | **Border radius** | `RADIUS_SM`, `RADIUS_MD`, `RADIUS_LG`, `RADIUS_XL`, `RADIUS_FULL` |
//! | **Shadows** | `SHADOW_SM`, `SHADOW_MD`, `SHADOW_LG`, `SHADOW_XL` |
//! | **Z-index** | `Z_DROPDOWN`, `Z_OVERLAY`, `Z_MODAL`, `Z_TOAST`, `Z_TOOLTIP` |
//! | **Transitions** | `TRANSITION_FAST`, `TRANSITION_NORMAL`, `TRANSITION_SLOW` |

use std::fmt::Write;

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

/// Semantic color roles used throughout the component library.
///
/// Each role maps to a CSS custom property (`--rye-*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorTokens {
    pub primary: String,
    pub primary_hover: String,
    pub primary_active: String,
    pub primary_fg: String,

    pub secondary: String,
    pub secondary_hover: String,
    pub secondary_fg: String,

    pub success: String,
    pub success_fg: String,
    pub warning: String,
    pub warning_fg: String,
    pub danger: String,
    pub danger_fg: String,
    pub danger_bg: String,
    pub info: String,
    pub info_fg: String,

    pub bg: String,
    pub bg_subtle: String,
    pub bg_muted: String,
    pub bg_elevated: String,

    pub text: String,
    pub text_muted: String,
    pub text_subtle: String,

    pub border: String,
    pub border_strong: String,
    pub border_subtle: String,

    pub input_bg: String,
    pub input_border: String,
    pub input_border_focus: String,

    pub overlay: String,
    pub ring: String,

    pub code_bg: String,
    pub code_header_bg: String,
    pub code_border: String,
    pub code_text: String,
    pub code_line_number: String,
}

impl ColorTokens {
    /// Light color palette.
    pub fn light() -> Self {
        Self {
            primary: "#2563eb".into(),
            primary_hover: "#1d4ed8".into(),
            primary_active: "#1e40af".into(),
            primary_fg: "#ffffff".into(),

            secondary: "#64748b".into(),
            secondary_hover: "#475569".into(),
            secondary_fg: "#ffffff".into(),

            success: "#16a34a".into(),
            success_fg: "#ffffff".into(),
            warning: "#d97706".into(),
            warning_fg: "#ffffff".into(),
            danger: "#dc2626".into(),
            danger_fg: "#ffffff".into(),
            danger_bg: "#fef2f2".into(),
            info: "#0891b2".into(),
            info_fg: "#ffffff".into(),

            bg: "#ffffff".into(),
            bg_subtle: "#f8fafc".into(),
            bg_muted: "#f1f5f9".into(),
            bg_elevated: "#ffffff".into(),

            text: "#1e293b".into(),
            text_muted: "#64748b".into(),
            text_subtle: "#94a3b8".into(),

            border: "#e2e8f0".into(),
            border_strong: "#cbd5e1".into(),
            border_subtle: "#f1f5f9".into(),

            input_bg: "#ffffff".into(),
            input_border: "#cbd5e1".into(),
            input_border_focus: "#2563eb".into(),

            overlay: "rgba(0,0,0,0.5)".into(),
            ring: "rgba(37,99,235,0.4)".into(),

            code_bg: "#0f172a".into(),
            code_header_bg: "#1e293b".into(),
            code_border: "#334155".into(),
            code_text: "#e2e8f0".into(),
            code_line_number: "#475569".into(),
        }
    }

    /// Dark color palette.
    pub fn dark() -> Self {
        Self {
            primary: "#3b82f6".into(),
            primary_hover: "#60a5fa".into(),
            primary_active: "#2563eb".into(),
            primary_fg: "#ffffff".into(),

            secondary: "#94a3b8".into(),
            secondary_hover: "#cbd5e1".into(),
            secondary_fg: "#0f172a".into(),

            success: "#22c55e".into(),
            success_fg: "#052e16".into(),
            warning: "#fbbf24".into(),
            warning_fg: "#451a03".into(),
            danger: "#ef4444".into(),
            danger_fg: "#ffffff".into(),
            danger_bg: "#450a0a".into(),
            info: "#06b6d4".into(),
            info_fg: "#083344".into(),

            bg: "#0f172a".into(),
            bg_subtle: "#1e293b".into(),
            bg_muted: "#334155".into(),
            bg_elevated: "#1e293b".into(),

            text: "#f1f5f9".into(),
            text_muted: "#94a3b8".into(),
            text_subtle: "#64748b".into(),

            border: "#334155".into(),
            border_strong: "#475569".into(),
            border_subtle: "#1e293b".into(),

            input_bg: "#1e293b".into(),
            input_border: "#475569".into(),
            input_border_focus: "#3b82f6".into(),

            overlay: "rgba(0,0,0,0.7)".into(),
            ring: "rgba(59,130,246,0.4)".into(),

            code_bg: "#0d1117".into(),
            code_header_bg: "#161b22".into(),
            code_border: "#30363d".into(),
            code_text: "#c9d1d9".into(),
            code_line_number: "#6e7681".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypographyTokens {
    pub font_family: String,
    pub font_family_mono: String,
    pub font_size_xs: String,
    pub font_size_sm: String,
    pub font_size_md: String,
    pub font_size_lg: String,
    pub font_size_xl: String,
    pub font_size_2xl: String,
    pub font_weight_normal: String,
    pub font_weight_medium: String,
    pub font_weight_semibold: String,
    pub font_weight_bold: String,
    pub line_height: String,
    pub line_height_tight: String,
}

impl TypographyTokens {
    pub fn default_tokens() -> Self {
        Self {
            font_family: "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif".into(),
            font_family_mono: "'Fira Code', 'JetBrains Mono', monospace".into(),
            font_size_xs: "11px".into(),
            font_size_sm: "12px".into(),
            font_size_md: "14px".into(),
            font_size_lg: "16px".into(),
            font_size_xl: "18px".into(),
            font_size_2xl: "24px".into(),
            font_weight_normal: "400".into(),
            font_weight_medium: "500".into(),
            font_weight_semibold: "600".into(),
            font_weight_bold: "700".into(),
            line_height: "1.5".into(),
            line_height_tight: "1.25".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Spacing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacingTokens {
    pub space_0: String,
    pub space_1: String,
    pub space_2: String,
    pub space_3: String,
    pub space_4: String,
    pub space_5: String,
    pub space_6: String,
    pub space_8: String,
    pub space_10: String,
    pub space_12: String,
    pub space_16: String,
    pub space_20: String,
}

impl SpacingTokens {
    pub fn default_tokens() -> Self {
        Self {
            space_0: "0".into(),
            space_1: "4px".into(),
            space_2: "8px".into(),
            space_3: "12px".into(),
            space_4: "16px".into(),
            space_5: "20px".into(),
            space_6: "24px".into(),
            space_8: "32px".into(),
            space_10: "40px".into(),
            space_12: "48px".into(),
            space_16: "64px".into(),
            space_20: "80px".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Border, shadow, z-index, transition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorderTokens {
    pub radius_none: String,
    pub radius_sm: String,
    pub radius_md: String,
    pub radius_lg: String,
    pub radius_xl: String,
    pub radius_full: String,
    pub width_thin: String,
    pub width_thick: String,
}

impl BorderTokens {
    pub fn default_tokens() -> Self {
        Self {
            radius_none: "0".into(),
            radius_sm: "4px".into(),
            radius_md: "6px".into(),
            radius_lg: "8px".into(),
            radius_xl: "12px".into(),
            radius_full: "9999px".into(),
            width_thin: "1px".into(),
            width_thick: "2px".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowTokens {
    pub none: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
}

impl ShadowTokens {
    pub fn default_tokens() -> Self {
        Self {
            none: "none".into(),
            sm: "0 1px 2px rgba(0,0,0,0.05)".into(),
            md: "0 4px 6px rgba(0,0,0,0.1), 0 2px 4px rgba(0,0,0,0.06)".into(),
            lg: "0 10px 15px rgba(0,0,0,0.1), 0 4px 6px rgba(0,0,0,0.05)".into(),
            xl: "0 20px 25px -5px rgba(0,0,0,0.1), 0 10px 10px -5px rgba(0,0,0,0.04)".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZIndexTokens {
    pub base: String,
    pub dropdown: String,
    pub sticky: String,
    pub overlay: String,
    pub modal: String,
    pub toast: String,
    pub tooltip: String,
}

impl ZIndexTokens {
    pub fn default_tokens() -> Self {
        Self {
            base: "0".into(),
            dropdown: "1000".into(),
            sticky: "1100".into(),
            overlay: "1100".into(),
            modal: "1200".into(),
            toast: "1300".into(),
            tooltip: "1400".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionTokens {
    pub fast: String,
    pub normal: String,
    pub slow: String,
}

impl TransitionTokens {
    pub fn default_tokens() -> Self {
        Self {
            fast: "150ms ease".into(),
            normal: "200ms ease".into(),
            slow: "300ms ease".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// DesignTokens — the full bundle
// ---------------------------------------------------------------------------

/// Complete set of design tokens for the component library.
///
/// Contains all CSS custom properties that components reference. A
/// [`ThemeProvider`](crate::ThemeProvider) renders these as a `<style>` tag
/// with `:root` (or `[data-theme]`) selector, enabling runtime theme switching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTokens {
    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub border: BorderTokens,
    pub shadow: ShadowTokens,
    pub z_index: ZIndexTokens,
    pub transition: TransitionTokens,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self::light()
    }
}

impl DesignTokens {
    /// Light theme tokens.
    pub fn light() -> Self {
        Self {
            colors: ColorTokens::light(),
            typography: TypographyTokens::default_tokens(),
            spacing: SpacingTokens::default_tokens(),
            border: BorderTokens::default_tokens(),
            shadow: ShadowTokens::default_tokens(),
            z_index: ZIndexTokens::default_tokens(),
            transition: TransitionTokens::default_tokens(),
        }
    }

    /// Dark theme tokens.
    pub fn dark() -> Self {
        Self {
            colors: ColorTokens::dark(),
            ..Self::light()
        }
    }

    // -- Color overrides (builder) ------------------------------------------

    pub fn primary(mut self, color: impl Into<String>) -> Self {
        self.colors.primary = color.into();
        self
    }
    pub fn secondary(mut self, color: impl Into<String>) -> Self {
        self.colors.secondary = color.into();
        self
    }
    pub fn bg(mut self, color: impl Into<String>) -> Self {
        self.colors.bg = color.into();
        self
    }
    pub fn text(mut self, color: impl Into<String>) -> Self {
        self.colors.text = color.into();
        self
    }
    pub fn border(mut self, color: impl Into<String>) -> Self {
        self.colors.border = color.into();
        self
    }
    pub fn radius(mut self, r: impl Into<String>) -> Self {
        self.border.radius_md = r.into();
        self
    }
    pub fn font_family(mut self, f: impl Into<String>) -> Self {
        self.typography.font_family = f.into();
        self
    }

    // -- CSS generation -----------------------------------------------------

    /// Generate a CSS string with all tokens as `:root` custom properties.
    ///
    /// The output looks like:
    /// ```css
    /// :root {
    ///   --rye-primary: #2563eb;
    ///   --rye-bg: #ffffff;
    ///   ...
    /// }
    /// ```
    pub fn to_css(&self) -> String {
        let mut css = String::from(":root {\n");
        self.write_tokens(&mut css);
        css.push_str("}\n");
        css
    }

    /// Generate a CSS string scoped to a `[data-theme="..."]` selector.
    ///
    /// This allows having both light and dark themes loaded simultaneously
    /// and switching by setting `data-theme` on a parent element.
    pub fn to_css_scoped(&self, selector: &str) -> String {
        let mut css = String::with_capacity(2048);
        let _ = write!(css, "{} {{\n", selector);
        self.write_tokens(&mut css);
        css.push_str("}\n");
        css
    }

    /// Generate both light and dark CSS with `[data-theme]` selectors.
    pub fn to_css_with_dark(dark: &Self) -> String {
        let light = Self::light();
        format!(
            "{}\n{}\n",
            light.to_css_scoped("[data-theme=\"light\"]"),
            dark.to_css_scoped("[data-theme=\"dark\"]"),
        )
    }

    fn write_tokens(&self, css: &mut String) {
        let c = &self.colors;
        let t = &self.typography;
        let s = &self.spacing;
        let b = &self.border;
        let sh = &self.shadow;
        let z = &self.z_index;
        let tr = &self.transition;

        // Colors
        write_kv(css, "rye-primary", &c.primary);
        write_kv(css, "rye-primary-hover", &c.primary_hover);
        write_kv(css, "rye-primary-active", &c.primary_active);
        write_kv(css, "rye-primary-fg", &c.primary_fg);
        write_kv(css, "rye-secondary", &c.secondary);
        write_kv(css, "rye-secondary-hover", &c.secondary_hover);
        write_kv(css, "rye-secondary-fg", &c.secondary_fg);
        write_kv(css, "rye-success", &c.success);
        write_kv(css, "rye-success-fg", &c.success_fg);
        write_kv(css, "rye-warning", &c.warning);
        write_kv(css, "rye-warning-fg", &c.warning_fg);
        write_kv(css, "rye-danger", &c.danger);
        write_kv(css, "rye-danger-fg", &c.danger_fg);
        write_kv(css, "rye-danger-bg", &c.danger_bg);
        write_kv(css, "rye-info", &c.info);
        write_kv(css, "rye-info-fg", &c.info_fg);
        write_kv(css, "rye-bg", &c.bg);
        write_kv(css, "rye-bg-subtle", &c.bg_subtle);
        write_kv(css, "rye-bg-muted", &c.bg_muted);
        write_kv(css, "rye-bg-elevated", &c.bg_elevated);
        write_kv(css, "rye-text", &c.text);
        write_kv(css, "rye-text-muted", &c.text_muted);
        write_kv(css, "rye-text-subtle", &c.text_subtle);
        write_kv(css, "rye-border", &c.border);
        write_kv(css, "rye-border-strong", &c.border_strong);
        write_kv(css, "rye-border-subtle", &c.border_subtle);
        write_kv(css, "rye-input-bg", &c.input_bg);
        write_kv(css, "rye-input-border", &c.input_border);
        write_kv(css, "rye-input-border-focus", &c.input_border_focus);
        write_kv(css, "rye-overlay", &c.overlay);
        write_kv(css, "rye-ring", &c.ring);
        write_kv(css, "rye-code-bg", &c.code_bg);
        write_kv(css, "rye-code-header-bg", &c.code_header_bg);
        write_kv(css, "rye-code-border", &c.code_border);
        write_kv(css, "rye-code-text", &c.code_text);
        write_kv(css, "rye-code-line-number", &c.code_line_number);

        // Typography
        write_kv(css, "rye-font-family", &t.font_family);
        write_kv(css, "rye-font-family-mono", &t.font_family_mono);
        write_kv(css, "rye-font-size-xs", &t.font_size_xs);
        write_kv(css, "rye-font-size-sm", &t.font_size_sm);
        write_kv(css, "rye-font-size-md", &t.font_size_md);
        write_kv(css, "rye-font-size-lg", &t.font_size_lg);
        write_kv(css, "rye-font-size-xl", &t.font_size_xl);
        write_kv(css, "rye-font-size-2xl", &t.font_size_2xl);
        write_kv(css, "rye-font-weight-normal", &t.font_weight_normal);
        write_kv(css, "rye-font-weight-medium", &t.font_weight_medium);
        write_kv(css, "rye-font-weight-semibold", &t.font_weight_semibold);
        write_kv(css, "rye-font-weight-bold", &t.font_weight_bold);
        write_kv(css, "rye-line-height", &t.line_height);
        write_kv(css, "rye-line-height-tight", &t.line_height_tight);

        // Spacing
        write_kv(css, "rye-space-0", &s.space_0);
        write_kv(css, "rye-space-1", &s.space_1);
        write_kv(css, "rye-space-2", &s.space_2);
        write_kv(css, "rye-space-3", &s.space_3);
        write_kv(css, "rye-space-4", &s.space_4);
        write_kv(css, "rye-space-5", &s.space_5);
        write_kv(css, "rye-space-6", &s.space_6);
        write_kv(css, "rye-space-8", &s.space_8);
        write_kv(css, "rye-space-10", &s.space_10);
        write_kv(css, "rye-space-12", &s.space_12);
        write_kv(css, "rye-space-16", &s.space_16);
        write_kv(css, "rye-space-20", &s.space_20);

        // Border
        write_kv(css, "rye-radius-sm", &b.radius_sm);
        write_kv(css, "rye-radius-md", &b.radius_md);
        write_kv(css, "rye-radius-lg", &b.radius_lg);
        write_kv(css, "rye-radius-xl", &b.radius_xl);
        write_kv(css, "rye-radius-full", &b.radius_full);
        write_kv(css, "rye-border-width", &b.width_thin);

        // Shadow
        write_kv(css, "rye-shadow-sm", &sh.sm);
        write_kv(css, "rye-shadow-md", &sh.md);
        write_kv(css, "rye-shadow-lg", &sh.lg);
        write_kv(css, "rye-shadow-xl", &sh.xl);

        // Z-index
        write_kv(css, "rye-z-dropdown", &z.dropdown);
        write_kv(css, "rye-z-overlay", &z.overlay);
        write_kv(css, "rye-z-modal", &z.modal);
        write_kv(css, "rye-z-toast", &z.toast);
        write_kv(css, "rye-z-tooltip", &z.tooltip);

        // Transition
        write_kv(css, "rye-transition-fast", &tr.fast);
        write_kv(css, "rye-transition-normal", &tr.normal);
        write_kv(css, "rye-transition-slow", &tr.slow);
    }
}

fn write_kv(css: &mut String, key: &str, value: &str) {
    let _ = write!(css, "  --{}: {};\n", key, value);
}

// ---------------------------------------------------------------------------
// CSS variable reference helpers — used by components
// ---------------------------------------------------------------------------

/// Helper to reference a CSS variable: `var(--rye-primary)`.
pub fn v(name: &str) -> String {
    format!("var(--rye-{})", name)
}

/// Reference with fallback: `var(--rye-primary, #2563eb)`.
pub fn vf(name: &str, fallback: &str) -> String {
    format!("var(--rye-{}, {})", name, fallback)
}

/// Common CSS variable references used across components.
pub mod vars {
    // Colors
    pub const PRIMARY: &str = "var(--rye-primary)";
    pub const PRIMARY_HOVER: &str = "var(--rye-primary-hover)";
    pub const PRIMARY_FG: &str = "var(--rye-primary-fg)";
    pub const SECONDARY: &str = "var(--rye-secondary)";
    pub const SECONDARY_HOVER: &str = "var(--rye-secondary-hover)";
    pub const SECONDARY_FG: &str = "var(--rye-secondary-fg)";
    pub const SUCCESS: &str = "var(--rye-success)";
    pub const SUCCESS_FG: &str = "var(--rye-success-fg)";
    pub const WARNING: &str = "var(--rye-warning)";
    pub const WARNING_FG: &str = "var(--rye-warning-fg)";
    pub const DANGER: &str = "var(--rye-danger)";
    pub const DANGER_FG: &str = "var(--rye-danger-fg)";
    pub const DANGER_BG: &str = "var(--rye-danger-bg)";
    pub const INFO: &str = "var(--rye-info)";
    pub const INFO_FG: &str = "var(--rye-info-fg)";

    pub const BG: &str = "var(--rye-bg)";
    pub const BG_SUBTLE: &str = "var(--rye-bg-subtle)";
    pub const BG_MUTED: &str = "var(--rye-bg-muted)";
    pub const BG_ELEVATED: &str = "var(--rye-bg-elevated)";

    pub const TEXT: &str = "var(--rye-text)";
    pub const TEXT_MUTED: &str = "var(--rye-text-muted)";
    pub const TEXT_SUBTLE: &str = "var(--rye-text-subtle)";

    pub const BORDER: &str = "var(--rye-border)";
    pub const BORDER_STRONG: &str = "var(--rye-border-strong)";
    pub const BORDER_SUBTLE: &str = "var(--rye-border-subtle)";

    pub const INPUT_BG: &str = "var(--rye-input-bg)";
    pub const INPUT_BORDER: &str = "var(--rye-input-border)";
    pub const INPUT_BORDER_FOCUS: &str = "var(--rye-input-border-focus)";

    pub const OVERLAY: &str = "var(--rye-overlay)";
    pub const RING: &str = "var(--rye-ring)";

    // Code block colors
    pub const CODE_BG: &str = "var(--rye-code-bg)";
    pub const CODE_HEADER_BG: &str = "var(--rye-code-header-bg)";
    pub const CODE_BORDER: &str = "var(--rye-code-border)";
    pub const CODE_TEXT: &str = "var(--rye-code-text)";
    pub const CODE_LINE_NUMBER: &str = "var(--rye-code-line-number)";

    // Typography
    pub const FONT_FAMILY: &str = "var(--rye-font-family)";
    pub const FONT_FAMILY_MONO: &str = "var(--rye-font-family-mono)";
    pub const FONT_MONO: &str = "var(--rye-font-family-mono)";
    pub const FONT_SIZE_XS: &str = "var(--rye-font-size-xs)";
    pub const FONT_SIZE_SM: &str = "var(--rye-font-size-sm)";
    pub const FONT_SIZE_MD: &str = "var(--rye-font-size-md)";
    pub const FONT_SIZE_LG: &str = "var(--rye-font-size-lg)";
    pub const FONT_SIZE_XL: &str = "var(--rye-font-size-xl)";
    pub const FONT_WEIGHT_NORMAL: &str = "var(--rye-font-weight-normal)";
    pub const FONT_WEIGHT_MEDIUM: &str = "var(--rye-font-weight-medium)";
    pub const FONT_WEIGHT_SEMIBOLD: &str = "var(--rye-font-weight-semibold)";
    pub const FONT_WEIGHT_BOLD: &str = "var(--rye-font-weight-bold)";
    pub const LINE_HEIGHT: &str = "var(--rye-line-height)";

    // Spacing
    pub const SPACE_1: &str = "var(--rye-space-1)";
    pub const SPACE_2: &str = "var(--rye-space-2)";
    pub const SPACE_3: &str = "var(--rye-space-3)";
    pub const SPACE_4: &str = "var(--rye-space-4)";
    pub const SPACE_5: &str = "var(--rye-space-5)";
    pub const SPACE_6: &str = "var(--rye-space-6)";
    pub const SPACE_8: &str = "var(--rye-space-8)";

    // Border
    pub const RADIUS_SM: &str = "var(--rye-radius-sm)";
    pub const RADIUS_MD: &str = "var(--rye-radius-md)";
    pub const RADIUS_LG: &str = "var(--rye-radius-lg)";
    pub const RADIUS_XL: &str = "var(--rye-radius-xl)";
    pub const RADIUS_FULL: &str = "var(--rye-radius-full)";

    // Shadow
    pub const SHADOW_SM: &str = "var(--rye-shadow-sm)";
    pub const SHADOW_MD: &str = "var(--rye-shadow-md)";
    pub const SHADOW_LG: &str = "var(--rye-shadow-lg)";
    pub const SHADOW_XL: &str = "var(--rye-shadow-xl)";

    // Z-index
    pub const Z_DROPDOWN: &str = "var(--rye-z-dropdown)";
    pub const Z_OVERLAY: &str = "var(--rye-z-overlay)";
    pub const Z_MODAL: &str = "var(--rye-z-modal)";
    pub const Z_TOAST: &str = "var(--rye-z-toast)";
    pub const Z_TOOLTIP: &str = "var(--rye-z-tooltip)";

    // Transition
    pub const TRANSITION_FAST: &str = "var(--rye-transition-fast)";
    pub const TRANSITION_NORMAL: &str = "var(--rye-transition-normal)";
    pub const TRANSITION_SLOW: &str = "var(--rye-transition-slow)";
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_tokens() {
        let t = DesignTokens::light();
        assert_eq!(t.colors.primary, "#2563eb");
        assert_eq!(t.colors.bg, "#ffffff");
        assert_eq!(t.colors.text, "#1e293b");
    }

    #[test]
    fn test_dark_tokens() {
        let t = DesignTokens::dark();
        assert_eq!(t.colors.primary, "#3b82f6");
        assert_eq!(t.colors.bg, "#0f172a");
        assert_eq!(t.colors.text, "#f1f5f9");
    }

    #[test]
    fn test_builder_overrides() {
        let t = DesignTokens::light()
            .primary("#7c3aed")
            .radius("12px")
            .font_family("Inter, sans-serif");
        assert_eq!(t.colors.primary, "#7c3aed");
        assert_eq!(t.border.radius_md, "12px");
        assert_eq!(t.typography.font_family, "Inter, sans-serif");
    }

    #[test]
    fn test_to_css_contains_all_vars() {
        let css = DesignTokens::light().to_css();
        assert!(css.contains(":root"));
        assert!(css.contains("--rye-primary"));
        assert!(css.contains("--rye-bg"));
        assert!(css.contains("--rye-text"));
        assert!(css.contains("--rye-border"));
        assert!(css.contains("--rye-font-family"));
        assert!(css.contains("--rye-space-4"));
        assert!(css.contains("--rye-radius-md"));
        assert!(css.contains("--rye-shadow-md"));
        assert!(css.contains("--rye-z-modal"));
        assert!(css.contains("--rye-transition-normal"));
    }

    #[test]
    fn test_to_css_scoped() {
        let css = DesignTokens::dark().to_css_scoped("[data-theme=\"dark\"]");
        assert!(css.contains("[data-theme=\"dark\"]"));
        assert!(css.contains("--rye-bg: #0f172a;"));
    }

    #[test]
    fn test_to_css_with_dark() {
        let dark = DesignTokens::dark();
        let css = DesignTokens::to_css_with_dark(&dark);
        assert!(css.contains("[data-theme=\"light\"]"));
        assert!(css.contains("[data-theme=\"dark\"]"));
    }

    #[test]
    fn test_v_helper() {
        assert_eq!(v("primary"), "var(--rye-primary)");
    }

    #[test]
    fn test_vf_helper() {
        assert_eq!(vf("primary", "#2563eb"), "var(--rye-primary, #2563eb)");
    }

    #[test]
    fn test_vars_constants() {
        assert_eq!(vars::PRIMARY, "var(--rye-primary)");
        assert_eq!(vars::BG, "var(--rye-bg)");
        assert_eq!(vars::TEXT, "var(--rye-text)");
        assert_eq!(vars::DANGER, "var(--rye-danger)");
        assert_eq!(vars::RADIUS_MD, "var(--rye-radius-md)");
        assert_eq!(vars::SHADOW_LG, "var(--rye-shadow-lg)");
        assert_eq!(vars::Z_MODAL, "var(--rye-z-modal)");
        assert_eq!(vars::TRANSITION_NORMAL, "var(--rye-transition-normal)");
    }

    #[test]
    fn test_light_and_dark_differ() {
        let light = DesignTokens::light();
        let dark = DesignTokens::dark();
        assert_ne!(light.colors.bg, dark.colors.bg);
        assert_ne!(light.colors.text, dark.colors.text);
        assert_ne!(light.colors.border, dark.colors.border);
        // Typography/spacing/border/shadow are the same
        assert_eq!(light.typography, dark.typography);
        assert_eq!(light.spacing, dark.spacing);
    }

    #[test]
    fn test_default_is_light() {
        let t = DesignTokens::default();
        assert_eq!(t, DesignTokens::light());
    }
}
