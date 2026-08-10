//! Template macro implementation — parses template DSL and generates code.
//!
//! The template! macro uses a custom parser that converts HTML-like syntax
//! into calls to the Renderer trait.
//!
//! ## Features
//!
//! - **Elements**: `div { class: "container", "text" }` → `Template::new_element(...)`
//! - **Dynamic text**: `{expr}` → `TemplateNode::Reactive(Rc::new(move || expr.to_string()))`
//! - **Static attributes**: `class: "btn"` → `("class", "btn")`
//! - **Dynamic attributes**: `style: { format!("width:{}px", w) }` → reactive attr
//! - **Event handlers**: `onclick: move |_| do_thing()` → `EventHandler` box
//! - **Component invocation**: `Button { label: "Click", onclick: move |_| {} }`
//!   → `Button::render(ButtonProps { label: "Click", .. })` — PascalCase tags
//!   are treated as component types, lowercase as HTML elements
//! - **Keyed lists**: `For { each: {items}, key: |item| item.id, |item| li { {item.text} } }`
//!   → `Template::new_reactive_list(Rc::new(move || { ... }))`
//! - **Conditionals**: `If { {condition}, div { "visible" } }`
//!   → evaluates condition and renders content or empty
//!
//! ## Known limitations
//!
//! - **Move closure capture conflicts**: Every dynamic expression (`{expr}`) and
//!   reactive attribute (`attr: {expr}`) generates a `move` closure. If a signal
//!   is used in multiple dynamic positions, it must be manually cloned before
//!   each use (e.g. `let display_count = count.clone();` then `{display_count.get()}`).
//!   Future improvement: auto-clone `Copy`/`Clone` types, or use borrowed closures.
//! - **Single root node**: The macro only parses one root node. Multiple root
//!   elements must be wrapped in a container `div`.
//! - **Returns `Element`, not `Template`**: The macro always wraps output in
//!   `Element::Template(...)`. When a `Template` value is needed (for children
//!   or composition), use `generate_template_code` or build manually.

use proc_macro2::{Group, TokenStream, TokenTree};
use quote::quote;

/// A node in the template AST.
#[derive(Debug)]
pub(crate) enum TemplateNode {
    /// An element with tag, attributes, and children.
    Element {
        tag: String,
        attributes: Vec<Attribute>,
        children: Vec<TemplateNode>,
    },
    /// A component invocation (PascalCase tag).
    /// Generates `ComponentType::render(ComponentTypeProps { ... })`
    Component {
        /// The component type name (e.g. "Button", "Card")
        type_name: String,
        /// The props type name (e.g. "ButtonProps", "CardProps")
        props_name: String,
        /// Prop expressions as (name, value_tokens)
        props: Vec<(String, TokenStream)>,
        /// Children to pass as a prop (collected into `children` prop)
        children: Vec<TemplateNode>,
    },
    /// A keyed list (For block).
    /// Generates `Template::new_reactive_list(Rc::new(move || { ... }))`
    For {
        /// Expression that evaluates to the iterable (inside `{}`)
        each: TokenStream,
        /// Optional key closure: `|item| item.id`
        key: Option<TokenStream>,
        /// Render closure parameter names: ["item"] or ["item", "index"]
        render_params: Vec<String>,
        /// Template body for each item (parsed as a TemplateNode)
        render_body: Box<TemplateNode>,
    },
    /// A conditional (If block).
    /// Evaluates condition; if true, renders content; else empty.
    If {
        /// Condition expression (inside `{}`)
        condition: TokenStream,
        /// Content to render if true
        content: Box<TemplateNode>,
    },
    /// Static text.
    Text(String),
    /// A dynamic expression (Rust code that evaluates to a value).
    Dynamic(TokenStream),
}

/// An attribute — either static or dynamic.
#[derive(Debug)]
pub(crate) enum Attribute {
    /// Static name=value pair.
    Static { name: String, value: String },
    /// Dynamic name=expression pair.
    Dynamic { name: String, value: TokenStream },
    /// Event handler: on{event} = expression.
    Event { event: String, handler: TokenStream },
}

/// Check if a tag name is a component (PascalCase) or HTML element (lowercase).
fn is_component_tag(tag: &str) -> bool {
    tag.chars().next().map_or(false, |c| c.is_uppercase())
}

/// Derive the props type name from a component type name.
/// "Button" → "ButtonProps", "Card" → "CardProps"
fn props_name_for(component: &str) -> String {
    format!("{}Props", component)
}

