//! Load balancing for distributed HAL9 deployment

use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::net::SocketAddr;

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    IPHash,
    Random,
    ResponseTime,
}

/// Server instance information
#[derive(Debug, Clone)]
pub struct ServerInstance {
    pub id: String,
    pub address: SocketAddr,
    pub weight: u32,
    pub region: String,
    pub health_check_url: String,
    pub max_connections: u32,
}

/// Server metrics for load balancing decisions
#[derive(Debug)]
struct ServerMetrics {
    active_connections: AtomicUsize,
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_response_time_ms: AtomicU64,
    last_health_check: RwLock<Instant>,
    is_healthy: RwLock<bool>,
    circuit_breaker_state: RwLock<CircuitBreakerState>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open(Instant),
    HalfOpen,
}

/// Load balancer implementation
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    servers: Arc<RwLock<Vec<ServerInstance>>>,
    metrics: Arc<RwLock<HashMap<String, ServerMetrics>>>,
    round_robin_counter: AtomicUsize,
    weighted_indices: Arc<RwLock<Vec<usize>>>,
    health_check_interval: Duration,
}

impl LoadBalancer {
    /// Create new load balancer
    pub fn new(
        strategy: LoadBalancingStrategy,
        servers: Vec<ServerInstance>,
        health_check_interval: Duration,
    ) -> Self {
        let mut metrics = HashMap::new();
        let mut weighted_indices = Vec::new();
        
        // Initialize metrics and weighted indices
        for (idx, server) in servers.iter().enumerate() {
            metrics.insert(
                server.id.clone(),
                ServerMetrics {
                    active_connections: AtomicUsize::new(0),
                    total_requests: AtomicU64::new(0),
                    failed_requests: AtomicU64::new(0),
                    total_response_time_ms: AtomicU64::new(0),
                    last_health_check: RwLock::new(Instant::now()),
                    is_healthy: RwLock::new(true),
                    circuit_breaker_state: RwLock::new(CircuitBreakerState::Closed),
                }
            );
            
            // Add server index multiple times based on weight
            for _ in 0..server.weight {
                weighted_indices.push(idx);
            }
        }
        
        Self {
            strategy,
            servers: Arc::new(RwLock::new(servers)),
            metrics: Arc::new(RwLock::new(metrics)),
            round_robin_counter: AtomicUsize::new(0),
            weighted_indices: Arc::new(RwLock::new(weighted_indices)),
            health_check_interval,
        }
    }
    
