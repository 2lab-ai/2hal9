//! Rollback capabilities for safe migration recovery

use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::{Result, Error};
use sysinfo::{System, SystemExt, ProcessExt};
use reqwest;

/// Rollback strategy options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RollbackStrategy {
    /// Immediate full rollback to flat system
    Immediate,
    /// Gradual rollback over time
    Gradual { duration: std::time::Duration },
    /// Partial rollback of specific components
    Partial { components: Vec<String> },
}

/// Rollback manager for migration recovery
pub struct RollbackManager {
    snapshots: Arc<RwLock<Vec<SystemSnapshot>>>,
    strategy: Arc<RwLock<RollbackStrategy>>,
    rollback_history: Arc<RwLock<Vec<RollbackEvent>>>,
    is_rolling_back: Arc<RwLock<bool>>,
}

impl RollbackManager {
    pub fn new(strategy: RollbackStrategy) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
            strategy: Arc::new(RwLock::new(strategy)),
            rollback_history: Arc::new(RwLock::new(Vec::new())),
            is_rolling_back: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Create a snapshot before migration phase
    pub async fn create_snapshot(&self, phase_name: &str) -> Result<Uuid> {
        let snapshot = SystemSnapshot {
            id: Uuid::new_v4(),
            phase_name: phase_name.to_string(),
            timestamp: chrono::Utc::now(),
            system_state: self.capture_system_state().await?,
            feature_flags: self.capture_feature_flags().await?,
            routing_config: self.capture_routing_config().await?,
            metadata: self.capture_metadata().await?,
        };
        
        let snapshot_id = snapshot.id;
        self.snapshots.write().push(snapshot);
        
        tracing::info!("Created snapshot {} for phase {}", snapshot_id, phase_name);
        Ok(snapshot_id)
    }
    
    /// Execute rollback to a specific phase
    pub async fn rollback_to_phase(&self, phase_index: usize) -> Result<()> {
        // Check if already rolling back
        if *self.is_rolling_back.read() {
            return Err(Error::Migration("Rollback already in progress".to_string()));
        }
        
        *self.is_rolling_back.write() = true;
        
        let result = self.execute_rollback(phase_index).await;
        
        *self.is_rolling_back.write() = false;
        
        result
    }
    
    async fn execute_rollback(&self, phase_index: usize) -> Result<()> {
        tracing::warn!("Initiating rollback to phase {}", phase_index);
        
        // Find appropriate snapshot
        let snapshot = {
            let snapshots = self.snapshots.read();
            snapshots.get(phase_index)
                .ok_or_else(|| Error::Migration(format!("No snapshot for phase {}", phase_index)))?
                .clone()
        };
        
        // Record rollback event
        let event = RollbackEvent {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            from_phase: phase_index + 1,
            to_phase: phase_index,
            reason: "Manual rollback triggered".to_string(),
            strategy: self.strategy.read().clone(),
        };
        
        self.rollback_history.write().push(event.clone());
        
        // Execute based on strategy
        match &*self.strategy.read() {
            RollbackStrategy::Immediate => {
                self.execute_immediate_rollback(&snapshot).await?;
            }
            RollbackStrategy::Gradual { duration } => {
                self.execute_gradual_rollback(&snapshot, *duration).await?;
            }
            RollbackStrategy::Partial { components } => {
                self.execute_partial_rollback(&snapshot, components).await?;
            }
        }
        
        tracing::info!("Rollback completed successfully");
        Ok(())
    }
    
    async fn execute_immediate_rollback(&self, snapshot: &SystemSnapshot) -> Result<()> {
        tracing::info!("Executing immediate rollback");
        
        // 1. Stop all traffic to hierarchical system
        self.stop_hierarchical_traffic().await?;
        
        // 2. Drain in-flight requests
        self.drain_requests().await?;
        
        // 3. Restore system state
        self.restore_system_state(&snapshot.system_state).await?;
        
        // 4. Restore feature flags
        self.restore_feature_flags(&snapshot.feature_flags).await?;
        
        // 5. Restore routing configuration
        self.restore_routing_config(&snapshot.routing_config).await?;
        
        // 6. Verify system health
        self.verify_system_health().await?;
        
        Ok(())
    }
    
