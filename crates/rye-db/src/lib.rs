//! # rye-db
//!
//! Database integration layer for rye — connection pooling, query builder,
//! and reactive queries. `use_query_db()` that re-runs when dependent signals change.
//! Adapters for SQLx, Diesel, SeaORM.

pub mod pool;
pub mod query;
pub mod reactive;

pub use pool::{ConnectionPool, PooledConnection, PoolConfig};
pub use query::{QueryBuilder, Query, QueryResult, ValueType, Condition, OrderDirection};
pub use reactive::{use_query_db, ReactiveQuery, QueryState};
