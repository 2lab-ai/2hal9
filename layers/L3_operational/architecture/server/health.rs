//! Enhanced health check system for HAL9 server

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use tracing::{error, warn, info};

use crate::{
    server::HAL9Server,
    error::ServerError,
    database::DatabasePool,
};

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            HealthStatus::Healthy => StatusCode::OK,
            HealthStatus::Degraded => StatusCode::OK, // Still return 200 for degraded
            HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Overall health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks_passed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks_total: Option<usize>,
}

/// Health check query parameters
#[derive(Debug, Deserialize)]
pub struct HealthCheckQuery {
    /// Include detailed component checks
    #[serde(default)]
    pub detailed: bool,
    /// Timeout for health checks in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000 // 5 seconds default
}

/// Simple health check endpoint (fast)
pub async fn health_check_simple() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "hal9-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Liveness probe endpoint (Kubernetes)
pub async fn liveness_probe() -> impl IntoResponse {
    // Basic liveness check - is the service running?
    (StatusCode::OK, "OK")
}

/// Readiness probe endpoint (Kubernetes)
pub async fn readiness_probe(
    State(server): State<Arc<HAL9Server>>,
) -> impl IntoResponse {
    // Check if server is ready to accept traffic
    let start = Instant::now();
    
    // Quick database connectivity check
    if let Some(db) = &server.db {
        match timeout(Duration::from_secs(2), check_database_ready(db)).await {
            Ok(Ok(_)) => {},
            Ok(Err(_)) | Err(_) => {
                return (StatusCode::SERVICE_UNAVAILABLE, "Database not ready");
            }
        }
    }
    
    // Check if neurons are initialized
    let neurons = server.list_neurons().await.unwrap_or_default();
    if neurons.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "No neurons initialized");
    }
    
    let latency = start.elapsed().as_millis();
    if latency > 1000 {
        warn!("Readiness probe slow: {}ms", latency);
    }
    
    (StatusCode::OK, "Ready")
}

