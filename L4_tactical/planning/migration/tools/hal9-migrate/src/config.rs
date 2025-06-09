use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub server_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub monitoring: MonitoringConfig,
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_interval: u64,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub error_rate: f32,
    pub latency_p99_ms: f32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub auto_rollback: bool,
    pub health_check_interval: u64,
    pub min_success_rate: f32,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3030".to_string(),
            timeout_seconds: 300,
            retry_attempts: 3,
            monitoring: MonitoringConfig {
                metrics_interval: 5,
                alert_thresholds: AlertThresholds {
                    error_rate: 0.05,
                    latency_p99_ms: 100.0,
                    cpu_usage: 0.8,
                    memory_usage: 0.9,
                },
            },
            safety: SafetyConfig {
                auto_rollback: true,
                health_check_interval: 30,
                min_success_rate: 0.95,
            },
        }
    }
}

impl MigrationConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}