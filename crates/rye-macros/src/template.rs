//! Template macro implementation — parses template DSL and generates code.
//!
//! The template! macro uses a custom parser that converts HTML-like syntax
//! into calls to the Renderer trait.

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

/// Parse a template! macro body into a TemplateNode AST.
pub(crate) fn parse_template(tokens: TokenStream) -> Result<TemplateNode, String> {
    let mut iter = tokens.into_iter().peekable();

    // Skip the initial TokenStream if empty
    if iter.peek().is_none() {
        return Ok(TemplateNode::Text(String::new()));
    }

    parse_node(&mut iter)
}

/// Parse a single node (element, text, or dynamic expression).
fn parse_node(
    iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> Result<TemplateNode, String> {
    match iter.peek() {
        Some(TokenTree::Ident(ident)) => {
            let tag = ident.to_string();
            iter.next(); // consume the tag ident

            // Check for `{` block (element with children/attrs) or self-closing
            match iter.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                    let group = g.clone();
                    iter.next(); // consume the group
                    parse_element_body(&tag, group)
                }
                _ => {
                    // Self-closing element (no children)
                    Ok(TemplateNode::Element {
                        tag,
                        attributes: Vec::new(),
                        children: Vec::new(),
                    })
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
                // We need to clone the iterator state to peek ahead
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
        // Parse the handler expression — collect tokens until we hit the next attribute or child
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
            // Collect expression tokens
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
                    // Check if this is the start of a new attribute or child
                    // If the previous token is not a punct or is a comma, break
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
pub(crate) fn generate_code(node: &TemplateNode) -> TokenStream {
    if matches!(node, TemplateNode::Text(t) if t.is_empty()) {
        return quote! { ::rye_core::Element::none() };
    }
    let template = generate_template_code(node);
    quote! { ::rye_core::Element::Template(#template) }
}
