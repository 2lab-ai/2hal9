//! Refactored database operations for enterprise features
//! This version avoids using SQLX Any type with Json

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, PgPool, SqlitePool};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use super::rbac::{Role, Permission, RoleAssignment, AssignmentScope};
use super::organization::Organization;
use super::team::Team;
use super::audit::{AuditEvent as AuditLog, AuditAction};

/// Database abstraction that handles both Postgres and SQLite
pub enum EnterpriseDb {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl EnterpriseDb {
    /// Create from database URL
    pub async fn from_url(database_url: &str) -> Result<Self> {
        if database_url.starts_with("postgres") {
            let pool = PgPool::connect(database_url).await?;
            Ok(EnterpriseDb::Postgres(pool))
        } else {
            let pool = SqlitePool::connect(database_url).await?;
            Ok(EnterpriseDb::Sqlite(pool))
        }
    }
    
    /// Create role
    pub async fn create_role(&self, role: &Role) -> Result<()> {
        match self {
            EnterpriseDb::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO roles (id, name, description, permissions, is_system, organization_id, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                       ON CONFLICT (id) DO NOTHING"#,
                    role.id,
                    role.name,
                    role.description,
                    serde_json::to_value(&role.permissions)?,
                    role.is_system,
                    role.organization_id,
                    role.created_at,
                    role.updated_at
                )
                .execute(pool)
                .await?;
            }
            EnterpriseDb::Sqlite(pool) => {
                let permissions_json = serde_json::to_string(&role.permissions)?;
                sqlx::query!(
                    r#"INSERT OR IGNORE INTO roles (id, name, description, permissions, is_system, organization_id, created_at, updated_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                    role.id,
                    role.name,
                    role.description,
                    permissions_json,
                    role.is_system,
                    role.organization_id,
                    role.created_at,
                    role.updated_at
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Get role by ID
    pub async fn get_role(&self, id: Uuid) -> Result<Option<Role>> {
        match self {
            EnterpriseDb::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT * FROM roles WHERE id = $1",
                    id
                )
                .fetch_optional(pool)
                .await?;
                
                if let Some(row) = row {
                    let permissions: Vec<Permission> = serde_json::from_value(row.permissions)?;
                    Ok(Some(Role {
                        id: row.id,
                        name: row.name,
                        description: row.description,
                        permissions,
                        is_system: row.is_system,
                        organization_id: row.organization_id,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    }))
                } else {
                    Ok(None)
                }
            }
            EnterpriseDb::Sqlite(pool) => {
                let row = sqlx::query!(
                    "SELECT * FROM roles WHERE id = ?1",
                    id
                )
                .fetch_optional(pool)
                .await?;
                
                if let Some(row) = row {
                    let permissions: Vec<Permission> = serde_json::from_str(&row.permissions)?;
                    Ok(Some(Role {
                        id: row.id,
                        name: row.name,
                        description: row.description.unwrap_or_default(),
                        permissions,
                        is_system: row.is_system,
                        organization_id: row.organization_id,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// Create organization
    pub async fn create_organization(&self, org: &Organization) -> Result<()> {
        match self {
            EnterpriseDb::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO organizations (id, name, slug, settings, subscription_tier, subscription_expires_at, is_active, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                       ON CONFLICT (id) DO NOTHING"#,
                    org.id,
                    org.name,
                    org.slug,
                    serde_json::to_value(&org.settings)?,
                    org.subscription_tier as i32,
                    org.subscription_expires_at,
                    org.is_active,
                    org.created_at,
                    org.updated_at
                )
                .execute(pool)
                .await?;
            }
            EnterpriseDb::Sqlite(pool) => {
                let settings_json = serde_json::to_string(&org.settings)?;
                sqlx::query!(
                    r#"INSERT OR IGNORE INTO organizations (id, name, slug, settings, subscription_tier, subscription_expires_at, is_active, created_at, updated_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                    org.id,
                    org.name,
                    org.slug,
                    settings_json,
                    org.subscription_tier,
                    org.subscription_expires_at,
                    org.is_active,
                    org.created_at,
                    org.updated_at
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Create audit log
    pub async fn create_audit_log(&self, log: &AuditLog) -> Result<()> {
        match self {
            EnterpriseDb::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, metadata, ip_address, user_agent, created_at)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
                    log.id,
                    log.organization_id,
                    log.user_id,
                    log.action.to_string(),
                    log.resource_type,
                    log.resource_id,
                    serde_json::to_value(&log.metadata)?,
                    log.ip_address,
                    log.user_agent,
                    log.created_at
                )
                .execute(pool)
                .await?;
            }
            EnterpriseDb::Sqlite(pool) => {
                let metadata_json = serde_json::to_string(&log.metadata)?;
                sqlx::query!(
                    r#"INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, metadata, ip_address, user_agent, created_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
                    log.id,
                    log.organization_id,
                    log.user_id,
                    log.action.to_string(),
                    log.resource_type,
                    log.resource_id,
                    metadata_json,
                    log.ip_address,
                    log.user_agent,
                    log.created_at
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}