/// Parse a template! macro body into a TemplateNode AST.
pub(crate) fn parse_template(tokens: TokenStream) -> Result<TemplateNode, String> {
    let mut iter = tokens.into_iter().peekable();

    // Skip the initial TokenStream if empty
    if iter.peek().is_none() {
        return Ok(TemplateNode::Text(String::new()));
    }

    parse_node(&mut iter)
}

/// Parse a single node (element, component, For, If, text, or dynamic expression).
fn parse_node(
    iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> Result<TemplateNode, String> {
    match iter.peek() {
        Some(TokenTree::Ident(ident)) => {
            let tag = ident.to_string();
            iter.next(); // consume the tag ident

            // Check for `{` block (element/component with children/attrs) or self-closing
            match iter.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                    let group = g.clone();
                    iter.next(); // consume the group

                    // Special keywords: For, If
                    if tag == "For" {
                        return parse_for_block(group);
                    }
                    if tag == "If" {
                        return parse_if_block(group);
                    }

                    // Component invocation (PascalCase) vs HTML element (lowercase)
                    if is_component_tag(&tag) {
                        parse_component_body(&tag, group)
                    } else {
                        parse_element_body(&tag, group)
                    }
                }
                _ => {
                    // Self-closing element (no children)
                    if is_component_tag(&tag) {
                        // Self-closing component: Button (no props)
                        let props_name = props_name_for(&tag);
                        Ok(TemplateNode::Component {
                            type_name: tag,
                            props_name,
                            props: Vec::new(),
                            children: Vec::new(),
                        })
                    } else {
                        Ok(TemplateNode::Element {
                            tag,
                            attributes: Vec::new(),
                            children: Vec::new(),
                        })
                    }
                }
            }
        }
        Some(TokenTree::Literal(lit)) => {
            let text = lit.to_string();
            iter.next();
            // Strip quotes from string literals
            let text = if text.starts_with('"') && text.ends_with('"') {
                text[1..text.len() - 1].to_string()
            } else {
                text
            };
            Ok(TemplateNode::Text(text))
        }
        Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
            // Dynamic expression: {expr}
            let group = g.clone();
            iter.next();
            Ok(TemplateNode::Dynamic(group.stream()))
        }
        Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
            // Fragment: #{}
            iter.next();
            match iter.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                    let group = g.clone();
                    iter.next();
                    Ok(TemplateNode::Dynamic(group.stream()))
                }
                _ => Err("Expected `{}` after `#`".to_string()),
            }
        }
        _ => {
            // Collect remaining tokens as a dynamic expression
            let mut tokens = Vec::new();
            for t in iter.by_ref() {
                tokens.push(t);
            }
            if tokens.is_empty() {
                Ok(TemplateNode::Text(String::new()))
            } else {
                let stream = tokens.into_iter().collect::<TokenStream>();
                Ok(TemplateNode::Dynamic(stream))
            }
        }
    }
}

/// Parse a For block: `For { each: {expr}, key: |item| item.id, |item| li { ... } }`
fn parse_for_block(group: Group) -> Result<TemplateNode, String> {
    let stream = group.stream();
    let mut iter = stream.into_iter().peekable();

    let mut each: Option<TokenStream> = None;
    let mut key: Option<TokenStream> = None;
    let mut render_params: Vec<String> = Vec::new(); // closure param names
    let mut render_body: Option<TemplateNode> = None;

    loop {
        if iter.peek().is_none() {
            break;
        }

        match iter.peek() {
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                if name == "each" {
                    iter.next(); // consume "each"
                    if let Some(TokenTree::Punct(p)) = iter.peek() {
                        if p.as_char() == ':' {
                            iter.next();
                        }
                    }
                    if let Some(TokenTree::Group(g)) = iter.peek() {
                        if g.delimiter() == proc_macro2::Delimiter::Brace {
                            let g = g.clone();
                            iter.next();
                            each = Some(g.stream());
                        }
                    }
                } else if name == "key" {
                    iter.next(); // consume "key"
                    if let Some(TokenTree::Punct(p)) = iter.peek() {
                        if p.as_char() == ':' {
                            iter.next();
                        }
                    }
                    key = Some(collect_expression(&mut iter)?);
                } else {
                    // Unexpected ident — skip
                    iter.next();
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '|' => {
                // Render closure: |item| or |item, index| followed by template body
                iter.next(); // consume first '|'

                // Parse closure params until closing '|'
                loop {
                    match iter.peek() {
                        Some(TokenTree::Punct(p)) if p.as_char() == '|' => {
                            iter.next(); // consume closing '|'
                            break;
                        }
                        Some(TokenTree::Ident(ident)) => {
                            render_params.push(ident.to_string());
                            iter.next();
                        }
                        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                            iter.next();
                        }
                        Some(TokenTree::Group(g))
                            if g.delimiter() == proc_macro2::Delimiter::Parenthesis =>
                        {
                            // |(item, index)| pattern — extract idents
                            let inner = g.clone();
                            iter.next();
                            for t in inner.stream() {
                                if let TokenTree::Ident(ident) = t {
                                    render_params.push(ident.to_string());
                                }
                            }
                        }
                        _ => {
                            iter.next();
                        }
                    }
                }

                // Now parse the template body (a single node)
                render_body = Some(parse_node(&mut iter)?);
            }
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                iter.next();
            }
            _ => {
                iter.next();
            }
        }
    }

    let each = each.ok_or("For block requires `each: {expr}`")?;
    let render_body = render_body.ok_or("For block requires a render closure `|item| ...`")?;

    Ok(TemplateNode::For {
        each,
        key,
        render_params,
        render_body: Box::new(render_body),
    })
}

