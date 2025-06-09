use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use super::api::Permission;

// ============ Security Policy ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    // Resource access
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_system_time: bool,
    pub allow_random: bool,
    pub allow_environment: bool,
    
    // Resource limits
    pub max_memory_bytes: u64,
    pub max_cpu_percent: f32,
    
    // Network restrictions
    pub allowed_hosts: Vec<String>,
    pub blocked_hosts: Vec<String>,
    pub allowed_ports: Option<Vec<u16>>,
    pub max_concurrent_connections: u32,
    pub network_timeout: Duration,
    
    // Filesystem restrictions
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub max_file_handles: u32,
    pub max_file_size: u64,
    
    // Execution limits
    pub max_execution_time: Duration,
    pub max_stack_depth: u32,
    pub max_heap_allocations: u32,
    
    // API restrictions
    pub allowed_hal9_apis: HashSet<String>,
    pub rate_limits: RateLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub api_calls_per_second: u32,
    pub api_calls_per_minute: u32,
    pub memory_operations_per_second: u32,
    pub network_requests_per_minute: u32,
}

// ============ Resource Limits ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_percent: f32,
    pub max_execution_time_ms: u64,
    pub max_file_size_bytes: u64,
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024,  // 64MB
            max_cpu_percent: 25.0,
            max_execution_time_ms: 5000,          // 5 seconds
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
            max_network_connections: 10,
        }
    }
}

// ============ Security Sandbox ============

pub struct SecuritySandbox {
    policy: SecurityPolicy,
    resource_tracker: ResourceTracker,
}

#[derive(Default)]
struct ResourceTracker {
    memory_used: u64,
    cpu_time_used: Duration,
    file_handles_open: u32,
    network_connections: u32,
    api_calls: APICallTracker,
}

#[derive(Default)]
struct APICallTracker {
    calls_this_second: u32,
    calls_this_minute: u32,
    last_second_reset: std::time::Instant,
    last_minute_reset: std::time::Instant,
}

