pub mod pre_check;
pub mod status;
pub mod migrate;
pub mod rollback;
pub mod verify;
pub mod feature;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Common types used across commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub current_phase: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub progress: f32,
    pub is_healthy: bool,
    pub active_features: Vec<String>,
    pub metrics: MigrationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetrics {
    pub total_neurons: usize,
    pub migrated_neurons: usize,
    pub error_rate: f32,
    pub latency_p99: f32,
    pub throughput_rps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;
        match self {
            HealthStatus::Healthy => write!(f, "{}", "✓ Healthy".green()),
            HealthStatus::Degraded => write!(f, "{}", "⚠ Degraded".yellow()),
            HealthStatus::Unhealthy => write!(f, "{}", "✗ Unhealthy".red()),
            HealthStatus::Unknown => write!(f, "{}", "? Unknown".dimmed()),
        }
    }
}

/// Format output based on selected format
pub fn format_output<T: Serialize>(data: &T, format: &crate::OutputFormat) -> anyhow::Result<()> {
    match format {
        crate::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        crate::OutputFormat::Table | crate::OutputFormat::Pretty => {
            // Commands will handle their own pretty/table formatting
            // This is a fallback
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}