//! Server actions — type-safe RPC from client to server.
//!
//! The `#[server]` macro (in `rye-macros`) transforms a Rust function into
//! a server-callable action. On the server, the function runs directly.
//! On the client (Wasm), the function is replaced with a stub that
//! serializes arguments via `rye-serialize` and calls the server via HTTP.
//!
//! ## Architecture
//!
//! ```text
//! Client (Wasm)                          Server
//! ┌──────────┐    POST /api/actions      ┌──────────────┐
//! │ #[server] │ ──┐   action_id + input   │  Registry     │
//! │  stub     │   │ ──────────────────►   │  invoke(id,  │
//! │           │   │                       │   input)      │
//! │           │   │ ◄──────────────────   │  → output    │
//! └──────────┘   │                       └──────────────┘
//!                 │
//!    rye_serialize::serialize(args)
//!    → call_server(id, serialized)
//!    → rye_serialize::deserialize(output)
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! #[server]
//! async fn create_user(name: String, email: String) -> Result<User, ServerError> {
//!     db.insert(name, email).await
//! }
//! ```
//!
//! On the client, calling `create_user(name, email)` makes an HTTP POST
//! to `/api/actions/create_user` with the serialized arguments.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

/// A boxed async future returned by server action handlers.
pub type ActionFuture = Pin<Box<dyn Future<Output = Result<String, ServerError>> + Send>>;

/// A server action handler — deserializes input, runs the action, serializes output.
pub type ActionHandler = Box<dyn Fn(&str) -> ActionFuture + Send + Sync>;

/// Error type for server actions.
#[derive(Debug, Clone)]
pub enum ServerError {
    /// Network error (fetch failed, timeout, etc.).
    Network(String),
    /// Server returned a non-200 status code.
    Status(u16),
    /// Failed to deserialize the server's response.
    Deserialize(String),
    /// Failed to serialize the request arguments.
    Serialize(String),
    /// Custom error message from the server action.
    Message(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Network(msg) => write!(f, "Network error: {}", msg),
            ServerError::Status(code) => write!(f, "Server returned status: {}", code),
            ServerError::Deserialize(msg) => write!(f, "Deserialization error: {}", msg),
            ServerError::Serialize(msg) => write!(f, "Serialization error: {}", msg),
            ServerError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

impl From<String> for ServerError {
    fn from(msg: String) -> Self {
        ServerError::Message(msg)
    }
}

// === Registry ===

/// Global registry of server actions.
static REGISTRY: Mutex<Option<HashMap<String, ActionHandler>>> = Mutex::new(None);

/// Register a server action handler.
///
/// Called automatically by the `#[server]` macro on the server target.
/// Each action is identified by a unique string ID (typically the function name).
pub fn register_action(id: &str, handler: ActionHandler) {
    let mut registry = REGISTRY.lock().unwrap();
    if registry.is_none() {
        *registry = Some(HashMap::new());
    }
    registry.as_mut().unwrap().insert(id.to_string(), handler);
}

/// Invoke a registered server action by ID with serialized input.
///
/// Used on the server side to dispatch incoming HTTP requests to the
/// correct action handler. Also used as a fallback in tests.
pub async fn invoke_action(id: &str, input: &str) -> Result<String, ServerError> {
    let future = {
        let registry = REGISTRY.lock().unwrap();
        if let Some(ref reg) = *registry {
            reg.get(id).map(|h| h(input))
        } else {
            None
        }
    };

    match future {
        Some(fut) => fut.await,
        None => Err(ServerError::Message(format!(
            "Unknown server action: {}",
            id
        ))),
    }
}

/// Get a list of all registered action IDs.
pub fn list_actions() -> Vec<String> {
    let registry = REGISTRY.lock().unwrap();
    if let Some(ref reg) = *registry {
        reg.keys().cloned().collect()
    } else {
        Vec::new()
    }
}

// === Transport ===

/// Transport trait for calling server actions from the client.
///
/// On Wasm, this is backed by `web_sys::fetch`.
/// On native, this can be backed by any HTTP client.
/// For testing, `InProcessTransport` calls actions directly.
pub trait ServerTransport: Send + Sync + 'static {
    /// Call a server action by ID with serialized input.
    fn call(&self, action_id: &str, input: &str) -> ActionFuture;
}

/// In-process transport — invokes actions directly without HTTP.
///
/// Used for testing and SSR (where the server and client run in the same process).
pub struct InProcessTransport;

impl ServerTransport for InProcessTransport {
    fn call(&self, action_id: &str, input: &str) -> ActionFuture {
        let id = action_id.to_string();
        let input = input.to_string();
        Box::pin(async move { invoke_action(&id, &input).await })
    }
}

/// Global transport for client-side server action calls.
static TRANSPORT: Mutex<Option<Box<dyn ServerTransport>>> = Mutex::new(None);

/// Set the global server transport.
///
/// Called once at application startup. On Wasm, this should be a
/// `WebFetchTransport`. For testing, use `InProcessTransport`.
pub fn set_transport(transport: Box<dyn ServerTransport>) {
    let mut t = TRANSPORT.lock().unwrap();
    *t = Some(transport);
}

/// Call a server action via the global transport.
///
/// This is the function called by `#[server]` client stubs.
/// If no transport is set, falls back to in-process invocation (for testing).
pub async fn call_server(action_id: &str, input: &str) -> Result<String, ServerError> {
    let future = {
        let transport = TRANSPORT.lock().unwrap();
        if let Some(ref t) = *transport {
            Some(t.call(action_id, input))
        } else {
            None
        }
    };

    match future {
        Some(fut) => fut.await,
        None => {
            // Fallback: invoke in-process (for tests without a transport set)
            invoke_action(action_id, input).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_list() {
        // Clear registry for test isolation
        {
            let mut registry = REGISTRY.lock().unwrap();
            *registry = Some(HashMap::new());
        }

        register_action(
            "test_action",
            Box::new(|_input: &str| Box::pin(async { Ok("42".to_string()) })),
        );

        let actions = list_actions();
        assert!(actions.contains(&"test_action".to_string()));
    }

    #[tokio::test]
    async fn test_invoke_action() {
        // Use unique action ID to avoid races with other tests
        register_action(
            "test_invoke_add",
            Box::new(|input: &str| {
                let input = input.to_string();
                Box::pin(async move {
                    let (a, b): (i32, i32) =
                        rye_serialize::deserialize(&input).ok_or_else(|| {
                            ServerError::Deserialize("Failed to parse input".to_string())
                        })?;
                    Ok(rye_serialize::serialize(&(a + b)))
                })
            }),
        );

        let input = rye_serialize::serialize(&(3i32, 4i32));
        let result = invoke_action("test_invoke_add", &input).await.unwrap();
        let value: i32 = rye_serialize::deserialize(&result).unwrap();
        assert_eq!(value, 7);
    }

    #[tokio::test]
    async fn test_call_server_in_process() {
        // Use unique action ID to avoid races with other tests
        register_action(
            "test_call_echo",
            Box::new(|input: &str| {
                let input = input.to_string();
                Box::pin(async move { Ok(input) })
            }),
        );

        let result = call_server("test_call_echo", "hello").await.unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_unknown_action() {
        let result = invoke_action("nonexistent_test_action_12345", "").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ServerError::Message(msg) => assert!(msg.contains("nonexistent_test_action_12345")),
            _ => panic!("Expected ServerError::Message"),
        }
    }
}
