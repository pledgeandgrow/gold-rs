# rye Framework Spec for AI

> Paste this file into any LLM's context (system prompt or first message) to enable
> correct rye code generation. No fine-tuning or few-shot examples required.

---

## Quick Start

```rust
use rye::prelude::*;

#[component]
fn App() -> impl IntoView {
    let count = Signal::new(0);
    template! {
        <div>
            <h1>{"Counter"}</h1>
            <p>{count.get()}</p>
            <button on:click=move |_| count.set(count.get() + 1)>{"+1"}</button>
        </div>
    }
}
```

---

## Core API (Memorize These)

### State — `Signal<T>`

```rust
let count = Signal::new(0i32);     // create
count.get()                         // read (tracks dependency in effects/memos)
count.set(5)                        // write (notifies subscribers)
count.update(|v| *v + 1)            // mutate in-place
```

### Derived — `Memo<T>`

```rust
let doubled = Memo::new(move || count.get() * 2);
doubled.get()                       // read (auto-tracks count)
```

### Side Effect — `Effect`

```rust
Effect::new(move || {
    println!("count is {}", count.get());
});
```

### Async — `Resource<T>`

```rust
let data = Resource::new(async {
    fetch_data().await
});

// In template:
template! {
    <Suspense fallback=template! { <p>{"Loading..."}</p> }>
        {data.get().map(|d| template! { <p>{d}</p> })}
    </Suspense>
}
```

### Context

```rust
// Provide (in parent component)
provide_context(Theme::default());

// Consume (in any child component)
let theme = use_context::<Theme>();
```

### Utilities

```rust
batch(|| {                         // batch multiple signal writes into one update
    count.set(1);
    name.set("hello");
});

untrack(|| {                       // read signals without tracking
    count.get()
});

on_cleanup(|| {                    // register cleanup for current scope
    cancel_request();
});
```

---

## Components

### Function Component (preferred)

```rust
#[derive(Props)]
struct ButtonProps {
    label: String,                         // required prop
    #[prop(default)]
    disabled: bool,                        // optional prop (defaults to false)
    on_click: Option<EventHandler>,        // optional event handler
}

#[component]
fn Button(props: ButtonProps) -> impl IntoView {
    template! {
        <button disabled={props.disabled} on:click={props.on_click}>
            {props.label}
        </button>
    }
}
```

### Trait Component (advanced — lifecycle control)

```rust
impl Component for Counter {
    type Props = CounterProps;

    fn create(props: Self::Props) -> Self { ... }
    fn render(&self) -> impl IntoView { ... }
    fn destroy(&mut self) { ... }
}
```

---

## Template Syntax

### Elements

```rust
template! {
    <div class="container" id="main">
        <h1>{"Title"}</h1>
        <p>{some_string.get()}</p>
        <button>{"Text"}</button>
    </div>
}
```

### Dynamic attributes

```rust
template! {
    <div class={class_name.get()}        // dynamic class
         style={format!("color: {}", color.get())}  // dynamic style
         data-id={item.id.to_string()}>  // data attributes
        {item.name}
    </div>
}
```

### Event handlers

```rust
template! {
    <button on:click=move |_| { count.set(count.get() + 1); }>
        {"Click"}
    </button>
    <input on:input=move |e| { name.set(e.target_value()); } />
}
```

**Event handler rules:**
- Always use `move |event| { ... }` — the `move` keyword is required
- Event types: `on:click`, `on:input`, `on:focus`, `on:blur`, `on:submit`, `on:keydown`, `on:keyup`, `on:mouseenter`, `on:mouseleave`, `on:scroll`, `on:resize`
- The event parameter is a typed `Event` — use `e.target_value()`, `e.target_checked()`, etc.

### Conditional rendering

```rust
template! {
    <div>
        {if show.get() {
            template! { <p>{"Visible"}</p> }.into_view()
        } else {
            template! { <p>{"Hidden"}</p> }.into_view()
        }}
    </div>
}
```

Or use the `Show` component:

```rust
template! {
    <Show when={move || count.get() > 0}>
        {"Count is positive"}
    </Show>
}
```

### List rendering

```rust
template! {
    <For each={items.get()} key=|item| item.id>
        {move |item| template! {
            <div>{item.name}</div>
        }}
    </For>
}
```

**Key rules:**
- `key` must return a unique, stable value per item
- The closure receives each item by value
- Always use `For` for dynamic lists — never manually map

### Component composition

```rust
template! {
    <Card>
        <CardHeader>{"Title"}</CardHeader>
        <CardBody>
            <p>{"Content"}</p>
        </CardBody>
    </Card>
}
```

