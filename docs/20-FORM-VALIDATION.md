# Form & Validation System Design

> Goal 22 — Built-in form handling: reactive form state, schema validation, field-level + form-level validation, async validation, error display helpers.

---

## Design Goals

- **Reactive** — Form state is signal-based, UI updates automatically
- **Schema validation** — Declarative validation rules, integrate with `validator` crate
- **Multi-level** — Field-level and form-level validation
- **Async validation** — Server-side checks (e.g., username availability)
- **Ergonomic** — Minimal boilerplate for common cases
- **Error display** — Built-in helpers for showing validation errors

---

## Basic Usage

### Simple form with validation

```rust
use rye::prelude::*;
use rye::forms::{Form, Field, Validators};

#[derive(Form)]
struct LoginForm {
    #[field(
        label = "Email",
        validators = [Validators::required(), Validators::email()],
    )]
    email: String,

    #[field(
        label = "Password",
        validators = [Validators::required(), Validators::min_length(8)],
    )]
    password: String,
}

#[component]
fn Login() {
    let form = use_form(LoginForm::default());

    div {
        class: "login-form",
        form.on_submit(move |values: LoginForm| {
            // values are validated — safe to use
            log::info!("Login: {} / {}", values.email, values.password);
        }),

        Field { field: form.email, label: "Email" }
        Field { field: form.password, label: "Password", type: "password" }

        button {
            type: "submit",
            disabled: {form.is_invalid()},
            "Login"
        }
    }
}
```

### Manual form (without derive)

```rust
#[component]
fn ContactForm() {
    let name = use_field(String::new())
        .validate(|v| if v.is_empty() { Some("Name is required".into()) } else { None });

    let email = use_field(String::new())
        .validate(|v| if !v.contains('@') { Some("Invalid email".into()) } else { None });

    let message = use_field(String::new())
        .validate(|v| if v.len() < 10 { Some("Message too short".into()) } else { None });

    let is_valid = use_memo(move || {
        name.errors().is_empty() &&
        email.errors().is_empty() &&
        message.errors().is_empty()
    });

    div {
        input {
            value: {name.value()},
            oninput: move |e| name.set(e.value()),
            placeholder: "Name",
        }
        if !name.errors().is_empty() {
            span { class: "error", {name.errors().first().unwrap()} }
        }

        input {
            value: {email.value()},
            oninput: move |e| email.set(e.value()),
            placeholder: "Email",
        }
        if !email.errors().is_empty() {
            span { class: "error", {email.errors().first().unwrap()} }
        }

        textarea {
            value: {message.value()},
            oninput: move |e| message.set(e.value()),
            placeholder: "Message",
        }

        button {
            disabled: { !is_valid() },
            onclick: move |_| {
                // Submit
                submit_contact(name.value(), email.value(), message.value());
            },
            "Send"
        }
    }
}
```

---

## Validation

### Built-in validators

```rust
pub enum Validators {
    Required,
    Email,
    Url,
    MinLength(usize),
    MaxLength(usize),
    Min(i32),
    Max(i32),
    Pattern(Regex),
    Custom(Box<dyn Fn(&str) -> Option<String>>),
}

// Usage in derive:
#[field(validators = [Validators::required(), Validators::email()])]
email: String,

// Usage in manual:
let field = use_field(String::new())
    .validate(Validators::required())
    .validate(Validators::email())
    .validate(|v| {
        if v.contains("spam") { Some("No spam allowed".into()) } else { None }
    });
```

### Async validation

```rust
let username = use_field(String::new())
    .validate_async(move |value: String| async move {
        if value.is_empty() {
            return Some("Username is required".into());
        }
        // Check if username is available on server
        match check_username_available(&value).await {
            Ok(true) => None,
            Ok(false) => Some("Username already taken".into()),
            Err(_) => Some("Could not validate username".into()),
        }
    });

// Async validation runs after sync validation passes
// Debounced by default (300ms after last keystroke)
```

### Form-level validation

```rust
#[derive(Form)]
struct PasswordChangeForm {
    #[field(validators = [Validators::required(), Validators::min_length(8)])]
    new_password: String,

    #[field(validators = [Validators::required()])]
    confirm_password: String,
}

// Form-level validation (cross-field)
impl ValidateForm for PasswordChangeForm {
    fn validate_form(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.new_password != self.confirm_password {
            errors.push("Passwords do not match".into());
        }
        errors
    }
}
```

---

## Field State

```rust
/// A reactive form field.
pub struct Field<T: Clone + 'static> {
    /// Current value.
    value: Signal<T>,
    /// Validation errors.
    errors: Signal<Vec<String>>,
    /// Whether the field has been touched (blurred).
    touched: Signal<bool>,
    /// Whether the field has been modified.
    dirty: Signal<bool>,
}

impl<T: Clone + 'static> Field<T> {
    pub fn value(&self) -> T;
    pub fn set(&self, value: T);
    pub fn errors(&self) -> Vec<String>;
    pub fn is_valid(&self) -> bool;
    pub fn is_touched(&self) -> bool;
    pub fn is_dirty(&self) -> bool;
    pub fn touch(&self);  // Mark as touched
    pub fn reset(&self);  // Reset to initial value
}
```

---

## Form State

```rust
/// A reactive form.
pub struct Form<T: FormValues> {
    /// Form values.
    values: Signal<T>,
    /// Whether the form is valid.
    is_valid: Signal<bool>,
    /// Whether the form is submitting.
    is_submitting: Signal<bool>,
    /// Form-level errors.
    errors: Signal<Vec<String>>,
}

impl<T: FormValues> Form<T> {
    pub fn is_valid(&self) -> bool;
    pub fn is_invalid(&self) -> bool;
    pub fn is_submitting(&self) -> bool;
    pub fn is_dirty(&self) -> bool;
    pub fn reset(&self);
    pub fn on_submit<F: Fn(T) + 'static>(&self, handler: F);
}
```

---

## Error Display Helpers

```rust
// Show error only when field is touched and invalid
ErrorDisplay {
    field: form.email,
    class: "field-error",
}

// Equivalent to:
if form.email.is_touched() && !form.email.errors().is_empty() {
    span {
        class: "field-error",
        {form.email.errors().first().unwrap()}
    }
}

// Show all errors
ErrorList {
    field: form.email,
    class: "error-list",
}

// Show form-level errors
FormErrors {
    form: form,
    class: "form-errors",
}
```

---

## Comparison with Competitors

| Feature | React | Vue | Dioxus | Leptos | rye |
|---|---|---|---|---|---|
| Form state | Manual (useState) | Manual | Manual | Manual | Built-in (use_form) |
| Validation | External (zod, yup) | External (vee-validate) | Manual | Manual | Built-in + validator crate |
| Async validation | Manual | Manual | No | No | Built-in (debounced) |
| Field state (touched/dirty) | Manual | Manual | No | No | Built-in |
| Schema validation | External | External | No | No | Built-in (#[derive(Form)]) |
| Error display helpers | No | No | No | No | Yes |

---

*This document defines the form & validation system. **Implemented** in `rye-forms` crate — `#[derive(Form)]`, `use_form`, `use_field`, built-in validators, async validation with debounce, form-level cross-field validation, field state (touched/dirty), and error display helpers (`ErrorDisplay`, `ErrorList`, `FormErrors`).*
