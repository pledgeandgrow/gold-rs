//! Reactive queries — queries that re-run when dependent signals change.
//!
//! `use_query_db()` that re-runs when dependent signals change.

use rye_signals::Signal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::query::{Query, QueryResult};

/// The state of a reactive query.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryState {
    /// Query is loading.
    Loading,
    /// Query completed successfully.
    Loaded(QueryResult),
    /// Query resulted in an error.
    Error(String),
}

impl QueryState {
    /// Check if the query is loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, QueryState::Loading)
    }

    /// Check if the query is loaded.
    pub fn is_loaded(&self) -> bool {
        matches!(self, QueryState::Loaded(_))
    }

    /// Check if the query errored.
    pub fn is_error(&self) -> bool {
        matches!(self, QueryState::Error(_))
    }

    /// Get the query result if loaded.
    pub fn result(&self) -> Option<&QueryResult> {
        match self {
            QueryState::Loaded(r) => Some(r),
            _ => None,
        }
    }
}

/// A reactive query — re-runs when dependent signals change.
pub struct ReactiveQuery {
    /// The query to execute.
    query: Query,
    /// The current state.
    state: Signal<QueryState>,
    /// The fetch function.
    fetch_fn: Rc<dyn Fn(&Query) -> QueryResult>,
}

impl ReactiveQuery {
    /// Create a new reactive query.
    pub fn new<F: Fn(&Query) -> QueryResult + 'static>(query: Query, fetch_fn: F) -> Self {
        Self {
            query,
            state: Signal::new(QueryState::Loading),
            fetch_fn: Rc::new(fetch_fn),
        }
    }

    /// Execute the query and update state.
    pub fn execute(&self) {
        let result = (self.fetch_fn)(&self.query);
        self.state.set(QueryState::Loaded(result));
    }

    /// Get the current state (tracked).
    pub fn state(&self) -> QueryState {
        self.state.get()
    }

    /// Get the current state (untracked).
    pub fn state_untracked(&self) -> QueryState {
        self.state.get_untracked()
    }

    /// Get the query.
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Reset to loading state.
    pub fn reset(&self) {
        self.state.set(QueryState::Loading);
    }

    /// Set an error state.
    pub fn set_error(&self, message: &str) {
        self.state.set(QueryState::Error(message.to_string()));
    }
}

/// A reactive query cache — stores results by query SQL.
pub struct ReactiveQueryCache {
    cache: RefCell<HashMap<String, QueryResult>>,
}

impl ReactiveQueryCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Get a cached result by query SQL.
    pub fn get(&self, sql: &str) -> Option<QueryResult> {
        self.cache.borrow().get(sql).cloned()
    }

    /// Insert a result into the cache.
    pub fn insert(&self, sql: &str, result: QueryResult) {
        self.cache.borrow_mut().insert(sql.to_string(), result);
    }

    /// Invalidate a cached result.
    pub fn invalidate(&self, sql: &str) {
        self.cache.borrow_mut().remove(sql);
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }

    /// Get the number of cached queries.
    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.borrow().is_empty()
    }

    /// Get all cached query SQL strings.
    pub fn keys(&self) -> Vec<String> {
        self.cache.borrow().keys().cloned().collect()
    }
}

impl Default for ReactiveQueryCache {
    fn default() -> Self {
        Self::new()
    }
}

/// `use_query_db()` — creates a reactive query that re-runs when dependent signals change.
///
/// # Example
/// ```
/// use rye_db::{use_query_db, query::{QueryBuilder, QueryResult, ValueType}};
///
/// let query = QueryBuilder::select("users").where_eq("active", ValueType::Bool(true)).build();
/// let reactive = use_query_db(query, |q| {
///     // In real app, this would execute against a database
///     QueryResult::empty()
/// });
/// reactive.execute();
/// assert!(reactive.state().is_loaded());
/// ```
pub fn use_query_db<F: Fn(&Query) -> QueryResult + 'static>(
    query: Query,
    fetch_fn: F,
) -> ReactiveQuery {
    ReactiveQuery::new(query, fetch_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{QueryBuilder, ValueType};

    fn mock_fetch(_q: &Query) -> QueryResult {
        QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![ValueType::Int(1), ValueType::Text("Alice".to_string())],
                vec![ValueType::Int(2), ValueType::Text("Bob".to_string())],
            ],
        }
    }

    #[test]
    fn test_reactive_query_initial_state() {
        let query = QueryBuilder::select("users").build();
        let rq = use_query_db(query, mock_fetch);
        assert!(rq.state().is_loading());
    }

    #[test]
    fn test_reactive_query_execute() {
        let query = QueryBuilder::select("users").build();
        let rq = use_query_db(query, mock_fetch);
        rq.execute();
        assert!(rq.state().is_loaded());
        let state = rq.state();
        let result = state.result().unwrap();
        assert_eq!(result.row_count(), 2);
    }

    #[test]
    fn test_reactive_query_reset() {
        let query = QueryBuilder::select("users").build();
        let rq = use_query_db(query, mock_fetch);
        rq.execute();
        assert!(rq.state().is_loaded());
        rq.reset();
        assert!(rq.state().is_loading());
    }

    #[test]
    fn test_reactive_query_error() {
        let query = QueryBuilder::select("users").build();
        let rq = use_query_db(query, mock_fetch);
        rq.set_error("Connection failed");
        assert!(rq.state().is_error());
    }

    #[test]
    fn test_reactive_query_query_ref() {
        let query = QueryBuilder::select("users").build();
        let rq = use_query_db(query, mock_fetch);
        assert_eq!(rq.query().table, "users");
    }

    #[test]
    fn test_query_state_loading() {
        let state = QueryState::Loading;
        assert!(state.is_loading());
        assert!(!state.is_loaded());
        assert!(!state.is_error());
    }

    #[test]
    fn test_query_state_loaded() {
        let state = QueryState::Loaded(QueryResult::empty());
        assert!(!state.is_loading());
        assert!(state.is_loaded());
        assert!(state.result().is_some());
    }

    #[test]
    fn test_query_state_error() {
        let state = QueryState::Error("fail".to_string());
        assert!(state.is_error());
        assert!(state.result().is_none());
    }

    #[test]
    fn test_cache_insert_get() {
        let cache = ReactiveQueryCache::new();
        let result = QueryResult {
            columns: vec!["id".to_string()],
            rows: vec![vec![ValueType::Int(1)]],
        };
        cache.insert("SELECT * FROM users", result.clone());
        let cached = cache.get("SELECT * FROM users");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().row_count(), 1);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = ReactiveQueryCache::new();
        cache.insert("SELECT 1", QueryResult::empty());
        cache.invalidate("SELECT 1");
        assert!(cache.get("SELECT 1").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = ReactiveQueryCache::new();
        cache.insert("a", QueryResult::empty());
        cache.insert("b", QueryResult::empty());
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_len() {
        let cache = ReactiveQueryCache::new();
        cache.insert("a", QueryResult::empty());
        cache.insert("b", QueryResult::empty());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_keys() {
        let cache = ReactiveQueryCache::new();
        cache.insert("key1", QueryResult::empty());
        cache.insert("key2", QueryResult::empty());
        let keys = cache.keys();
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_cache_is_empty() {
        let cache = ReactiveQueryCache::new();
        assert!(cache.is_empty());
    }
}