    async fn execute_gradual_rollback(
        &self,
        snapshot: &SystemSnapshot,
        duration: std::time::Duration,
    ) -> Result<()> {
        tracing::info!("Executing gradual rollback over {:?}", duration);
        
        let steps = 10; // Rollback in 10 steps
        let step_duration = duration / steps as u32;
        
        for step in 0..steps {
            // Calculate traffic percentage for this step
            let hierarchical_percentage = 100.0 * (1.0 - (step as f32 / steps as f32));
            
            // Update routing gradually
            self.update_traffic_percentage(hierarchical_percentage).await?;
            
            // Wait before next step
            tokio::time::sleep(step_duration).await;
            
            // Check system health
            if !self.is_system_healthy().await? {
                tracing::warn!("System unhealthy during gradual rollback, accelerating");
                return self.execute_immediate_rollback(snapshot).await;
            }
        }
        
        // Final restoration
        self.restore_system_state(&snapshot.system_state).await?;
        self.restore_feature_flags(&snapshot.feature_flags).await?;
        
        Ok(())
    }
    
    async fn execute_partial_rollback(
        &self,
        snapshot: &SystemSnapshot,
        components: &[String],
    ) -> Result<()> {
        tracing::info!("Executing partial rollback for components: {:?}", components);
        
        for component in components {
            match component.as_str() {
                "routing" => {
                    self.restore_routing_config(&snapshot.routing_config).await?;
                }
                "features" => {
                    self.restore_feature_flags(&snapshot.feature_flags).await?;
                }
                "state" => {
                    self.restore_partial_state(&snapshot.system_state, component).await?;
                }
                _ => {
                    tracing::warn!("Unknown component for rollback: {}", component);
                }
            }
        }
        
        Ok(())
    }
    