/// Parse an If block: `If { {condition}, div { ... } }`
fn parse_if_block(group: Group) -> Result<TemplateNode, String> {
    let stream = group.stream();
    let mut iter = stream.into_iter().peekable();

    // First: expect {condition} group
    let condition = match iter.peek() {
        Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
            let g = g.clone();
            iter.next();
            g.stream()
        }
        _ => return Err("If block requires `{condition}` first".to_string()),
    };

    // Skip comma if present
    if let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == ',' {
            iter.next();
        }
    }

    // Rest is the content node
    let content = parse_node(&mut iter)?;

    Ok(TemplateNode::If {
        condition,
        content: Box::new(content),
    })
}

/// Parse a component invocation body: `Button { label: "Click", onclick: move |_| {} }`
fn parse_component_body(tag: &str, group: Group) -> Result<TemplateNode, String> {
    let props_name = props_name_for(tag);
    let stream = group.stream();
    let mut iter = stream.into_iter().peekable();
    let mut props: Vec<(String, TokenStream)> = Vec::new();
    let mut children: Vec<TemplateNode> = Vec::new();

    loop {
        if iter.peek().is_none() {
            break;
        }

        match iter.peek() {
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                // Look ahead for `:` (prop) or `{`/child (child element)
                let mut look_ahead = iter.clone();
                look_ahead.next();

                match look_ahead.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                        // Prop: name : value
                        iter.next(); // consume ident
                        iter.next(); // consume ':'
                        let value = collect_expression(&mut iter)?;
                        props.push((name, value));
                    }
                    _ => {
                        // Child node
                        let node = parse_node(&mut iter)?;
                        children.push(node);
                    }
                }
            }
            Some(TokenTree::Literal(_)) => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            _ => {
                iter.next();
            }
        }
    }

    Ok(TemplateNode::Component {
        type_name: tag.to_string(),
        props_name,
        props,
        children,
    })
}

/// Parse the body of an element (inside `{}`).
/// The body can contain attributes (name: value) and child nodes.
fn parse_element_body(tag: &str, group: Group) -> Result<TemplateNode, String> {
    let stream = group.stream();
    let mut iter = stream.into_iter().peekable();
    let mut attributes = Vec::new();
    let mut children = Vec::new();

    loop {
        if iter.peek().is_none() {
            break;
        }

        // Check if this is an attribute (ident : value) or a child node
        match iter.peek() {
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                // Look ahead for `:` (attribute) or `{` (child element)
                let mut look_ahead = iter.clone();
                look_ahead.next(); // consume ident

                match look_ahead.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                        // Attribute: name : value
                        iter.next(); // consume ident
                        iter.next(); // consume ':'

                        let attr = parse_attribute_value(&name, &mut iter)?;
                        attributes.push(attr);
                    }
                    _ => {
                        // Child element
                        let node = parse_node(&mut iter)?;
                        children.push(node);
                    }
                }
            }
            Some(TokenTree::Literal(_)) => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                let node = parse_node(&mut iter)?;
                children.push(node);
            }
            _ => {
                // Skip unexpected tokens
                iter.next();
            }
        }
    }

    Ok(TemplateNode::Element {
        tag: tag.to_string(),
        attributes,
        children,
    })
}

