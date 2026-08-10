//! FormValidator — validation rule engine + error display.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Email,
    Url,
    Min(f64),
    Max(f64),
    Pattern(String),
    Custom(String),
}

impl ValidationRule {
    pub fn validate(&self, value: &str) -> Option<String> {
        let err = match self {
            Self::Required => {
                if value.trim().is_empty() {
                    Some("This field is required".to_string())
                } else {
                    None
                }
            }
            Self::MinLength(n) => {
                if value.len() < *n {
                    Some(format!("Must be at least {} characters", n))
                } else {
                    None
                }
            }
            Self::MaxLength(n) => {
                if value.len() > *n {
                    Some(format!("Must be at most {} characters", n))
                } else {
                    None
                }
            }
            Self::Email => {
                if value.is_empty() || (value.contains('@') && value.contains('.')) {
                    None
                } else {
                    Some("Invalid email address".to_string())
                }
            }
            Self::Url => {
                if value.is_empty() || value.starts_with("http://") || value.starts_with("https://")
                {
                    None
                } else {
                    Some("Invalid URL".to_string())
                }
            }
            Self::Min(n) => value
                .parse::<f64>()
                .ok()
                .filter(|v| *v >= *n)
                .map(|_| ())
                .map_or_else(|| Some(format!("Must be at least {}", n)), |_| None),
            Self::Max(n) => value
                .parse::<f64>()
                .ok()
                .filter(|v| *v <= *n)
                .map(|_| ())
                .map_or_else(|| Some(format!("Must be at most {}", n)), |_| None),
            Self::Pattern(p) => {
                if value.is_empty() {
                    None
                } else {
                    let ok = p.chars().all(|pc| value.contains(pc));
                    if ok {
                        None
                    } else {
                        Some(format!("Must match pattern: {}", p))
                    }
                }
            }
            Self::Custom(msg) => Some(msg.clone()),
        };
        err
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub field: String,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FieldValidator {
    pub field: String,
    pub rules: Vec<ValidationRule>,
}

impl FieldValidator {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            rules: Vec::new(),
        }
    }
    pub fn rule(mut self, r: ValidationRule) -> Self {
        self.rules.push(r);
        self
    }
    pub fn required(mut self) -> Self {
        self.rules.push(ValidationRule::Required);
        self
    }
    pub fn min_length(mut self, n: usize) -> Self {
        self.rules.push(ValidationRule::MinLength(n));
        self
    }
    pub fn max_length(mut self, n: usize) -> Self {
        self.rules.push(ValidationRule::MaxLength(n));
        self
    }
    pub fn email(mut self) -> Self {
        self.rules.push(ValidationRule::Email);
        self
    }

    pub fn validate(&self, value: &str) -> ValidationResult {
        let errors: Vec<String> = self
            .rules
            .iter()
            .filter_map(|r| r.validate(value))
            .collect();
        ValidationResult {
            field: self.field.clone(),
            errors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormValidator {
    pub fields: Vec<FieldValidator>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }
    pub fn field(mut self, f: FieldValidator) -> Self {
        self.fields.push(f);
        self
    }

    pub fn validate(
        &self,
        values: &std::collections::HashMap<String, String>,
    ) -> Vec<ValidationResult> {
        self.fields
            .iter()
            .map(|f| {
                let value = values.get(&f.field).map(|s| s.as_str()).unwrap_or("");
                f.validate(value)
            })
            .collect()
    }

    pub fn is_valid(&self, values: &std::collections::HashMap<String, String>) -> bool {
        self.validate(values).iter().all(|r| r.is_valid())
    }

    pub fn render_errors(&self, values: &std::collections::HashMap<String, String>) -> Element {
        let results = self.validate(values);
        let errors: Vec<Template> = results.iter().filter(|r| !r.is_valid()).flat_map(|r| {
            r.errors.iter().map(|err| {
                Template::new_element("div",
                    vec![("style".to_string(), format!("padding:8px 12px;background:{};color:{};border-radius:var(--rye-radius-md);font-size:var(--rye-font-size-sm);margin-bottom:4px;", vars::DANGER_BG, vars::DANGER)),
                         ("class".to_string(), "rye-form-validator-error".to_string())],
                    Vec::new(), vec![Template::text(&format!("{}: {}", r.field, err))])
            }).collect::<Vec<_>>()
        }).collect();

        if errors.is_empty() {
            Element::None
        } else {
            Element::Template(Template::new_element(
                "div",
                vec![("class".to_string(), "rye-form-validator".to_string())],
                Vec::new(),
                errors,
            ))
        }
    }
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_required() {
        assert!(ValidationRule::Required.validate("").is_some());
        assert!(ValidationRule::Required.validate("value").is_none());
    }

    #[test]
    fn test_rule_min_length() {
        assert!(ValidationRule::MinLength(5).validate("ab").is_some());
        assert!(ValidationRule::MinLength(5).validate("abcde").is_none());
    }

    #[test]
    fn test_rule_email() {
        assert!(ValidationRule::Email.validate("notanemail").is_some());
        assert!(ValidationRule::Email.validate("user@example.com").is_none());
    }

    #[test]
    fn test_rule_min() {
        assert!(ValidationRule::Min(10.0).validate("5").is_some());
        assert!(ValidationRule::Min(10.0).validate("15").is_none());
    }

    #[test]
    fn test_field_validator() {
        let fv = FieldValidator::new("email")
            .required()
            .email()
            .min_length(5);
        let result = fv.validate("ab");
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_field_validator_valid() {
        let fv = FieldValidator::new("email").required().email();
        let result = fv.validate("user@test.com");
        assert!(result.is_valid());
    }

    #[test]
    fn test_form_validator() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "".to_string());
        values.insert("email".to_string(), "user@test.com".to_string());

        let fv = FormValidator::new()
            .field(FieldValidator::new("name").required())
            .field(FieldValidator::new("email").required().email());

        assert!(!fv.is_valid(&values));

        values.insert("name".to_string(), "Alice".to_string());
        assert!(fv.is_valid(&values));
    }

    #[test]
    fn test_form_validator_render_errors() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "".to_string());

        let fv = FormValidator::new().field(FieldValidator::new("name").required());

        let el = fv.render_errors(&values);
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_form_validator_render_no_errors() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());

        let fv = FormValidator::new().field(FieldValidator::new("name").required());

        let el = fv.render_errors(&values);
        assert!(matches!(el, Element::None));
    }
}
