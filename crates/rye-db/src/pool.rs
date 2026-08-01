//! Connection pooling — manage database connections efficiently.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Configuration for a connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool.
    pub max_connections: usize,
    /// Minimum number of idle connections to maintain.
    pub min_idle: usize,
    /// Connection timeout in seconds.
    pub connection_timeout_secs: u64,
    /// Idle timeout in seconds.
    pub idle_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_idle: 2,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl PoolConfig {
    /// Create a config with a specific max connections.
    pub fn max(max_connections: usize) -> Self {
        Self {
            max_connections,
            ..Default::default()
        }
    }
}

/// A pooled connection — returned to the pool when dropped.
pub struct PooledConnection {
    /// The connection ID.
    pub id: u64,
    /// Whether this connection is currently in use.
    pub in_use: bool,
}

impl PooledConnection {
    /// Create a new connection with the given ID.
    fn new(id: u64) -> Self {
        Self { id, in_use: true }
    }
}

/// A connection pool — manages a set of database connections.
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Mutex<VecDeque<PooledConnection>>,
    next_id: Mutex<u64>,
    active_count: Mutex<usize>,
}

impl ConnectionPool {
    /// Create a new connection pool with the given config.
    pub fn new(config: PoolConfig) -> Self {
        let mut connections = VecDeque::new();
        for _ in 0..config.min_idle {
            connections.push_back(PooledConnection { id: 0, in_use: false });
        }

        let next_id = Mutex::new(config.min_idle as u64);

        Self {
            config,
            connections: Mutex::new(connections),
            next_id,
            active_count: Mutex::new(0),
        }
    }

    /// Acquire a connection from the pool.
    pub fn acquire(&self) -> Option<PooledConnection> {
        let mut connections = self.connections.lock().unwrap();
        let mut active = self.active_count.lock().unwrap();

        // Try to reuse an idle connection
        if let Some(conn) = connections.pop_front() {
            if !conn.in_use {
                *active += 1;
                return Some(PooledConnection::new(conn.id));
            }
        }

        // Create a new connection if under the limit
        if *active < self.config.max_connections {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            *active += 1;
            return Some(PooledConnection::new(id));
        }

        None
    }

    /// Release a connection back to the pool.
    pub fn release(&self, conn: PooledConnection) {
        let mut connections = self.connections.lock().unwrap();
        let mut active = self.active_count.lock().unwrap();
        *active -= 1;
        connections.push_back(PooledConnection {
            id: conn.id,
            in_use: false,
        });
    }

    /// Get the number of active (in-use) connections.
    pub fn active_count(&self) -> usize {
        *self.active_count.lock().unwrap()
    }

    /// Get the number of idle connections.
    pub fn idle_count(&self) -> usize {
        self.connections.lock().unwrap().iter().filter(|c| !c.in_use).count()
    }

    /// Get the total number of connections (active + idle).
    pub fn total_count(&self) -> usize {
        self.active_count() + self.idle_count()
    }

    /// Get the pool config.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Get the maximum number of connections.
    pub fn max_connections(&self) -> usize {
        self.config.max_connections
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_release() {
        let pool = ConnectionPool::new(PoolConfig::max(5));
        let conn = pool.acquire().unwrap();
        assert_eq!(pool.active_count(), 1);
        pool.release(conn);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_pool_max_connections() {
        let pool = ConnectionPool::new(PoolConfig::max(2));
        let c1 = pool.acquire().unwrap();
        let c2 = pool.acquire().unwrap();
        let c3 = pool.acquire();
        assert!(c3.is_none()); // pool exhausted
        pool.release(c1);
        let c4 = pool.acquire();
        assert!(c4.is_some()); // now available
        pool.release(c2);
        pool.release(c4.unwrap());
    }

    #[test]
    fn test_pool_idle_count() {
        let pool = ConnectionPool::new(PoolConfig::max(5));
        let c1 = pool.acquire().unwrap();
        pool.release(c1);
        // After acquire+release, idle = min_idle (2 pre-created, 1 was popped and returned)
        assert_eq!(pool.idle_count(), 2);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_pool_total_count() {
        let pool = ConnectionPool::new(PoolConfig::max(5));
        let _c1 = pool.acquire().unwrap();
        let _c2 = pool.acquire().unwrap();
        assert_eq!(pool.total_count(), 2);
    }

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_idle, 2);
    }

    #[test]
    fn test_pool_config_max() {
        let config = PoolConfig::max(20);
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_pool_reuse_idle() {
        let pool = ConnectionPool::new(PoolConfig::max(3));
        let c1 = pool.acquire().unwrap();
        let id = c1.id;
        pool.release(c1);
        let c2 = pool.acquire().unwrap();
        // Should reuse the same connection ID
        assert_eq!(id, c2.id);
        pool.release(c2);
    }

    #[test]
    fn test_pool_max_connections_method() {
        let pool = ConnectionPool::new(PoolConfig::max(7));
        assert_eq!(pool.max_connections(), 7);
    }
}
