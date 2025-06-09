use anyhow::Result;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// Client for communicating with HAL9 migration API
pub struct MigrationClient {
    client: Client,
    base_url: Url,
}

impl MigrationClient {
    pub fn new(server: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        let base_url = Url::parse(server)?;
        
        Ok(Self { client, base_url })
    }
    
    /// Get current migration status
    pub async fn get_status(&self) -> Result<MigrationStatusResponse> {
        let url = self.base_url.join("/api/migration/status")?;
        debug!("Fetching migration status from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Start migration phase
    pub async fn start_migration(&self, request: StartMigrationRequest) -> Result<MigrationResponse> {
        let url = self.base_url.join("/api/migration/start")?;
        debug!("Starting migration: {:?}", request);
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Rollback migration
    pub async fn rollback(&self, request: RollbackRequest) -> Result<MigrationResponse> {
        let url = self.base_url.join("/api/migration/rollback")?;
        debug!("Rolling back migration: {:?}", request);
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Get health checks
    pub async fn get_health(&self) -> Result<Vec<HealthCheckResponse>> {
        let url = self.base_url.join("/api/health")?;
        debug!("Fetching health status from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Get feature flags
    pub async fn get_features(&self) -> Result<Vec<FeatureFlagResponse>> {
        let url = self.base_url.join("/api/migration/features")?;
        debug!("Fetching feature flags from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Update feature flag
    pub async fn update_feature(&self, name: &str, request: UpdateFeatureRequest) -> Result<FeatureFlagResponse> {
        let url = self.base_url.join(&format!("/api/migration/features/{}", name))?;
        debug!("Updating feature {}: {:?}", name, request);
        
        let response = self.client
            .put(url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Export migration state
    pub async fn export_state(&self) -> Result<MigrationStateExport> {
        let url = self.base_url.join("/api/migration/state/export")?;
        debug!("Exporting migration state");
        
        let response = self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Import migration state
    pub async fn import_state(&self, state: MigrationStateExport) -> Result<MigrationResponse> {
        let url = self.base_url.join("/api/migration/state/import")?;
        debug!("Importing migration state");
        
        let response = self.client
            .post(url)
            .json(&state)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        
        Ok(response)
    }
}

// Request/Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStatusResponse {
    pub phase: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub progress: f32,
    pub is_healthy: bool,
    pub metrics: MigrationMetrics,
    pub active_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationMetrics {
    pub total_neurons: usize,
    pub migrated_neurons: usize,
    pub error_rate: f32,
    pub latency_p99: f32,
    pub throughput_rps: f32,
}

#[derive(Debug, Serialize)]
pub struct StartMigrationRequest {
    pub phase: String,
    pub percentage: Option<u8>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RollbackRequest {
    pub to_phase: Option<String>,
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct MigrationResponse {
    pub success: bool,
    pub message: String,
    pub phase: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthCheckResponse {
    pub component: String,
    pub status: String,
    pub message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureFlagResponse {
    pub name: String,
    pub enabled: bool,
    pub percentage: Option<u8>,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateFeatureRequest {
    pub enabled: bool,
    pub percentage: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStateExport {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub phase: String,
    pub features: Vec<FeatureFlagResponse>,
    pub checkpoints: Vec<MigrationCheckpoint>,
    pub metrics: MigrationMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationCheckpoint {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub phase: String,
    pub description: Option<String>,
}