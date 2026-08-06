//! # rye-demo
//!
//! A polished multi-page demo app proving rye runs in a browser via WASM.
//!
//! Pages:
//! - Dashboard — stat cards, progress bars, activity feed
//! - Counter — interactive counter with increment/decrement
//! - Todo — add/remove todos with reactive list
//! - Components — showcase of rye-ui components (Badge, Switch, Progress, Card)
//!
//! ## Running
//!
//! ```sh
//! wasm-pack build crates/rye-demo --target web --out-dir www/pkg --no-opt
//! # Then serve crates/rye-demo/www/ with any static server
//! ```

use rye_core::Element;
use rye_core::template::{Template, TemplateNode, ReactiveFn, ReactiveListFn, SharedEventHandler, shared_event_handler};
use rye_signals::Signal;
use rye_ui::{
    Alert, AlertProps, AlertVariant,
    Avatar, AvatarProps,
    Badge, BadgeProps,
    Breadcrumb, BreadcrumbProps, BreadcrumbItem,
    Button, ButtonProps,
    Card, CardProps,
    Checkbox, CheckboxProps,
    CircularProgress, CircularProgressProps,
    CodeBlock, CodeBlockProps, CodeLanguage,
    Divider, DividerProps,
    EmptyState, EmptyStateProps,
    Input, InputProps,
    Label, LabelProps,
    Link, LinkProps,
    List, ListProps, ListItem, ListVariant,
    Notification, NotificationProps, NotificationVariant,
    Progress, ProgressProps,
    RadioGroup, RadioGroupProps,
    Select, SelectProps, SelectOption,
    Skeleton, SkeletonProps, SkeletonShape,
    Spinner, SpinnerProps,
    Stat, StatProps, StatTrend,
    Tag, TagProps,
    Textarea, TextareaProps,
    Timeline, TimelineProps, TimelineItem, TimelineVariant,
    Variant,
};
use rye_ui::theme::Size;
use rye_ui::switch::SwitchSize;
use std::rc::Rc;

// ─── Helper: Element → Vec<Template> ─────────────────────────────────────────

/// Flatten an Element into a Vec<Template> for use as children of new_element.
fn to_templates(el: Element) -> Vec<Template> {
    match el {
        Element::Template(t) => vec![t],
        Element::Fragment(els) => {
            els.into_iter().flat_map(to_templates).collect()
        }
        Element::None => vec![],
        Element::Component(_) => vec![],
    }
}

/// Combine multiple Templates into one by flattening their nodes.
fn combine(templates: Vec<Template>) -> Template {
    let nodes: Vec<TemplateNode> = templates
        .into_iter()
        .flat_map(|t| t.nodes)
        .collect();
    Template::new(nodes)
}

// ─── Todo item model ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct TodoItem {
    text: String,
    done: bool,
}

// ─── WASM event helpers ──────────────────────────────────────────────────────

/// Simple random number generator (works on all targets).
fn rye_rand() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = Math)]
            fn random() -> f64;
        }
        random()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as f64 / 1_000_000_000.0)
            .unwrap_or(0.5);
        nanos
    }
}

/// Extract the value from an input event (WASM only).
#[cfg(target_arch = "wasm32")]
fn get_input_value(event: &dyn std::any::Any) -> String {
    use wasm_bindgen::JsCast;
    if let Some(e) = event.downcast_ref::<web_sys::Event>() {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                return input.value();
            }
        }
    }
    String::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_input_value(_event: &dyn std::any::Any) -> String {
    String::new()
}

// ─── Page routing ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Counter,
    Todo,
    Components,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Counter => "Counter",
            Self::Todo => "Todo List",
            Self::Components => "Components",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "\u{25A3}",
            Self::Counter => "\u{2295}",
            Self::Todo => "\u{2713}",
            Self::Components => "\u{2699}",
        }
    }
}

// ─── Sidebar ────────────────────────────────────────────────────────────────

