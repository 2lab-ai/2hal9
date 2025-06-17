//! Audit logging for compliance and security monitoring

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};
use uuid::Uuid;
use std::net::IpAddr;
use std::collections::HashMap;

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub details: serde_json::Value,
    pub risk_score: f32,
    pub status: EventStatus,
    pub error_message: Option<String>,
}

/// Audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Authentication
    Login,
    Logout,
    FailedLogin,
    PasswordChange,
    PasswordReset,
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorVerified,
    
    // Data access
    DataAccess,
    DataExport,
    DataModification,
    DataDeletion,
    DataSharing,
    
    // Administration
    UserCreation,
    UserDeletion,
    UserModification,
    RoleAssignment,
    RoleRevocation,
    PermissionChange,
    
    // Organization
    OrganizationCreated,
    OrganizationUpdated,
    OrganizationDeleted,
    TeamCreated,
    TeamUpdated,
    TeamDeleted,
    
    // Neuron operations
    NeuronCreated,
    NeuronUpdated,
    NeuronDeleted,
    NeuronExecuted,
    SignalProcessed,
    
    // API operations
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiCall,
    ApiRateLimitExceeded,
    
    // Security
    SecurityAlert,
    PolicyViolation,
    SuspiciousActivity,
    AccessDenied,
    
    // Compliance
    ComplianceCheck,
    DataRetentionApplied,
    DataPurged,
    ConsentUpdated,
    
    // Custom
    Custom(String),
}

/// Event status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Success,
    Failed,
    Partial,
    Pending,
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.8 => Self::Critical,
            s if s >= 0.6 => Self::High,
            s if s >= 0.4 => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// Audit event builder
pub struct AuditEventBuilder {
    event: AuditEvent,
}

impl AuditEventBuilder {
    pub fn new(
        organization_id: Uuid,
        user_id: Uuid,
        action: AuditAction,
        resource_type: &str,
        resource_id: &str,
    ) -> Self {
        Self {
            event: AuditEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                organization_id,
                user_id,
                session_id: None,
                action,
                resource_type: resource_type.to_string(),
                resource_id: resource_id.to_string(),
                ip_address: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                user_agent: String::new(),
                details: serde_json::Value::Object(serde_json::Map::new()),
                risk_score: 0.0,
                status: EventStatus::Success,
                error_message: None,
            },
        }
    }
    
    pub fn session_id(mut self, session_id: Uuid) -> Self {
        self.event.session_id = Some(session_id);
        self
    }
    
    pub fn ip_address(mut self, ip: IpAddr) -> Self {
        self.event.ip_address = ip;
        self
    }
    
    pub fn user_agent(mut self, agent: &str) -> Self {
        self.event.user_agent = agent.to_string();
        self
    }
    
    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.event.details = details;
        self
    }
    
    pub fn risk_score(mut self, score: f32) -> Self {
        self.event.risk_score = score.clamp(0.0, 1.0);
        self
    }
    
    pub fn status(mut self, status: EventStatus) -> Self {
        self.event.status = status;
        self
    }
    
    pub fn error(mut self, message: &str) -> Self {
        self.event.status = EventStatus::Failed;
        self.event.error_message = Some(message.to_string());
        self
    }
    
    pub fn build(self) -> AuditEvent {
        self.event
    }
}

/// Audit logger
pub struct AuditLogger {
    pool: AuditPool,
    risk_analyzer: RiskAnalyzer,
    retention_policy: RetentionPolicy,
}

enum AuditPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

/// Risk analyzer for detecting suspicious activities
struct RiskAnalyzer {
    rules: Vec<RiskRule>,
}

#[derive(Clone)]
struct RiskRule {
    name: String,
    condition: Box<dyn Fn(&AuditEvent) -> bool + Send + Sync>,
    score_modifier: f32,
}

/// Retention policy
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub default_retention_days: u32,
    pub action_retention: HashMap<String, u32>,
    pub compliance_mode: ComplianceMode,
}

#[derive(Debug, Clone, Copy)]
pub enum ComplianceMode {
    Standard,
    Gdpr,
    Hipaa,
    Soc2,
    Custom,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            default_retention_days: 90,
            action_retention: HashMap::new(),
            compliance_mode: ComplianceMode::Standard,
        }
    }
}

