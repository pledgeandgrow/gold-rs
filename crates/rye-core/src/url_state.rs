//! Reactive URL state synchronization.
//!
//! `use_url_state()` hook that bidirectionally syncs a signal with
//! URL search params. Changing the signal updates the URL; navigating
//! back/forward updates the signal.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A URL state binding — syncs a signal-like value with a URL search param.
///
/// In a browser environment, this uses `window.location` and the
/// History API. In non-browser contexts (tests, SSR), it operates
/// on an in-memory URL representation.
pub struct UrlState {
    key: String,
    value: Rc<RefCell<String>>,
    listeners: Rc<RefCell<Vec<Rc<dyn Fn(&str)>>>>,
}

impl UrlState {
    /// Create a new URL state binding for the given key.
    ///
    /// Reads the initial value from the current URL's search params.
    /// If the key is not present, uses the default.
    pub fn new(key: &str, default: &str) -> Self {
        let initial = Self::read_url_param(key).unwrap_or_else(|| default.to_string());
        Self {
            key: key.to_string(),
            value: Rc::new(RefCell::new(initial)),
            listeners: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> String {
        self.value.borrow().clone()
    }

    /// Set a new value — updates both the in-memory state and the URL.
    pub fn set(&self, value: &str) {
        *self.value.borrow_mut() = value.to_string();
        Self::write_url_param(&self.key, value);

        // Notify listeners
        let listeners = self.listeners.borrow().clone();
        for listener in &listeners {
            listener(value);
        }
    }

    /// Register a callback that fires when the value changes.
    pub fn on_change<F: Fn(&str) + 'static>(&self, callback: F) {
        self.listeners.borrow_mut().push(Rc::new(callback));
    }

    /// Get the URL parameter key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Sync from the URL — reads the current URL and updates the value.
    pub fn sync_from_url(&self) {
        if let Some(val) = Self::read_url_param(&self.key) {
            let changed = self.value.borrow().as_str() != val.as_str();
            *self.value.borrow_mut() = val.clone();
            if changed {
                let listeners = self.listeners.borrow().clone();
                for listener in &listeners {
                    listener(&val);
                }
            }
        }
    }

    /// Remove the parameter from the URL.
    pub fn clear(&self) {
        *self.value.borrow_mut() = String::new();
        Self::remove_url_param(&self.key);
    }

    /// Read a URL search parameter.
    fn read_url_param(key: &str) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let location = web_sys::window()?.location();
            let search = location.search().ok()?;
            let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
            params.get(key)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            IN_MEMORY_URL.with(|url| {
                url.borrow().get(key).cloned()
            })
        }
    }

    /// Write a URL search parameter.
    fn write_url_param(key: &str, value: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                if let Ok(search) = location.search() {
                    if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                        params.set(key, value);
                        let new_search = params.to_string();
                        let path = location.pathname().unwrap_or_default();
                        let url = format!("{}?{}", path, new_search);
                        if let Ok(history) = window.history() {
                            let _ = history.push_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&url),
                            );
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            IN_MEMORY_URL.with(|url| {
                url.borrow_mut().insert(key.to_string(), value.to_string());
            });
        }
    }

    /// Remove a URL search parameter.
    fn remove_url_param(key: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                if let Ok(search) = location.search() {
                    if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                        params.delete(key);
                        let new_search = params.to_string();
                        let path = location.pathname().unwrap_or_default();
                        let url = format!("{}?{}", path, new_search);
                        if let Ok(history) = window.history() {
                            let _ = history.push_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&url),
                            );
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            IN_MEMORY_URL.with(|url| {
                url.borrow_mut().remove(key);
            });
        }
    }
}

