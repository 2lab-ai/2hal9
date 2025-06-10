//! Health checking for distributed HAL9 system

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
    
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Component types to monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Database,
    Cache,
    MessageQueue,
    ApiServer,
    NeuronWorker,
    LoadBalancer,
    Storage,
    External,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub component_type: ComponentType,
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub checked_at: DateTime<Utc>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub check_interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub degraded_threshold_ms: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            degraded_threshold_ms: 1000,
        }
    }
}

/// Component health state
#[derive(Debug, Clone)]
struct ComponentHealth {
    pub status: HealthStatus,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_check: Option<Instant>,
    pub last_result: Option<HealthCheckResult>,
    pub check_function: Arc<dyn Fn() -> futures::future::BoxFuture<'static, Result<HealthCheckResult>> + Send + Sync>,
}

/// Health checker for distributed system
pub struct HealthChecker {
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    config: HealthCheckConfig,
    overall_status: Arc<RwLock<SystemHealth>>,
}

/// Overall system health
#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub healthy_components: u32,
    pub degraded_components: u32,
    pub unhealthy_components: u32,
    pub last_update: DateTime<Utc>,
    pub uptime: Duration,
    pub started_at: DateTime<Utc>,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        let overall_status = SystemHealth {
            status: HealthStatus::Unknown,
            healthy_components: 0,
            degraded_components: 0,
            unhealthy_components: 0,
            last_update: Utc::now(),
            uptime: Duration::from_secs(0),
            started_at: Utc::now(),
        };
        
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            config,
            overall_status: Arc::new(RwLock::new(overall_status)),
        }
    }
    
    /// Register a component for health checking
    pub async fn register_component<F>(
        &self,
        name: String,
        component_type: ComponentType,
        check_fn: F,
    ) where
        F: Fn() -> futures::future::BoxFuture<'static, Result<HealthCheckResult>> + Send + Sync + 'static,
    {
        let mut components = self.components.write().await;
        
        components.insert(name.clone(), ComponentHealth {
            status: HealthStatus::Unknown,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_check: None,
            last_result: None,
            check_function: Arc::new(check_fn),
        });
        
        tracing::info!("Registered health check for component: {}", name);
    }
    
    /// Register standard database health check
    pub async fn register_database(&self, name: String, pool: sqlx::PgPool) {
        let check_fn = move || {
            let pool = pool.clone();
            let component_name = name.clone();
            Box::pin(async move {
                let start = Instant::now();
                
                match sqlx::query("SELECT 1").fetch_one(&pool).await {
                    Ok(_) => {
                        let latency = start.elapsed().as_millis() as u64;
                        Ok(HealthCheckResult {
                            component: component_name,
                            component_type: ComponentType::Database,
                            status: HealthStatus::Healthy,
                            latency_ms: latency,
                            message: None,
                            metadata: HashMap::new(),
                            checked_at: Utc::now(),
                        })
                    }
                    Err(e) => {
                        Ok(HealthCheckResult {
                            component: component_name,
                            component_type: ComponentType::Database,
                            status: HealthStatus::Unhealthy,
                            latency_ms: start.elapsed().as_millis() as u64,
                            message: Some(format!("Database error: {}", e)),
                            metadata: HashMap::new(),
                            checked_at: Utc::now(),
                        })
                    }
                }
            })
        };
        
        self.register_component(name, ComponentType::Database, check_fn).await;
    }
    
    /// Register Redis health check
    pub async fn register_redis(&self, name: String, pool: crate::cache::RedisPool) {
        let check_fn = move || {
            let pool = pool.clone();
            let component_name = name.clone();
            Box::pin(async move {
                let start = Instant::now();
                
                match pool.get().await {
                    Ok(mut conn) => {
                        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                            Ok(pong) if pong == "PONG" => {
                                let latency = start.elapsed().as_millis() as u64;
                                Ok(HealthCheckResult {
                                    component: component_name,
                                    component_type: ComponentType::Cache,
                                    status: HealthStatus::Healthy,
                                    latency_ms: latency,
                                    message: None,
                                    metadata: HashMap::new(),
                                    checked_at: Utc::now(),
                                })
                            }
                            _ => {
                                Ok(HealthCheckResult {
                                    component: component_name,
                                    component_type: ComponentType::Cache,
                                    status: HealthStatus::Unhealthy,
                                    latency_ms: start.elapsed().as_millis() as u64,
                                    message: Some("Redis ping failed".to_string()),
                                    metadata: HashMap::new(),
                                    checked_at: Utc::now(),
                                })
                            }
                        }
                    }
                    Err(e) => {
                        Ok(HealthCheckResult {
                            component: component_name,
                            component_type: ComponentType::Cache,
                            status: HealthStatus::Unhealthy,
                            latency_ms: start.elapsed().as_millis() as u64,
                            message: Some(format!("Redis connection error: {}", e)),
                            metadata: HashMap::new(),
                            checked_at: Utc::now(),
                        })
                    }
                }
            })
        };
        
        self.register_component(name, ComponentType::Cache, check_fn).await;
    }
    
    /// Run health checks for all components
    pub async fn run_checks(&self) -> Result<Vec<HealthCheckResult>> {
        let mut components = self.components.write().await;
        let mut results = Vec::new();
        
        for (name, health) in components.iter_mut() {
            // Skip if recently checked
            if let Some(last_check) = health.last_check {
                if last_check.elapsed() < self.config.check_interval {
                    if let Some(last_result) = &health.last_result {
                        results.push(last_result.clone());
                        continue;
                    }
                }
            }
            
            // Run health check with timeout
            let check_fn = health.check_function.clone();
            let timeout_duration = self.config.timeout;
            
            let result = match tokio::time::timeout(timeout_duration, check_fn()).await {
                Ok(Ok(mut check_result)) => {
                    // Apply degraded threshold
                    if check_result.status == HealthStatus::Healthy 
                        && check_result.latency_ms > self.config.degraded_threshold_ms {
                        check_result.status = HealthStatus::Degraded;
                        check_result.message = Some(format!(
                            "High latency: {}ms (threshold: {}ms)",
                            check_result.latency_ms,
                            self.config.degraded_threshold_ms
                        ));
                    }
                    check_result
                }
                Ok(Err(e)) => HealthCheckResult {
                    component: name.clone(),
                    component_type: ComponentType::External,
                    status: HealthStatus::Unhealthy,
                    latency_ms: timeout_duration.as_millis() as u64,
                    message: Some(format!("Check failed: {}", e)),
                    metadata: HashMap::new(),
                    checked_at: Utc::now(),
                },
                Err(_) => HealthCheckResult {
                    component: name.clone(),
                    component_type: ComponentType::External,
                    status: HealthStatus::Unhealthy,
                    latency_ms: timeout_duration.as_millis() as u64,
                    message: Some("Health check timeout".to_string()),
                    metadata: HashMap::new(),
                    checked_at: Utc::now(),
                },
            };
            
            // Update component health state
            self.update_component_health(health, &result);
            
            health.last_check = Some(Instant::now());
            health.last_result = Some(result.clone());
            results.push(result);
        }
        
        // Update overall system health
        self.update_system_health(&results).await;
        
        Ok(results)
    }
    
    /// Update component health based on result
    fn update_component_health(&self, health: &mut ComponentHealth, result: &HealthCheckResult) {
        match result.status {
            HealthStatus::Healthy | HealthStatus::Degraded => {
                health.consecutive_failures = 0;
                health.consecutive_successes += 1;
                
                if health.consecutive_successes >= self.config.success_threshold {
                    health.status = result.status;
                }
            }
            HealthStatus::Unhealthy => {
                health.consecutive_successes = 0;
                health.consecutive_failures += 1;
                
                if health.consecutive_failures >= self.config.failure_threshold {
                    health.status = HealthStatus::Unhealthy;
                }
            }
            HealthStatus::Unknown => {
                health.status = HealthStatus::Unknown;
            }
        }
    }
    
    /// Update overall system health
    async fn update_system_health(&self, results: &[HealthCheckResult]) {
        let mut overall = self.overall_status.write().await;
        
        overall.healthy_components = results.iter()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count() as u32;
        
        overall.degraded_components = results.iter()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count() as u32;
        
        overall.unhealthy_components = results.iter()
            .filter(|r| r.status == HealthStatus::Unhealthy)
            .count() as u32;
        
        overall.last_update = Utc::now();
        overall.uptime = Utc::now().signed_duration_since(overall.started_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
        
        // Determine overall status
        if overall.unhealthy_components > 0 {
            overall.status = HealthStatus::Unhealthy;
        } else if overall.degraded_components > 0 {
            overall.status = HealthStatus::Degraded;
        } else if overall.healthy_components > 0 {
            overall.status = HealthStatus::Healthy;
        } else {
            overall.status = HealthStatus::Unknown;
        }
    }
    
    /// Get current system health
    pub async fn get_system_health(&self) -> SystemHealth {
        self.overall_status.read().await.clone()
    }
    
    /// Get health status for specific component
    pub async fn get_component_health(&self, component: &str) -> Option<HealthCheckResult> {
        let components = self.components.read().await;
        components.get(component)
            .and_then(|h| h.last_result.clone())
    }
    
    /// Get all component health results
    pub async fn get_all_health(&self) -> HashMap<String, HealthCheckResult> {
        let components = self.components.read().await;
        components.iter()
            .filter_map(|(name, health)| {
                health.last_result.as_ref()
                    .map(|result| (name.clone(), result.clone()))
            })
            .collect()
    }
    
    /// Start background health check task
    pub fn start_background_checks(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.run_checks().await {
                    tracing::error!("Health check error: {}", e);
                }
            }
        });
    }
}