impl AuditLogger {
    /// Create new audit logger with PostgreSQL
    pub fn new_postgres(pool: PgPool) -> Self {
        Self {
            pool: AuditPool::Postgres(pool),
            risk_analyzer: RiskAnalyzer::default(),
            retention_policy: RetentionPolicy::default(),
        }
    }
    
    /// Create new audit logger with SQLite
    pub fn new_sqlite(pool: SqlitePool) -> Self {
        Self {
            pool: AuditPool::Sqlite(pool),
            risk_analyzer: RiskAnalyzer::default(),
            retention_policy: RetentionPolicy::default(),
        }
    }
    
    /// Log audit event
    pub async fn log(&self, mut event: AuditEvent) -> Result<()> {
        // Analyze risk
        event.risk_score = self.risk_analyzer.analyze(&event);
        
        // Store event
        match &self.pool {
            AuditPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO audit_log (
                        id, timestamp, organization_id, user_id, session_id,
                        action, resource_type, resource_id, ip_address, user_agent,
                        details, risk_score, status, error_message
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                    "#,
                    event.id,
                    event.timestamp,
                    event.organization_id,
                    event.user_id,
                    event.session_id,
                    serde_json::to_string(&event.action)?,
                    event.resource_type,
                    event.resource_id,
                    event.ip_address.to_string(),
                    event.user_agent,
                    event.details,
                    event.risk_score,
                    serde_json::to_string(&event.status)?,
                    event.error_message
                )
                .execute(pool)
                .await?;
            }
            AuditPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO audit_log (
                        id, timestamp, organization_id, user_id, session_id,
                        action, resource_type, resource_id, ip_address, user_agent,
                        details, risk_score, status, error_message
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    "#,
                    event.id.to_string(),
                    event.timestamp.timestamp(),
                    event.organization_id.to_string(),
                    event.user_id.to_string(),
                    event.session_id.map(|id| id.to_string()),
                    serde_json::to_string(&event.action)?,
                    event.resource_type,
                    event.resource_id,
                    event.ip_address.to_string(),
                    event.user_agent,
                    serde_json::to_string(&event.details)?,
                    event.risk_score,
                    serde_json::to_string(&event.status)?,
                    event.error_message
                )
                .execute(pool)
                .await?;
            }
        }
        
        // Check for high-risk events
        if event.risk_score >= 0.8 {
            self.trigger_security_alert(&event).await?;
        }
        
        Ok(())
    }
    
    /// Log multiple events as a batch
    pub async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<()> {
        for event in events {
            self.log(event).await?;
        }
        Ok(())
    }
    
    /// Query audit events
    pub async fn query(
        &self,
        filter: AuditFilter,
    ) -> Result<Vec<AuditEvent>> {
        match &self.pool {
            AuditPool::Postgres(pool) => {
                let mut query = String::from("SELECT * FROM audit_log WHERE 1=1");
                let mut binds = vec![];
                
                if let Some(org_id) = filter.organization_id {
                    query.push_str(" AND organization_id = $1");
                    binds.push(org_id.to_string());
                }
                
                if let Some(user_id) = filter.user_id {
                    query.push_str(&format!(" AND user_id = ${}", binds.len() + 1));
                    binds.push(user_id.to_string());
                }
                
                if let Some(start) = filter.start_time {
                    query.push_str(&format!(" AND timestamp >= ${}", binds.len() + 1));
                    binds.push(start.to_rfc3339());
                }
                
                if let Some(end) = filter.end_time {
                    query.push_str(&format!(" AND timestamp <= ${}", binds.len() + 1));
                    binds.push(end.to_rfc3339());
                }
                
                if let Some(action) = filter.action {
                    query.push_str(&format!(" AND action = ${}", binds.len() + 1));
                    binds.push(serde_json::to_string(&action)?);
                }
                
                query.push_str(" ORDER BY timestamp DESC");
                
                if let Some(limit) = filter.limit {
                    query.push_str(&format!(" LIMIT {}", limit));
                }
                
                // Execute dynamic query
                let rows = sqlx::query(&query)
                    .fetch_all(pool)
                    .await?;
                
                // Convert rows to AuditEvent
                Ok(vec![])  // TODO: Implement row mapping
            }
            AuditPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(vec![])
            }
        }
    }
    
    /// Get audit statistics
    pub async fn get_statistics(
        &self,
        organization_id: Uuid,
        period: StatisticsPeriod,
    ) -> Result<AuditStatistics> {
        let start_time = match period {
            StatisticsPeriod::Day => Utc::now() - chrono::Duration::days(1),
            StatisticsPeriod::Week => Utc::now() - chrono::Duration::weeks(1),
            StatisticsPeriod::Month => Utc::now() - chrono::Duration::days(30),
            StatisticsPeriod::Year => Utc::now() - chrono::Duration::days(365),
        };
        
        match &self.pool {
            AuditPool::Postgres(pool) => {
                // Count events by action
                let action_counts = sqlx::query!(
                    r#"
                    SELECT action, COUNT(*) as count
                    FROM audit_log
                    WHERE organization_id = $1 AND timestamp >= $2
                    GROUP BY action
                    "#,
                    organization_id,
                    start_time
                )
                .fetch_all(pool)
                .await?;
                
                // Count events by risk level
                let risk_counts = sqlx::query!(
                    r#"
                    SELECT 
                        CASE 
                            WHEN risk_score >= 0.8 THEN 'critical'
                            WHEN risk_score >= 0.6 THEN 'high'
                            WHEN risk_score >= 0.4 THEN 'medium'
                            ELSE 'low'
                        END as risk_level,
                        COUNT(*) as count
                    FROM audit_log
                    WHERE organization_id = $1 AND timestamp >= $2
                    GROUP BY risk_level
                    "#,
                    organization_id,
                    start_time
                )
                .fetch_all(pool)
                .await?;
                
                // Get top users by activity
                let top_users = sqlx::query!(
                    r#"
                    SELECT user_id, COUNT(*) as event_count
                    FROM audit_log
                    WHERE organization_id = $1 AND timestamp >= $2
                    GROUP BY user_id
                    ORDER BY event_count DESC
                    LIMIT 10
                    "#,
                    organization_id,
                    start_time
                )
                .fetch_all(pool)
                .await?;
                
                Ok(AuditStatistics {
                    total_events: action_counts.iter().map(|r| r.count.unwrap_or(0) as u64).sum(),
                    events_by_action: action_counts.into_iter()
                        .map(|r| (r.action, r.count.unwrap_or(0) as u64))
                        .collect(),
                    events_by_risk: risk_counts.into_iter()
                        .map(|r| (r.risk_level.unwrap_or_default(), r.count.unwrap_or(0) as u64))
                        .collect(),
                    top_users: top_users.into_iter()
                        .map(|r| (r.user_id, r.event_count.unwrap_or(0) as u64))
                        .collect(),
                    period,
                })
            }
            AuditPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(AuditStatistics::default())
            }
        }
    }
    
    /// Apply retention policy
    pub async fn apply_retention(&self) -> Result<u64> {
        let mut deleted_count = 0u64;
        
        match &self.pool {
            AuditPool::Postgres(pool) => {
                // Delete events older than default retention
                let cutoff = Utc::now() - chrono::Duration::days(self.retention_policy.default_retention_days as i64);
                
                let result = sqlx::query!(
                    "DELETE FROM audit_log WHERE timestamp < $1",
                    cutoff
                )
                .execute(pool)
                .await?;
                
                deleted_count = result.rows_affected();
            }
            AuditPool::Sqlite(pool) => {
                let cutoff = Utc::now() - chrono::Duration::days(self.retention_policy.default_retention_days as i64);
                
                let result = sqlx::query!(
                    "DELETE FROM audit_log WHERE timestamp < ?1",
                    cutoff.timestamp()
                )
                .execute(pool)
                .await?;
                
                deleted_count = result.rows_affected();
            }
        }
        
        Ok(deleted_count)
    }
    
    /// Export audit logs
    pub async fn export(
        &self,
        filter: AuditFilter,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        let events = self.query(filter).await?;
        
        match format {
            ExportFormat::Json => {
                Ok(serde_json::to_vec_pretty(&events)?)
            }
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                
                // Write headers
                wtr.write_record(&[
                    "timestamp", "organization_id", "user_id", "action",
                    "resource_type", "resource_id", "ip_address", "risk_score",
                    "status", "details"
                ])?;
                
                // Write records
                for event in events {
                    wtr.write_record(&[
                        &event.timestamp.to_rfc3339(),
                        &event.organization_id.to_string(),
                        &event.user_id.to_string(),
                        &serde_json::to_string(&event.action)?,
                        &event.resource_type,
                        &event.resource_id,
                        &event.ip_address.to_string(),
                        &event.risk_score.to_string(),
                        &serde_json::to_string(&event.status)?,
                        &event.details.to_string(),
                    ])?;
                }
                
                wtr.flush()?;
                Ok(wtr.into_inner()?)
            }
        }
    }
    
    /// Trigger security alert for high-risk events
    async fn trigger_security_alert(&self, event: &AuditEvent) -> Result<()> {
        // TODO: Implement alerting mechanism (email, webhook, etc.)
        eprintln!(
            "SECURITY ALERT: High-risk event detected - Action: {:?}, User: {}, Risk Score: {}",
            event.action, event.user_id, event.risk_score
        );
        Ok(())
    }
}