    async fn capture_system_state(&self) -> Result<SystemState> {
        // Capture actual system state from metrics
        use sysinfo::{System, SystemExt, ProcessExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        // Get memory usage
        let memory_usage_mb = (sys.used_memory() / 1024) as u64;
        
        // Get CPU usage
        let cpu_usage_percent = sys.global_cpu_info().cpu_usage();
        
        // Count active neurons by checking running processes
        // This is a heuristic - in production, query actual neuron registry
        let active_neurons = sys.processes().values()
            .filter(|p| p.name().contains("neuron") || p.name().contains("hal9"))
            .count();
        
        // Get active connections from /proc/net/tcp (Linux) or netstat
        let active_connections = self.count_active_connections().await?;
        
        // Capture current configuration
        let configuration = self.get_current_configuration().await?;
        
        Ok(SystemState {
            active_neurons: active_neurons.max(1), // At least 1
            memory_usage_mb,
            cpu_usage_percent,
            active_connections,
            configuration,
        })
    }
    
    async fn capture_feature_flags(&self) -> Result<serde_json::Value> {
        // Read feature flags from configuration file or environment
        let flags_file = std::env::var("HAL9_FEATURE_FLAGS_PATH")
            .unwrap_or_else(|_| "/etc/hal9/feature_flags.json".to_string());
        
        if let Ok(content) = tokio::fs::read_to_string(&flags_file).await {
            if let Ok(flags) = serde_json::from_str(&content) {
                return Ok(flags);
            }
        }
        
        // Fallback to environment variables
        let hierarchical_enabled = std::env::var("HAL9_HIERARCHICAL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
            
        let traffic_percentage = std::env::var("HAL9_HIERARCHICAL_TRAFFIC_PERCENTAGE")
            .unwrap_or_else(|_| "0.0".to_string())
            .parse::<f32>()
            .unwrap_or(0.0);
        
        Ok(serde_json::json!({
            "hierarchical_enabled": hierarchical_enabled,
            "hierarchical_traffic_percentage": traffic_percentage,
            "shadow_mode": std::env::var("HAL9_SHADOW_MODE").unwrap_or_else(|_| "false".to_string()),
            "canary_enabled": std::env::var("HAL9_CANARY_ENABLED").unwrap_or_else(|_| "false".to_string()),
            "migration_phase": std::env::var("HAL9_MIGRATION_PHASE").unwrap_or_else(|_| "0".to_string()),
        }))
    }
    
    async fn capture_routing_config(&self) -> Result<serde_json::Value> {
        // Read routing configuration from configuration service
        let config_path = std::env::var("HAL9_ROUTING_CONFIG_PATH")
            .unwrap_or_else(|_| "/etc/hal9/routing.json".to_string());
        
        if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
            if let Ok(config) = serde_json::from_str(&content) {
                return Ok(config);
            }
        }
        
        // Fallback configuration
        Ok(serde_json::json!({
            "shadow_mode": false,
            "routing_strategy": "percentage",
            "load_balancer": {
                "algorithm": "round_robin",
                "health_check_interval": 5000,
                "timeout_ms": 30000
            },
            "circuit_breaker": {
                "enabled": true,
                "failure_threshold": 5,
                "timeout_ms": 60000
            },
            "retry_policy": {
                "max_attempts": 3,
                "backoff_ms": 1000
            }
        }))
    }
    
    async fn capture_metadata(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "captured_at": chrono::Utc::now(),
            "version": "1.0.0",
        }))
    }
    
    async fn stop_hierarchical_traffic(&self) -> Result<()> {
        tracing::info!("Stopping hierarchical traffic");
        
        // Update load balancer to stop routing to hierarchical system
        let lb_endpoint = std::env::var("HAL9_LOAD_BALANCER_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8081/admin".to_string());
        
        // Send request to update routing rules
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/routing/hierarchical", lb_endpoint))
            .json(&serde_json::json!({
                "enabled": false,
                "drain_timeout_ms": 30000
            }))
            .send()
            .await
            .map_err(|e| Error::Migration(format!("Failed to stop traffic: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::Migration(format!("Load balancer returned: {}", response.status())));
        }
        
        // Update feature flag immediately
        std::env::set_var("HAL9_HIERARCHICAL_TRAFFIC_PERCENTAGE", "0.0");
        
        tracing::info!("Hierarchical traffic stopped successfully");
        Ok(())
    }
    
    async fn drain_requests(&self) -> Result<()> {
        tracing::info!("Draining in-flight requests");
        
        let drain_timeout = std::time::Duration::from_secs(30);
        let start_time = std::time::Instant::now();
        
        // Monitor active connections and wait for them to complete
        loop {
            let active_count = self.count_active_connections().await?;
            
            if active_count == 0 {
                tracing::info!("All requests drained successfully");
                break;
            }
            
            if start_time.elapsed() >= drain_timeout {
                tracing::warn!("Drain timeout reached with {} active connections", active_count);
                break;
            }
            
            tracing::info!("Waiting for {} active connections to complete...", active_count);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        
        // Give a final grace period
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(())
    }
    
    async fn restore_system_state(&self, state: &SystemState) -> Result<()> {
        tracing::info!("Restoring system state from snapshot");
        
        // Write configuration back to file
        let config_path = std::env::var("HAL9_CONFIG_PATH")
            .unwrap_or_else(|_| "/etc/hal9/system.json".to_string());
        
        tokio::fs::create_dir_all(std::path::Path::new(&config_path).parent().unwrap())
            .await
            .map_err(|e| Error::Migration(format!("Failed to create config directory: {}", e)))?;
        
        tokio::fs::write(
            &config_path,
            serde_json::to_string_pretty(&state.configuration)?
        )
        .await
        .map_err(|e| Error::Migration(format!("Failed to write configuration: {}", e)))?;
        
        // Restart services if neuron count differs significantly
        let current_neurons = self.count_active_neurons().await?;
        if (current_neurons as i32 - state.active_neurons as i32).abs() > 10 {
            tracing::warn!("Neuron count mismatch: current={}, snapshot={}", current_neurons, state.active_neurons);
            // In production, trigger service restart here
        }
        
        tracing::info!("System state restored successfully");
        Ok(())
    }
    
    async fn restore_feature_flags(&self, flags: &serde_json::Value) -> Result<()> {
        tracing::info!("Restoring feature flags");
        
        // Write to feature flags file
        let flags_file = std::env::var("HAL9_FEATURE_FLAGS_PATH")
            .unwrap_or_else(|_| "/etc/hal9/feature_flags.json".to_string());
        
        tokio::fs::create_dir_all(std::path::Path::new(&flags_file).parent().unwrap())
            .await
            .map_err(|e| Error::Migration(format!("Failed to create flags directory: {}", e)))?;
        
        tokio::fs::write(
            &flags_file,
            serde_json::to_string_pretty(flags)?
        )
        .await
        .map_err(|e| Error::Migration(format!("Failed to write feature flags: {}", e)))?;
        
        // Also update environment variables for immediate effect
        if let Some(hierarchical_enabled) = flags.get("hierarchical_enabled") {
            std::env::set_var("HAL9_HIERARCHICAL_ENABLED", hierarchical_enabled.to_string());
        }
        if let Some(traffic_pct) = flags.get("hierarchical_traffic_percentage") {
            std::env::set_var("HAL9_HIERARCHICAL_TRAFFIC_PERCENTAGE", traffic_pct.to_string());
        }
        
        // Notify services of flag changes
        self.notify_flag_changes(flags).await?;
        
        tracing::info!("Feature flags restored successfully");
        Ok(())
    }
    
    async fn restore_routing_config(&self, config: &serde_json::Value) -> Result<()> {
        tracing::info!("Restoring routing configuration");
        
        // Write to routing config file
        let config_path = std::env::var("HAL9_ROUTING_CONFIG_PATH")
            .unwrap_or_else(|_| "/etc/hal9/routing.json".to_string());
        
        tokio::fs::write(
            &config_path,
            serde_json::to_string_pretty(config)?
        )
        .await
        .map_err(|e| Error::Migration(format!("Failed to write routing config: {}", e)))?;
        
        // Update load balancer configuration
        let lb_endpoint = std::env::var("HAL9_LOAD_BALANCER_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8081/admin".to_string());
        
        let client = reqwest::Client::new();
        let response = client
            .put(format!("{}/config", lb_endpoint))
            .json(config)
            .send()
            .await
            .map_err(|e| Error::Migration(format!("Failed to update load balancer: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::Migration(format!("Load balancer config update failed: {}", response.status())));
        }
        
        tracing::info!("Routing configuration restored successfully");
        Ok(())
    }
    
    async fn restore_partial_state(&self, state: &SystemState, component: &str) -> Result<()> {
        // TODO: Implement partial state restoration
        tracing::info!("Restoring partial state for component: {}", component);
        Ok(())
    }
    
    async fn verify_system_health(&self) -> Result<()> {
        tracing::info!("Verifying system health after rollback");
        
        let max_retries = 10;
        let mut retry_count = 0;
        
        while retry_count < max_retries {
            if self.is_system_healthy().await? {
                tracing::info!("System health verified successfully");
                return Ok(());
            }
            
            retry_count += 1;
            tracing::warn!("Health check failed, retry {}/{}", retry_count, max_retries);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        
        Err(Error::Migration("System health check failed after rollback".to_string()))
    }
    
    async fn is_system_healthy(&self) -> Result<bool> {
        // Check multiple health endpoints
        let health_checks = vec![
            ("http://localhost:8080/health", "Main API"),
            ("http://localhost:8081/health", "Load Balancer"),
            ("http://localhost:9090/api/v1/query?query=up", "Prometheus"),
        ];
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| Error::Migration(format!("Failed to create HTTP client: {}", e)))?;
        
        let mut all_healthy = true;
        
        for (endpoint, name) in health_checks {
            match client.get(endpoint).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        tracing::debug!("{} health check passed", name);
                    } else {
                        tracing::warn!("{} health check failed: {}", name, response.status());
                        all_healthy = false;
                    }
                }
                Err(e) => {
                    tracing::warn!("{} health check error: {}", name, e);
                    all_healthy = false;
                }
            }
        }
        
        // Check system resources
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let memory_percent = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
        if memory_percent > 90.0 {
            tracing::warn!("Memory usage critical: {:.1}%", memory_percent);
            all_healthy = false;
        }
        
        Ok(all_healthy)
    }
    
    async fn update_traffic_percentage(&self, percentage: f32) -> Result<()> {
        tracing::info!("Updating hierarchical traffic to {}%", percentage);
        
        // Update environment variable
        std::env::set_var("HAL9_HIERARCHICAL_TRAFFIC_PERCENTAGE", percentage.to_string());
        
        // Update load balancer
        let lb_endpoint = std::env::var("HAL9_LOAD_BALANCER_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8081/admin".to_string());
        
        let client = reqwest::Client::new();
        let response = client
            .patch(format!("{}/routing/traffic-split", lb_endpoint))
            .json(&serde_json::json!({
                "hierarchical_percentage": percentage,
                "flat_percentage": 100.0 - percentage
            }))
            .send()
            .await
            .map_err(|e| Error::Migration(format!("Failed to update traffic split: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::Migration(format!("Traffic update failed: {}", response.status())));
        }
        
        // Update feature flags file
        let flags_file = std::env::var("HAL9_FEATURE_FLAGS_PATH")
            .unwrap_or_else(|_| "/etc/hal9/feature_flags.json".to_string());
        
        if let Ok(content) = tokio::fs::read_to_string(&flags_file).await {
            if let Ok(mut flags) = serde_json::from_str::<serde_json::Value>(&content) {
                flags["hierarchical_traffic_percentage"] = serde_json::json!(percentage);
                let _ = tokio::fs::write(&flags_file, serde_json::to_string_pretty(&flags)?).await;
            }
        }
        
        Ok(())
    }
    
    /// Get rollback history
    pub fn get_history(&self) -> Vec<RollbackEvent> {
        self.rollback_history.read().clone()
    }
    
    /// Check if rollback is in progress
    pub fn is_rolling_back(&self) -> bool {
        *self.is_rolling_back.read()
    }
    
    // Helper methods for system operations
    async fn count_active_connections(&self) -> Result<usize> {
        // On Linux, check /proc/net/tcp for established connections
        // On macOS, use netstat or lsof
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = tokio::fs::read_to_string("/proc/net/tcp").await {
                let established_count = content.lines()
                    .skip(1) // Skip header
                    .filter(|line| line.contains("01")) // 01 = ESTABLISHED
                    .count();
                return Ok(established_count);
            }
        }
        
        // Fallback: count connections to our port
        use std::process::Command;
        let output = Command::new("lsof")
            .args(&["-i", ":8080", "-sTCP:ESTABLISHED"])
            .output()
            .map_err(|e| Error::Migration(format!("Failed to run lsof: {}", e)))?;
        
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1) // Skip header
            .count();
        
        Ok(count)
    }
    
    async fn get_current_configuration(&self) -> Result<serde_json::Value> {
        let config_path = std::env::var("HAL9_CONFIG_PATH")
            .unwrap_or_else(|_| "/etc/hal9/system.json".to_string());
        
        if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
            if let Ok(config) = serde_json::from_str(&content) {
                return Ok(config);
            }
        }
        
        // Return default configuration
        Ok(serde_json::json!({
            "version": "1.0.0",
            "system": {
                "max_neurons": 10000,
                "memory_limit_mb": 8192,
                "cpu_cores": 4
            },
            "networking": {
                "port": 8080,
                "max_connections": 1000
            }
        }))
    }
    
    async fn count_active_neurons(&self) -> Result<usize> {
        // In production, query the neuron registry
        // For now, estimate based on process count
        use sysinfo::{System, SystemExt, ProcessExt};
        
        let mut sys = System::new_all();
        sys.refresh_processes();
        
        let neuron_count = sys.processes().values()
            .filter(|p| {
                let name = p.name();
                name.contains("neuron") || name.contains("hal9") || name.contains("worker")
            })
            .count();
        
        Ok(neuron_count.max(1))
    }
    
    async fn notify_flag_changes(&self, flags: &serde_json::Value) -> Result<()> {
        // Notify all services about feature flag changes
        // In production, this would use a message bus or configuration service
        tracing::info!("Notifying services of feature flag changes");
        
        // For now, just log the changes
        tracing::info!("Updated feature flags: {}", serde_json::to_string_pretty(flags)?);
        
        // Touch a timestamp file to signal changes
        let signal_file = "/tmp/hal9-feature-flags-updated";
        tokio::fs::write(signal_file, chrono::Utc::now().to_rfc3339())
            .await
            .map_err(|e| Error::Migration(format!("Failed to write signal file: {}", e)))?;
        
        Ok(())
    }
    
    async fn restore_partial_state(&self, state: &SystemState, component: &str) -> Result<()> {
        tracing::info!("Restoring partial state for component: {}", component);
        
        match component {
            "neurons" => {
                // Restore neuron-specific configuration
                if let Some(neuron_config) = state.configuration.get("neurons") {
                    let config_path = "/etc/hal9/neurons.json";
                    tokio::fs::write(config_path, serde_json::to_string_pretty(neuron_config)?)
                        .await
                        .map_err(|e| Error::Migration(format!("Failed to restore neuron config: {}", e)))?;
                }
            }
            "memory" => {
                // Restore memory-specific configuration
                if let Some(memory_config) = state.configuration.get("memory") {
                    let config_path = "/etc/hal9/memory.json";
                    tokio::fs::write(config_path, serde_json::to_string_pretty(memory_config)?)
                        .await
                        .map_err(|e| Error::Migration(format!("Failed to restore memory config: {}", e)))?;
                }
            }
            "networking" => {
                // Restore networking configuration
                if let Some(net_config) = state.configuration.get("networking") {
                    let config_path = "/etc/hal9/networking.json";
                    tokio::fs::write(config_path, serde_json::to_string_pretty(net_config)?)
                        .await
                        .map_err(|e| Error::Migration(format!("Failed to restore network config: {}", e)))?;
                }
            }
            _ => {
                tracing::warn!("Unknown component for partial restoration: {}", component);
            }
        }
        
        Ok(())
    }
}

