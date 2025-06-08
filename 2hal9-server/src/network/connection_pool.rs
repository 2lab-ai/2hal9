//! Connection pooling for efficient network resource management

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{Semaphore, RwLock};
use tracing::debug;
use dashmap::DashMap;

use twohal9_core::{Result, Error};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum connections per server
    pub max_connections_per_server: usize,
    /// Maximum idle time before closing connection
    pub idle_timeout: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Maximum total connections
    pub max_total_connections: usize,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_server: 10,
            idle_timeout: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(10),
            max_total_connections: 100,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Pooled connection wrapper
struct PooledConnection {
    stream: TcpStream,
    created_at: Instant,
    last_used: RwLock<Instant>,
    server_id: String,
}

impl PooledConnection {
    /// Check if connection is healthy
    async fn is_healthy(&self) -> bool {
        // For now, just check if stream exists
        // In production, implement proper health check
        true
    }
    
    /// Update last used time
    async fn touch(&self) {
        *self.last_used.write().await = Instant::now();
    }
    
    /// Check if connection is idle
    async fn is_idle(&self, timeout: Duration) -> bool {
        let last_used = *self.last_used.read().await;
        Instant::now().duration_since(last_used) > timeout
    }
}

/// Connection pool for managing TCP connections
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Arc<DashMap<String, Vec<Arc<PooledConnection>>>>,
    total_connections: Arc<Semaphore>,
    shutdown: Arc<RwLock<bool>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: PoolConfig) -> Self {
        let total_connections = Arc::new(Semaphore::new(config.max_total_connections));
        
        Self {
            config,
            connections: Arc::new(DashMap::new()),
            total_connections,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start background maintenance tasks
    pub fn start_maintenance(&self) {
        let connections = self.connections.clone();
        let config = self.config.clone();
        let shutdown = self.shutdown.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                if *shutdown.read().await {
                    break;
                }
                
                Self::perform_maintenance(&connections, &config).await;
            }
        });
    }
    
    /// Perform maintenance on connection pool
    async fn perform_maintenance(
        connections: &Arc<DashMap<String, Vec<Arc<PooledConnection>>>>,
        config: &PoolConfig,
    ) {
        let mut total_removed = 0;
        
        for mut entry in connections.iter_mut() {
            let server_id = entry.key().clone();
            let conns = entry.value_mut();
            
            // Remove unhealthy or idle connections
            let mut healthy_conns = Vec::new();
            
            for conn in conns.drain(..) {
                if conn.is_idle(config.idle_timeout).await {
                    debug!("Removing idle connection to {}", server_id);
                    total_removed += 1;
                } else if !conn.is_healthy().await {
                    debug!("Removing unhealthy connection to {}", server_id);
                    total_removed += 1;
                } else {
                    healthy_conns.push(conn);
                }
            }
            
            *conns = healthy_conns;
        }
        
        if total_removed > 0 {
            debug!("Connection pool maintenance removed {} connections", total_removed);
        }
    }
    
    /// Get a connection to a server
    pub async fn get_connection(
        &self,
        server_id: &str,
        address: SocketAddr,
    ) -> Result<Arc<PooledConnection>> {
        // Check if shutting down
        if *self.shutdown.read().await {
            return Err(Error::InvalidState("Connection pool is shutting down".to_string()));
        }
        
        // Try to get existing connection
        if let Some(mut entry) = self.connections.get_mut(server_id) {
            let conns = entry.value_mut();
            
            // Find a healthy connection
            while let Some(conn) = conns.pop() {
                if conn.is_healthy().await {
                    conn.touch().await;
                    return Ok(conn);
                }
            }
        }
        
        // No existing connection, create new one
        self.create_connection(server_id, address).await
    }
    
    /// Create a new connection
    async fn create_connection(
        &self,
        server_id: &str,
        address: SocketAddr,
    ) -> Result<Arc<PooledConnection>> {
        // Acquire permit for total connections
        let _permit = self.total_connections.acquire().await
            .map_err(|_| Error::Network("Connection pool exhausted".to_string()))?;
            
        // Check per-server limit
        let current_count = self.connections.get(server_id)
            .map(|entry| entry.value().len())
            .unwrap_or(0);
            
        if current_count >= self.config.max_connections_per_server {
            return Err(Error::Network(format!(
                "Maximum connections to {} reached", server_id
            )));
        }
        
        // Create connection with timeout
        let stream = tokio::time::timeout(
            self.config.connection_timeout,
            TcpStream::connect(address)
        ).await
            .map_err(|_| Error::Network(format!("Connection timeout to {}", address)))?
            .map_err(|e| Error::Network(format!("Failed to connect to {}: {}", address, e)))?;
            
        // Configure socket
        stream.set_nodelay(true)?;
        
        let conn = Arc::new(PooledConnection {
            stream,
            created_at: Instant::now(),
            last_used: RwLock::new(Instant::now()),
            server_id: server_id.to_string(),
        });
        
        debug!("Created new connection to {}", server_id);
        
        // Don't store in pool - let caller use it
        // Connection will be returned to pool later if needed
        
        Ok(conn)
    }
    
    /// Return a connection to the pool
    pub async fn return_connection(&self, conn: Arc<PooledConnection>) {
        if *self.shutdown.read().await {
            return;
        }
        
        let server_id = &conn.server_id;
        
        // Check if connection is still healthy
        if !conn.is_healthy().await {
            debug!("Not returning unhealthy connection to pool");
            return;
        }
        
        // Update last used time
        conn.touch().await;
        
        // Add to pool
        self.connections.entry(server_id.clone())
            .or_insert_with(Vec::new)
            .push(conn);
    }
    
    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let mut total_connections = 0;
        let mut connections_by_server = std::collections::HashMap::new();
        
        for entry in self.connections.iter() {
            let count = entry.value().len();
            total_connections += count;
            connections_by_server.insert(entry.key().clone(), count);
        }
        
        PoolStats {
            total_connections,
            connections_by_server,
            max_total: self.config.max_total_connections,
            max_per_server: self.config.max_connections_per_server,
        }
    }
    
    /// Shutdown the connection pool
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
        
        // Clear all connections
        self.connections.clear();
        
        debug!("Connection pool shut down");
    }
}

