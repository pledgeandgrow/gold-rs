# Custom Error Reporting Design

> Goal 14 — Build a custom diagnostic layer that transforms opaque Rust macro errors into friendly, colored, contextual messages. A major differentiator — Rust's #1 pain is bad error messages.

---

## Design Goals

- **Friendly** — Errors feel like they're from a framework, not from Rust macros
- **Contextual** — Show the exact template location, not the macro expansion site
- **Actionable** — Every error includes a hint or fix suggestion
- **Colored** — Use terminal colors for readability (when supported)
- **Mapped** — Map macro expansion errors back to source code locations

---

## Architecture

```
User writes template! { ... }
         │
         ▼
   ┌─────────────┐
   │  Parser     │  ← Custom parser for template syntax
   └──────┬──────┘
          │
          ▼
   ┌─────────────┐
   │ Validator   │  ← Checks tags, attrs, props, types
   └──────┬──────┘
          │
     ┌────┴────┐
     │ Error?  │
     └────┬────┘
          │
          ▼
   ┌─────────────┐
   │ Diagnostic  │  ← Custom diagnostic emitter
   │ Emitter     │    Uses proc_macro2::Span for source mapping
   └──────┬──────┘
          │
          ▼
   ┌─────────────┐
   │ Compiler    │  ← Rust compiler displays the error
   │ Output      │    with our custom formatting
   └─────────────┘
```

---

## Error Categories

### 1. Parse Errors

| Error | Message | Hint |
|---|---|---|
| Unclosed brace | `Unexpected end of template, expected '}'` | Show where the opening `{` was |
| Invalid tag name | `Unknown HTML tag '{tag}'` | Suggest closest match (Levenshtein) |
| Invalid attribute | `Invalid attribute '{name}'` | List valid attributes for the tag |
| Missing comma | `Expected ',' between attributes` | Show the correct position |

### 2. Validation Errors

| Error | Message | Hint |
|---|---|---|
| Missing required prop | `Missing required prop '{prop}' for component '{name}'` | Show all props with (required)/(optional) |
| Wrong prop type | `Prop '{prop}' expects {expected}, got {actual}` | Show correct usage example |
| Invalid event name | `Unknown event '{event}'` | List valid events for the element |
| Missing key in For | `For loop requires a 'key:' attribute` | Show example with key |
| Duplicate attribute | `Attribute '{name}' specified twice` | Show both locations |

### 3. Type Errors

| Error | Message | Hint |
|---|---|---|
| Signal type mismatch | `Signal<{T}> cannot be used where {U} is expected` | Show where the signal is read |
| Non-Clone type in template | `Type '{T}' does not implement Clone` | Suggest wrapping in `Arc` or using `Signal` |
| Missing Display impl | `Type '{T}' cannot be rendered as text` | Suggest implementing `std::fmt::Display` |

---

## Diagnostic Format

```
error[{code}]: {message}
  --> {file}:{line}:{col}
   |
{line} |     {source_line}
   |                {pointer}
   |
   = hint: {hint_text}
   = help: {help_text}
   = note: {note_text}
```

### Example: Missing prop

```
error[R001]: Missing required prop 'label' for component 'Button'
  --> src/components/form.rs:23:9
   |
23 |     Button { disabled: true }
   |     ^^^^^^
   |
   = hint: Component 'Button' requires these props:
   |
   |     label: String    (required)
   |     disabled: bool   (optional, default: false)
   |     variant: String  (optional, default: "primary")
   |
   = help: Add the missing prop:
   |
   |     Button { label: "Submit", disabled: true }
```

### Example: Unknown tag

```
error[R002]: Unknown HTML tag 'buton'
  --> src/components/nav.rs:15:5
   |
15 |     buton { "Click me" }
   |     ^^^^^
   |
   = hint: Did you mean 'button'?
   = help: Valid HTML tags: a, abbr, article, aside, b, blockquote, br,
   |        button, canvas, code, div, em, footer, form, h1-h6, header,
   |        hr, i, img, input, label, li, nav, ol, p, pre, section,
   |        select, span, strong, table, tbody, td, th, thead, tr, ul, ...
```

### Example: Type mismatch

```
error[R003]: Prop 'size' expects i32, got &str
  --> src/components/icon.rs:31:22
   |
31 |     Icon { name: "home", size: "large" }
   |                      ^^^^^^^
   |
   = help: Use an integer:
   |
   |     Icon { name: "home", size: 32 }
```

---

## Error Codes

| Code Range | Category |
|---|---|
| R001–R099 | Parse errors |
| R100–R199 | Validation errors |
| R200–R299 | Type errors |
| R300–R399 | Reactivity errors |
| R400–R499 | Renderer errors |
| R500–R599 | Router errors |
| R600–R699 | SSR errors |
| R700–R799 | CLI errors |
| R800–R899 | AI-specific errors (common AI code generation mistakes) |

### AI-Specific Error Codes (R800–R899)

