//! Goal 189: Distributed SSR with session affinity.
//!
//! For multi-server SSR deployments, route requests to the server that holds
//! the user's session. Session-aware load balancing.

use std::collections::HashMap;
use std::sync::Mutex;

/// A server node in a distributed SSR cluster.
#[derive(Debug, Clone)]
pub struct ServerNode {
    /// Unique server ID.
    pub id: String,
    /// Server URL (e.g. "https://ssr-1.example.com").
    pub url: String,
    /// Server region (e.g. "us-east-1").
    pub region: String,
    /// Whether the server is healthy.
    pub healthy: bool,
    /// Current load (0.0 to 1.0).
    pub load: f64,
    /// Maximum concurrent requests.
    pub max_connections: usize,
    /// Current active connections.
    pub active_connections: usize,
}

impl ServerNode {
    /// Create a new server node.
    pub fn new(id: &str, url: &str, region: &str) -> Self {
        Self {
            id: id.to_string(),
            url: url.to_string(),
            region: region.to_string(),
            healthy: true,
            load: 0.0,
            max_connections: 1000,
            active_connections: 0,
        }
    }

    /// Get the utilization ratio (0.0 to 1.0).
    pub fn utilization(&self) -> f64 {
        if self.max_connections == 0 {
            return 1.0;
        }
        self.active_connections as f64 / self.max_connections as f64
    }

    /// Check if the server can accept more connections.
    pub fn can_accept(&self) -> bool {
        self.healthy && self.active_connections < self.max_connections
    }
}

/// Session affinity strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffinityStrategy {
    /// Route to the server that holds the session.
    Sticky,
    /// Route to the server with the least connections.
    LeastConnections,
    /// Route to the server in the same region.
    RegionPreferred,
    /// Round-robin with session pinning.
    RoundRobin,
}

/// The session affinity router — routes requests to the correct server.
pub struct SessionAffinityRouter {
    servers: Vec<ServerNode>,
    session_map: Mutex<HashMap<String, String>>, // session_id -> server_id
    strategy: AffinityStrategy,
    round_robin_idx: Mutex<usize>,
}

impl SessionAffinityRouter {
    /// Create a new router with the given strategy.
    pub fn new(strategy: AffinityStrategy) -> Self {
        Self {
            servers: Vec::new(),
            session_map: Mutex::new(HashMap::new()),
            strategy,
            round_robin_idx: Mutex::new(0),
        }
    }

    /// Add a server node.
    pub fn add_server(&mut self, server: ServerNode) {
        self.servers.push(server);
    }

    /// Remove a server node by ID.
    pub fn remove_server(&mut self, id: &str) {
        self.servers.retain(|s| s.id != id);
        // Clean up session mappings to this server
        let mut map = self.session_map.lock().unwrap();
        map.retain(|_, server_id| server_id != id);
    }

    /// Get all servers.
    pub fn servers(&self) -> &[ServerNode] {
        &self.servers
    }

    /// Get healthy servers.
    pub fn healthy_servers(&self) -> Vec<&ServerNode> {
        self.servers.iter().filter(|s| s.healthy).collect()
    }

    /// Route a request to a server, considering session affinity.
    pub fn route(&self, session_id: Option<&str>, region: Option<&str>) -> Option<&ServerNode> {
        // Check if session is already pinned to a server
        if let Some(sid) = session_id {
            let map = self.session_map.lock().unwrap();
            if let Some(server_id) = map.get(sid) {
                if let Some(server) = self
                    .servers
                    .iter()
                    .find(|s| s.id == *server_id && s.healthy)
                {
                    return Some(server);
                }
            }
        }

        // No existing session or server unavailable — pick a new server
        let server = match self.strategy {
            AffinityStrategy::Sticky | AffinityStrategy::RoundRobin => self.round_robin_select(),
            AffinityStrategy::LeastConnections => self.least_connections_select(),
            AffinityStrategy::RegionPreferred => self.region_preferred_select(region),
        };

        // Pin the session to the selected server
        if let (Some(sid), Some(ref server)) = (session_id, server) {
            let mut map = self.session_map.lock().unwrap();
            map.insert(sid.to_string(), server.id.clone());
        }

        server
    }

