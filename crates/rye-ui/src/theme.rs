//! Shared theme types — colors, sizes, variants used across all components.
//!
//! This module re-exports the design token system from [`tokens`] and provides
//! the [`Size`] and [`Variant`] enums used by individual components.
//!
//! ## Variants
//!
//! [`Variant`] maps semantic visual styles (Primary, Secondary, Ghost, etc.)
//! to CSS variable references. All color methods (`background()`, `color()`,
//! `border()`, `hover_background()`) return `var(--rye-*)` strings, so theme
//! switching is automatic when the `data-theme` attribute changes.
//!
//! ## Sizes
//!
//! [`Size`] provides `padding()` and `font_size()` helpers for consistent
//! sizing across components.
//!
//! See [`crate::ThemeProvider`] for injecting CSS variables into the DOM.

pub use crate::tokens::{
    DesignTokens, ColorTokens, TypographyTokens, SpacingTokens,
    BorderTokens, ShadowTokens, ZIndexTokens, TransitionTokens,
    vars, v, vf,
};

/// Color scheme for themed components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

/// Component size variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "sm",
            Self::Medium => "md",
            Self::Large => "lg",
        }
    }

    pub fn padding(&self) -> &'static str {
        match self {
            Self::Small => "4px 8px",
            Self::Medium => "8px 16px",
            Self::Large => "12px 24px",
        }
    }

    pub fn font_size(&self) -> &'static str {
        match self {
            Self::Small => "12px",
            Self::Medium => "14px",
            Self::Large => "16px",
        }
    }
}

/// Visual variant for buttons, alerts, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
    Outline,
    Success,
    Warning,
    Info,
}

impl Variant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }

    /// Background color as a CSS variable reference.
    pub fn background(&self) -> &'static str {
        match self {
            Self::Primary => vars::PRIMARY,
            Self::Secondary => vars::SECONDARY,
            Self::Ghost => "transparent",
            Self::Destructive => vars::DANGER,
            Self::Outline => "transparent",
            Self::Success => vars::SUCCESS,
            Self::Warning => vars::WARNING,
            Self::Info => vars::INFO,
        }
    }

    /// Text color as a CSS variable reference.
    pub fn color(&self) -> &'static str {
        match self {
            Self::Ghost | Self::Outline => vars::TEXT,
            _ => vars::PRIMARY_FG,
        }
    }

    /// Border color as a CSS variable reference.
    pub fn border(&self) -> &'static str {
        match self {
            Self::Outline => vars::BORDER_STRONG,
            _ => "transparent",
        }
    }

    /// Hover background color as a CSS variable reference.
    pub fn hover_background(&self) -> &'static str {
        match self {
            Self::Primary => vars::PRIMARY_HOVER,
            Self::Secondary => vars::SECONDARY_HOVER,
            Self::Ghost => vars::BG_MUTED,
            Self::Destructive => vars::DANGER,
            Self::Outline => vars::BG_SUBTLE,
            Self::Success => vars::SUCCESS,
            Self::Warning => vars::WARNING,
            Self::Info => vars::INFO,
        }
    }
}

/// Theme configuration — a thin wrapper around [`DesignTokens`].
///
/// Kept for backwards compatibility. New code should use [`DesignTokens`]
/// and [`crate::ThemeProvider`] directly.
#[derive(Debug, Clone)]
pub struct Theme {
    pub color_scheme: ColorScheme,
    pub tokens: DesignTokens,
}

impl Default for Theme {
    fn default() -> Self {
        Self { color_scheme: ColorScheme::Light, tokens: DesignTokens::light() }
    }
}

impl Theme {
    /// Create a dark theme.
    pub fn dark() -> Self {
        Self { color_scheme: ColorScheme::Dark, tokens: DesignTokens::dark() }
    }

    /// Create a light theme.
    pub fn light() -> Self {
        Self { color_scheme: ColorScheme::Light, tokens: DesignTokens::light() }
    }

    /// Generate CSS custom properties for this theme.
    pub fn to_css(&self) -> String {
        self.tokens.to_css()
    }

    /// Builder: override primary color.
    pub fn primary(mut self, color: impl Into<String>) -> Self {
        self.tokens = self.tokens.primary(color); self
    }

    /// Builder: override border radius.
    pub fn radius(mut self, r: impl Into<String>) -> Self {
        self.tokens = self.tokens.radius(r); self
    }

    /// Builder: override font family.
    pub fn font_family(mut self, f: impl Into<String>) -> Self {
        self.tokens = self.tokens.font_family(f); self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_as_str() {
        assert_eq!(Size::Small.as_str(), "sm");
        assert_eq!(Size::Medium.as_str(), "md");
        assert_eq!(Size::Large.as_str(), "lg");
    }

    #[test]
    fn test_size_padding() {
        assert_eq!(Size::Small.padding(), "4px 8px");
        assert_eq!(Size::Large.padding(), "12px 24px");
    }

    #[test]
    fn test_variant_as_str() {
        assert_eq!(Variant::Primary.as_str(), "primary");
        assert_eq!(Variant::Destructive.as_str(), "destructive");
    }

    #[test]
    fn test_variant_colors() {
        assert_eq!(Variant::Primary.background(), vars::PRIMARY);
        assert_eq!(Variant::Primary.color(), vars::PRIMARY_FG);
        assert_eq!(Variant::Ghost.color(), vars::TEXT);
    }

    #[test]
    fn test_variant_hover() {
        assert_eq!(Variant::Primary.hover_background(), vars::PRIMARY_HOVER);
        assert_eq!(Variant::Secondary.hover_background(), vars::SECONDARY_HOVER);
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.color_scheme, ColorScheme::Light);
        assert_eq!(theme.tokens.colors.primary, "#2563eb");
    }

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert_eq!(theme.color_scheme, ColorScheme::Dark);
        assert_eq!(theme.tokens.colors.bg, "#0f172a");
    }

    #[test]
    fn test_theme_builder() {
        let theme = Theme::light().primary("#7c3aed").radius("12px");
        assert_eq!(theme.tokens.colors.primary, "#7c3aed");
        assert_eq!(theme.tokens.border.radius_md, "12px");
    }

    #[test]
    fn test_theme_to_css() {
        let theme = Theme::default();
        let css = theme.to_css();
        assert!(css.contains("--rye-primary"));
    }
}