/// Pool statistics
#[derive(Debug)]
pub struct PoolStats {
    pub total_connections: usize,
    pub connections_by_server: std::collections::HashMap<String, usize>,
    pub max_total: usize,
    pub max_per_server: usize,
}

/// Connection manager that integrates with the pool
pub struct ConnectionManager {
    pool: Arc<ConnectionPool>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
    
    /// Get a connection for use
    pub async fn get(
        &self,
        server_id: &str,
        address: SocketAddr,
    ) -> Result<ManagedConnection> {
        let conn = self.pool.get_connection(server_id, address).await?;
        
        Ok(ManagedConnection {
            conn,
            pool: self.pool.clone(),
            returned: false,
        })
    }
}

/// Managed connection that returns to pool when dropped
pub struct ManagedConnection {
    conn: Arc<PooledConnection>,
    pool: Arc<ConnectionPool>,
    returned: bool,
}

impl ManagedConnection {
    /// Get the underlying TCP stream
    pub fn stream(&self) -> &TcpStream {
        &self.conn.stream
    }
    
    /// Mark connection as failed (won't return to pool)
    pub fn mark_failed(&mut self) {
        self.returned = true; // Prevent return to pool
    }
}

impl Drop for ManagedConnection {
    fn drop(&mut self) {
        if !self.returned {
            let conn = self.conn.clone();
            let pool = self.pool.clone();
            
            // Return to pool asynchronously
            tokio::spawn(async move {
                pool.return_connection(conn).await;
            });
        }
    }
}