//! Runtime database queries for multi-database support

use anyhow::Result;
use chrono::{DateTime, Utc};
use hal9_core::NeuronSignal;
use sqlx::{Any, AnyPool, Row};
use uuid::Uuid;

/// Database operations that work with both SQLite and PostgreSQL
pub struct RuntimeDatabase {
    pool: AnyPool,
    db_type: DatabaseType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl RuntimeDatabase {
    pub fn new(pool: AnyPool, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
    
    /// Insert a signal into the database
    pub async fn insert_signal(&self, signal: &NeuronSignal) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            }
            DatabaseType::Postgres => {
                "INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            }
        };
        
        sqlx::query(query)
            .bind(signal.signal_id.to_string())
            .bind(&signal.from_neuron)
            .bind(&signal.to_neuron)
            .bind(&signal.layer_from)
            .bind(&signal.layer_to)
            .bind(&signal.payload.activation.content)
            .bind(signal.timestamp.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Update neuron state
    pub async fn update_neuron_state(&self, neuron_id: Uuid, state: String) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "UPDATE neurons SET state = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2"
            }
            DatabaseType::Postgres => {
                "UPDATE neurons SET state = $1, updated_at = NOW() WHERE id = $2"
            }
        };
        
        sqlx::query(query)
            .bind(state)
            .bind(neuron_id.to_string())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Get signal by ID
    pub async fn get_signal(&self, signal_id: Uuid) -> Result<Option<SignalRecord>> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "SELECT id, content, source, target, signal_type, priority, created_at, parent_id 
                 FROM signals WHERE id = ?1"
            }
            DatabaseType::Postgres => {
                "SELECT id, content, source, target, signal_type, priority, created_at, parent_id 
                 FROM signals WHERE id = $1"
            }
        };
        
        let row = sqlx::query(query)
            .bind(signal_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
            Ok(Some(SignalRecord {
                id: row.try_get::<String, _>("id")?.parse()?,
                content: row.try_get("content")?,
                source: row.try_get("source")?,
                target: row.try_get("target")?,
                signal_type: row.try_get("signal_type")?,
                priority: row.try_get("priority")?,
                created_at: {
                    let timestamp: i64 = row.try_get("created_at")?;
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                },
                parent_id: row.try_get::<Option<String>, _>("parent_id")?
                    .map(|s| s.parse()).transpose()?,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Clean old signals
    pub async fn clean_old_signals(&self, days: i32) -> Result<u64> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "DELETE FROM signals WHERE timestamp < datetime('now', '-' || ? || ' days')"
            }
            DatabaseType::Postgres => {
                "DELETE FROM signals WHERE timestamp < NOW() - INTERVAL '$1 days'"
            }
        };
        
        let result = sqlx::query(query)
            .bind(days)
            .execute(&self.pool)
            .await?;
            
        Ok(result.rows_affected())
    }
    
    /// Insert audit log
    pub async fn insert_audit_log(&self, log: &AuditLog) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, timestamp) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            }
            DatabaseType::Postgres => {
                "INSERT INTO audit_logs (id, organization_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, timestamp) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            }
        };
        
        sqlx::query(query)
            .bind(log.id.to_string())
            .bind(log.organization_id.to_string())
            .bind(log.user_id.as_ref().map(|u| u.to_string()))
            .bind(&log.action)
            .bind(&log.resource_type)
            .bind(log.resource_id.as_ref())
            .bind(serde_json::to_string(&log.details)?)
            .bind(log.ip_address.as_ref())
            .bind(log.user_agent.as_ref())
            .bind(log.timestamp.timestamp())
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    /// Get audit logs for organization
    pub async fn get_audit_logs(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        let query = match self.db_type {
            DatabaseType::Sqlite => {
                "SELECT * FROM audit_logs WHERE organization_id = ?1 
                 ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3"
            }
            DatabaseType::Postgres => {
                "SELECT * FROM audit_logs WHERE organization_id = $1 
                 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
            }
        };
        
        let rows = sqlx::query(query)
            .bind(org_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            
        let mut logs = Vec::new();
        for row in rows {
            logs.push(AuditLog {
                id: row.try_get::<String, _>("id")?.parse()?,
                organization_id: row.try_get::<String, _>("organization_id")?.parse()?,
                user_id: row.try_get::<Option<String>, _>("user_id")?
                    .map(|s| s.parse()).transpose()?,
                action: row.try_get("action")?,
                resource_type: row.try_get("resource_type")?,
                resource_id: row.try_get("resource_id")?,
                details: serde_json::from_str(&row.try_get::<String, _>("details")?)?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                timestamp: DateTime::from_timestamp(row.try_get("timestamp")?, 0)
                    .unwrap_or_else(|| Utc::now()),
            });
        }
        
        Ok(logs)
    }
}

/// Signal record from database
#[derive(Debug, Clone)]
pub struct SignalRecord {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub target: String,
    pub signal_type: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<Uuid>,
}

/// Audit log record
#[derive(Debug, Clone)]
pub struct AuditLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_type() {
        assert_eq!(DatabaseType::Sqlite, DatabaseType::Sqlite);
        assert_ne!(DatabaseType::Sqlite, DatabaseType::Postgres);
    }
}