fn sidebar(active: &Signal<Page>, dark_mode: &Signal<bool>) -> Template {
    let pages = [Page::Dashboard, Page::Counter, Page::Todo, Page::Components];

    let nav_children: Vec<Template> = pages
        .iter()
        .map(|&page| {
            let page_signal = active.clone();
            let active_clone = active.clone();
            let label = page.label();
            let icon = page.icon();

            let handler: SharedEventHandler = shared_event_handler(move |_| {
                page_signal.set(page);
            });

            // Reactive class — updates when active signal changes
            let class_fn: ReactiveFn = Rc::new(move || {
                if active_clone.get() == page {
                    "nav-item active".to_string()
                } else {
                    "nav-item".to_string()
                }
            });

            Template::new_element_reactive(
                "div",
                Vec::new(),
                vec![("class".to_string(), class_fn)],
                vec![("click".to_string(), handler)],
                vec![
                    Template::new_element(
                        "span",
                        vec![("class".to_string(), "nav-icon".to_string())],
                        Vec::new(),
                        vec![Template::text(icon)],
                    ),
                    Template::new_element(
                        "span",
                        Vec::new(),
                        Vec::new(),
                        vec![Template::text(label)],
                    ),
                ],
            )
        })
        .collect();

    let header = Template::new_element(
        "div",
        vec![("class".to_string(), "sidebar-header".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h1", Vec::new(), Vec::new(), vec![Template::text("rye")]),
            Template::new_element(
                "span",
                vec![("class".to_string(), "sidebar-sub".to_string())],
                Vec::new(),
                vec![Template::text("WASM Demo")],
            ),
        ],
    );

    let nav = Template::new_element(
        "nav",
        vec![("class".to_string(), "sidebar-nav".to_string())],
        Vec::new(),
        nav_children,
    );

    let footer = Template::new_element(
        "div",
        vec![("class".to_string(), "sidebar-footer".to_string())],
        Vec::new(),
        vec![
            Template::new_element(
                "span",
                Vec::new(),
                Vec::new(),
                vec![
                    Template::new_element(
                        "span",
                        vec![("class".to_string(), "sidebar-dot".to_string())],
                        Vec::new(),
                        vec![Template::text("\u{2B24}")],
                    ),
                    Template::text(" WASM"),
                ],
            ),
            {
                let dm = dark_mode.clone();
                let toggle_handler: SharedEventHandler = shared_event_handler(move |_| {
                    dm.set(!dm.get());
                });
                let dm_icon = dark_mode.clone();
                let icon_fn: ReactiveFn = Rc::new(move || {
                    if dm_icon.get() { "\u{1F319}".to_string() } else { "\u{2600}\u{FE0F}".to_string() }
                });
                let dm_label = dark_mode.clone();
                let label_fn: ReactiveFn = Rc::new(move || {
                    if dm_label.get() { "Dark".to_string() } else { "Light".to_string() }
                });
                Template::new_element(
                    "div",
                    vec![("class".to_string(), "theme-toggle".to_string())],
                    vec![("click".to_string(), toggle_handler)],
                    vec![
                        Template::new(vec![TemplateNode::Reactive(icon_fn)]),
                        Template::new(vec![TemplateNode::Reactive(label_fn)]),
                    ],
                )
            },
        ],
    );

    Template::new_element(
        "aside",
        vec![("class".to_string(), "sidebar".to_string())],
        Vec::new(),
        vec![header, nav, footer],
    )
}

// ─── Page wrapper (show/hide via reactive style) ─────────────────────────────

fn page_wrapper(page: Page, active: &Signal<Page>, content: Element) -> Template {
    let active_clone = active.clone();
    let style_fn: ReactiveFn = Rc::new(move || {
        if active_clone.get() == page {
            "display:block".to_string()
        } else {
            "display:none".to_string()
        }
    });

    Template::new_element_reactive(
        "div",
        vec![("class".to_string(), "page-wrapper".to_string())],
        vec![("style".to_string(), style_fn)],
        Vec::new(),
        to_templates(content),
    )
}

// ─── Dashboard page ─────────────────────────────────────────────────────────

fn dashboard_page() -> Element {
    // Live-updating progress signals
    let cpu = Signal::new(42.0f64);
    let mem = Signal::new(68.0f64);
    let disk = Signal::new(23.0f64);
    let net = Signal::new(91.0f64);

    // Simulate live updates via setInterval-like pattern using effects
    // We'll use a simple approach: each progress bar gets a reactive value
    let make_progress_card = |label: &str, color: &str, signal: &Signal<f64>| -> Template {
        let color = color.to_string();
        let val_signal = signal.clone();
        let val_fn: ReactiveFn = Rc::new(move || {
            format!("{:.0}%", val_signal.get())
        });

        let pct_signal = signal.clone();
        let pct_fn: ReactiveFn = Rc::new(move || {
            format!("width:{}%;height:100%;background:{};border-radius:inherit;transition:width 0.5s ease;",
                pct_signal.get().clamp(0.0, 100.0), color)
        });

        let bar_track = Template::new_element(
            "div",
            vec![("style".to_string(), "width:100%;height:6px;background:var(--border);border-radius:3px;overflow:hidden;".to_string())],
            Vec::new(),
            vec![
                Template::new_element_reactive(
                    "div",
                    Vec::new(),
                    vec![("style".to_string(), pct_fn)],
                    Vec::new(),
                    Vec::new(),
                ),
            ],
        );

        Template::new_element(
            "div",
            vec![("class".to_string(), "progress-card".to_string())],
            Vec::new(),
            vec![
                Template::new_element(
                    "div",
                    vec![("class".to_string(), "progress-card-header".to_string())],
                    Vec::new(),
                    vec![
                        Template::new_element("span", Vec::new(), Vec::new(), vec![Template::text(label)]),
                        Template::new_element_reactive(
                            "span",
                            vec![("class".to_string(), "progress-value".to_string())],
                            Vec::new(),
                            Vec::new(),
                            vec![Template::new(vec![TemplateNode::Reactive(val_fn)])],
                        ),
                    ],
                ),
                bar_track,
            ],
        )
    };

    let progress_cards = vec![
        make_progress_card("CPU Usage", "#3b82f6", &cpu),
        make_progress_card("Memory", "#f59e0b", &mem),
        make_progress_card("Disk", "#22c55e", &disk),
        make_progress_card("Network", "#ef4444", &net),
    ];

    // Interactive refresh button that randomizes values
    let cpu_r = cpu.clone();
    let mem_r = mem.clone();
    let disk_r = disk.clone();
    let net_r = net.clone();
    let refresh_btn = Button::render(
        ButtonProps::default()
            .label("\u{1F504} Refresh Metrics")
            .variant(Variant::Outline)
            .on_click(move |_| {
                cpu_r.set(10.0 + (rye_rand() * 80.0));
                mem_r.set(30.0 + (rye_rand() * 60.0));
                disk_r.set(5.0 + (rye_rand() * 40.0));
                net_r.set(50.0 + (rye_rand() * 48.0));
            }),
    );

    let stats = vec![
        Stat::render(
            StatProps::default()
                .label("Active Users")
                .value("12,847")
                .trend(StatTrend::Up)
                .trend_value("+12.5%")
                .icon("\u{1F465}"),
        ),
        Stat::render(
            StatProps::default()
                .label("Revenue")
                .value("$42,580")
                .trend(StatTrend::Up)
                .trend_value("+8.2%")
                .icon("\u{1F4B0}"),
        ),
        Stat::render(
            StatProps::default()
                .label("Sessions")
                .value("3,291")
                .trend(StatTrend::Down)
                .trend_value("-3.1%")
                .icon("\u{1F4CA}"),
        ),
        Stat::render(
            StatProps::default()
                .label("Uptime")
                .value("99.9%")
                .trend(StatTrend::Neutral)
                .trend_value("stable")
                .icon("\u{26A1}"),
        ),
    ];

    let activity = vec![
        ("\u{2705}", "Deploy succeeded", "2 min ago"),
        ("\u{1F41E}", "Bug #2841 resolved", "14 min ago"),
        ("\u{1F4E7}", "3 new messages", "1 hour ago"),
        ("\u{1F504}", "Cache refreshed", "3 hours ago"),
        ("\u{1F4C1}", "12 files uploaded", "5 hours ago"),
    ];

    let activity_items: Vec<Template> = activity
        .iter()
        .map(|(icon, text, time)| {
            Template::new_element(
                "div",
                vec![("class".to_string(), "activity-item".to_string())],
                Vec::new(),
                vec![
                    Template::new_element(
                        "span",
                        vec![("class".to_string(), "activity-icon".to_string())],
                        Vec::new(),
                        vec![Template::text(icon.to_string())],
                    ),
                    Template::new_element(
                        "div",
                        vec![("class".to_string(), "activity-content".to_string())],
                        Vec::new(),
                        vec![
                            Template::new_element("span", Vec::new(), Vec::new(), vec![Template::text(text.to_string())]),
                            Template::new_element(
                                "span",
                                vec![("class".to_string(), "activity-time".to_string())],
                                Vec::new(),
                                vec![Template::text(time.to_string())],
                            ),
                        ],
                    ),
                ],
            )
        })
        .collect();

    let header = Template::new_element(
        "div",
        vec![("class".to_string(), "page-header".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h1", Vec::new(), Vec::new(), vec![Template::text("Dashboard")]),
            Template::new_element("p", Vec::new(), Vec::new(), vec![Template::text("Real-time metrics powered by rye signals")]),
        ],
    );

    let stats_grid = Template::new_element(
        "div",
        vec![("class".to_string(), "stats-grid".to_string())],
        Vec::new(),
        to_templates(Element::Fragment(stats)),
    );

    let stats_title = Template::new_element(
        "div",
        vec![("class".to_string(), "section-title".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h2", Vec::new(), Vec::new(), vec![Template::text("System Health")]),
            Template::new_element("div",
                vec![("style".to_string(), "margin-left:auto;".to_string())],
                Vec::new(),
                to_templates(refresh_btn),
            ),
        ],
    );

    let progress_grid = Template::new_element(
        "div",
        vec![("class".to_string(), "progress-grid".to_string())],
        Vec::new(),
        progress_cards,
    );

    let activity_title = Template::new_element(
        "div",
        vec![("class".to_string(), "section-title".to_string())],
        Vec::new(),
        vec![Template::new_element("h2", Vec::new(), Vec::new(), vec![Template::text("Recent Activity")])],
    );

    let activity_list = Template::new_element(
        "div",
        vec![("class".to_string(), "activity-list".to_string())],
        Vec::new(),
        activity_items,
    );

    Element::Template(combine(vec![
        header,
        stats_grid,
        stats_title,
        progress_grid,
        activity_title,
        activity_list,
    ]))
}

// ─── Counter page ───────────────────────────────────────────────────────────

fn counter_page() -> Element {
    let count = Signal::new(0i32);
    let display = count.clone();
    let inc = count.clone();
    let dec = count.clone();
    let reset = count.clone();

    // Reactive class — updates when count changes
    let class_fn: ReactiveFn = {
        let c = count.clone();
        Rc::new(move || {
            let v = c.get();
            if v > 0 {
                "counter-value positive".to_string()
            } else if v < 0 {
                "counter-value negative".to_string()
            } else {
                "counter-value".to_string()
            }
        })
    };

    let header = Template::new_element(
        "div",
        vec![("class".to_string(), "page-header".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h1", Vec::new(), Vec::new(), vec![Template::text("Counter")]),
            Template::new_element("p", Vec::new(), Vec::new(), vec![Template::text("Fine-grained reactivity \u{2014} only the value updates, no VDOM diff")]),
        ],
    );

    let display_fn: ReactiveFn = {
        let d = display.clone();
        Rc::new(move || d.get().to_string())
    };

    let counter_display = Template::new_element_reactive(
        "div",
        Vec::new(),
        vec![("class".to_string(), class_fn)],
        Vec::new(),
        vec![Template::new(vec![TemplateNode::Reactive(display_fn)])],
    );

    let inc_handler: SharedEventHandler = shared_event_handler(move |_| {
        inc.set(inc.get() + 1);
    });
    let dec_handler: SharedEventHandler = shared_event_handler(move |_| {
        let c = dec.get();
        dec.set(c - 1);
    });
    let reset_handler: SharedEventHandler = shared_event_handler(move |_| {
        reset.set(0);
    });

    let buttons = Template::new_element(
        "div",
        vec![("class".to_string(), "counter-buttons".to_string())],
        Vec::new(),
        vec![
            Template::new_element(
                "button",
                vec![("class".to_string(), "btn-primary".to_string())],
                vec![("click".to_string(), inc_handler)],
                vec![Template::text("+ Increment")],
            ),
            Template::new_element(
                "button",
                vec![("class".to_string(), "btn-secondary".to_string())],
                vec![("click".to_string(), dec_handler)],
                vec![Template::text("\u{2212} Decrement")],
            ),
            Template::new_element(
                "button",
                vec![("class".to_string(), "btn-ghost".to_string())],
                vec![("click".to_string(), reset_handler)],
                vec![Template::text("Reset")],
            ),
        ],
    );

    let card = Template::new_element(
        "div",
        vec![("class".to_string(), "counter-card".to_string())],
        Vec::new(),
        vec![counter_display, buttons],
    );

    Element::Template(combine(vec![header, card]))
}

// ─── Todo page ──────────────────────────────────────────────────────────────

fn todo_page() -> Element {
    let todos = Signal::new(vec![
        TodoItem { text: "Build WASM demo".to_string(), done: false },
        TodoItem { text: "Test on mobile".to_string(), done: false },
        TodoItem { text: "Ship to production".to_string(), done: false },
    ]);
    let todo_input = Signal::new(String::new());
    let next_id = Signal::new(100usize);

    // ── Handlers ────────────────────────────────────────────────────────────

    let input_signal = todo_input.clone();
    let input_handler: SharedEventHandler = shared_event_handler(move |event| {
        input_signal.set(get_input_value(event));
    });

    let add_todos = todos.clone();
    let add_input = todo_input.clone();
    let add_next_id = next_id.clone();
    let add_btn = Button::render(
        ButtonProps::default()
            .label("+ Add")
            .variant(Variant::Primary)
            .on_click(move |_| {
                let text = add_input.get().trim().to_string();
                if !text.is_empty() {
                    let mut list = add_todos.get();
                    list.push(TodoItem { text, done: false });
                    add_todos.set(list);
                    add_input.set(String::new());
                    add_next_id.set(add_next_id.get() + 1);
                }
            }),
    );

    let keydown_input = todo_input.clone();
    let keydown_todos = todos.clone();
    let keydown_next_id = next_id.clone();
    let keydown_handler: SharedEventHandler = shared_event_handler(move |event| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(e) = event.downcast_ref::<web_sys::Event>() {
                if let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                    if ke.key() == "Enter" {
                        let text = keydown_input.get().trim().to_string();
                        if !text.is_empty() {
                            let mut list = keydown_todos.get();
                            list.push(TodoItem { text, done: false });
                            keydown_todos.set(list);
                            keydown_input.set(String::new());
                            keydown_next_id.set(keydown_next_id.get() + 1);
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        { let _ = (&event, &keydown_input, &keydown_todos, &keydown_next_id); }
    });

    // ── Reactive list with keyed reconciliation ─────────────────────────────

    let list_todos = todos.clone();
    let todo_list_fn: ReactiveListFn = Rc::new(move || {
        let list = list_todos.get();
        list.iter()
            .enumerate()
            .map(|(i, _item)| {
                let key = i;

                let check_todos = list_todos.clone();
                let check_fn: ReactiveFn = Rc::new(move || {
                    let l = check_todos.get();
                    if i < l.len() && l[i].done {
                        "\u{2713}".to_string()
                    } else {
                        "\u{25CB}".to_string()
                    }
                });

                let check_class_todos = list_todos.clone();
                let check_class_fn: ReactiveFn = Rc::new(move || {
                    let l = check_class_todos.get();
                    if i < l.len() && l[i].done {
                        "todo-check done".to_string()
                    } else {
                        "todo-check".to_string()
                    }
                });

                let text_todos = list_todos.clone();
                let text_fn: ReactiveFn = Rc::new(move || {
                    let l = text_todos.get();
                    if i < l.len() {
                        l[i].text.clone()
                    } else {
                        String::new()
                    }
                });

                let text_class_todos = list_todos.clone();
                let text_class_fn: ReactiveFn = Rc::new(move || {
                    let l = text_class_todos.get();
                    if i < l.len() && l[i].done {
                        "todo-text done".to_string()
                    } else {
                        "todo-text".to_string()
                    }
                });

                let toggle_todos = list_todos.clone();
                let toggle_handler: SharedEventHandler = shared_event_handler(move |_| {
                    let mut l = toggle_todos.get();
                    if i < l.len() {
                        l[i].done = !l[i].done;
                        toggle_todos.set(l);
                    }
                });

                let remove_todos = list_todos.clone();
                let remove_handler: SharedEventHandler = shared_event_handler(move |_| {
                    let mut l = remove_todos.get();
                    if i < l.len() {
                        l.remove(i);
                        remove_todos.set(l);
                    }
                });

                let item_template = Template::new_element(
                    "div",
                    vec![("class".to_string(), "todo-item".to_string())],
                    Vec::new(),
                    vec![
                        Template::new_element_reactive(
                            "span",
                            Vec::new(),
                            vec![("class".to_string(), check_class_fn)],
                            vec![("click".to_string(), toggle_handler)],
                            vec![Template::new(vec![TemplateNode::Reactive(check_fn)])],
                        ),
                        Template::new_element_reactive(
                            "span",
                            Vec::new(),
                            vec![("class".to_string(), text_class_fn)],
                            Vec::new(),
                            vec![Template::new(vec![TemplateNode::Reactive(text_fn)])],
                        ),
                        Template::new_element(
                            "button",
                            vec![("class".to_string(), "todo-remove".to_string())],
                            vec![("click".to_string(), remove_handler)],
                            vec![Template::text("\u{00D7}")],
                        ),
                    ],
                );

                (key, item_template)
            })
            .collect()
    });

    // ── Layout ──────────────────────────────────────────────────────────────

    let header = Template::new_element(
        "div",
        vec![("class".to_string(), "page-header".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h1", Vec::new(), Vec::new(), vec![Template::text("Todo List")]),
            Template::new_element("p", Vec::new(), Vec::new(), vec![Template::text("Add, toggle done, and remove items \u{2014} powered by Signal<Vec<TodoItem>>")]),
        ],
    );

    let input_fn: ReactiveFn = {
        let iv = todo_input.clone();
        Rc::new(move || iv.get())
    };

    let input_field = Template::new_element_reactive(
        "input",
        vec![
            ("class".to_string(), "todo-input".to_string()),
            ("placeholder".to_string(), "What needs to be done?".to_string()),
            ("type".to_string(), "text".to_string()),
        ],
        vec![("value".to_string(), input_fn)],
        vec![
            ("input".to_string(), input_handler),
            ("keydown".to_string(), keydown_handler),
        ],
        Vec::new(),
    );

    let mut input_children = vec![input_field];
    input_children.extend(to_templates(add_btn));
    let input_row = Template::new_element(
        "div",
        vec![("class".to_string(), "todo-input-row".to_string())],
        Vec::new(),
        input_children,
    );

    let count_fn: ReactiveFn = {
        let ct = todos.clone();
        Rc::new(move || {
            let list = ct.get();
            let done = list.iter().filter(|t| t.done).count();
            format!("{} items \u{2014} {} done", list.len(), done)
        })
    };

    let count_display = Template::new_element_reactive(
        "div",
        vec![("class".to_string(), "todo-count".to_string())],
        Vec::new(),
        Vec::new(),
        vec![Template::new(vec![TemplateNode::Reactive(count_fn)])],
    );

    let todo_list = Template::new_element(
        "div",
        vec![("class".to_string(), "todo-list".to_string())],
        Vec::new(),
        vec![Template::new_reactive_list(todo_list_fn)],
    );

    // Reactive empty state — show when list is empty
    let empty_todos = todos.clone();
    let empty_style_fn: ReactiveFn = Rc::new(move || {
        if empty_todos.get().is_empty() {
            "display:block".to_string()
        } else {
            "display:none".to_string()
        }
    });

    let empty_state = Template::new_element_reactive(
        "div",
        vec![("class".to_string(), "todo-empty".to_string())],
        vec![("style".to_string(), empty_style_fn)],
        Vec::new(),
        vec![
            Template::text("\u{1F4DD}"),
            Template::new_element("p", Vec::new(), Vec::new(), vec![Template::text("No todos yet. Add one above!")]),
        ],
    );

    Element::Template(combine(vec![
        header,
        input_row,
        count_display,
        todo_list,
        empty_state,
    ]))
}

// ─── Interactive switch helper ───────────────────────────────────────────────

fn interactive_switch(label: &str, signal: &Signal<bool>, size: SwitchSize) -> Template {
    let (w, h, knob) = size.dimensions();

    let track_toggled = signal.clone();
    let track_fn: ReactiveFn = Rc::new(move || {
        let checked = track_toggled.get();
        let bg = if checked { "var(--rye-primary)" } else { "var(--rye-input-border)" };
        format!(
            "width:{}px;height:{}px;border-radius:{}px;background:{};position:relative;\
             transition:background 0.2s;flex-shrink:0;",
            w, h, h / 2, bg,
        )
    });

    let knob_toggled = signal.clone();
    let knob_fn: ReactiveFn = Rc::new(move || {
        let checked = knob_toggled.get();
        let knob_offset = if checked { w - knob - 2 } else { 2 };
        format!(
            "width:{}px;height:{}px;border-radius:50%;background:var(--rye-bg);\
             position:absolute;top:{}px;left:{}px;transition:left 0.2s;box-shadow:var(--rye-shadow-sm);",
            knob, knob, (h - knob) / 2, knob_offset,
        )
    });

    let toggle_signal = signal.clone();
    let click_handler: SharedEventHandler = shared_event_handler(move |_| {
        toggle_signal.set(!toggle_signal.get());
    });

    let label_text = label.to_string();

    Template::new_element(
        "label",
        vec![("style".to_string(), "display:inline-flex;align-items:center;gap:8px;cursor:pointer;font-size:14px;".to_string())],
        vec![("click".to_string(), click_handler)],
        vec![
            Template::new_element_reactive(
                "span",
                vec![("class".to_string(), "rye-switch-track".to_string())],
                vec![("style".to_string(), track_fn)],
                Vec::new(),
                vec![
                    Template::new_element_reactive(
                        "span",
                        Vec::new(),
                        vec![("style".to_string(), knob_fn)],
                        Vec::new(),
                        Vec::new(),
                    ),
                ],
            ),
            Template::text(&label_text),
        ],
    )
}

// ─── Components showcase page ───────────────────────────────────────────────

fn components_page() -> Element {
    // Interactive switch signals
    let sw1 = Signal::new(true);
    let sw2 = Signal::new(false);
    let sw3 = Signal::new(true);

    let badges = vec![
        Badge::render(BadgeProps::default().text("Primary").variant(Variant::Primary)),
        Badge::render(BadgeProps::default().text("Success").variant(Variant::Success).dot(true)),
        Badge::render(BadgeProps::default().text("Warning").variant(Variant::Warning)),
        Badge::render(BadgeProps::default().text("Danger").variant(Variant::Destructive).dot(true)),
        Badge::render(BadgeProps::default().text("Info").variant(Variant::Info)),
    ];

    let switches = vec![
        interactive_switch("Notifications", &sw1, SwitchSize::Medium),
        interactive_switch("Dark mode", &sw2, SwitchSize::Medium),
        interactive_switch("Auto-save", &sw3, SwitchSize::Large),
    ];

    let buttons = vec![
        Button::render(ButtonProps::default().label("Primary").variant(Variant::Primary)),
        Button::render(ButtonProps::default().label("Secondary").variant(Variant::Secondary)),
        Button::render(ButtonProps::default().label("Outline").variant(Variant::Outline)),
        Button::render(ButtonProps::default().label("Ghost").variant(Variant::Ghost)),
        Button::render(ButtonProps::default().label("Danger").variant(Variant::Destructive)),
        Button::render(ButtonProps::default().label("Success").variant(Variant::Success)),
    ];

    let progress_bars = vec![
        Progress::render(ProgressProps::default().value(25.0).color("#3b82f6")),
        Progress::render(ProgressProps::default().value(50.0).color("#f59e0b")),
        Progress::render(ProgressProps::default().value(75.0).color("#22c55e")),
        Progress::render(ProgressProps::default().value(100.0).color("#8b5cf6")),
    ];

    let card = Card::render(
        CardProps::default()
            .padding("24px")
            .background("#1a1a2e"),
    );

    // ── New component showcases ──────────────────────────────────────────────

    let alerts = vec![
        Alert::render(AlertProps::default().title("Info").message("This is an informational alert.").variant(AlertVariant::Info)),
        Alert::render(AlertProps::default().title("Success").message("Operation completed successfully!").variant(AlertVariant::Success).dismissible(true)),
        Alert::render(AlertProps::default().title("Warning").message("Please review before proceeding.").variant(AlertVariant::Warning)),
        Alert::render(AlertProps::default().title("Error").message("Something went wrong.").variant(AlertVariant::Error).dismissible(true)),
    ];

    let spinners = vec![
        Spinner::render(SpinnerProps::default().size(Size::Small)),
        Spinner::render(SpinnerProps::default().size(Size::Medium).label("Loading...")),
        Spinner::render(SpinnerProps::default().size(Size::Large).color("#f59e0b")),
    ];

    let avatars = vec![
        Avatar::render(AvatarProps::default().name("John Doe").size(Size::Small)),
        Avatar::render(AvatarProps::default().name("Jane Smith").size(Size::Medium)),
        Avatar::render(AvatarProps::default().name("Bob").size(Size::Large)),
    ];

    let tags = vec![
        Tag::render(TagProps::default().text("Rust").variant(Variant::Primary)),
        Tag::render(TagProps::default().text("WASM").variant(Variant::Success)),
        Tag::render(TagProps::default().text("Beta").variant(Variant::Warning).removable(true)),
        Tag::render(TagProps::default().text("Deprecated").variant(Variant::Destructive)),
    ];

    let dividers = vec![
        Divider::render(DividerProps::default()),
        Divider::render(DividerProps::default().color("#3b82f6").thickness("2px")),
    ];

    let inputs = vec![
        Input::render(InputProps::default().placeholder("Enter your name").label("Name")),
        Input::render(InputProps::default().placeholder("Disabled input").disabled(true).label("Disabled")),
        Input::render(InputProps::default().placeholder("Error state").error("This field is required").label("With Error")),
    ];

    let textareas = vec![
        Textarea::render(TextareaProps::default().placeholder("Write something...").label("Message").rows(3)),
    ];

    let selects = vec![
        Select::render(SelectProps::default()
            .label("Framework")
            .placeholder("Select...")
            .options(vec![
                SelectOption { value: "rust".to_string(), label: "Rust".to_string(), disabled: false },
                SelectOption { value: "wasm".to_string(), label: "WASM".to_string(), disabled: false },
            ])),
    ];

    let checkboxes = vec![
        Checkbox::render(CheckboxProps::default().label("Accept terms").checked(true)),
        Checkbox::render(CheckboxProps::default().label("Subscribe to newsletter")),
        Checkbox::render(CheckboxProps::default().label("Indeterminate option").indeterminate(true)),
    ];

    let radios = vec![
        RadioGroup::render(RadioGroupProps::default()
            .name("plan")
            .label("Subscription Plan")
            .options(vec![
                ("free".to_string(), "Free".to_string()),
                ("pro".to_string(), "Pro".to_string()),
                ("enterprise".to_string(), "Enterprise".to_string()),
            ])
            .selected("pro")),
    ];

    let labels = vec![
        Label::render(LabelProps::default().text("Email Address").required(true)),
        Label::render(LabelProps::default().text("Optional Field")),
    ];

    let links = vec![
        Link::render(LinkProps::default().text("Primary Link").href("#")),
        Link::render(LinkProps::default().text("Secondary").href("#").variant(rye_ui::link::LinkVariant::Secondary)),
        Link::render(LinkProps::default().text("Muted").href("#").variant(rye_ui::link::LinkVariant::Muted)),
    ];

    let skeletons = vec![
        Skeleton::render(SkeletonProps::default().shape(SkeletonShape::Text).width("200px").count(3)),
        Skeleton::render(SkeletonProps::default().shape(SkeletonShape::Circle).width("48px").height("48px")),
        Skeleton::render(SkeletonProps::default().shape(SkeletonShape::Rect).width("100%").height("100px")),
    ];

    let circular_progress = vec![
        CircularProgress::render(CircularProgressProps::default().value(35.0).show_percentage(true)),
        CircularProgress::render(CircularProgressProps::default().value(70.0).color("#22c55e").show_percentage(true)),
        CircularProgress::render(CircularProgressProps::default().indeterminate().color("#f59e0b")),
    ];

    let empty_state = vec![
        EmptyState::render(EmptyStateProps::default()
            .icon("\u{1F50D}")
            .title("No results found")
            .description("Try adjusting your search or filters.")
            .action("Clear Filters")),
    ];

    let notifications = vec![
        Notification::render(NotificationProps::default().title("Success").body("Your changes have been saved.").variant(NotificationVariant::Success)),
        Notification::render(NotificationProps::default().title("Warning").body("Your session expires in 5 minutes.").variant(NotificationVariant::Warning)),
        Notification::render(NotificationProps::default().title("Info").body("A new version is available.").variant(NotificationVariant::Info)),
    ];

    let breadcrumb = vec![
        Breadcrumb::render(BreadcrumbProps::default()
            .items(vec![
                BreadcrumbItem::new("Home").href("#"),
                BreadcrumbItem::new("Settings").href("#"),
                BreadcrumbItem::new("Profile").current(),
            ])),
    ];

    let timeline = vec![
        Timeline::render(TimelineProps::default()
            .items(vec![
                TimelineItem::new("Project created").timestamp("Jan 1").variant(TimelineVariant::Success),
                TimelineItem::new("Development started").description("Initial commit").timestamp("Jan 5").variant(TimelineVariant::Info),
                TimelineItem::new("Beta release").timestamp("Feb 10").variant(TimelineVariant::Warning),
                TimelineItem::new("Production deploy").timestamp("Mar 1").variant(TimelineVariant::Default),
            ])),
    ];

    let code_block = vec![
        CodeBlock::render(CodeBlockProps::default()
            .code("fn main() {\n    println!(\"Hello from rye!\");\n}")
            .language(CodeLanguage::Rust)
            .title("main.rs")),
    ];

    let accordion_signal = Signal::new(0usize);
    let accordion_items = ["Section 1: Getting Started", "Section 2: Configuration", "Section 3: Advanced Usage"];
    let accordion_children: Vec<Template> = accordion_items
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let sig = accordion_signal.clone();
            let header_fn: ReactiveFn = Rc::new(move || {
                if sig.get() == i { "\u{25BC}".to_string() } else { "\u{25B6}".to_string() }
            });
            let sig2 = accordion_signal.clone();
            let content_style_fn: ReactiveFn = Rc::new(move || {
                if sig2.get() == i { "display:block;padding:12px 0;color:var(--text-muted);font-size:0.85rem;".to_string() }
                else { "display:none;".to_string() }
            });
            let sig3 = accordion_signal.clone();
            let click_handler: SharedEventHandler = shared_event_handler(move |_| {
                sig3.set(if sig3.get() == i { usize::MAX } else { i });
            });

            Template::new_element(
                "div",
                vec![("style".to_string(), "border-bottom:1px solid var(--border);".to_string())],
                Vec::new(),
                vec![
                    Template::new_element(
                        "div",
                        vec![("style".to_string(), "display:flex;align-items:center;gap:8px;padding:12px 0;cursor:pointer;font-weight:500;font-size:0.9rem;".to_string())],
                        vec![("click".to_string(), click_handler)],
                        vec![
                            Template::new(vec![TemplateNode::Reactive(header_fn)]),
                            Template::text(*title),
                        ],
                    ),
                    Template::new_element_reactive(
                        "div",
                        Vec::new(),
                        vec![("style".to_string(), content_style_fn)],
                        Vec::new(),
                        vec![Template::text("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.")],
                    ),
                ],
            )
        })
        .collect();

    let accordion = vec![Template::new_element(
        "div",
        vec![("style".to_string(), "border:1px solid var(--border);border-radius:var(--radius);padding:0 16px;".to_string())],
        Vec::new(),
        accordion_children,
    )];

    let tab_signal = Signal::new(0usize);
    let tab_labels = ["Overview", "Details", "Settings"];
    let tab_contents = [
        "This is the overview tab. Here you'll find a high-level summary of the project.",
        "The details tab contains in-depth information about the architecture and components.",
        "Configure your preferences here. Settings persist across sessions.",
    ];

    let tab_headers: Vec<Template> = tab_labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let sig = tab_signal.clone();
            let class_fn: ReactiveFn = Rc::new(move || {
                if sig.get() == i { "rye-tab active".to_string() } else { "rye-tab".to_string() }
            });
            let sig2 = tab_signal.clone();
            let click_handler: SharedEventHandler = shared_event_handler(move |_| {
                sig2.set(i);
            });

            Template::new_element_reactive(
                "div",
                Vec::new(),
                vec![("class".to_string(), class_fn)],
                vec![("click".to_string(), click_handler)],
                vec![Template::text(*label)],
            )
        })
        .collect();

    let tab_panels: Vec<Template> = tab_contents
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let sig = tab_signal.clone();
            let style_fn: ReactiveFn = Rc::new(move || {
                if sig.get() == i { "display:block;padding:16px 0;color:var(--text-muted);font-size:0.85rem;".to_string() }
                else { "display:none;".to_string() }
            });
            Template::new_element_reactive(
                "div",
                Vec::new(),
                vec![("style".to_string(), style_fn)],
                Vec::new(),
                vec![Template::text(*content)],
            )
        })
        .collect();

    let tabs = vec![Template::new_element(
        "div",
        Vec::new(),
        Vec::new(),
        vec![
            Template::new_element(
                "div",
                vec![("style".to_string(), "display:flex;gap:0;border-bottom:2px solid var(--border);margin-bottom:4px;".to_string())],
                Vec::new(),
                tab_headers,
            ),
            Template::new_element("div", Vec::new(), Vec::new(), tab_panels),
        ],
    )];

    let rating_signal = Signal::new(3usize);
    let rating_stars: Vec<Template> = (0..5)
        .map(|i| {
            let sig = rating_signal.clone();
            let star_fn: ReactiveFn = Rc::new(move || {
                if sig.get() > i { "\u{2605}".to_string() } else { "\u{2606}".to_string() }
            });
            let sig2 = rating_signal.clone();
            let star_color_fn: ReactiveFn = Rc::new(move || {
                if sig2.get() > i { "color:var(--warning);".to_string() } else { "color:var(--text-dim);".to_string() }
            });
            let sig3 = rating_signal.clone();
            let click_handler: SharedEventHandler = shared_event_handler(move |_| {
                sig3.set(i + 1);
            });

            Template::new_element_reactive(
                "span",
                vec![("style".to_string(), "font-size:1.5rem;cursor:pointer;transition:color 0.15s;".to_string())],
                vec![("style".to_string(), star_color_fn)],
                vec![("click".to_string(), click_handler)],
                vec![
                    Template::new(vec![TemplateNode::Reactive(star_fn)]),
                ],
            )
        })
        .collect();

    let rating_value_fn: ReactiveFn = {
        let sig = rating_signal.clone();
        Rc::new(move || format!("{} / 5 stars", sig.get()))
    };

    let rating = vec![Template::new_element(
        "div",
        vec![("style".to_string(), "display:flex;flex-direction:column;gap:8px;".to_string())],
        Vec::new(),
        vec![
            Template::new_element("div", Vec::new(), Vec::new(), rating_stars),
            Template::new_element_reactive(
                "span",
                vec![("style".to_string(), "font-size:0.8rem;color:var(--text-muted);".to_string())],
                Vec::new(),
                Vec::new(),
                vec![Template::new(vec![TemplateNode::Reactive(rating_value_fn)])],
            ),
        ],
    )];

    let list = vec![
        List::render(ListProps::default()
            .items(vec![
                ListItem::new("First item"),
                ListItem::new("Second item"),
                ListItem::new("Third item"),
            ])
            .variant(ListVariant::Unordered)),
    ];

    let stats = vec![
        Stat::render(StatProps::default().label("Users").value("1,234").trend(StatTrend::Up).trend_value("+12%").icon("\u{1F465}")),
        Stat::render(StatProps::default().label("Revenue").value("$9.8K").trend(StatTrend::Up).trend_value("+5%").icon("\u{1F4B0}")),
    ];

    // ── Layout ───────────────────────────────────────────────────────────────

    let header = Template::new_element(
        "div",
        vec![("class".to_string(), "page-header".to_string())],
        Vec::new(),
        vec![
            Template::new_element("h1", Vec::new(), Vec::new(), vec![Template::text("Components")]),
            Template::new_element("p", Vec::new(), Vec::new(), vec![Template::text("Showcase of all rye-ui components \u{2014} built with the template system")]),
        ],
    );

    let section = |title: &str, children: Vec<Template>| -> Template {
        Template::new_element(
            "div",
            vec![("class".to_string(), "showcase-section".to_string())],
            Vec::new(),
            vec![
                Template::new_element("h2", Vec::new(), Vec::new(), vec![Template::text(title)]),
                Template::new_element(
                    "div",
                    vec![("class".to_string(), "showcase-row".to_string())],
                    Vec::new(),
                    children,
                ),
            ],
        )
    };

    let section_col = |title: &str, children: Vec<Template>| -> Template {
        Template::new_element(
            "div",
            vec![("class".to_string(), "showcase-section".to_string())],
            Vec::new(),
            vec![
                Template::new_element("h2", Vec::new(), Vec::new(), vec![Template::text(title)]),
                Template::new_element(
                    "div",
                    vec![("class".to_string(), "showcase-col".to_string())],
                    Vec::new(),
                    children,
                ),
            ],
        )
    };

    // Build all sections
    let badges_section = section("Badges", to_templates(Element::Fragment(badges)));
    let buttons_section = section("Buttons", to_templates(Element::Fragment(buttons)));
    let switches_section = section_col("Switches (Interactive)", switches);
    let progress_section = section_col("Progress Bars", to_templates(Element::Fragment(progress_bars)));
    let alerts_section = section_col("Alerts", to_templates(Element::Fragment(alerts)));
    let spinners_section = section("Spinners", to_templates(Element::Fragment(spinners)));
    let avatars_section = section("Avatars", to_templates(Element::Fragment(avatars)));
    let tags_section = section("Tags", to_templates(Element::Fragment(tags)));
    let dividers_section = section_col("Dividers", to_templates(Element::Fragment(dividers)));
    let inputs_section = section_col("Inputs", to_templates(Element::Fragment(inputs)));
    let textareas_section = section_col("Textareas", to_templates(Element::Fragment(textareas)));
    let selects_section = section_col("Selects", to_templates(Element::Fragment(selects)));
    let checkboxes_section = section("Checkboxes", to_templates(Element::Fragment(checkboxes)));
    let radios_section = section_col("Radio Groups", to_templates(Element::Fragment(radios)));
    let labels_section = section("Labels", to_templates(Element::Fragment(labels)));
    let links_section = section("Links", to_templates(Element::Fragment(links)));
    let skeletons_section = section_col("Skeletons", to_templates(Element::Fragment(skeletons)));
    let circular_section = section("Circular Progress", to_templates(Element::Fragment(circular_progress)));
    let empty_section = section_col("Empty State", to_templates(Element::Fragment(empty_state)));
    let notifications_section = section_col("Notifications", to_templates(Element::Fragment(notifications)));
    let breadcrumb_section = section_col("Breadcrumb", to_templates(Element::Fragment(breadcrumb)));
    let timeline_section = section_col("Timeline", to_templates(Element::Fragment(timeline)));
    let code_section = section_col("Code Block", to_templates(Element::Fragment(code_block)));
    let accordion_section = section_col("Accordion", accordion);
    let tabs_section = section_col("Tabs", tabs);
    let rating_section = section("Rating", rating);
    let list_section = section_col("List", to_templates(Element::Fragment(list)));
    let stats_section = section("Stats", to_templates(Element::Fragment(stats)));

    let mut card_children = vec![
        Template::new_element("h2", Vec::new(), Vec::new(), vec![Template::text("Card")]),
    ];
    card_children.extend(to_templates(card));
    let card_section = Template::new_element(
        "div",
        vec![("class".to_string(), "showcase-section".to_string())],
        Vec::new(),
        card_children,
    );

    Element::Template(combine(vec![
        header,
        badges_section,
        buttons_section,
        switches_section,
        progress_section,
        alerts_section,
        spinners_section,
        avatars_section,
        tags_section,
        dividers_section,
        inputs_section,
        textareas_section,
        selects_section,
        checkboxes_section,
        radios_section,
        labels_section,
        links_section,
        skeletons_section,
        circular_section,
        empty_section,
        notifications_section,
        breadcrumb_section,
        timeline_section,
        code_section,
        accordion_section,
        tabs_section,
        rating_section,
        list_section,
        stats_section,
        card_section,
    ]))
}

// ─── Main app ───────────────────────────────────────────────────────────────

pub fn build_app() -> Element {
    let active_page = Signal::new(Page::Dashboard);
    let dark_mode = Signal::new(true);

    let sidebar_el = sidebar(&active_page, &dark_mode);

    let dashboard = page_wrapper(Page::Dashboard, &active_page, dashboard_page());
    let counter = page_wrapper(Page::Counter, &active_page, counter_page());
    let todo = page_wrapper(Page::Todo, &active_page, todo_page());
    let components = page_wrapper(Page::Components, &active_page, components_page());

    let main = Template::new_element(
        "main",
        vec![("class".to_string(), "main-content".to_string())],
        Vec::new(),
        vec![dashboard, counter, todo, components],
    );

    let dm_clone = dark_mode.clone();
    let theme_fn: ReactiveFn = Rc::new(move || {
        if dm_clone.get() { "dark".to_string() } else { "light".to_string() }
    });

    let layout = Template::new_element_reactive(
        "div",
        vec![("class".to_string(), "app-layout".to_string())],
        vec![("data-theme".to_string(), theme_fn)],
        Vec::new(),
        vec![sidebar_el, main],
    );

    Element::Template(layout)
}

// ─── WASM entry point (WebView backend) ─────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start() {
    use rye_core::mount;
    use rye_html::DomRenderer;

    console_error_panic_hook::set_once();

    let renderer = DomRenderer::new();
    renderer.setup_delegation();

    let scope = mount(|| build_app(), renderer);
    std::mem::forget(scope);
}

