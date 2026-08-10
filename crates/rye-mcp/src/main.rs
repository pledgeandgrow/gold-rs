//! # rye MCP Server — Goal 157
//!
//! Exposes all rye AI-native tooling as MCP (Model Context Protocol) tools.
//! AI agents (Claude, Cursor, Windsurf, Copilot) can call these tools
//! to discover components, explain errors, scaffold code, generate tests,
//! review code, and search for components using natural language.
//!
//! The server communicates over stdio using JSON-RPC 2.0.

use std::io::{self, BufRead, Write};
use std::sync::Mutex;

use rye_core::ai::{
    code_review, context_optimizer, error_recovery, nl_search, prompt_templates, usage_analytics,
};
use rye_core::component_registry;
use rye_core::error_codes;

mod scaffold_gen;
mod test_gen_mcp;

struct McpTool {
    name: &'static str,
    description: &'static str,
    input_schema: serde_json::Value,
}

fn tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "rye_explain_error",
            description: "Explain a rye error code with causes, suggestion, and correct example",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "Error code (e.g. R802)" },
                    "json": { "type": "boolean", "description": "Output as JSON", "default": false }
                },
                "required": ["code"]
            }),
        },
        McpTool {
            name: "rye_list_error_codes",
            description: "List all rye error codes, optionally filtered by category",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Filter by category" }
                }
            }),
        },
        McpTool {
            name: "rye_search_error_codes",
            description: "Search error codes by keyword",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keyword" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "rye_get_recovery_plan",
            description: "Get step-by-step recovery plan for an error code",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "Error code" }
                },
                "required": ["code"]
            }),
        },
        McpTool {
            name: "rye_list_components",
            description: "List all registered rye components",
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "rye_find_component",
            description: "Find a rye component by exact name",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Component name (PascalCase)" }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "rye_search_components",
            description: "Search components by keyword",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keyword" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "rye_nl_search_components",
            description: "Natural language component search",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language query" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "rye_list_prompt_templates",
            description: "List available AI prompt templates",
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "rye_get_prompt_template",
            description: "Get a specific prompt template by ID with placeholders filled",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Template ID" },
                    "values": { "type": "object", "description": "Placeholder values" }
                },
                "required": ["id"]
            }),
        },
        McpTool {
            name: "rye_review_code",
            description: "Review rye source code for common issues",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "Source code to review" },
                    "file_path": { "type": "string", "description": "File path", "default": "anonymous" }
                },
                "required": ["source"]
            }),
        },
        McpTool {
            name: "rye_get_context",
            description: "Get optimized context package for AI agents",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "max_tokens": { "type": "integer", "description": "Max tokens", "default": 4000 }
                }
            }),
        },
        McpTool {
            name: "rye_get_focused_context",
            description: "Get focused context for a specific query",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Query to focus on" },
                    "max_tokens": { "type": "integer", "description": "Max tokens", "default": 4000 }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "rye_scaffold_component",
            description: "Generate component source code (returns code as string)",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Component name" },
                    "props": { "type": "string", "description": "Props as 'name:Type,...'" },
                    "style": { "type": "boolean", "description": "Include style block", "default": false },
                    "island": { "type": "boolean", "description": "Mark as island", "default": false },
                    "test": { "type": "boolean", "description": "Also generate test code", "default": false }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "rye_scaffold_test",
            description: "Generate test scaffolding for a component (returns code as string)",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source": { "type": "string", "description": "Component source code" }
                },
                "required": ["source"]
            }),
        },
        McpTool {
            name: "rye_component_usage_stats",
            description: "Get usage analytics for components",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["all", "most_used", "unused"], "default": "all" },
                    "limit": { "type": "integer", "description": "Limit for most_used", "default": 10 }
                }
            }),
        },
    ]
}

fn parse_category(s: &str) -> Option<error_codes::ErrorCategory> {
    match s.to_lowercase().as_str() {
        "parse" => Some(error_codes::ErrorCategory::Parse),
        "validation" => Some(error_codes::ErrorCategory::Validation),
        "type" => Some(error_codes::ErrorCategory::Type),
        "reactivity" => Some(error_codes::ErrorCategory::Reactivity),
        "renderer" => Some(error_codes::ErrorCategory::Renderer),
        "router" => Some(error_codes::ErrorCategory::Router),
        "ssr" => Some(error_codes::ErrorCategory::Ssr),
        "cli" => Some(error_codes::ErrorCategory::Cli),
        "ai" => Some(error_codes::ErrorCategory::Ai),
        _ => None,
    }
}

