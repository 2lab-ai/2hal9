//! Organization management for enterprise deployments

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};
use uuid::Uuid;
use std::collections::HashMap;

/// Organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub subscription_tier: SubscriptionTier,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// SSO configuration
    pub sso_enabled: bool,
    pub sso_provider: Option<String>,
    pub sso_config: Option<serde_json::Value>,
    
    /// Security settings
    pub enforce_2fa: bool,
    pub password_policy: PasswordPolicy,
    pub session_timeout_minutes: u32,
    pub allowed_ip_ranges: Vec<String>,
    
    /// Usage limits
    pub max_users: Option<u32>,
    pub max_api_calls_per_month: Option<u64>,
    pub max_neurons: Option<u32>,
    pub max_storage_gb: Option<u32>,
    
    /// Features
    pub enabled_features: Vec<String>,
    pub custom_branding: Option<BrandingConfig>,
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            sso_enabled: false,
            sso_provider: None,
            sso_config: None,
            enforce_2fa: false,
            password_policy: PasswordPolicy::default(),
            session_timeout_minutes: 480, // 8 hours
            allowed_ip_ranges: vec!["0.0.0.0/0".to_string()],
            max_users: None,
            max_api_calls_per_month: None,
            max_neurons: None,
            max_storage_gb: None,
            enabled_features: vec![],
            custom_branding: None,
        }
    }
}

/// Subscription tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom(String),
}

impl SubscriptionTier {
    /// Get tier limits
    pub fn limits(&self) -> TierLimits {
        match self {
            Self::Free => TierLimits {
                max_users: 5,
                max_api_calls_per_month: 10_000,
                max_neurons: 3,
                max_storage_gb: 1,
                sso_enabled: false,
                audit_retention_days: 7,
            },
            Self::Starter => TierLimits {
                max_users: 20,
                max_api_calls_per_month: 100_000,
                max_neurons: 10,
                max_storage_gb: 10,
                sso_enabled: false,
                audit_retention_days: 30,
            },
            Self::Professional => TierLimits {
                max_users: 100,
                max_api_calls_per_month: 1_000_000,
                max_neurons: 50,
                max_storage_gb: 100,
                sso_enabled: true,
                audit_retention_days: 90,
            },
            Self::Enterprise => TierLimits {
                max_users: u32::MAX,
                max_api_calls_per_month: u64::MAX,
                max_neurons: u32::MAX,
                max_storage_gb: u32::MAX,
                sso_enabled: true,
                audit_retention_days: 365,
            },
            Self::Custom(_) => TierLimits {
                max_users: u32::MAX,
                max_api_calls_per_month: u64::MAX,
                max_neurons: u32::MAX,
                max_storage_gb: u32::MAX,
                sso_enabled: true,
                audit_retention_days: 365,
            },
        }
    }
}

/// Tier limits
#[derive(Debug, Clone)]
pub struct TierLimits {
    pub max_users: u32,
    pub max_api_calls_per_month: u64,
    pub max_neurons: u32,
    pub max_storage_gb: u32,
    pub sso_enabled: bool,
    pub audit_retention_days: u32,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: u32,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            max_age_days: None,
            prevent_reuse_count: 5,
        }
    }
}

/// Custom branding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub custom_css: Option<String>,
    pub favicon_url: Option<String>,
}

/// Organization manager
pub struct OrganizationManager {
    pool: OrganizationPool,
}

enum OrganizationPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl OrganizationManager {
    /// Create new organization manager
    pub fn new_postgres(pool: PgPool) -> Self {
        Self {
            pool: OrganizationPool::Postgres(pool),
        }
    }
    
    pub fn new_sqlite(pool: SqlitePool) -> Self {
        Self {
            pool: OrganizationPool::Sqlite(pool),
        }
    }
    