impl SecuritySandbox {
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy,
            resource_tracker: ResourceTracker::default(),
        }
    }
    
    /// Check if a permission is allowed
    pub fn check_permission(&self, permission: &Permission) -> Result<(), SecurityError> {
        match permission {
            Permission::NetworkHttp | Permission::NetworkHttps => {
                if !self.policy.allow_network {
                    return Err(SecurityError::NetworkAccessDenied);
                }
            }
            Permission::FileRead(path) | Permission::FileWrite(path) | Permission::FileCreate(path) => {
                if !self.policy.allow_filesystem {
                    return Err(SecurityError::FilesystemAccessDenied);
                }
                self.check_path_access(path)?;
            }
            Permission::SystemTime => {
                if !self.policy.allow_system_time {
                    return Err(SecurityError::SystemTimeAccessDenied);
                }
            }
            Permission::SystemRandom => {
                if !self.policy.allow_random {
                    return Err(SecurityError::RandomAccessDenied);
                }
            }
            Permission::SystemEnv(_) => {
                if !self.policy.allow_environment {
                    return Err(SecurityError::EnvironmentAccessDenied);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Check if a network host is allowed
    pub fn check_network_access(&self, host: &str, port: u16) -> Result<(), SecurityError> {
        if !self.policy.allow_network {
            return Err(SecurityError::NetworkAccessDenied);
        }
        
        // Check blocked hosts
        for blocked in &self.policy.blocked_hosts {
            if host_matches(host, blocked) {
                return Err(SecurityError::HostBlocked(host.to_string()));
            }
        }
        
        // Check allowed hosts
        if !self.policy.allowed_hosts.is_empty() {
            let mut allowed = false;
            for allowed_host in &self.policy.allowed_hosts {
                if host_matches(host, allowed_host) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Err(SecurityError::HostNotAllowed(host.to_string()));
            }
        }
        
        // Check allowed ports
        if let Some(ref allowed_ports) = self.policy.allowed_ports {
            if !allowed_ports.contains(&port) {
                return Err(SecurityError::PortNotAllowed(port));
            }
        }
        
        // Check connection limit
        if self.resource_tracker.network_connections >= self.policy.max_concurrent_connections {
            return Err(SecurityError::ConnectionLimitExceeded);
        }
        
        Ok(())
    }
    
    /// Check if a file path is allowed
    pub fn check_path_access(&self, path: &str) -> Result<(), SecurityError> {
        let path = PathBuf::from(path);
        
        // Check blocked paths
        for blocked in &self.policy.blocked_paths {
            if path.starts_with(blocked) {
                return Err(SecurityError::PathBlocked(path.display().to_string()));
            }
        }
        
        // Check allowed paths
        if !self.policy.allowed_paths.is_empty() {
            let mut allowed = false;
            for allowed_path in &self.policy.allowed_paths {
                if path.starts_with(allowed_path) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Err(SecurityError::PathNotAllowed(path.display().to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Check memory allocation
    pub fn check_memory_allocation(&mut self, bytes: u64) -> Result<(), SecurityError> {
        if self.resource_tracker.memory_used + bytes > self.policy.max_memory_bytes {
            return Err(SecurityError::MemoryLimitExceeded);
        }
        
        self.resource_tracker.memory_used += bytes;
        Ok(())
    }
    
    /// Release memory
    pub fn release_memory(&mut self, bytes: u64) {
        self.resource_tracker.memory_used = 
            self.resource_tracker.memory_used.saturating_sub(bytes);
    }
    
    /// Check API rate limits
    pub fn check_api_rate_limit(&mut self) -> Result<(), SecurityError> {
        let now = std::time::Instant::now();
        let mut tracker = &mut self.resource_tracker.api_calls;
        
        // Reset counters if needed
        if now.duration_since(tracker.last_second_reset) >= Duration::from_secs(1) {
            tracker.calls_this_second = 0;
            tracker.last_second_reset = now;
        }
        
        if now.duration_since(tracker.last_minute_reset) >= Duration::from_secs(60) {
            tracker.calls_this_minute = 0;
            tracker.last_minute_reset = now;
        }
        
        // Check limits
        if tracker.calls_this_second >= self.policy.rate_limits.api_calls_per_second {
            return Err(SecurityError::RateLimitExceeded("per second".to_string()));
        }
        
        if tracker.calls_this_minute >= self.policy.rate_limits.api_calls_per_minute {
            return Err(SecurityError::RateLimitExceeded("per minute".to_string()));
        }
        
        // Increment counters
        tracker.calls_this_second += 1;
        tracker.calls_this_minute += 1;
        
        Ok(())
    }
}

// ============ Security Errors ============

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Network access denied")]
    NetworkAccessDenied,
    
    #[error("Filesystem access denied")]
    FilesystemAccessDenied,
    
    #[error("System time access denied")]
    SystemTimeAccessDenied,
    
    #[error("Random number access denied")]
    RandomAccessDenied,
    
    #[error("Environment variable access denied")]
    EnvironmentAccessDenied,
    
    #[error("Host blocked: {0}")]
    HostBlocked(String),
    
    #[error("Host not allowed: {0}")]
    HostNotAllowed(String),
    
    #[error("Port not allowed: {0}")]
    PortNotAllowed(u16),
    
    #[error("Path blocked: {0}")]
    PathBlocked(String),
    
    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),
    
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,
    
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

// ============ Helper Functions ============

fn host_matches(host: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    if pattern.starts_with("*.") {
        let suffix = &pattern[2..];
        return host.ends_with(suffix) || host == &suffix[1..];
    }
    
    host == pattern
}

// ============ Default Policies ============

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_network: false,
            allow_filesystem: false,
            allow_system_time: true,
            allow_random: true,
            allow_environment: false,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_percent: 25.0,
            allowed_hosts: vec![],
            blocked_hosts: vec![],
            allowed_ports: None,
            max_concurrent_connections: 10,
            network_timeout: Duration::from_secs(30),
            allowed_paths: vec![],
            blocked_paths: vec![],
            max_file_handles: 10,
            max_file_size: 10 * 1024 * 1024,
            max_execution_time: Duration::from_secs(5),
            max_stack_depth: 1000,
            max_heap_allocations: 10000,
            allowed_hal9_apis: HashSet::new(),
            rate_limits: RateLimits {
                api_calls_per_second: 10,
                api_calls_per_minute: 100,
                memory_operations_per_second: 100,
                network_requests_per_minute: 60,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_host_matching() {
        assert!(host_matches("example.com", "*"));
        assert!(host_matches("api.example.com", "*.example.com"));
        assert!(host_matches("example.com", "*.example.com"));
        assert!(!host_matches("badexample.com", "*.example.com"));
        assert!(host_matches("example.com", "example.com"));
    }
}