fn handle_tool_call(name: &str, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    match name {
        "rye_explain_error" => {
            let code = args["code"].as_str().ok_or("Missing 'code' parameter")?;
            let json_mode = args.get("json").and_then(|v| v.as_bool()).unwrap_or(false);
            match error_codes::lookup(code) {
                Some(entry) => {
                    if json_mode {
                        Ok(serde_json::Value::String(entry.format_json()))
                    } else {
                        Ok(serde_json::Value::String(entry.format_text()))
                    }
                }
                None => Err(format!("Unknown error code: {}", code)),
            }
        }
        "rye_list_error_codes" => {
            let category = args.get("category").and_then(|v| v.as_str());
            let codes = match category {
                Some(cat) => {
                    let cat_enum =
                        parse_category(cat).ok_or(format!("Unknown category: {}", cat))?;
                    error_codes::list_category(cat_enum)
                }
                None => error_codes::all_codes().iter().collect(),
            };
            let entries: Vec<String> = codes.iter().map(|c| c.format_json()).collect();
            Ok(serde_json::Value::String(format!(
                "[{}]",
                entries.join(",")
            )))
        }
        "rye_search_error_codes" => {
            let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
            let results = error_codes::search(query);
            let entries: Vec<String> = results.iter().map(|c| c.format_json()).collect();
            Ok(serde_json::Value::String(format!(
                "[{}]",
                entries.join(",")
            )))
        }
        "rye_get_recovery_plan" => {
            let code = args["code"].as_str().ok_or("Missing 'code' parameter")?;
            match error_recovery::get_recovery_plan(code) {
                Some(plan) => Ok(serde_json::Value::String(plan.format_json())),
                None => Err(format!("No recovery plan for error code: {}", code)),
            }
        }
        "rye_list_components" => Ok(serde_json::Value::String(
            component_registry::format_all_json(),
        )),
        "rye_find_component" => {
            let name = args["name"].as_str().ok_or("Missing 'name' parameter")?;
            match component_registry::find(name) {
                Some(comp) => Ok(serde_json::Value::String(comp.format_json())),
                None => Err(format!("Component not found: {}", name)),
            }
        }
        "rye_search_components" => {
            let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
            let results = component_registry::search(query);
            let entries: Vec<String> = results.iter().map(|c| c.format_json()).collect();
            Ok(serde_json::Value::String(format!(
                "[{}]",
                entries.join(",")
            )))
        }
        "rye_nl_search_components" => {
            let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
            let results = nl_search::search_nl(query);
            Ok(serde_json::Value::String(nl_search::format_results_json(
                &results,
            )))
        }
        "rye_list_prompt_templates" => Ok(serde_json::Value::String(
            prompt_templates::format_all_json(),
        )),
        "rye_get_prompt_template" => {
            let id = args["id"].as_str().ok_or("Missing 'id' parameter")?;
            let template =
                prompt_templates::get_template(id).ok_or(format!("Template not found: {}", id))?;
            let values_map: std::collections::HashMap<&str, String> = args
                .get("values")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.as_str(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            Ok(serde_json::Value::String(template.fill(&values_map)))
        }
        "rye_review_code" => {
            let source = args["source"]
                .as_str()
                .ok_or("Missing 'source' parameter")?;
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("anonymous");
            let result = code_review::review_source(file_path, source);
            Ok(serde_json::Value::String(result.format_json()))
        }
        "rye_get_context" => {
            let max_tokens = args
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(4000) as usize;
            let context = context_optimizer::generate_context_package(max_tokens);
            Ok(serde_json::Value::String(context))
        }
        "rye_get_focused_context" => {
            let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
            let max_tokens = args
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(4000) as usize;
            let context = context_optimizer::generate_focused_context(query, max_tokens);
            Ok(serde_json::Value::String(context))
        }
        "rye_scaffold_component" => {
            let name = args["name"].as_str().ok_or("Missing 'name' parameter")?;
            let props_str = args.get("props").and_then(|v| v.as_str()).unwrap_or("");
            let with_style = args.get("style").and_then(|v| v.as_bool()).unwrap_or(false);
            let is_island = args
                .get("island")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let with_test = args.get("test").and_then(|v| v.as_bool()).unwrap_or(false);

            let props = scaffold_gen::parse_props(props_str);
            let code = scaffold_gen::generate_component_code(name, &props, with_style, is_island);

            if with_test {
                let test_code = scaffold_gen::generate_component_test(name, &props);
                Ok(serde_json::json!({ "component": code, "test": test_code }))
            } else {
                Ok(serde_json::json!({ "component": code }))
            }
        }
        "rye_scaffold_test" => {
            let source = args["source"]
                .as_str()
                .ok_or("Missing 'source' parameter")?;
            let test_code = test_gen_mcp::generate_test_from_source(source);
            Ok(serde_json::Value::String(test_code))
        }
        "rye_component_usage_stats" => {
            let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("all");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            match action {
                "most_used" => {
                    let stats = usage_analytics::most_used(limit);
                    let entries: Vec<String> = stats.iter().map(|s| s.format_json()).collect();
                    Ok(serde_json::Value::String(format!(
                        "[{}]",
                        entries.join(",")
                    )))
                }
                "unused" => {
                    let unused = usage_analytics::unused_components();
                    Ok(serde_json::json!(unused))
                }
                _ => {
                    let stats = usage_analytics::all_stats();
                    let entries: Vec<String> = stats.iter().map(|s| s.format_json()).collect();
                    Ok(serde_json::Value::String(format!(
                        "[{}]",
                        entries.join(",")
                    )))
                }
            }
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

struct ServerState {
    initialized: bool,
}

static STATE: Mutex<ServerState> = Mutex::new(ServerState { initialized: false });

fn process_request(request: &serde_json::Value) -> Option<serde_json::Value> {
    let id = request.get("id").cloned();
    let method = request.get("method").and_then(|v| v.as_str())?;

    match method {
        "initialize" => {
            {
                let mut state = STATE.lock().unwrap();
                state.initialized = true;
            }
            Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "rye-mcp", "version": "0.1.0" }
                }
            }))
        }
        "initialized" => None,
        "tools/list" => {
            let tools_list: Vec<serde_json::Value> = tools()
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema,
                    })
                })
                .collect();
            Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools_list }
            }))
        }
        "tools/call" => {
            let params = request.get("params")?;
            let tool_name = params.get("name").and_then(|v| v.as_str())?;
            let arguments = params.get("arguments").unwrap_or(&serde_json::Value::Null);

            match handle_tool_call(tool_name, arguments) {
                Ok(result) => {
                    let content = match result {
                        serde_json::Value::String(s) => {
                            vec![serde_json::json!({ "type": "text", "text": s })]
                        }
                        other => vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&other).unwrap_or_default()
                        })],
                    };
                    Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": content }
                    }))
                }
                Err(e) => Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": e }
                })),
            }
        }
        _ => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {}", method) }
        })),
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let error = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                let _ = writeln!(stdout, "{}", error);
                let _ = stdout.flush();
                continue;
            }
        };

        if let Some(response) = process_request(&request) {
            let _ = writeln!(stdout, "{}", response);
            let _ = stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_count() {
        assert!(tools().len() >= 16);
    }

    #[test]
    fn test_explain_error() {
        let args = serde_json::json!({"code": "R802"});
        let result = handle_tool_call("rye_explain_error", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("R802"));
    }

    #[test]
    fn test_explain_error_json() {
        let args = serde_json::json!({"code": "R802", "json": true});
        let result = handle_tool_call("rye_explain_error", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explain_unknown_error() {
        let args = serde_json::json!({"code": "R999"});
        assert!(handle_tool_call("rye_explain_error", &args).is_err());
    }

    #[test]
    fn test_search_error_codes() {
        let args = serde_json::json!({"query": "signal"});
        assert!(handle_tool_call("rye_search_error_codes", &args).is_ok());
    }

    #[test]
    fn test_recovery_plan() {
        let args = serde_json::json!({"code": "R802"});
        assert!(handle_tool_call("rye_get_recovery_plan", &args).is_ok());
    }

    #[test]
    fn test_list_components() {
        let args = serde_json::json!({});
        assert!(handle_tool_call("rye_list_components", &args).is_ok());
    }

    #[test]
    fn test_list_prompt_templates() {
        let args = serde_json::json!({});
        assert!(handle_tool_call("rye_list_prompt_templates", &args).is_ok());
    }

    #[test]
    fn test_get_prompt_template() {
        let args = serde_json::json!({
            "id": "component",
            "values": { "name": "Button", "props": "label: String", "description": "a button", "events": "click" }
        });
        let result = handle_tool_call("rye_get_prompt_template", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("Button"));
    }

    #[test]
    fn test_review_code() {
        let args = serde_json::json!({"source": "#[component]\nfn MyComp() { template! { div { \"Hi\" } } }"});
        assert!(handle_tool_call("rye_review_code", &args).is_ok());
    }

    #[test]
    fn test_get_context() {
        let args = serde_json::json!({"max_tokens": 2000});
        assert!(handle_tool_call("rye_get_context", &args).is_ok());
    }

    #[test]
    fn test_scaffold_component() {
        let args = serde_json::json!({"name": "Button", "props": "label:String", "style": true, "test": true});
        assert!(handle_tool_call("rye_scaffold_component", &args).is_ok());
    }

    #[test]
    fn test_scaffold_test() {
        let args = serde_json::json!({"source": "#[component]\nfn Button() { div { \"Click\" } }"});
        assert!(handle_tool_call("rye_scaffold_test", &args).is_ok());
    }

    #[test]
    fn test_unknown_tool() {
        let args = serde_json::json!({});
        assert!(handle_tool_call("unknown_tool", &args).is_err());
    }

    #[test]
    fn test_process_initialize() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        let resp = process_request(&req);
        assert!(resp.is_some());
        assert!(resp.unwrap()["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn test_process_tools_list() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"});
        let resp = process_request(&req);
        assert!(resp.is_some());
        let resp = resp.unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert!(tools.len() >= 16);
    }

    #[test]
    fn test_process_tools_call() {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "rye_explain_error", "arguments": {"code": "R802"} }
        });
        let resp = process_request(&req);
        assert!(resp.is_some());
        let resp = resp.unwrap();
        assert!(resp["result"]["content"].is_array());
    }

    #[test]
    fn test_process_unknown_method() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 4, "method": "unknown/method"});
        let resp = process_request(&req);
        assert!(resp.is_some());
        assert_eq!(resp.unwrap()["error"]["code"], -32601);
    }
}