### Spread attributes

```rust
template! {
    <div ..{extra_attrs}>
        {"Content"}
    </div>
}
```

### Slots / children

```rust
#[derive(Props)]
struct CardProps {
    #[prop(children)]
    children: Children,
}

#[component]
fn Card(props: CardProps) -> impl IntoView {
    template! {
        <div class="card">
            {props.children()}
        </div>
    }
}
```

---

## Built-in Components

| Component | Purpose |
|-----------|---------|
| `<Show when={bool}>` | Conditional rendering |
| `<For each={iter} key={fn}>` | Keyed list rendering |
| `<Suspense fallback={view}>` | Async boundary |
| `<ErrorBoundary fallback={fn}>` | Error catching |
| `<Fragment>` | Group without wrapper element |
| `<KeepAlive>` | Cache component state |

---

## Styling

### Scoped CSS

```rust
style! {
    .button {
        background: blue;
        color: white;
        padding: 8px 16px;
    }
    .button:hover {
        background: darkblue;
    }
}

#[component]
fn MyButton() -> impl IntoView {
    template! {
        <button class="button">{"Click"}</button>
    }
}
```

### CSS-in-Rust (typed)

```rust
let styles = css! {
    color: Color::Red,
    padding: Px(16),
    margin: Auto,
};
```

### Tailwind

```rust
template! {
    <div class="flex items-center justify-center p-4">
        {"Centered content"}
    </div>
}
```

---

## Routing

```rust
use rye_router::{Router, Route, Params};

#[derive(Params)]
struct UserParams {
    id: u32,
}

#[component]
fn App() -> impl IntoView {
    template! {
        <Router>
            <Route path="/" view=Home />
            <Route path="/users/:id" view=UserPage />
            <Route path="*" view=NotFound />
        </Router>
    }
}

#[component]
fn UserPage(props: UserPageProps) -> impl IntoView {
    let params = use_params::<UserParams>();
    template! {
        <h1>{format!("User #{}", params.get().id)}</h1>
    }
}
```

---

## Forms

```rust
use rye_forms::{Form, use_field, validators};

#[derive(Form)]
struct LoginForm {
    #[field(validate = validators::email())]
    email: String,
    #[field(validate = validators::min_len(8))]
    password: String,
}

#[component]
fn Login() -> impl IntoView {
    let form = use_form::<LoginForm>();

    template! {
        <form on:submit=move |e| { e.prevent_default(); form.submit(); }>
            <input on:input=move |ev| form.email.set(ev.target_value()) />
            <ErrorDisplay field={form.email} />
            <input type="password" on:input=move |ev| form.password.set(ev.target_value()) />
            <ErrorDisplay field={form.password} />
            <button type="submit" disabled={form.is_submitting()}>
                {"Submit"}
            </button>
        </form>
    }
}
```

---

## i18n

```rust
use rye_i18n::t;

#[component]
fn Greeting() -> impl IntoView {
    template! {
        <h1>{t!("welcome", name = "World")}</h1>
    }
}
```

---

## Testing

```rust
use rye_testing::{TestRenderer, assert_html, fire_event};

#[test]
fn counter_increments() {
    let renderer = TestRenderer::new(|| {
        let count = Signal::new(0);
        template! {
            <button on:click=move |_| count.set(count.get() + 1)>
                {count.get()}
            </button>
        }
    });

    assert_html!(renderer, "<button>0</button>");
    fire_event!(renderer, "click", "button");
    assert_html!(renderer, "<button>1</button>");
}
```

---

## Rules (Follow These Exactly)

1. **Always use `.get()` to read signals** — never access the inner value directly
2. **Always use `.set()` to write signals** — never mutate through a reference
3. **Event handlers need `move` keyword** — `move |e| { ... }` not `|e| { ... }`
4. **Component names are PascalCase** — `Button`, not `button`
5. **HTML elements are lowercase in templates** — `<div>`, not `<Div>`
6. **Props are passed as attributes** — `prop_name={value}` in templates
7. **Events use `on:event_name`** — `on:click`, `on:input`, `on:focus`
8. **Always import the prelude** — `use rye::prelude::*;` at the top of every file
9. **Templates return `impl IntoView`** — components return `impl IntoView`
10. **Use `For` for lists** — never manually map over collections in templates
11. **Use `Show` for conditionals** — or `if/else` returning `.into_view()`
12. **Signal closures capture by move** — `move || count.get()` not `|| count.get()`

---