/// System snapshot for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemSnapshot {
    id: Uuid,
    phase_name: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    system_state: SystemState,
    feature_flags: serde_json::Value,
    routing_config: serde_json::Value,
    metadata: serde_json::Value,
}

/// System state at snapshot time
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemState {
    active_neurons: usize,
    memory_usage_mb: u64,
    cpu_usage_percent: f32,
    active_connections: usize,
    configuration: serde_json::Value,
}

/// Rollback event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackEvent {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub from_phase: usize,
    pub to_phase: usize,
    pub reason: String,
    pub strategy: RollbackStrategy,
}

/// Rollback trigger conditions
pub struct RollbackTrigger {
    error_threshold: f32,
    latency_threshold: f32,
    health_threshold: f32,
}

impl RollbackTrigger {
    pub fn should_rollback(&self, metrics: &RollbackMetrics) -> bool {
        metrics.error_rate > self.error_threshold ||
        metrics.p99_latency > self.latency_threshold ||
        metrics.health_score < self.health_threshold
    }
}

/// Metrics for rollback decisions
pub struct RollbackMetrics {
    pub error_rate: f32,
    pub p99_latency: f32,
    pub health_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_snapshot_creation() {
        let manager = RollbackManager::new(RollbackStrategy::Immediate);
        
        let snapshot_id = manager.create_snapshot("test_phase").await.unwrap();
        assert_ne!(snapshot_id, Uuid::nil());
        
        let snapshots = manager.snapshots.read();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].phase_name, "test_phase");
    }
    
    #[test]
    fn test_rollback_trigger() {
        let trigger = RollbackTrigger {
            error_threshold: 0.05,
            latency_threshold: 100.0,
            health_threshold: 0.8,
        };
        
        let metrics = RollbackMetrics {
            error_rate: 0.06,
            p99_latency: 90.0,
            health_score: 0.9,
        };
        
        assert!(trigger.should_rollback(&metrics));
        
        let good_metrics = RollbackMetrics {
            error_rate: 0.01,
            p99_latency: 50.0,
            health_score: 0.95,
        };
        
        assert!(!trigger.should_rollback(&good_metrics));
    }
}