/// Comprehensive health check endpoint
pub async fn health_check_detailed(
    State(server): State<Arc<HAL9Server>>,
    Query(params): Query<HealthCheckQuery>,
) -> Result<Response, ServerError> {
    let start = Instant::now();
    let check_timeout = Duration::from_millis(params.timeout_ms);
    
    // Get server status
    let status = server.get_status().await?;
    let mut components = Vec::new();
    let mut overall_status = HealthStatus::Healthy;
    
    if params.detailed {
        // Database health check
        if let Some(db) = &server.db {
            let db_health = timeout(
                check_timeout,
                check_database_health(db.clone())
            ).await;
            
            match db_health {
                Ok(Ok(health)) => {
                    if health.status != HealthStatus::Healthy {
                        overall_status = HealthStatus::Degraded;
                    }
                    components.push(health);
                },
                Ok(Err(e)) => {
                    overall_status = HealthStatus::Unhealthy;
                    components.push(ComponentHealth {
                        name: "database".to_string(),
                        status: HealthStatus::Unhealthy,
                        message: Some(format!("Database check failed: {}", e)),
                        latency_ms: check_timeout.as_millis() as u64,
                        metadata: HashMap::new(),
                    });
                },
                Err(_) => {
                    overall_status = HealthStatus::Unhealthy;
                    components.push(ComponentHealth {
                        name: "database".to_string(),
                        status: HealthStatus::Unhealthy,
                        message: Some("Database check timed out".to_string()),
                        latency_ms: check_timeout.as_millis() as u64,
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        // Redis health check
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let redis_health = timeout(
                check_timeout,
                check_redis_health(&redis_url)
            ).await;
            
            match redis_health {
                Ok(Ok(health)) => {
                    if health.status != HealthStatus::Healthy {
                        overall_status = HealthStatus::Degraded;
                    }
                    components.push(health);
                },
                Ok(Err(_)) | Err(_) => {
                    // Redis is optional, so just mark as degraded
                    if overall_status == HealthStatus::Healthy {
                        overall_status = HealthStatus::Degraded;
                    }
                    components.push(ComponentHealth {
                        name: "redis".to_string(),
                        status: HealthStatus::Degraded,
                        message: Some("Redis unavailable".to_string()),
                        latency_ms: 0,
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        // Neuron health checks
        let neuron_health = check_neurons_health(&server, &status).await;
        if neuron_health.status != HealthStatus::Healthy {
            overall_status = neuron_health.status;
        }
        components.push(neuron_health);
        
        // Memory health check
        let memory_health = check_memory_health();
        if memory_health.status != HealthStatus::Healthy {
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Degraded;
            }
        }
        components.push(memory_health);
        
        // Disk space health check
        let disk_health = check_disk_health();
        if disk_health.status != HealthStatus::Healthy {
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Degraded;
            }
        }
        components.push(disk_health);
    }
    
    let checks_passed = components.iter().filter(|c| c.status == HealthStatus::Healthy).count();
    let checks_total = components.len();
    
    let response = HealthCheckResponse {
        status: overall_status,
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: status.uptime.as_secs(),
        components,
        checks_passed: if params.detailed { Some(checks_passed) } else { None },
        checks_total: if params.detailed { Some(checks_total) } else { None },
    };
    
    let elapsed = start.elapsed();
    if elapsed > Duration::from_secs(1) {
        warn!("Health check took {}ms", elapsed.as_millis());
    }
    
    Ok((overall_status.to_status_code(), Json(response)).into_response())
}

/// Check database health
async fn check_database_health(db: Arc<DatabasePool>) -> Result<ComponentHealth, ServerError> {
    let start = Instant::now();
    let mut metadata = HashMap::new();
    
    // Connection pool stats
    let pool_size = db.size();
    let num_idle = db.num_idle();
    metadata.insert("pool_size".to_string(), serde_json::json!(pool_size));
    metadata.insert("connections_idle".to_string(), serde_json::json!(num_idle));
    metadata.insert("connections_active".to_string(), serde_json::json!(pool_size - num_idle));
    
    // Try a simple query
    match sqlx::query("SELECT 1").fetch_one(db.as_ref()).await {
        Ok(_) => {
            let latency = start.elapsed().as_millis() as u64;
            metadata.insert("query_latency_ms".to_string(), serde_json::json!(latency));
            
            let status = if latency > 100 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };
            
            Ok(ComponentHealth {
                name: "database".to_string(),
                status,
                message: None,
                latency_ms: latency,
                metadata,
            })
        },
        Err(e) => {
            error!("Database health check failed: {}", e);
            Ok(ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Unhealthy,
                message: Some(format!("Query failed: {}", e)),
                latency_ms: start.elapsed().as_millis() as u64,
                metadata,
            })
        }
    }
}

/// Quick database readiness check
async fn check_database_ready(db: &DatabasePool) -> Result<(), ServerError> {
    sqlx::query("SELECT 1")
        .fetch_one(db)
        .await
        .map_err(|e| ServerError::Internal(format!("Database not ready: {}", e)))?;
    Ok(())
}

/// Check Redis health
async fn check_redis_health(redis_url: &str) -> Result<ComponentHealth, ServerError> {
    let start = Instant::now();
    let mut metadata = HashMap::new();
    
    match redis::Client::open(redis_url) {
        Ok(client) => {
            match client.get_tokio_connection().await {
                Ok(mut conn) => {
                    // Ping Redis
                    let ping_result: Result<String, _> = redis::cmd("PING")
                        .query_async(&mut conn)
                        .await;
                    
                    match ping_result {
                        Ok(_) => {
                            // Get Redis info
                            if let Ok(info) = redis::cmd("INFO")
                                .arg("server")
                                .query_async(&mut conn)
                                .await
                            {
                                // Parse version from info
                                for line in info.lines() {
                                    if line.starts_with("redis_version:") {
                                        let version = line.trim_start_matches("redis_version:");
                                        metadata.insert("version".to_string(), serde_json::json!(version));
                                        break;
                                    }
                                }
                            }
                            
                            let latency = start.elapsed().as_millis() as u64;
                            Ok(ComponentHealth {
                                name: "redis".to_string(),
                                status: HealthStatus::Healthy,
                                message: None,
                                latency_ms: latency,
                                metadata,
                            })
                        },
                        Err(e) => Ok(ComponentHealth {
                            name: "redis".to_string(),
                            status: HealthStatus::Unhealthy,
                            message: Some(format!("Ping failed: {}", e)),
                            latency_ms: start.elapsed().as_millis() as u64,
                            metadata,
                        })
                    }
                },
                Err(e) => Ok(ComponentHealth {
                    name: "redis".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Connection failed: {}", e)),
                    latency_ms: start.elapsed().as_millis() as u64,
                    metadata,
                })
            }
        },
        Err(e) => Ok(ComponentHealth {
            name: "redis".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("Client creation failed: {}", e)),
            latency_ms: start.elapsed().as_millis() as u64,
            metadata,
        })
    }
}

/// Check neuron health
async fn check_neurons_health(
    server: &HAL9Server,
    status: &crate::server::ServerStatus,
) -> ComponentHealth {
    let start = Instant::now();
    let mut metadata = HashMap::new();
    
    let total_neurons = status.neurons.len();
    let healthy_neurons = status.neurons.iter().filter(|n| n.is_healthy).count();
    let unhealthy_neurons = total_neurons - healthy_neurons;
    
    metadata.insert("total".to_string(), serde_json::json!(total_neurons));
    metadata.insert("healthy".to_string(), serde_json::json!(healthy_neurons));
    metadata.insert("unhealthy".to_string(), serde_json::json!(unhealthy_neurons));
    
    // Check layer distribution
    let mut layer_counts = HashMap::new();
    for neuron in &status.neurons {
        *layer_counts.entry(neuron.layer.clone()).or_insert(0) += 1;
    }
    metadata.insert("layer_distribution".to_string(), serde_json::json!(layer_counts));
    
    let status = if unhealthy_neurons == 0 {
        HealthStatus::Healthy
    } else if unhealthy_neurons < total_neurons / 2 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    ComponentHealth {
        name: "neurons".to_string(),
        status,
        message: if unhealthy_neurons > 0 {
            Some(format!("{} unhealthy neurons", unhealthy_neurons))
        } else {
            None
        },
        latency_ms: start.elapsed().as_millis() as u64,
        metadata,
    }
}

/// Check memory health
fn check_memory_health() -> ComponentHealth {
    let mut metadata = HashMap::new();
    
    // Get memory info using sysinfo
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();
    let memory_percent = (used_memory as f64 / total_memory as f64) * 100.0;
    
    metadata.insert("total_mb".to_string(), serde_json::json!(total_memory / 1024));
    metadata.insert("used_mb".to_string(), serde_json::json!(used_memory / 1024));
    metadata.insert("available_mb".to_string(), serde_json::json!(available_memory / 1024));
    metadata.insert("usage_percent".to_string(), serde_json::json!(memory_percent));
    
    let status = if memory_percent > 90.0 {
        HealthStatus::Unhealthy
    } else if memory_percent > 80.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };
    
    ComponentHealth {
        name: "memory".to_string(),
        status,
        message: if memory_percent > 80.0 {
            Some(format!("High memory usage: {:.1}%", memory_percent))
        } else {
            None
        },
        latency_ms: 0,
        metadata,
    }
}

/// Check disk space health
fn check_disk_health() -> ComponentHealth {
    let mut metadata = HashMap::new();
    
    // Check disk space for data directory
    let data_dir = std::env::var("HAL9_DATA_DIR").unwrap_or_else(|_| "/app/data".to_string());
    
    match fs2::available_space(&data_dir) {
        Ok(available_bytes) => {
            match fs2::total_space(&data_dir) {
                Ok(total_bytes) => {
                    let used_bytes = total_bytes - available_bytes;
                    let usage_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;
                    
                    metadata.insert("path".to_string(), serde_json::json!(data_dir));
                    metadata.insert("total_gb".to_string(), serde_json::json!(total_bytes / 1_073_741_824));
                    metadata.insert("available_gb".to_string(), serde_json::json!(available_bytes / 1_073_741_824));
                    metadata.insert("usage_percent".to_string(), serde_json::json!(usage_percent));
                    
                    let status = if usage_percent > 95.0 {
                        HealthStatus::Unhealthy
                    } else if usage_percent > 85.0 {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Healthy
                    };
                    
                    ComponentHealth {
                        name: "disk".to_string(),
                        status,
                        message: if usage_percent > 85.0 {
                            Some(format!("High disk usage: {:.1}%", usage_percent))
                        } else {
                            None
                        },
                        latency_ms: 0,
                        metadata,
                    }
                },
                Err(e) => ComponentHealth {
                    name: "disk".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Failed to get total space: {}", e)),
                    latency_ms: 0,
                    metadata,
                }
            }
        },
        Err(e) => ComponentHealth {
            name: "disk".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("Failed to check disk space: {}", e)),
            latency_ms: 0,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_status_serialization() {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy).unwrap(),
            "\"healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Degraded).unwrap(),
            "\"degraded\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy).unwrap(),
            "\"unhealthy\""
        );
    }
    
    #[test]
    fn test_health_status_to_status_code() {
        assert_eq!(HealthStatus::Healthy.to_status_code(), StatusCode::OK);
        assert_eq!(HealthStatus::Degraded.to_status_code(), StatusCode::OK);
        assert_eq!(HealthStatus::Unhealthy.to_status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }
}