//! Migration infrastructure for transitioning from flat to hierarchical architecture
//!
//! This module provides all the components needed for a safe, zero-downtime migration:
//! - Feature flags for gradual rollout
//! - Traffic routing between systems
//! - State migration engine
//! - Rollback capabilities
//! - Monitoring and observability

pub mod feature_flags;
pub mod router;
pub mod state_migration;
pub mod rollback;
pub mod monitoring;

pub use feature_flags::{FeatureFlags, FeatureFlagManager, RequestContext};
pub use router::{MigrationRouter, RoutingDecision};
pub use state_migration::{StateMigrationEngine, MigrationProgress};
pub use rollback::{RollbackManager, RollbackStrategy};
pub use monitoring::{MigrationMonitor, MigrationMetrics};

use crate::{Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main migration orchestrator that coordinates all migration activities
pub struct MigrationOrchestrator {
    feature_flags: Arc<FeatureFlagManager>,
    #[allow(dead_code)]
    router: Arc<MigrationRouter>,
    state_migrator: Arc<StateMigrationEngine>,
    rollback_manager: Arc<RollbackManager>,
    monitor: Arc<MigrationMonitor>,
    phases: Vec<MigrationPhase>,
    current_phase: Arc<RwLock<usize>>,
}

impl MigrationOrchestrator {
    pub fn new(
        feature_flags: Arc<FeatureFlagManager>,
        router: Arc<MigrationRouter>,
        state_migrator: Arc<StateMigrationEngine>,
        rollback_manager: Arc<RollbackManager>,
        monitor: Arc<MigrationMonitor>,
    ) -> Self {
        Self {
            feature_flags,
            router,
            state_migrator,
            rollback_manager,
            monitor,
            phases: Self::create_phases(),
            current_phase: Arc::new(RwLock::new(0)),
        }
    }
    
    fn create_phases() -> Vec<MigrationPhase> {
        vec![
            MigrationPhase {
                name: "Shadow Mode".to_string(),
                description: "Run both systems in parallel, compare outputs".to_string(),
                duration: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 1 week
                traffic_percentage: 0.0,
                features: vec!["shadow_mode".to_string()],
                validations: vec![
                    MigrationValidation::OutputParity { threshold: 0.99 },
                    MigrationValidation::PerformanceWithin { margin: 1.1 },
                ],
            },
            MigrationPhase {
                name: "Canary Deployment".to_string(),
                description: "Route small percentage of traffic to hierarchical".to_string(),
                duration: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 1 week
                traffic_percentage: 5.0,
                features: vec!["hierarchical_routing".to_string()],
                validations: vec![
                    MigrationValidation::ErrorRateBelow { threshold: 0.001 },
                    MigrationValidation::LatencyP99Below { threshold_ms: 100 },
                ],
            },
            MigrationPhase {
                name: "State Migration".to_string(),
                description: "Migrate neuron states to hierarchical system".to_string(),
                duration: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 1 week
                traffic_percentage: 25.0,
                features: vec![
                    "state_migration".to_string(),
                    "cognitive_layers".to_string(),
                ],
                validations: vec![
                    MigrationValidation::StateIntegrity { check_all: true },
                    MigrationValidation::NoDataLoss,
                ],
            },
            MigrationPhase {
                name: "Traffic Ramp-up".to_string(),
                description: "Gradually increase traffic to hierarchical".to_string(),
                duration: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 1 week
                traffic_percentage: 75.0,
                features: vec![
                    "protocol_layer".to_string(),
                    "substrate_layer".to_string(),
                ],
                validations: vec![
                    MigrationValidation::SystemStability { duration: std::time::Duration::from_secs(3600) },
                    MigrationValidation::ResourceUsageOptimal,
                ],
            },
            MigrationPhase {
                name: "Full Migration".to_string(),
                description: "Route all traffic to hierarchical system".to_string(),
                duration: std::time::Duration::from_secs(3 * 24 * 60 * 60), // 3 days
                traffic_percentage: 100.0,
                features: vec!["intelligence_layer".to_string()],
                validations: vec![
                    MigrationValidation::AllMetricsHealthy,
                    MigrationValidation::CustomerSatisfaction { threshold: 0.95 },
                ],
            },
        ]
    }
    
    /// Execute the migration process
    pub async fn execute(&self) -> Result<()> {
        tracing::info!("Starting HAL9 migration from flat to hierarchical architecture");
        
        // Start monitoring
        self.monitor.start().await?;
        
        // Execute each phase
        for (idx, phase) in self.phases.iter().enumerate() {
            *self.current_phase.write().await = idx;
            
            tracing::info!("Starting migration phase {}: {}", idx + 1, phase.name);
            
            // Execute phase
            match self.execute_phase(phase).await {
                Ok(_) => {
                    tracing::info!("Phase {} completed successfully", phase.name);
                }
                Err(e) => {
                    tracing::error!("Phase {} failed: {}", phase.name, e);
                    
                    // Attempt rollback
                    self.rollback_manager.rollback_to_phase(idx.saturating_sub(1)).await?;
                    
                    return Err(Error::Migration(format!("Migration failed at phase {}: {}", phase.name, e)));
                }
            }
            
            // Wait before next phase
            if idx < self.phases.len() - 1 {
                tracing::info!("Waiting before next phase...");
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await; // 1 hour
            }
        }
        
        tracing::info!("Migration completed successfully!");
        Ok(())
    }
    
    async fn execute_phase(&self, phase: &MigrationPhase) -> Result<()> {
        // Update feature flags
        self.update_features_for_phase(phase).await?;
        
        // Perform pre-checks
        for validation in &phase.validations {
            self.validate(validation).await?;
        }
        
        // Monitor phase execution
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < phase.duration {
            // Check health metrics
            let health = self.monitor.get_health_score().await?;
            if health < 0.8 {
                return Err(Error::Migration("Health score below threshold".to_string()));
            }
            
            // Perform periodic validations
            for validation in &phase.validations {
                if let Err(e) = self.validate(validation).await {
                    tracing::warn!("Validation failed during phase execution: {}", e);
                    // Give it some time to recover
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    
                    // Re-check
                    self.validate(validation).await?;
                }
            }
            
            // Sleep before next check
            tokio::time::sleep(std::time::Duration::from_secs(300)).await; // 5 minutes
        }
        
        // Final validation
        for validation in &phase.validations {
            self.validate(validation).await?;
        }
        
        Ok(())
    }
    
    async fn update_features_for_phase(&self, phase: &MigrationPhase) -> Result<()> {
        let mut flags = self.feature_flags.get_flags();
        
        // Update traffic percentage
        flags.hierarchical_traffic_percentage = phase.traffic_percentage;
        
        // Enable phase features
        for feature in &phase.features {
            if let Some(config) = flags.features.get_mut(feature) {
                config.enabled = true;
                config.percentage = 100.0;
            }
        }
        
        self.feature_flags.update_flags(flags).await?;
        Ok(())
    }
    
    async fn validate(&self, validation: &MigrationValidation) -> Result<()> {
        match validation {
            MigrationValidation::OutputParity { threshold } => {
                let parity = self.monitor.get_output_parity().await?;
                if parity < *threshold {
                    return Err(Error::Migration(format!("Output parity {} below threshold {}", parity, threshold)));
                }
            }
            MigrationValidation::PerformanceWithin { margin } => {
                let perf_ratio = self.monitor.get_performance_ratio().await?;
                if perf_ratio > *margin {
                    return Err(Error::Migration(format!("Performance ratio {} exceeds margin {}", perf_ratio, margin)));
                }
            }
            MigrationValidation::ErrorRateBelow { threshold } => {
                let error_rate = self.monitor.get_error_rate().await?;
                if error_rate > *threshold {
                    return Err(Error::Migration(format!("Error rate {} exceeds threshold {}", error_rate, threshold)));
                }
            }
            MigrationValidation::LatencyP99Below { threshold_ms } => {
                let p99 = self.monitor.get_p99_latency().await?;
                if p99 > *threshold_ms as f64 {
                    return Err(Error::Migration(format!("P99 latency {}ms exceeds threshold {}ms", p99, threshold_ms)));
                }
            }
            MigrationValidation::StateIntegrity { check_all } => {
                let integrity = self.state_migrator.verify_integrity(*check_all).await?;
                if !integrity {
                    return Err(Error::Migration("State integrity check failed".to_string()));
                }
            }
            MigrationValidation::NoDataLoss => {
                let data_loss = self.state_migrator.check_data_loss().await?;
                if data_loss {
                    return Err(Error::Migration("Data loss detected".to_string()));
                }
            }
            MigrationValidation::SystemStability { duration } => {
                // Monitor system for specified duration
                let stable = self.monitor.is_stable_for(*duration).await?;
                if !stable {
                    return Err(Error::Migration("System not stable".to_string()));
                }
            }
            MigrationValidation::ResourceUsageOptimal => {
                let optimal = self.monitor.is_resource_usage_optimal().await?;
                if !optimal {
                    return Err(Error::Migration("Resource usage not optimal".to_string()));
                }
            }
            MigrationValidation::AllMetricsHealthy => {
                let healthy = self.monitor.are_all_metrics_healthy().await?;
                if !healthy {
                    return Err(Error::Migration("Not all metrics are healthy".to_string()));
                }
            }
            MigrationValidation::CustomerSatisfaction { threshold } => {
                // This would integrate with customer feedback systems
                let satisfaction = 0.98; // Placeholder
                if satisfaction < *threshold {
                    return Err(Error::Migration(format!("Customer satisfaction {} below threshold {}", satisfaction, threshold)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get current migration status
    pub async fn get_status(&self) -> MigrationStatus {
        let current_phase = *self.current_phase.read().await;
        let progress = self.state_migrator.get_progress().await;
        let metrics = self.monitor.get_current_metrics().await;
        let is_healthy = metrics.is_healthy();
        
        MigrationStatus {
            current_phase,
            total_phases: self.phases.len(),
            phase_name: self.phases.get(current_phase).map(|p| p.name.clone()).unwrap_or_default(),
            state_migration_progress: progress,
            metrics,
            is_healthy,
        }
    }
}

/// Migration phase definition
#[derive(Debug, Clone)]
pub struct MigrationPhase {
    pub name: String,
    pub description: String,
    pub duration: std::time::Duration,
    pub traffic_percentage: f32,
    pub features: Vec<String>,
    pub validations: Vec<MigrationValidation>,
}

/// Validation criteria for migration phases
#[derive(Debug, Clone)]
pub enum MigrationValidation {
    OutputParity { threshold: f32 },
    PerformanceWithin { margin: f32 },
    ErrorRateBelow { threshold: f32 },
    LatencyP99Below { threshold_ms: u32 },
    StateIntegrity { check_all: bool },
    NoDataLoss,
    SystemStability { duration: std::time::Duration },
    ResourceUsageOptimal,
    AllMetricsHealthy,
    CustomerSatisfaction { threshold: f32 },
}

/// Current migration status
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub current_phase: usize,
    pub total_phases: usize,
    pub phase_name: String,
    pub state_migration_progress: MigrationProgress,
    pub metrics: MigrationMetrics,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migration_phases() {
        let phases = MigrationOrchestrator::create_phases();
        assert_eq!(phases.len(), 5);
        assert_eq!(phases[0].name, "Shadow Mode");
        assert_eq!(phases[4].name, "Full Migration");
        assert_eq!(phases[4].traffic_percentage, 100.0);
    }
}