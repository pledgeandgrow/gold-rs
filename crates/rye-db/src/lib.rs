//! # rye-db
//!
//! Database integration layer for rye — connection pooling, query builder,
//! and reactive queries. `use_query_db()` that re-runs when dependent signals change.
//! Adapters for SQLx, Diesel, SeaORM.

pub mod pool;
pub mod query;
pub mod reactive;

pub use pool::{ConnectionPool, PoolConfig, PooledConnection};
pub use query::{Condition, OrderDirection, Query, QueryBuilder, QueryResult, ValueType};
pub use reactive::{use_query_db, QueryState, ReactiveQuery};
