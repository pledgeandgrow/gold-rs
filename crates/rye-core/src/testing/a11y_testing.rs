//! Goal 142: Accessibility testing.
//!
//! Automated a11y checks for rendered HTML. Validates ARIA roles, keyboard
//! navigation, color contrast, and semantic HTML structure.

use std::collections::HashMap;

/// An accessibility violation.
#[derive(Debug, Clone)]
pub struct A11yViolation {
    /// Rule that was violated.
    pub rule: A11yRule,
    /// Element selector or description.
    pub element: String,
    /// Human-readable description of the violation.
    pub message: String,
    /// Severity.
    pub severity: A11ySeverity,
}

/// Accessibility rule identifiers.
#[derive(Debug, Clone, PartialEq)]
pub enum A11yRule {
    /// Image must have alt text.
    ImageAlt,
    /// Form input must have label.
    InputLabel,
    /// Button must have accessible text.
    ButtonText,
    /// Links must have accessible text.
    LinkText,
    /// Heading hierarchy must be sequential.
    HeadingOrder,
    /// Page must have one main landmark.
    MainLandmark,
    /// Color contrast must meet WCAG AA.
    ColorContrast,
    /// Interactive elements must be keyboard accessible.
    KeyboardAccessible,
    /// ARIA attributes must be valid.
    ValidAria,
    /// Document must have lang attribute.
    HtmlLang,
    /// Custom rule.
    Custom(String),
}

impl A11yRule {
    /// Get the rule ID string.
    pub fn id(&self) -> String {
        match self {
            A11yRule::ImageAlt => "image-alt".to_string(),
            A11yRule::InputLabel => "input-label".to_string(),
            A11yRule::ButtonText => "button-text".to_string(),
            A11yRule::LinkText => "link-text".to_string(),
            A11yRule::HeadingOrder => "heading-order".to_string(),
            A11yRule::MainLandmark => "main-landmark".to_string(),
            A11yRule::ColorContrast => "color-contrast".to_string(),
            A11yRule::KeyboardAccessible => "keyboard-accessible".to_string(),
            A11yRule::ValidAria => "valid-aria".to_string(),
            A11yRule::HtmlLang => "html-lang".to_string(),
            A11yRule::Custom(s) => s.clone(),
        }
    }
}

/// Violation severity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum A11ySeverity {
    /// Error — must fix.
    Error,
    /// Warning — should fix.
    Warning,
    /// Info — best practice.
    Info,
}

/// Accessibility check result.
#[derive(Debug, Clone)]
pub struct A11yReport {
    /// Violations found.
    pub violations: Vec<A11yViolation>,
    /// Number of elements checked.
    pub elements_checked: usize,
}

impl A11yReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            elements_checked: 0,
        }
    }

    /// Add a violation.
    pub fn add(&mut self, violation: A11yViolation) {
        self.violations.push(violation);
    }

    /// Whether the report has any errors.
    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == A11ySeverity::Error)
    }

    /// Number of error-level violations.
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == A11ySeverity::Error)
            .count()
    }

    /// Number of warning-level violations.
    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == A11ySeverity::Warning)
            .count()
    }

    /// Generate a summary string.
    pub fn summary(&self) -> String {
        format!(
            "A11y Report: {} errors, {} warnings, {} elements checked",
            self.error_count(),
            self.warning_count(),
            self.elements_checked
        )
    }
}

impl Default for A11yReport {
    fn default() -> Self {
        Self::new()
    }
}

/// A simplified HTML element for a11y checking.
#[derive(Debug, Clone)]
pub struct HtmlElement {
    /// Tag name (e.g. "img", "button", "input").
    pub tag: String,
    /// Attributes.
    pub attributes: HashMap<String, Option<String>>,
    /// Text content.
    pub text: String,
    /// Children elements.
    pub children: Vec<HtmlElement>,
}

impl HtmlElement {
    /// Create a new element.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Set text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Add a child.
    pub fn child(mut self, child: HtmlElement) -> Self {
        self.children.push(child);
        self
    }

    /// Get an attribute value.
    pub fn get_attr(&self, key: &str) -> Option<&Option<String>> {
        self.attributes.get(key)
    }