    /// Round-robin server selection.
    fn round_robin_select(&self) -> Option<&ServerNode> {
        let healthy: Vec<&ServerNode> = self.servers.iter().filter(|s| s.healthy).collect();
        if healthy.is_empty() {
            return None;
        }

        let mut idx = self.round_robin_idx.lock().unwrap();
        let server = &healthy[*idx % healthy.len()];
        *idx = (*idx + 1) % healthy.len();
        Some(server)
    }

    /// Least connections server selection.
    fn least_connections_select(&self) -> Option<&ServerNode> {
        self.servers
            .iter()
            .filter(|s| s.healthy && s.can_accept())
            .min_by_key(|s| s.active_connections)
    }

    /// Region-preferred server selection.
    fn region_preferred_select(&self, preferred_region: Option<&str>) -> Option<&ServerNode> {
        if let Some(region) = preferred_region {
            // Try to find a healthy server in the preferred region
            let in_region: Vec<&ServerNode> = self
                .servers
                .iter()
                .filter(|s| s.healthy && s.region == region)
                .collect();
            if !in_region.is_empty() {
                return in_region
                    .iter()
                    .min_by_key(|s| s.active_connections)
                    .copied();
            }
        }
        // Fall back to least connections
        self.least_connections_select()
    }

    /// Pin a session to a specific server.
    pub fn pin_session(&self, session_id: &str, server_id: &str) {
        let mut map = self.session_map.lock().unwrap();
        map.insert(session_id.to_string(), server_id.to_string());
    }

    /// Unpin a session.
    pub fn unpin_session(&self, session_id: &str) {
        let mut map = self.session_map.lock().unwrap();
        map.remove(session_id);
    }

    /// Get the server a session is pinned to.
    pub fn get_session_server(&self, session_id: &str) -> Option<String> {
        let map = self.session_map.lock().unwrap();
        map.get(session_id).cloned()
    }