// ─── Native desktop entry point (Native GPU backend) ────────────────────────

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    use rye_core::mount;
    use rye_core::RenderBackend;
    use rye_desktop::NativeRenderer;
    use rye_desktop::window::{run, WindowConfig};

    let config = WindowConfig {
        title: "rye — Demo".to_string(),
        width: 1024.0,
        height: 720.0,
        resizable: true,
    };

    let render_callback = move |renderer: &mut NativeRenderer| {
        mount(|| build_app(), NativeRenderer::new());
    };

    let input_callback = move |_event| {};

    run(config, render_callback, input_callback)
}

/// Get the active render backend for this build.
pub fn backend() -> rye_core::RenderBackend {
    #[cfg(target_arch = "wasm32")]
    { rye_core::RenderBackend::WebView }

    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    { rye_core::RenderBackend::Native }

    #[cfg(all(not(feature = "native"), not(target_arch = "wasm32")))]
    { rye_core::RenderBackend::WebView }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_app_renders() {
        let el = build_app();
        assert!(matches!(el, Element::Fragment(_)));
    }

    #[test]
    fn test_dashboard_page_renders() {
        let el = dashboard_page();
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_counter_page_renders() {
        let el = counter_page();
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_todo_page_renders() {
        let el = todo_page();
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_components_page_renders() {
        let el = components_page();
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_sidebar_renders() {
        let page = Signal::new(Page::Dashboard);
        let el = sidebar(&page);
        assert!(el.nodes.len() == 1);
    }

    #[test]
    fn test_page_labels() {
        assert_eq!(Page::Dashboard.label(), "Dashboard");
        assert_eq!(Page::Counter.label(), "Counter");
        assert_eq!(Page::Todo.label(), "Todo List");
        assert_eq!(Page::Components.label(), "Components");
    }

    #[test]
    fn test_to_templates_fragment() {
        let frag = Element::Fragment(vec![
            Element::Template(Template::text("a")),
            Element::Template(Template::text("b")),
        ]);
        let templates = to_templates(frag);
        assert_eq!(templates.len(), 2);
    }

    #[test]
    fn test_to_templates_none() {
        let templates = to_templates(Element::none());
        assert!(templates.is_empty());
    }

    #[test]
    fn test_template_macro_basic() {
        let el = rye_macros::template! {
            div { "Hello" }
        };
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_backend() {
        let b = backend();
        #[cfg(target_arch = "wasm32")]
        assert_eq!(b, rye_core::RenderBackend::WebView);
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
        assert_eq!(b, rye_core::RenderBackend::WebView);
    }
}