/// Health check endpoint response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub system: SystemHealth,
    pub components: HashMap<String, ComponentHealthInfo>,
}

/// Component health info for API response
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealthInfo {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

impl From<HealthCheckResult> for ComponentHealthInfo {
    fn from(result: HealthCheckResult) -> Self {
        Self {
            status: result.status,
            latency_ms: result.latency_ms,
            message: result.message,
            last_check: result.checked_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_status_transitions() {
        let checker = HealthChecker::new(HealthCheckConfig {
            failure_threshold: 2,
            success_threshold: 2,
            ..Default::default()
        });
        
        // Register a test component
        let health_states = Arc::new(RwLock::new(vec![
            HealthStatus::Healthy,
            HealthStatus::Unhealthy,
            HealthStatus::Unhealthy,
            HealthStatus::Healthy,
            HealthStatus::Healthy,
        ]));
        
        let states_clone = health_states.clone();
        checker.register_component(
            "test".to_string(),
            ComponentType::External,
            move || {
                let states = states_clone.clone();
                Box::pin(async move {
                    let mut states = states.write().await;
                    let status = states.remove(0);
                    
                    Ok(HealthCheckResult {
                        component: "test".to_string(),
                        component_type: ComponentType::External,
                        status,
                        latency_ms: 10,
                        message: None,
                        metadata: HashMap::new(),
                        checked_at: Utc::now(),
                    })
                })
            }
        ).await;
        
        // Run checks and verify state transitions
        for _ in 0..5 {
            checker.run_checks().await.unwrap();
        }
        
        let health = checker.get_component_health("test").await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}