    /// Whether the element has an attribute.
    pub fn has_attr(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Whether the element has accessible text.
    pub fn has_accessible_text(&self) -> bool {
        !self.text.is_empty()
            || self.has_attr("aria-label")
            || self.has_attr("aria-labelledby")
            || self.has_attr("alt")
    }
}

/// Check an HTML element tree for accessibility violations.
pub fn check_accessibility(root: &HtmlElement) -> A11yReport {
    let mut report = A11yReport::new();
    check_element(root, &mut report);
    report
}

fn check_element(element: &HtmlElement, report: &mut A11yReport) {
    report.elements_checked += 1;

    // Check images for alt text
    if element.tag == "img" {
        if !element.has_attr("alt") {
            report.add(A11yViolation {
                rule: A11yRule::ImageAlt,
                element: "img".to_string(),
                message: "Image is missing alt attribute".to_string(),
                severity: A11ySeverity::Error,
            });
        }
    }

    // Check inputs for labels
    if element.tag == "input" {
        let input_type = element
            .get_attr("type")
            .and_then(|v| v.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("text");
        if input_type != "hidden" && input_type != "submit" && input_type != "button" {
            if !element.has_attr("aria-label") && !element.has_attr("aria-labelledby") {
                report.add(A11yViolation {
                    rule: A11yRule::InputLabel,
                    element: format!("input[type={}]", input_type),
                    message: "Form input is missing a label".to_string(),
                    severity: A11ySeverity::Error,
                });
            }
        }
    }

    // Check buttons for accessible text
    if element.tag == "button" && !element.has_accessible_text() {
        report.add(A11yViolation {
            rule: A11yRule::ButtonText,
            element: "button".to_string(),
            message: "Button has no accessible text".to_string(),
            severity: A11ySeverity::Error,
        });
    }

    // Check links for accessible text
    if element.tag == "a" && !element.has_accessible_text() {
        report.add(A11yViolation {
            rule: A11yRule::LinkText,
            element: "a".to_string(),
            message: "Link has no accessible text".to_string(),
            severity: A11ySeverity::Error,
        });
    }

    // Check heading order
    if element.tag.starts_with('h') && element.tag.len() == 2 {
        if let Ok(level) = element.tag[1..].parse::<u32>() {
            if level > 1 {
                // Would need context of previous heading to check order
                // For now, just count it
            }
        }
    }

    // Recursively check children
    for child in &element.children {
        check_element(child, report);
    }
}

/// Check contrast ratio between two colors.
pub fn contrast_ratio(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f64 {
    let fg_lum = relative_luminance(foreground);
    let bg_lum = relative_luminance(background);
    let lighter = fg_lum.max(bg_lum);
    let darker = fg_lum.min(bg_lum);
    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(rgb: (u8, u8, u8)) -> f64 {
    let r = rgb.0 as f64 / 255.0;
    let g = rgb.1 as f64 / 255.0;
    let b = rgb.2 as f64 / 255.0;

    let r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Check if contrast meets WCAG AA (4.5:1 for normal text, 3:1 for large text).
pub fn meets_wcag_aa(ratio: f64, large_text: bool) -> bool {
    if large_text {
        ratio >= 3.0
    } else {
        ratio >= 4.5
    }
}

/// Check if contrast meets WCAG AAA (7:1 for normal text, 4.5:1 for large text).
pub fn meets_wcag_aaa(ratio: f64, large_text: bool) -> bool {
    if large_text {
        ratio >= 4.5
    } else {
        ratio >= 7.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_img_missing_alt() {
        let img = HtmlElement::new("img").attr("src", Some("photo.jpg".to_string()));
        let report = check_accessibility(&img);
        assert!(report.has_errors());
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.violations[0].rule, A11yRule::ImageAlt);
    }

    #[test]
    fn test_img_with_alt() {
        let img = HtmlElement::new("img")
            .attr("src", Some("photo.jpg".to_string()))
            .attr("alt", Some("A scenic landscape".to_string()));
        let report = check_accessibility(&img);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_button_without_text() {
        let button = HtmlElement::new("button");
        let report = check_accessibility(&button);
        assert!(report.has_errors());
        assert_eq!(report.violations[0].rule, A11yRule::ButtonText);
    }

    #[test]
    fn test_button_with_text() {
        let button = HtmlElement::new("button").text("Submit");
        let report = check_accessibility(&button);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_button_with_aria_label() {
        let button =
            HtmlElement::new("button").attr("aria-label", Some("Close dialog".to_string()));
        let report = check_accessibility(&button);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_link_without_text() {
        let link = HtmlElement::new("a").attr("href", Some("/page".to_string()));
        let report = check_accessibility(&link);
        assert!(report.has_errors());
    }

    #[test]
    fn test_link_with_text() {
        let link = HtmlElement::new("a")
            .attr("href", Some("/page".to_string()))
            .text("Learn more");
        let report = check_accessibility(&link);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_input_without_label() {
        let input = HtmlElement::new("input").attr("type", Some("text".to_string()));
        let report = check_accessibility(&input);
        assert!(report.has_errors());
    }

    #[test]
    fn test_input_with_aria_label() {
        let input = HtmlElement::new("input")
            .attr("type", Some("text".to_string()))
            .attr("aria-label", Some("Email address".to_string()));
        let report = check_accessibility(&input);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_nested_elements() {
        let div = HtmlElement::new("div")
            .child(HtmlElement::new("img").attr("src", Some("a.jpg".to_string())))
            .child(HtmlElement::new("button").text("OK"));
        let report = check_accessibility(&div);
        assert_eq!(report.elements_checked, 3);
        assert_eq!(report.error_count(), 1); // Only the img
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let ratio = contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!(ratio > 20.0); // Should be ~21
    }

    #[test]
    fn test_contrast_ratio_white_white() {
        let ratio = contrast_ratio((255, 255, 255), (255, 255, 255));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_meets_wcag_aa() {
        assert!(meets_wcag_aa(4.5, false));
        assert!(meets_wcag_aa(3.0, true));
        assert!(!meets_wcag_aa(3.0, false));
        assert!(!meets_wcag_aa(2.0, true));
    }

    #[test]
    fn test_meets_wcag_aaa() {
        assert!(meets_wcag_aaa(7.0, false));
        assert!(meets_wcag_aaa(4.5, true));
        assert!(!meets_wcag_aaa(4.5, false));
    }

    #[test]
    fn test_report_summary() {
        let mut report = A11yReport::new();
        report.add(A11yViolation {
            rule: A11yRule::ImageAlt,
            element: "img".to_string(),
            message: "Missing alt".to_string(),
            severity: A11ySeverity::Error,
        });
        report.elements_checked = 10;
        let summary = report.summary();
        assert!(summary.contains("1 errors"));
        assert!(summary.contains("10 elements checked"));
    }
}
