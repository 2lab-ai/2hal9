//! Runtime database queries for enterprise features

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Any, AnyPool, Row};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use super::rbac::{Role, Permission, RoleAssignment, AssignmentScope};
use super::organization::Organization;
use super::team::Team;
use super::audit::{AuditEvent as AuditLog, AuditAction};

/// Database operations for enterprise features
pub struct EnterpriseDatabase {
    pool: AnyPool,
    db_type: DatabaseType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl EnterpriseDatabase {
    pub fn new(pool: AnyPool, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
    
    /// Create role
    pub async fn create_role(&self, role: &Role) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"INSERT INTO roles (id, name, description, permissions, is_system, organization_id, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                   ON CONFLICT (id) DO NOTHING"#
            }
            DatabaseType::Sqlite => {
                r#"INSERT OR IGNORE INTO roles (id, name, description, permissions, is_system, organization_id, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#
            }
        };
        
        let permissions_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&role.permissions)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&role.permissions)?),
        };
        
        sqlx::query(query)
            .bind(role.id.to_string())
            .bind(&role.name)
            .bind(&role.description)
            .bind(permissions_json)
            .bind(role.is_system)
            .bind(role.organization_id.map(|id| id.to_string()))
            .bind(role.created_at.timestamp())
            .bind(role.updated_at.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Get role by ID
    pub async fn get_role(&self, id: Uuid) -> Result<Option<Role>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT * FROM roles WHERE id = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT * FROM roles WHERE id = ?1"
            }
        };
        
        let row = sqlx::query(query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
            let permissions: Vec<Permission> = match self.db_type {
                DatabaseType::Postgres => {
                    let json_val: JsonValue = row.try_get("permissions")?;
                    serde_json::from_value(json_val)?
                }
                DatabaseType::Sqlite => {
                    let json_str: String = row.try_get("permissions")?;
                    serde_json::from_str(&json_str)?
                }
            };
            
            Ok(Some(Role {
                id: row.try_get::<String, _>("id")?.parse()?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                permissions,
                is_system: row.try_get("is_system")?,
                organization_id: row.try_get::<Option<String>, _>("organization_id")?
                    .map(|s| s.parse()).transpose()?,
                created_at: {
                    let timestamp: i64 = row.try_get("created_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
                updated_at: {
                    let timestamp: i64 = row.try_get("updated_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Create role assignment
    pub async fn create_role_assignment(&self, assignment: &RoleAssignment) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"INSERT INTO role_assignments (user_id, role_id, scope, granted_by, granted_at, expires_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            }
            DatabaseType::Sqlite => {
                r#"INSERT INTO role_assignments (user_id, role_id, scope, granted_by, granted_at, expires_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#
            }
        };
        
        let scope_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&assignment.scope)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&assignment.scope)?),
        };
        
        sqlx::query(query)
            .bind(assignment.user_id.to_string())
            .bind(assignment.role_id.to_string())
            .bind(scope_json)
            .bind(assignment.granted_by.to_string())
            .bind(assignment.granted_at.timestamp())
            .bind(assignment.expires_at.map(|dt| dt.timestamp()))
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Delete role assignment
    pub async fn delete_role_assignment(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "DELETE FROM role_assignments WHERE user_id = $1 AND role_id = $2"
            }
            DatabaseType::Sqlite => {
                "DELETE FROM role_assignments WHERE user_id = ?1 AND role_id = ?2"
            }
        };
        
        sqlx::query(query)
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Get user roles
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<(Role, RoleAssignment)>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"SELECT r.*, ra.* FROM roles r
                   JOIN role_assignments ra ON r.id = ra.role_id
                   WHERE ra.user_id = $1 
                   AND (ra.expires_at IS NULL OR ra.expires_at > NOW())"#
            }
            DatabaseType::Sqlite => {
                r#"SELECT r.*, ra.* FROM roles r
                   JOIN role_assignments ra ON r.id = ra.role_id
                   WHERE ra.user_id = ?1 
                   AND (ra.expires_at IS NULL OR ra.expires_at > strftime('%s', 'now'))"#
            }
        };
        
        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
        let mut results = Vec::new();
        for row in rows {
            let permissions: Vec<Permission> = match self.db_type {
                DatabaseType::Postgres => {
                    let json_val: JsonValue = row.try_get("permissions")?;
                    serde_json::from_value(json_val)?
                }
                DatabaseType::Sqlite => {
                    let json_str: String = row.try_get("permissions")?;
                    serde_json::from_str(&json_str)?
                }
            };
            
            let scope: AssignmentScope = match self.db_type {
                DatabaseType::Postgres => {
                    let json_val: JsonValue = row.try_get("scope")?;
                    serde_json::from_value(json_val)?
                }
                DatabaseType::Sqlite => {
                    let json_str: String = row.try_get("scope")?;
                    serde_json::from_str(&json_str)?
                }
            };
            
            let role = Role {
                id: row.try_get::<String, _>("id")?.parse()?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                permissions,
                is_system: row.try_get("is_system")?,
                organization_id: row.try_get::<Option<String>, _>("organization_id")?
                    .map(|s| s.parse()).transpose()?,
                created_at: {
                    let timestamp: i64 = row.try_get("created_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
                updated_at: {
                    let timestamp: i64 = row.try_get("updated_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
            };
            
            let assignment = RoleAssignment {
                user_id: row.try_get::<String, _>("user_id")?.parse()?,
                role_id: row.try_get::<String, _>("role_id")?.parse()?,
                scope,
                granted_by: row.try_get::<String, _>("granted_by")?.parse()?,
                granted_at: {
                    let timestamp: i64 = row.try_get("granted_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
                expires_at: {
                    let timestamp: Option<i64> = row.try_get("expires_at")?;
                    timestamp.and_then(|t| DateTime::from_timestamp(t, 0))
                },
            };
            
            results.push((role, assignment));
        }
        
        Ok(results)
    }
    
    /// Create organization
    pub async fn create_organization(&self, org: &Organization) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"INSERT INTO organizations (id, name, description, settings, features, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#
            }
            DatabaseType::Sqlite => {
                r#"INSERT INTO organizations (id, name, description, settings, features, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#
            }
        };
        
        let settings_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&org.settings)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&org.settings)?),
        };
        
        let features_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&org.features)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&org.features)?),
        };
        
        sqlx::query(query)
            .bind(org.id.to_string())
            .bind(&org.name)
            .bind(&org.description)
            .bind(settings_json)
            .bind(features_json)
            .bind(org.is_active)
            .bind(org.created_at.timestamp())
            .bind(org.updated_at.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Create team
    pub async fn create_team(&self, team: &Team) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"INSERT INTO teams (id, organization_id, name, description, settings, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#
            }
            DatabaseType::Sqlite => {
                r#"INSERT INTO teams (id, organization_id, name, description, settings, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#
            }
        };
        
        let settings_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&team.settings)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&team.settings)?),
        };
        
        sqlx::query(query)
            .bind(team.id.to_string())
            .bind(team.organization_id.to_string())
            .bind(&team.name)
            .bind(&team.description)
            .bind(settings_json)
            .bind(team.created_at.timestamp())
            .bind(team.updated_at.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Create audit log
    pub async fn create_audit_log(&self, log: &AuditLog) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, timestamp)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
            }
            DatabaseType::Sqlite => {
                r#"INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, timestamp)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#
            }
        };
        
        let details_json = match self.db_type {
            DatabaseType::Postgres => serde_json::to_value(&log.details)?,
            DatabaseType::Sqlite => JsonValue::String(serde_json::to_string(&log.details)?),
        };
        
        sqlx::query(query)
            .bind(log.id.to_string())
            .bind(log.organization_id.to_string())
            .bind(log.user_id.to_string())
            .bind(format!("{:?}", log.action))
            .bind(&log.resource_type)
            .bind(&log.resource_id)
            .bind(details_json)
            .bind(&log.ip_address)
            .bind(&log.user_agent)
            .bind(log.timestamp.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Delete expired role assignments
    pub async fn cleanup_expired_assignments(&self) -> Result<u64> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "DELETE FROM role_assignments WHERE expires_at < NOW()"
            }
            DatabaseType::Sqlite => {
                "DELETE FROM role_assignments WHERE expires_at < strftime('%s', 'now')"
            }
        };
        
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await?;
            
        Ok(result.rows_affected())
    }
    
    /// Delete role and its assignments
    pub async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        // Start transaction
        let mut tx = self.pool.begin().await?;
        
        // Delete assignments
        let query1 = match self.db_type {
            DatabaseType::Postgres => {
                "DELETE FROM role_assignments WHERE role_id = $1"
            }
            DatabaseType::Sqlite => {
                "DELETE FROM role_assignments WHERE role_id = ?1"
            }
        };
        
        sqlx::query(query1)
            .bind(role_id.to_string())
            .execute(&mut *tx)
            .await?;
        
        // Delete role
        let query2 = match self.db_type {
            DatabaseType::Postgres => {
                "DELETE FROM roles WHERE id = $1 AND is_system = false"
            }
            DatabaseType::Sqlite => {
                "DELETE FROM roles WHERE id = ?1 AND is_system = 0"
            }
        };
        
        sqlx::query(query2)
            .bind(role_id.to_string())
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        
        Ok(())
    }
}