/// Audit filter
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub organization_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_risk_score: Option<f32>,
    pub status: Option<EventStatus>,
    pub limit: Option<u32>,
}

/// Statistics period
#[derive(Debug, Clone, Copy)]
pub enum StatisticsPeriod {
    Day,
    Week,
    Month,
    Year,
}

/// Audit statistics
#[derive(Debug, Clone, Default)]
pub struct AuditStatistics {
    pub total_events: u64,
    pub events_by_action: HashMap<String, u64>,
    pub events_by_risk: HashMap<String, u64>,
    pub top_users: Vec<(Uuid, u64)>,
    pub period: StatisticsPeriod,
}

impl Default for StatisticsPeriod {
    fn default() -> Self {
        Self::Day
    }
}

/// Export format
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl Default for RiskAnalyzer {
    fn default() -> Self {
        let mut analyzer = Self { rules: Vec::new() };
        
        // Add default risk rules
        analyzer.add_rule(
            "multiple_failed_logins",
            |event| matches!(event.action, AuditAction::FailedLogin),
            0.3,
        );
        
        analyzer.add_rule(
            "data_export",
            |event| matches!(event.action, AuditAction::DataExport),
            0.2,
        );
        
        analyzer.add_rule(
            "permission_change",
            |event| matches!(event.action, AuditAction::PermissionChange | AuditAction::RoleAssignment),
            0.4,
        );
        
        analyzer.add_rule(
            "suspicious_activity",
            |event| matches!(event.action, AuditAction::SuspiciousActivity),
            0.8,
        );
        
        analyzer
    }
}