    /// Select next server based on strategy
    pub async fn select_server(&self, client_ip: Option<&str>) -> Result<ServerInstance> {
        let servers = self.servers.read().await;
        let metrics = self.metrics.read().await;
        
        // Filter healthy servers
        let healthy_servers: Vec<(usize, &ServerInstance)> = servers
            .iter()
            .enumerate()
            .filter(|(_, server)| {
                if let Some(m) = metrics.get(&server.id) {
                    if let Ok(healthy) = m.is_healthy.try_read() {
                        return *healthy;
                    }
                }
                false
            })
            .collect();
        
        if healthy_servers.is_empty() {
            return Err(anyhow::anyhow!("No healthy servers available"));
        }
        
        let selected_idx = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(&healthy_servers)
            }
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_servers, &metrics).await
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_servers).await
            }
            LoadBalancingStrategy::IPHash => {
                self.select_ip_hash(&healthy_servers, client_ip)
            }
            LoadBalancingStrategy::Random => {
                self.select_random(&healthy_servers)
            }
            LoadBalancingStrategy::ResponseTime => {
                self.select_by_response_time(&healthy_servers, &metrics).await
            }
        };
        
        Ok(servers[selected_idx].clone())
    }
    
    /// Round-robin selection
    fn select_round_robin(&self, healthy_servers: &[(usize, &ServerInstance)]) -> usize {
        let counter = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let idx = counter % healthy_servers.len();
        healthy_servers[idx].0
    }
    
    /// Least connections selection
    async fn select_least_connections(
        &self,
        healthy_servers: &[(usize, &ServerInstance)],
        metrics: &HashMap<String, ServerMetrics>,
    ) -> usize {
        let mut min_connections = usize::MAX;
        let mut selected_idx = 0;
        
        for (idx, server) in healthy_servers {
            if let Some(m) = metrics.get(&server.id) {
                let connections = m.active_connections.load(Ordering::Relaxed);
                if connections < min_connections {
                    min_connections = connections;
                    selected_idx = *idx;
                }
            }
        }
        
        selected_idx
    }
    
    /// Weighted round-robin selection
    async fn select_weighted_round_robin(&self, healthy_servers: &[(usize, &ServerInstance)]) -> usize {
        let weighted_indices = self.weighted_indices.read().await;
        let counter = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let weighted_idx = counter % weighted_indices.len();
        
        // Find corresponding healthy server
        let server_idx = weighted_indices[weighted_idx];
        if healthy_servers.iter().any(|(idx, _)| *idx == server_idx) {
            server_idx
        } else {
            // Fallback to first healthy server
            healthy_servers[0].0
        }
    }
    
    /// IP hash selection
    fn select_ip_hash(&self, healthy_servers: &[(usize, &ServerInstance)], client_ip: Option<&str>) -> usize {
        if let Some(ip) = client_ip {
            let hash = self.hash_string(ip);
            let idx = (hash as usize) % healthy_servers.len();
            healthy_servers[idx].0
        } else {
            // Fallback to random
            self.select_random(healthy_servers)
        }
    }
    
    /// Random selection
    fn select_random(&self, healthy_servers: &[(usize, &ServerInstance)]) -> usize {
        let idx = rand::random::<usize>() % healthy_servers.len();
        healthy_servers[idx].0
    }
    
    /// Response time based selection
    async fn select_by_response_time(
        &self,
        healthy_servers: &[(usize, &ServerInstance)],
        metrics: &HashMap<String, ServerMetrics>,
    ) -> usize {
        let mut best_avg_time = f64::MAX;
        let mut selected_idx = 0;
        
        for (idx, server) in healthy_servers {
            if let Some(m) = metrics.get(&server.id) {
                let total_requests = m.total_requests.load(Ordering::Relaxed);
                if total_requests > 0 {
                    let total_time = m.total_response_time_ms.load(Ordering::Relaxed);
                    let avg_time = total_time as f64 / total_requests as f64;
                    
                    if avg_time < best_avg_time {
                        best_avg_time = avg_time;
                        selected_idx = *idx;
                    }
                }
            }
        }
        
        selected_idx
    }
    
    /// Hash string for consistent hashing
    fn hash_string(&self, s: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Mark request start
    pub async fn mark_request_start(&self, server_id: &str) -> Result<()> {
        let metrics = self.metrics.read().await;
        if let Some(m) = metrics.get(server_id) {
            m.active_connections.fetch_add(1, Ordering::Relaxed);
            m.total_requests.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
    
    /// Mark request end
    pub async fn mark_request_end(
        &self,
        server_id: &str,
        response_time_ms: u64,
        success: bool,
    ) -> Result<()> {
        let metrics = self.metrics.read().await;
        if let Some(m) = metrics.get(server_id) {
            m.active_connections.fetch_sub(1, Ordering::Relaxed);
            m.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);
            
            if !success {
                m.failed_requests.fetch_add(1, Ordering::Relaxed);
                
                // Check circuit breaker
                self.check_circuit_breaker(server_id, m).await?;
            }
        }
        Ok(())
    }
    
    /// Check and update circuit breaker state
    async fn check_circuit_breaker(
        &self,
        server_id: &str,
        metrics: &ServerMetrics,
    ) -> Result<()> {
        let total_requests = metrics.total_requests.load(Ordering::Relaxed);
        let failed_requests = metrics.failed_requests.load(Ordering::Relaxed);
        
        if total_requests > 100 {
            let failure_rate = failed_requests as f64 / total_requests as f64;
            
            let mut state = metrics.circuit_breaker_state.write().await;
            
            match *state {
                CircuitBreakerState::Closed => {
                    if failure_rate > 0.5 {
                        *state = CircuitBreakerState::Open(Instant::now());
                        let mut healthy = metrics.is_healthy.write().await;
                        *healthy = false;
                        tracing::warn!("Circuit breaker opened for server {}", server_id);
                    }
                }
                CircuitBreakerState::Open(opened_at) => {
                    if opened_at.elapsed() > Duration::from_secs(30) {
                        *state = CircuitBreakerState::HalfOpen;
                        tracing::info!("Circuit breaker half-open for server {}", server_id);
                    }
                }
                CircuitBreakerState::HalfOpen => {
                    if failure_rate < 0.1 {
                        *state = CircuitBreakerState::Closed;
                        let mut healthy = metrics.is_healthy.write().await;
                        *healthy = true;
                        tracing::info!("Circuit breaker closed for server {}", server_id);
                    } else if failure_rate > 0.5 {
                        *state = CircuitBreakerState::Open(Instant::now());
                        tracing::warn!("Circuit breaker re-opened for server {}", server_id);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Run health checks
    pub async fn run_health_checks(&self) -> Result<()> {
        let servers = self.servers.read().await;
        let mut metrics = self.metrics.write().await;
        
        for server in servers.iter() {
            if let Some(m) = metrics.get_mut(&server.id) {
                let mut last_check = m.last_health_check.write().await;
                
                if last_check.elapsed() >= self.health_check_interval {
                    *last_check = Instant::now();
                    
                    // Perform health check (simplified)
                    let is_healthy = self.check_server_health(server).await;
                    
                    let mut healthy = m.is_healthy.write().await;
                    *healthy = is_healthy;
                    
                    if !is_healthy {
                        tracing::warn!("Server {} failed health check", server.id);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check individual server health
    async fn check_server_health(&self, _server: &ServerInstance) -> bool {
        // In production, would make HTTP request to health endpoint
        // For now, simulate with random success
        rand::random::<f32>() > 0.05 // 95% success rate
    }
    
    /// Get current server statistics
    pub async fn get_statistics(&self) -> Result<Vec<ServerStats>> {
        let servers = self.servers.read().await;
        let metrics = self.metrics.read().await;
        
        let mut stats = Vec::new();
        
        for server in servers.iter() {
            if let Some(m) = metrics.get(&server.id) {
                let total_requests = m.total_requests.load(Ordering::Relaxed);
                let failed_requests = m.failed_requests.load(Ordering::Relaxed);
                let total_response_time = m.total_response_time_ms.load(Ordering::Relaxed);
                let active_connections = m.active_connections.load(Ordering::Relaxed);
                
                let avg_response_time = if total_requests > 0 {
                    total_response_time as f64 / total_requests as f64
                } else {
                    0.0
                };
                
                let success_rate = if total_requests > 0 {
                    (total_requests - failed_requests) as f64 / total_requests as f64
                } else {
                    1.0
                };
                
                stats.push(ServerStats {
                    server_id: server.id.clone(),
                    address: server.address,
                    region: server.region.clone(),
                    active_connections,
                    total_requests,
                    failed_requests,
                    avg_response_time_ms: avg_response_time,
                    success_rate,
                    is_healthy: *m.is_healthy.read().await,
                });
            }
        }
        
        Ok(stats)
    }
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub server_id: String,
    pub address: SocketAddr,
    pub region: String,
    pub active_connections: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_round_robin() {
        let servers = vec![
            ServerInstance {
                id: "server1".to_string(),
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001),
                weight: 1,
                region: "us-west".to_string(),
                health_check_url: "http://localhost:9001/health".to_string(),
                max_connections: 100,
            },
            ServerInstance {
                id: "server2".to_string(),
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9002),
                weight: 1,
                region: "us-west".to_string(),
                health_check_url: "http://localhost:9002/health".to_string(),
                max_connections: 100,
            },
        ];
        
        let lb = LoadBalancer::new(
            LoadBalancingStrategy::RoundRobin,
            servers,
            Duration::from_secs(30),
        );
        
        // Should alternate between servers
        let server1 = lb.select_server(None).await.unwrap();
        let server2 = lb.select_server(None).await.unwrap();
        let server3 = lb.select_server(None).await.unwrap();
        
        assert_eq!(server1.id, "server1");
        assert_eq!(server2.id, "server2");
        assert_eq!(server3.id, "server1");
    }
    
    #[tokio::test]
    async fn test_ip_hash() {
        let servers = vec![
            ServerInstance {
                id: "server1".to_string(),
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001),
                weight: 1,
                region: "us-west".to_string(),
                health_check_url: "http://localhost:9001/health".to_string(),
                max_connections: 100,
            },
            ServerInstance {
                id: "server2".to_string(),
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9002),
                weight: 1,
                region: "us-west".to_string(),
                health_check_url: "http://localhost:9002/health".to_string(),
                max_connections: 100,
            },
        ];
        
        let lb = LoadBalancer::new(
            LoadBalancingStrategy::IPHash,
            servers,
            Duration::from_secs(30),
        );
        
        // Same IP should always get same server
        let client_ip = "192.168.1.100";
        let server1 = lb.select_server(Some(client_ip)).await.unwrap();
        let server2 = lb.select_server(Some(client_ip)).await.unwrap();
        
        assert_eq!(server1.id, server2.id);
    }
}