## Common Mistakes (Avoid These)

| Mistake | Wrong | Correct |
|---------|-------|---------|
| Reading signal without `.get()` | `{count}` | `{count.get()}` |
| Writing signal without `.set()` | `count = 5` | `count.set(5)` |
| Missing `move` in handler | `on:click=\|_\| {...}` | `on:click=move \|_\| {...}` |
| Lowercase component | `<button>` (HTML) vs `<Button>` (component) | Use PascalCase for components |
| Missing prelude import | `Signal::new(...)` fails | `use rye::prelude::*;` first |
| Forgetting `.into_view()` | `if x { template!{...} }` | `if x { template!{...}.into_view() } else { ... }` |
| Not using `For` for lists | `items.get().map(\|i\| ...)` | `<For each={items.get()} key=\|i\| i.id>` |
| Missing `key` on `For` | `<For each={items.get()}>` | `<For each={items.get()} key=\|i\| i.id>` |

---

## Error Codes

If you get a compile error with code `R0XX`, run `rpg explain R0XX` for a detailed
explanation and correct usage example. Common codes:

| Code | Meaning |
|------|---------|
| R001 | Invalid HTML element in template |
| R002 | Unknown attribute |
| R003 | Unknown component (check spelling/imports) |
| R004 | Missing required prop |
| R005 | Wrong prop type |
| R006 | Invalid event name |
| R800 | Wrong prop type (expected String, got &str — use .to_string()) |
| R801 | Missing `move` keyword in event handler closure |
| R802 | Signal read without `.get()` |
| R803 | Signal write without `.set()` |
| R804 | Component name not capitalized |
| R805 | Missing `#[component]` macro |
| R806 | `use_effect` for derived state — use `Memo` instead |
| R807 | Unnecessary `.clone()` — rye props are borrowed |
| R808 | Prop drilling — use `provide_context`/`use_context` |
| R809 | Raw async spawn — use `Resource::new()` instead |
| R810 | `template!` outside `#[component]` function |

For step-by-step recovery plans, run `rpg explain R802 --json` or use the MCP tool `rye_get_recovery_plan`.

---

*This spec is the complete reference for writing rye code. Any LLM that reads this
file can generate correct rye components, templates, and tests without additional
documentation.*

---

## CLI Commands for AI Agents

These CLI commands help AI agents work with rye projects:

| Command | Purpose |
|---------|---------|
| `rpg explain R003` | Look up an error code with detailed fix |
| `rpg explain --list` | List all error codes |
| `rpg explain --search "signal"` | Search errors by keyword |
| `rpg explain R802 --json` | Machine-readable error explanation |
| `rpg scaffold component Button --props label:String --style --test` | Generate component boilerplate |
| `rpg scaffold page About --route /about` | Generate page boilerplate |
| `rpg scaffold store UserStore --fields name:String` | Generate store boilerplate |
| `rpg scaffold action GetUser --params id:u32` | Generate server action boilerplate |
| `rpg test --generate src/components/button.rs` | Generate test scaffolding |
| `rpg test --generate --all` | Generate tests for all components |
| `rpg lint --ai src/components/button.rs` | AI-aware linter for common mistakes |
| `rpg lint --ai --dir src/components` | Lint an entire directory |
| `rpg lint --ai --json src/components/button.rs` | Lint output as JSON |
| `rpg doctor` | Project health check |
| `rpg doctor --json` | Health check as JSON |
| `rpg playground` | Launch web-based code editor with live preview |
| `rpg profile` | Performance profiler with flamegraph output |
| `rpg bundle` | Bundle size analyzer with tree map visualization |
| `rpg init` | Interactive project wizard |
| `rpg generate openapi spec.yaml` | Generate API client + server actions from OpenAPI |
| `rpg generate schema schema.sql` | Generate CRUD components from database schema |
| `rpg monorepo init` | Initialize monorepo workspace |
| `rpg monorepo build` | Build all workspace members |
| `rpg publish` | Publish component library to rye registry |
| `rpg theme create dark` | Create a new design theme |
| `rpg theme export --format=css` | Export theme as CSS custom properties |
| `rpg theme diff light dark` | Diff two themes |
| `rpg docs` | Start local documentation server |
| `rpg ci github` | Generate GitHub Actions CI/CD config |
| `rpg ci gitlab` | Generate GitLab CI config |

---

## MCP Server Tools

The `rye-mcp` crate exposes 16 tools over the Model Context Protocol (JSON-RPC 2.0 over stdio).
Any MCP-compatible AI agent (Claude, Cursor, Windsurf, Copilot) can call these directly:

| Tool | Purpose |
|------|---------|
| `rye_explain_error` | Get detailed explanation for an error code |
| `rye_list_error_codes` | List all error codes with optional category filter |
| `rye_search_error_codes` | Search error codes by keyword |
| `rye_get_recovery_plan` | Get step-by-step recovery plan for an error code |
| `rye_list_components` | List all registered components |
| `rye_find_component` | Find a specific component by name |
| `rye_search_components` | Search components by keyword |
| `rye_nl_search_components` | Natural language component search |
| `rye_list_prompt_templates` | List AI prompt templates for common patterns |
| `rye_get_prompt_template` | Get a specific prompt template with placeholders filled |
| `rye_review_code` | AI code review — checks for common mistakes |
| `rye_get_context` | Get optimized context package within token budget |
| `rye_get_focused_context` | Get focused context for a specific query |
| `rye_scaffold_component` | Generate component code (no filesystem I/O) |
| `rye_scaffold_test` | Generate test code from source (no filesystem I/O) |
| `rye_component_usage_stats` | Get usage analytics for components |

---

## AI Prompt Templates

Pre-built prompt templates for common rye patterns (available via `rye_list_prompt_templates` MCP tool):

| Template | Category | Description |
|----------|----------|-------------|
| `component` | ui | Basic component with props and template |
| `form` | ui | Form with validation and error display |
| `list` | ui | List rendering with For and key |
| `page` | routing | Route page with params and data loading |
| `store` | state | Global store with GlobalSignal |
| `action` | server | Server action with error handling |
| `island` | islands | Interactive island component |
| `crud` | fullstack | Full CRUD page with list, create, edit, delete |
| `modal` | ui | Modal dialog with open/close state |
| `auth` | fullstack | Authentication form with login/logout |

---

## Ecosystem & Interop

### React Component Wrapping

```rust
// Wrap a React component for use in rye templates
wrap_react_component!(Button, "./Button", {
    label: "text",
    onclick: "onClick",
});
```

### Vue Component Wrapping

```rust
// Wrap a Vue SFC for use in rye templates
wrap_vue_component!(Card, "./Card.vue", {
    title: "title",
    close: "onClose",
});
```

### Tailwind 4.0

Zero-config Tailwind 4.0 (Oxide engine) with arbitrary values, container queries, and 3D transforms. Utility classes are processed at build time.

### WebGPU Compute Shaders

```rust
use rye::interop::ComputeShader;

let shader = ComputeShader::new("process", "// shader body")
    .with_workgroup_size(8, 8, 1)
    .add_input(ComputeBinding { name: "data", binding_type: ComputeBindingType::StorageBuffer, size: 1024 })
    .add_output(ComputeBinding { name: "result", binding_type: ComputeBindingType::StorageBuffer, size: 1024 });
```

### Figma Design-to-Code

Figma plugin exports designs directly to rye component code — layout, text, images, and interactive states.

---

## Advanced Testing

### Integration Testing

```rust
use rye_testing::{MockSsrServer, TestRequest, IntegrationTestCase, IntegrationTestRunner};

let mut server = MockSsrServer::new();
server.route("/", home_handler);

let mut runner = IntegrationTestRunner::new(server);
runner.add_test(
    IntegrationTestCase::new("home_page", TestRequest::get("/"))
        .expect_contains("Welcome"),
);
let (passed, failed) = runner.run_all();
```

### Component Contract Tests

```rust
use rye_testing::ComponentContract;

let contract = ComponentContract::new("Button")
    .add_prop("label", "String", true)
    .add_event("click")
    .add_slot("default");

let new_contract = ComponentContract::new("Button")
    .add_prop("label", "String", true)
    .add_prop("color", "String", false)
    .add_event("click")
    .add_slot("default");

assert!(contract.is_compatible_with(&new_contract)); // No breaking changes
```

### Semantic Snapshot Testing

```rust
use rye_testing::SemanticNode;

let expected = SemanticNode::element("div")
    .add_prop("class", "container")
    .add_child(SemanticNode::text("Hello"));

let actual = SemanticNode::element("div")
    .add_prop("class", "container")
    .add_child(SemanticNode::text("Hello"));

assert_eq!(actual.diff(&expected).len(), 0); // No differences
```

### Signal Update Ordering

```rust
use rye_testing::SignalGraph;

let mut graph = SignalGraph::new();
graph.add_dependency("c", "a");
graph.add_dependency("c", "b");

let order = graph.topological_order(); // a, b, c (or b, a, c)
```