impl Clone for UrlState {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: Rc::clone(&self.value),
            listeners: Rc::clone(&self.listeners),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static IN_MEMORY_URL: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Clear all in-memory URL state (for testing).
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_url_state() {
    IN_MEMORY_URL.with(|url| url.borrow_mut().clear());
}

/// Get all URL params as a HashMap (for debugging/inspection).
pub fn get_all_url_params() -> HashMap<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut result = HashMap::new();
        if let Some(window) = web_sys::window() {
            let location = window.location();
            if let Ok(search) = location.search() {
                result = parse_query_string(&search);
            }
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        IN_MEMORY_URL.with(|url| url.borrow().clone())
    }
}

/// Parse a URL query string into a HashMap.
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let query = query.trim_start_matches('?');
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some(eq_pos) = pair.find('=') {
            let key = pair[..eq_pos].to_string();
            let value = pair[eq_pos + 1..].to_string();
            params.insert(key, value);
        } else {
            params.insert(pair.to_string(), String::new());
        }
    }

    params
}

/// Build a URL query string from a HashMap.
pub fn build_query_string(params: &HashMap<String, String>) -> String {
    let pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    if pairs.is_empty() {
        String::new()
    } else {
        format!("?{}", pairs.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_state_default() {
        clear_url_state();
        let state = UrlState::new("filter", "all");
        assert_eq!(state.get(), "all");
    }

    #[test]
    fn test_url_state_set() {
        clear_url_state();
        let state = UrlState::new("sort", "asc");
        state.set("desc");
        assert_eq!(state.get(), "desc");
        // Verify it was written to URL
        let params = get_all_url_params();
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_url_state_reads_from_url() {
        clear_url_state();
        IN_MEMORY_URL.with(|url| {
            url.borrow_mut().insert("page".to_string(), "5".to_string());
        });
        let state = UrlState::new("page", "1");
        assert_eq!(state.get(), "5");
    }

    #[test]
    fn test_url_state_on_change() {
        clear_url_state();
        let state = UrlState::new("tab", "home");
        let received = Rc::new(RefCell::new(String::new()));
        let received_clone = Rc::clone(&received);
        state.on_change(move |v| {
            *received_clone.borrow_mut() = v.to_string();
        });
        state.set("settings");
        assert_eq!(*received.borrow(), "settings");
    }

    #[test]
    fn test_url_state_clear() {
        clear_url_state();
        let state = UrlState::new("temp", "value");
        state.clear();
        assert_eq!(state.get(), "");
        let params = get_all_url_params();
        assert!(!params.contains_key("temp"));
    }

    #[test]
    fn test_url_state_sync_from_url() {
        clear_url_state();
        let state = UrlState::new("q", "initial");
        // Simulate external URL change
        IN_MEMORY_URL.with(|url| {
            url.borrow_mut().insert("q".to_string(), "external".to_string());
        });
        state.sync_from_url();
        assert_eq!(state.get(), "external");
    }

    #[test]
    fn test_url_state_clone() {
        clear_url_state();
        let state = UrlState::new("key", "val1");
        let state2 = state.clone();
        state.set("val2");
        assert_eq!(state2.get(), "val2");
    }

    #[test]
    fn test_parse_query_string() {
        let params = parse_query_string("?a=1&b=2&c=3");
        assert_eq!(params.get("a"), Some(&"1".to_string()));
        assert_eq!(params.get("b"), Some(&"2".to_string()));
        assert_eq!(params.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn test_parse_query_string_no_prefix() {
        let params = parse_query_string("a=1&b=2");
        assert_eq!(params.get("a"), Some(&"1".to_string()));
    }

    #[test]
    fn test_parse_query_string_empty() {
        let params = parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("a".to_string(), "1".to_string());
        params.insert("b".to_string(), "2".to_string());
        let qs = build_query_string(&params);
        assert!(qs.starts_with('?'));
        assert!(qs.contains("a=1"));
        assert!(qs.contains("b=2"));
    }

    #[test]
    fn test_build_query_string_empty() {
        let params = HashMap::new();
        let qs = build_query_string(&params);
        assert_eq!(qs, "");
    }
}