    /// Mark a server as unhealthy.
    pub fn mark_unhealthy(&mut self, server_id: &str) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.id == server_id) {
            server.healthy = false;
        }
    }

    /// Mark a server as healthy.
    pub fn mark_healthy(&mut self, server_id: &str) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.id == server_id) {
            server.healthy = true;
        }
    }

    /// Update server load.
    pub fn update_load(&mut self, server_id: &str, active_connections: usize) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.id == server_id) {
            server.active_connections = active_connections;
            server.load = server.utilization();
        }
    }

    /// Get the number of registered sessions.
    pub fn session_count(&self) -> usize {
        self.session_map.lock().unwrap().len()
    }

    /// Get the number of servers.
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// Get the strategy.
    pub fn strategy(&self) -> AffinityStrategy {
        self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_node_new() {
        let node = ServerNode::new("ssr-1", "https://ssr-1.example.com", "us-east-1");
        assert_eq!(node.id, "ssr-1");
        assert!(node.healthy);
        assert!(node.can_accept());
    }

    #[test]
    fn test_server_node_utilization() {
        let mut node = ServerNode::new("s1", "url", "region");
        node.active_connections = 50;
        node.max_connections = 100;
        assert_eq!(node.utilization(), 0.5);
    }

    #[test]
    fn test_server_node_can_accept() {
        let mut node = ServerNode::new("s1", "url", "region");
        node.healthy = false;
        assert!(!node.can_accept());
        node.healthy = true;
        node.active_connections = 100;
        node.max_connections = 100;
        assert!(!node.can_accept());
    }

    #[test]
    fn test_router_sticky_session() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::Sticky);
        router.add_server(ServerNode::new("s1", "url1", "us-east"));
        router.add_server(ServerNode::new("s2", "url2", "us-west"));

        // First request — no session, picks a server
        let server1 = router.route(None, None).unwrap();
        let session_id = "session-1";
        router.pin_session(session_id, &server1.id);

        // Second request with same session — should route to same server
        let server2 = router.route(Some(session_id), None).unwrap();
        assert_eq!(server1.id, server2.id);
    }

    #[test]
    fn test_router_round_robin() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::RoundRobin);
        router.add_server(ServerNode::new("s1", "url1", "r1"));
        router.add_server(ServerNode::new("s2", "url2", "r2"));
        router.add_server(ServerNode::new("s3", "url3", "r3"));

        let s1 = router.route(None, None).unwrap();
        let s2 = router.route(None, None).unwrap();
        let s3 = router.route(None, None).unwrap();

        // Should cycle through all servers
        assert_ne!(s1.id, s2.id);
        assert_ne!(s2.id, s3.id);
    }

    #[test]
    fn test_router_least_connections() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::LeastConnections);
        let mut s1 = ServerNode::new("s1", "url1", "r1");
        s1.active_connections = 50;
        let mut s2 = ServerNode::new("s2", "url2", "r2");
        s2.active_connections = 10;
        router.add_server(s1);
        router.add_server(s2);

        let server = router.route(None, None).unwrap();
        assert_eq!(server.id, "s2"); // s2 has fewer connections
    }

    #[test]
    fn test_router_region_preferred() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::RegionPreferred);
        router.add_server(ServerNode::new("s1", "url1", "us-east"));
        router.add_server(ServerNode::new("s2", "url2", "us-west"));

        let server = router.route(None, Some("us-west")).unwrap();
        assert_eq!(server.region, "us-west");
    }

    #[test]
    fn test_router_region_preferred_fallback() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::RegionPreferred);
        router.add_server(ServerNode::new("s1", "url1", "us-east"));

        // No server in eu-west — should fall back
        let server = router.route(None, Some("eu-west")).unwrap();
        assert_eq!(server.id, "s1");
    }

    #[test]
    fn test_router_mark_unhealthy() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::RoundRobin);
        router.add_server(ServerNode::new("s1", "url1", "r1"));
        router.add_server(ServerNode::new("s2", "url2", "r2"));

        router.mark_unhealthy("s1");
        let healthy = router.healthy_servers();
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].id, "s2");
    }

    #[test]
    fn test_router_remove_server() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::Sticky);
        router.add_server(ServerNode::new("s1", "url1", "r1"));
        router.add_server(ServerNode::new("s2", "url2", "r2"));
        router.pin_session("sess1", "s1");

        router.remove_server("s1");
        assert_eq!(router.server_count(), 1);
        // Session should be cleaned up
        assert!(router.get_session_server("sess1").is_none());
    }

    #[test]
    fn test_router_pin_unpin_session() {
        let router = SessionAffinityRouter::new(AffinityStrategy::Sticky);
        router.pin_session("sess1", "s1");
        assert_eq!(router.get_session_server("sess1"), Some("s1".to_string()));
        router.unpin_session("sess1");
        assert!(router.get_session_server("sess1").is_none());
    }

    #[test]
    fn test_router_update_load() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::LeastConnections);
        router.add_server(ServerNode::new("s1", "url1", "r1"));
        router.update_load("s1", 75);
        let server = router.servers().first().unwrap();
        assert_eq!(server.active_connections, 75);
    }

    #[test]
    fn test_router_no_healthy_servers() {
        let mut router = SessionAffinityRouter::new(AffinityStrategy::RoundRobin);
        router.add_server(ServerNode::new("s1", "url1", "r1"));
        router.mark_unhealthy("s1");
        assert!(router.route(None, None).is_none());
    }

    #[test]
    fn test_router_session_count() {
        let router = SessionAffinityRouter::new(AffinityStrategy::Sticky);
        router.pin_session("s1", "srv1");
        router.pin_session("s2", "srv1");
        assert_eq!(router.session_count(), 2);
    }

    #[test]
    fn test_affinity_strategy() {
        let router = SessionAffinityRouter::new(AffinityStrategy::LeastConnections);
        assert_eq!(router.strategy(), AffinityStrategy::LeastConnections);
    }
}