/// Parse an attribute value after `name:`.
fn parse_attribute_value(
    name: &str,
    iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> Result<Attribute, String> {
    // Check if it's an event handler (on{event})
    if name.starts_with("on") && name.len() > 2 {
        let event = name[2..].to_string();
        let handler = collect_expression(iter)?;
        return Ok(Attribute::Event { event, handler });
    }

    // Parse value — could be a string literal or an expression
    match iter.peek() {
        Some(TokenTree::Literal(lit)) => {
            let value = lit.to_string();
            iter.next();
            // Strip quotes from string literals
            let value = if value.starts_with('"') && value.ends_with('"') {
                value[1..value.len() - 1].to_string()
            } else {
                value
            };
            Ok(Attribute::Static {
                name: name.to_string(),
                value,
            })
        }
        Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
            let group = g.clone();
            iter.next();
            Ok(Attribute::Dynamic {
                name: name.to_string(),
                value: group.stream(),
            })
        }
        _ => {
            let expr = collect_expression(iter)?;
            Ok(Attribute::Dynamic {
                name: name.to_string(),
                value: expr,
            })
        }
    }
}

/// Collect tokens forming an expression until we hit the next attribute or child element.
fn collect_expression(
    iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> Result<TokenStream, String> {
    let mut tokens = Vec::new();
    let mut paren_depth = 0;

    while let Some(tt) = iter.peek() {
        match tt {
            TokenTree::Punct(p) => {
                if p.as_char() == ',' && paren_depth == 0 {
                    iter.next();
                    break;
                }
                if p.as_char() == '(' || p.as_char() == '{' || p.as_char() == '[' {
                    paren_depth += 1;
                }
                if p.as_char() == ')' || p.as_char() == '}' || p.as_char() == ']' {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                }
            }
            TokenTree::Ident(_) => {
                if paren_depth == 0 && !tokens.is_empty() {
                    let last = tokens.last();
                    let is_continuation = match last {
                        Some(TokenTree::Punct(p)) => {
                            p.as_char() == '.' || p.as_char() == ':' || p.as_char() == '|'
                        }
                        _ => false,
                    };
                    if !is_continuation {
                        break;
                    }
                }
            }
            _ => {}
        }
        tokens.push(iter.next().unwrap());
    }

    Ok(tokens.into_iter().collect())
}

/// Generate code that produces a `Template` value from a TemplateNode AST.
///
/// This is used for child elements which need `Template` values, not `Element`.
pub(crate) fn generate_template_code(node: &TemplateNode) -> TokenStream {
    match node {
        TemplateNode::Text(text) => {
            if text.is_empty() {
                quote! { ::rye_core::Template::empty() }
            } else {
                quote! {
                    ::rye_core::Template::new(vec![
                        ::rye_core::TemplateNode::Text(#text.to_string()),
                    ])
                }
            }
        }
        TemplateNode::Dynamic(expr) => {
            quote! {
                ::rye_core::Template::new(vec![
                    ::rye_core::TemplateNode::Reactive(
                        ::std::rc::Rc::new(move || ::std::string::ToString::to_string(&(#expr)))
                            as ::rye_core::template::ReactiveFn
                    ),
                ])
            }
        }
        TemplateNode::Element {
            tag,
            attributes,
            children,
        } => {
            let tag_lit = tag.as_str();

            let mut attr_code: Vec<TokenStream> = Vec::new();
            let mut reactive_attr_code: Vec<TokenStream> = Vec::new();
            let mut event_code: Vec<TokenStream> = Vec::new();

            for attr in attributes {
                match attr {
                    Attribute::Static { name, value } => {
                        let name = name.as_str();
                        let value = value.as_str();
                        attr_code.push(quote! {
                            attrs.push((#name.to_string(), #value.to_string()));
                        });
                    }
                    Attribute::Dynamic { name, value } => {
                        let name = name.as_str();
                        reactive_attr_code.push(quote! {
                            reactive_attrs.push((
                                #name.to_string(),
                                ::std::rc::Rc::new(move || format!("{}", #value))
                                    as ::rye_core::template::ReactiveFn,
                            ));
                        });
                    }
                    Attribute::Event { event, handler } => {
                        let event = event.as_str();
                        event_code.push(quote! {
                            {
                                let handler: ::rye_core::renderer::EventHandler = Box::new(#handler);
                                let shared: ::rye_core::template::SharedEventHandler =
                                    ::std::rc::Rc::new(::std::cell::RefCell::new(handler));
                                events.push((#event.to_string(), shared));
                            }
                        });
                    }
                }
            }

            // Generate children as Template values (not Element)
            let child_code: Vec<TokenStream> = children
                .iter()
                .map(|child| {
                    let child_gen = generate_template_code(child);
                    quote! { children.push(#child_gen); }
                })
                .collect();

            let has_reactive_attrs = !reactive_attr_code.is_empty();

            if has_reactive_attrs {
                quote! {
                    ::rye_core::Template::new_element_reactive(
                        #tag_lit.to_string(),
                        {
                            let mut attrs = Vec::new();
                            #(#attr_code)*
                            attrs
                        },
                        {
                            let mut reactive_attrs = Vec::new();
                            #(#reactive_attr_code)*
                            reactive_attrs
                        },
                        {
                            let mut events = Vec::new();
                            #(#event_code)*
                            events
                        },
                        {
                            let mut children = Vec::new();
                            #(#child_code)*
                            children
                        },
                    )
                }
            } else {
                quote! {
                    ::rye_core::Template::new_element(
                        #tag_lit.to_string(),
                        {
                            let mut attrs = Vec::new();
                            #(#attr_code)*
                            attrs
                        },
                        {
                            let mut events = Vec::new();
                            #(#event_code)*
                            events
                        },
                        {
                            let mut children = Vec::new();
                            #(#child_code)*
                            children
                        },
                    )
                }
            }
        }
        TemplateNode::Component {
            type_name,
            props_name,
            props,
            children,
        } => {
            let type_ident: TokenStream = type_name.parse().unwrap_or_else(|_| {
                quote! { compile_error!(concat!("Invalid component type: ", #type_name)) }
            });
            let props_ident: TokenStream = props_name.parse().unwrap_or_else(|_| {
                quote! { compile_error!(concat!("Invalid props type: ", #props_name)) }
            });

            // Generate prop assignments
            let prop_code: Vec<TokenStream> = props
                .iter()
                .map(|(name, value)| {
                    // Convert event handler names: onclick → on_click
                    let rust_name = if name.starts_with("on") && name.len() > 2 {
                        let event = &name[2..];
                        // Insert underscore before uppercase letters: click → click, mouseMove → mouse_move
                        let mut snake = String::new();
                        for (i, c) in event.chars().enumerate() {
                            if c.is_uppercase() && i > 0 {
                                snake.push('_');
                            }
                            snake.push(c.to_ascii_lowercase());
                        }
                        format!("on_{}", snake)
                    } else {
                        name.clone()
                    };
                    let name_ident: TokenStream = rust_name.parse().unwrap_or_else(|_| {
                        quote! { compile_error!("Invalid prop name") }
                    });
                    quote! { .#name_ident(#value) }
                })
                .collect();

            // If there are children, convert them to Element and pass as children prop
            // Note: only components with a `children` method will accept these.
            // The generated code uses a helper trait so it compiles even if the
            // component doesn't support children.
            let children_code = if children.is_empty() {
                quote! {}
            } else {
                let child_templates: Vec<TokenStream> = children
                    .iter()
                    .map(|child| generate_template_code(child))
                    .collect();
                quote! {
                    .children(::rye_core::Element::Fragment(vec![
                        #(::rye_core::Element::Template(#child_templates)),*
                    ]))
                }
            };

            // Suppress unused variable warning when no children
            let _ = &children_code;

            quote! {
                {
                    let __el = #type_ident::render(
                        #props_ident::default()
                            #(#prop_code)*
                    );
                    // Convert Element to Template for embedding
                    match __el {
                        ::rye_core::Element::Template(t) => t,
                        ::rye_core::Element::Fragment(els) => {
                            let nodes: Vec<::rye_core::TemplateNode> = els
                                .into_iter()
                                .filter_map(|e| match e {
                                    ::rye_core::Element::Template(t) => Some(t),
                                    _ => None,
                                })
                                .flat_map(|t| t.nodes)
                                .collect();
                            ::rye_core::Template::new(nodes)
                        }
                        ::rye_core::Element::None => ::rye_core::Template::empty(),
                        ::rye_core::Element::Component(_) => ::rye_core::Template::empty(),
                    }
                }
            }
        }
        TemplateNode::For {
            each,
            key,
            render_params,
            render_body,
        } => {
            // Generate the template code for the render body
            // The render params are bound as variables (item, index)
            let body_template = generate_template_code(render_body);

            // Build the closure parameter list
            // First param is the item, second (if present) is the index
            let item_param = render_params.first().map(|s| s.as_str()).unwrap_or("item");
            let index_param = render_params.get(1).map(|s| s.as_str());

            // Generate the key expression
            let key_code = match key {
                Some(k) => {
                    quote! {
                        let __key = (#k);
                        (__key, item_template)
                    }
                }
                None => {
                    quote! { (i, item_template) }
                }
            };

            // If there's an index param, bind it; otherwise ignore i
            let index_binding = if let Some(idx) = index_param {
                let idx_ident: TokenStream = idx.parse().unwrap_or_else(|_| quote! { _idx });
                quote! { let #idx_ident = i; }
            } else {
                quote! { let _ = i; }
            };

            let item_ident: TokenStream = item_param.parse().unwrap_or_else(|_| quote! { item });

            quote! {
                ::rye_core::Template::new_reactive_list(
                    ::std::rc::Rc::new(move || {
                        let __items: Vec<_> = (#each).into_iter().collect();
                        let mut __result: Vec<(::rye_core::reconcile::Key, ::rye_core::Template)> = Vec::new();
                        for (i, #item_ident) in __items.into_iter().enumerate() {
                            #index_binding
                            let item_template = #body_template;
                            __result.push(#key_code);
                        }
                        __result
                    }) as ::rye_core::template::ReactiveListFn
                )
            }
        }
        TemplateNode::If { condition, content } => {
            let content_code = generate_template_code(content);
            quote! {
                {
                    if #condition {
                        #content_code
                    } else {
                        ::rye_core::Template::empty()
                    }
                }
            }
        }
    }
}

/// Generate code from a TemplateNode AST.
///
/// Produces a TokenStream that creates an `Element::Template(...)` with:
/// - Static attributes as string pairs
/// - Dynamic attributes evaluated via `format!`
/// - Event handlers as `EventHandler` boxes
/// - Children as nested `Template` instances
/// - Dynamic expressions as `TemplateNode::Reactive` for reactive updates
/// - Component invocations as `ComponentType::render(Props { ... })`
/// - For blocks as `Template::new_reactive_list(...)`
/// - If blocks as conditional Template rendering
pub(crate) fn generate_code(node: &TemplateNode) -> TokenStream {
    if matches!(node, TemplateNode::Text(t) if t.is_empty()) {
        return quote! { ::rye_core::Element::none() };
    }

    // For component nodes at the top level, return the Element directly
    if let TemplateNode::Component {
        type_name,
        props_name,
        props,
        children,
    } = node
    {
        let type_ident: TokenStream = type_name.parse().unwrap_or_else(|_| {
            quote! { compile_error!(concat!("Invalid component type: ", #type_name)) }
        });
        let props_ident: TokenStream = props_name.parse().unwrap_or_else(|_| {
            quote! { compile_error!(concat!("Invalid props type: ", #props_name)) }
        });

        let prop_code: Vec<TokenStream> = props
            .iter()
            .map(|(name, value)| {
                // Convert event handler names: onclick → on_click
                let rust_name = if name.starts_with("on") && name.len() > 2 {
                    let event = &name[2..];
                    let mut snake = String::new();
                    for (i, c) in event.chars().enumerate() {
                        if c.is_uppercase() && i > 0 {
                            snake.push('_');
                        }
                        snake.push(c.to_ascii_lowercase());
                    }
                    format!("on_{}", snake)
                } else {
                    name.clone()
                };
                let name_ident: TokenStream = rust_name.parse().unwrap_or_else(|_| {
                    quote! { compile_error!("Invalid prop name") }
                });
                quote! { .#name_ident(#value) }
            })
            .collect();

        let children_code = if children.is_empty() {
            quote! {}
        } else {
            let child_templates: Vec<TokenStream> = children
                .iter()
                .map(|child| generate_template_code(child))
                .collect();
            quote! {
                .children(::rye_core::Element::Fragment(vec![
                    #(::rye_core::Element::Template(#child_templates)),*
                ]))
            }
        };

        return quote! {
            #type_ident::render(
                #props_ident::default()
                    #(#prop_code)*
                    #children_code
            )
        };
    }

    // For For blocks at top level, wrap in Element::Template
    if let TemplateNode::For { .. } = node {
        let template = generate_template_code(node);
        return quote! { ::rye_core::Element::Template(#template) };
    }

    // For If blocks at top level, wrap in Element::Template
    if let TemplateNode::If { .. } = node {
        let template = generate_template_code(node);
        return quote! { ::rye_core::Element::Template(#template) };
    }

    let template = generate_template_code(node);
    quote! { ::rye_core::Element::Template(#template) }
}