    /// Create new organization
    pub async fn create_organization(&self, name: String, domain: Option<String>) -> Result<Organization> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let org = Organization {
            id,
            name: name.clone(),
            domain: domain.clone(),
            subscription_tier: SubscriptionTier::Free,
            settings: OrganizationSettings::default(),
            created_at: now,
            updated_at: now,
        };
        
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO organizations (id, name, domain, subscription_tier, settings, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    org.id,
                    org.name,
                    org.domain,
                    serde_json::to_string(&org.subscription_tier)?,
                    serde_json::to_value(&org.settings)?,
                    org.created_at,
                    org.updated_at
                )
                .execute(pool)
                .await?;
            }
            OrganizationPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO organizations (id, name, domain, subscription_tier, settings, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                    org.id.to_string(),
                    org.name,
                    org.domain,
                    serde_json::to_string(&org.subscription_tier)?,
                    serde_json::to_string(&org.settings)?,
                    org.created_at.timestamp(),
                    org.updated_at.timestamp()
                )
                .execute(pool)
                .await?;
            }
        }
        
        Ok(org)
    }
    
    /// Get organization by ID
    pub async fn get_organization(&self, id: Uuid) -> Result<Option<Organization>> {
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT * FROM organizations WHERE id = $1",
                    id
                )
                .fetch_optional(pool)
                .await?;
                
                match row {
                    Some(r) => Ok(Some(Organization {
                        id: r.id,
                        name: r.name,
                        domain: r.domain,
                        subscription_tier: serde_json::from_str(&r.subscription_tier)?,
                        settings: serde_json::from_value(r.settings)?,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    })),
                    None => Ok(None),
                }
            }
            OrganizationPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(None)
            }
        }
    }
    
    /// Get organization by domain
    pub async fn get_organization_by_domain(&self, domain: &str) -> Result<Option<Organization>> {
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT * FROM organizations WHERE domain = $1",
                    domain
                )
                .fetch_optional(pool)
                .await?;
                
                match row {
                    Some(r) => Ok(Some(Organization {
                        id: r.id,
                        name: r.name,
                        domain: r.domain,
                        subscription_tier: serde_json::from_str(&r.subscription_tier)?,
                        settings: serde_json::from_value(r.settings)?,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    })),
                    None => Ok(None),
                }
            }
            OrganizationPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(None)
            }
        }
    }
    
    /// Update organization
    pub async fn update_organization(&self, org: &Organization) -> Result<()> {
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    UPDATE organizations 
                    SET name = $2, domain = $3, subscription_tier = $4, settings = $5, updated_at = $6
                    WHERE id = $1
                    "#,
                    org.id,
                    org.name,
                    org.domain,
                    serde_json::to_string(&org.subscription_tier)?,
                    serde_json::to_value(&org.settings)?,
                    Utc::now()
                )
                .execute(pool)
                .await?;
            }
            OrganizationPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    UPDATE organizations 
                    SET name = ?2, domain = ?3, subscription_tier = ?4, settings = ?5, updated_at = ?6
                    WHERE id = ?1
                    "#,
                    org.id.to_string(),
                    org.name,
                    org.domain,
                    serde_json::to_string(&org.subscription_tier)?,
                    serde_json::to_string(&org.settings)?,
                    Utc::now().timestamp()
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Delete organization
    pub async fn delete_organization(&self, id: Uuid) -> Result<()> {
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                sqlx::query!("DELETE FROM organizations WHERE id = $1", id)
                    .execute(pool)
                    .await?;
            }
            OrganizationPool::Sqlite(pool) => {
                sqlx::query!("DELETE FROM organizations WHERE id = ?1", id.to_string())
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }
    
    /// Check if user can perform action in organization
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        action: &str,
    ) -> Result<bool> {
        // Check user's role in organization
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT role FROM user_organizations WHERE user_id = $1 AND organization_id = $2",
                    user_id,
                    org_id
                )
                .fetch_optional(pool)
                .await?;
                
                match row {
                    Some(r) => {
                        // Check if role has permission for action
                        Ok(self.role_has_permission(&r.role, action))
                    }
                    None => Ok(false),
                }
            }
            OrganizationPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(false)
            }
        }
    }
    
    /// Check if role has permission
    fn role_has_permission(&self, role: &str, action: &str) -> bool {
        match role {
            "owner" => true, // Owners can do everything
            "admin" => !matches!(action, "delete_organization" | "change_owner"),
            "member" => matches!(action, "read" | "create_neuron" | "execute"),
            "read_only" => matches!(action, "read"),
            _ => false,
        }
    }
    
    /// Get organization usage stats
    pub async fn get_usage_stats(&self, org_id: Uuid) -> Result<UsageStats> {
        match &self.pool {
            OrganizationPool::Postgres(pool) => {
                // Count users
                let user_count = sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM user_organizations WHERE organization_id = $1",
                    org_id
                )
                .fetch_one(pool)
                .await?
                .unwrap_or(0) as u32;
                
                // Count API calls this month
                let api_calls = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) FROM audit_log 
                    WHERE organization_id = $1 
                    AND action = 'api_call'
                    AND created_at >= date_trunc('month', CURRENT_DATE)
                    "#,
                    org_id
                )
                .fetch_one(pool)
                .await?
                .unwrap_or(0) as u64;
                
                // Count neurons
                let neuron_count = sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM neurons WHERE organization_id = $1",
                    org_id
                )
                .fetch_one(pool)
                .await?
                .unwrap_or(0) as u32;
                
                Ok(UsageStats {
                    user_count,
                    api_calls_this_month: api_calls,
                    neuron_count,
                    storage_gb: 0.0, // TODO: Implement storage calculation
                })
            }
            OrganizationPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(UsageStats::default())
            }
        }
    }
}

/// Organization usage statistics
#[derive(Debug, Clone, Serialize)]
pub struct UsageStats {
    pub user_count: u32,
    pub api_calls_this_month: u64,
    pub neuron_count: u32,
    pub storage_gb: f64,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            user_count: 0,
            api_calls_this_month: 0,
            neuron_count: 0,
            storage_gb: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_tier_limits() {
        let free = SubscriptionTier::Free;
        let limits = free.limits();
        assert_eq!(limits.max_users, 5);
        assert!(!limits.sso_enabled);
        
        let enterprise = SubscriptionTier::Enterprise;
        let limits = enterprise.limits();
        assert_eq!(limits.max_users, u32::MAX);
        assert!(limits.sso_enabled);
    }
    
    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 8);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_numbers);
    }
}