impl RiskAnalyzer {
    fn add_rule<F>(&mut self, name: &str, condition: F, score_modifier: f32)
    where
        F: Fn(&AuditEvent) -> bool + Send + Sync + 'static,
    {
        self.rules.push(RiskRule {
            name: name.to_string(),
            condition: Box::new(condition),
            score_modifier,
        });
    }
    
    fn analyze(&self, event: &AuditEvent) -> f32 {
        let mut score = 0.0;
        
        for rule in &self.rules {
            if (rule.condition)(event) {
                score += rule.score_modifier;
            }
        }
        
        // Additional factors
        if event.status == EventStatus::Failed {
            score += 0.1;
        }
        
        // Normalize to 0-1 range
        score.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEventBuilder::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            AuditAction::Login,
            "session",
            "session123"
        )
        .ip_address(IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)))
        .user_agent("Mozilla/5.0")
        .risk_score(0.2)
        .build();
        
        assert_eq!(event.resource_type, "session");
        assert_eq!(event.resource_id, "session123");
        assert_eq!(event.risk_score, 0.2);
    }
    
    #[test]
    fn test_risk_level() {
        assert_eq!(RiskLevel::from_score(0.1), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.5), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.7), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::Critical);
    }
    
    #[test]
    fn test_risk_analyzer() {
        let analyzer = RiskAnalyzer::default();
        
        let login_event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            organization_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_id: None,
            action: AuditAction::FailedLogin,
            resource_type: "session".to_string(),
            resource_id: "test".to_string(),
            ip_address: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            user_agent: String::new(),
            details: serde_json::Value::Null,
            risk_score: 0.0,
            status: EventStatus::Failed,
            error_message: None,
        };
        
        let score = analyzer.analyze(&login_event);
        assert!(score > 0.0); // Failed login should increase risk
    }
}