These error codes target patterns that AI models frequently get wrong when generating rye code:

| Code | Message | Hint |
|---|---|---|
| R800 | Wrong prop type (expected `String`, got `&str`) | Use `.to_string()` to convert |
| R801 | Missing `move` keyword in event handler closure | Add `move` before the closure: `move \|e\| { ... }` |
| R802 | Signal read without `.get()` | Use `signal.get()` instead of `signal` |
| R803 | Signal write without `.set()` | Use `signal.set(value)` instead of `signal = value` |
| R804 | Component name not PascalCase | Rename to PascalCase (e.g., `Button` not `button`) |
| R805 | Missing `#[component]` macro | Add `#[component]` above the function |
| R806 | `use_effect` for derived state | Use `Memo::new(move \|\| ...)` instead |
| R807 | Unnecessary `.clone()` | Remove `.clone()` — rye props are borrowed |
| R808 | Prop drilling instead of context | Use `provide_context()`/`use_context()` |
| R809 | Raw async spawn instead of `use_resource` | Use `Resource::new(async { ... })` |
| R810 | `template!` outside `#[component]` function | Move `template!` inside a `#[component]` fn |

Each AI-specific error code includes:
- **Step-by-step recovery plan** — ordered steps with code examples (`rye-core/src/ai/error_recovery.rs`)
- **Common mistakes** — what NOT to try
- **Verification** — how to confirm the fix worked
- **Alternatives** — other valid approaches

AI agents can query these programmatically via:
```bash
$ rpg explain R802
$ rpg explain R802 --json
```

Or via MCP tools:
```
rye_explain_error(code: "R802")
rye_get_recovery_plan(code: "R802")
```

---

### proc_macro2 spans

The `template!` macro uses `proc_macro2::Span` to track source locations. When an error is detected, the macro emits a `compile_error!` with a custom message at the correct span:

```rust
// In the macro:
if !is_valid_tag(tag_name) {
    let span = tag_name.span();
    let suggestion = closest_match(tag_name, VALID_TAGS);
    let msg = format!(
        "Unknown HTML tag '{}'\n\n= hint: Did you mean '{}'?",
        tag_name, suggestion
    );
    return quote_spanned! { span =>
        compile_error!(#msg);
    }.into();
}
```

### Diagnostic builder

```rust
pub struct Diagnostic {
    code: &'static str,       // e.g. "R002"
    message: String,           // e.g. "Unknown HTML tag 'buton'"
    span: Span,                // Source location
    hints: Vec<String>,        // Actionable suggestions
    notes: Vec<String>,        // Additional context
}

impl Diagnostic {
    pub fn emit(&self) -> TokenStream {
        let msg = self.format();
        let span = self.span;
        quote_spanned! { span =>
            compile_error!(#msg);
        }
    }

    fn format(&self) -> String {
        let mut s = format!("error[{}]: {}\n", self.code, self.message);
        for hint in &self.hints {
            s.push_str(&format!("= hint: {}\n", hint));
        }
        for note in &self.notes {
            s.push_str(&format!("= note: {}\n", note));
        }
        s
    }
}
```

### Levenshtein distance for suggestions

```rust
fn closest_match(input: &str, candidates: &[&str]) -> Option<&'static str> {
    candidates
        .iter()
        .map(|c| (c, levenshtein(input, c)))
        .filter(|(_, dist)| *dist <= 3)
        .min_by_key(|(_, dist)| *dist)
        .map(|(c, _)| *c)
}
```

---

## Comparison with Competitors

| Feature | Rust macros | Dioxus | Leptos | Yew | rye |
|---|---|---|---|---|---|
| Error messages | Opaque panics | Opaque panics | Opaque panics | Opaque panics | Custom diagnostics |
| Source mapping | Lost in expansion | Lost | Lost | Lost | Preserved via spans |
| Suggestions | No | No | No | No | Yes (Levenshtein) |
| Error codes | No | No | No | No | Yes (R001–R899) |
| Multi-line errors | No | No | No | No | Yes (hints, notes, examples) |
| AI-specific codes | No | No | No | No | Yes (R800–R810) |
| Recovery plans | No | No | No | No | Yes (step-by-step) |
| CLI error lookup | No | No | No | No | Yes (`rpg explain`) |
| MCP error tools | No | No | No | No | Yes (`rye-mcp`) |

---

*This document defines the error reporting system. **Implemented** in `rye-macros/src/template.rs` — `Diagnostic` builder with error codes R001–R799, `proc_macro2` span tracking, Levenshtein distance suggestions, and multi-line error messages with hints/notes. AI-specific error codes R800–R810 implemented in `rye-core/src/error_codes.rs` with step-by-step recovery plans in `rye-core/src/ai/error_recovery.rs`. CLI lookup via `rpg explain` in `rye-cli/src/explain.rs`. MCP tools for error explanation and recovery in `crates/rye-mcp/`.*
