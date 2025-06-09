//! Team management within organizations

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};
use uuid::Uuid;
use std::collections::HashMap;

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: TeamPermissions,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Team permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPermissions {
    /// Neuron access
    pub allowed_neurons: Vec<String>,
    pub neuron_creation: bool,
    pub neuron_deletion: bool,
    pub neuron_modification: bool,
    
    /// API access
    pub api_rate_limit: Option<u32>,
    pub allowed_endpoints: Vec<String>,
    pub blocked_endpoints: Vec<String>,
    
    /// Resource limits
    pub max_concurrent_signals: Option<u32>,
    pub max_memory_gb: Option<u32>,
    pub max_cpu_cores: Option<u32>,
    
    /// Feature access
    pub enabled_features: Vec<String>,
    pub allow_browser_automation: bool,
    pub allow_code_generation: bool,
    pub allow_external_tools: bool,
}

impl Default for TeamPermissions {
    fn default() -> Self {
        Self {
            allowed_neurons: vec!["*".to_string()],
            neuron_creation: true,
            neuron_deletion: false,
            neuron_modification: true,
            api_rate_limit: Some(1000),
            allowed_endpoints: vec!["*".to_string()],
            blocked_endpoints: vec![],
            max_concurrent_signals: Some(100),
            max_memory_gb: Some(8),
            max_cpu_cores: Some(4),
            enabled_features: vec![],
            allow_browser_automation: false,
            allow_code_generation: true,
            allow_external_tools: false,
        }
    }
}

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

/// Team roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    ReadOnly,
}

impl TeamRole {
    /// Check if role can perform action
    pub fn can_perform(&self, action: TeamAction) -> bool {
        match self {
            Self::Owner => true,
            Self::Admin => !matches!(action, TeamAction::DeleteTeam | TeamAction::TransferOwnership),
            Self::Member => matches!(
                action,
                TeamAction::ViewTeam | TeamAction::UseResources | TeamAction::InviteMembers
            ),
            Self::ReadOnly => matches!(action, TeamAction::ViewTeam),
        }
    }
}

/// Team actions
#[derive(Debug, Clone, Copy)]
pub enum TeamAction {
    ViewTeam,
    EditTeam,
    DeleteTeam,
    InviteMembers,
    RemoveMembers,
    EditPermissions,
    UseResources,
    TransferOwnership,
}

/// Team manager
pub struct TeamManager {
    pool: TeamPool,
}

enum TeamPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl TeamManager {
    /// Create new team manager with PostgreSQL
    pub fn new_postgres(pool: PgPool) -> Self {
        Self {
            pool: TeamPool::Postgres(pool),
        }
    }
    
    /// Create new team manager with SQLite
    pub fn new_sqlite(pool: SqlitePool) -> Self {
        Self {
            pool: TeamPool::Sqlite(pool),
        }
    }
    
    /// Create new team
    pub async fn create_team(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
        owner_id: Uuid,
    ) -> Result<Team> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let team = Team {
            id,
            organization_id,
            name: name.clone(),
            description,
            permissions: TeamPermissions::default(),
            created_at: now,
            updated_at: now,
        };
        
        match &self.pool {
            TeamPool::Postgres(pool) => {
                // Start transaction
                let mut tx = pool.begin().await?;
                
                // Insert team
                sqlx::query!(
                    r#"
                    INSERT INTO teams (id, organization_id, name, description, permissions, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    team.id,
                    team.organization_id,
                    team.name,
                    team.description,
                    serde_json::to_value(&team.permissions)?,
                    team.created_at,
                    team.updated_at
                )
                .execute(&mut *tx)
                .await?;
                
                // Add owner as first member
                sqlx::query!(
                    r#"
                    INSERT INTO team_members (user_id, team_id, role, joined_at)
                    VALUES ($1, $2, $3, $4)
                    "#,
                    owner_id,
                    team.id,
                    "owner",
                    now
                )
                .execute(&mut *tx)
                .await?;
                
                tx.commit().await?;
            }
            TeamPool::Sqlite(pool) => {
                // SQLite transaction
                let mut tx = pool.begin().await?;
                
                sqlx::query!(
                    r#"
                    INSERT INTO teams (id, organization_id, name, description, permissions, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                    team.id.to_string(),
                    team.organization_id.to_string(),
                    team.name,
                    team.description,
                    serde_json::to_string(&team.permissions)?,
                    team.created_at.timestamp(),
                    team.updated_at.timestamp()
                )
                .execute(&mut *tx)
                .await?;
                
                sqlx::query!(
                    r#"
                    INSERT INTO team_members (user_id, team_id, role, joined_at)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    owner_id.to_string(),
                    team.id.to_string(),
                    "owner",
                    now.timestamp()
                )
                .execute(&mut *tx)
                .await?;
                
                tx.commit().await?;
            }
        }
        
        Ok(team)
    }
    
    /// Get team by ID
    pub async fn get_team(&self, id: Uuid) -> Result<Option<Team>> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT * FROM teams WHERE id = $1",
                    id
                )
                .fetch_optional(pool)
                .await?;
                
                match row {
                    Some(r) => Ok(Some(Team {
                        id: r.id,
                        organization_id: r.organization_id,
                        name: r.name,
                        description: r.description,
                        permissions: serde_json::from_value(r.permissions)?,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    })),
                    None => Ok(None),
                }
            }
            TeamPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(None)
            }
        }
    }
    
    /// Get teams for organization
    pub async fn get_organization_teams(&self, org_id: Uuid) -> Result<Vec<Team>> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let rows = sqlx::query!(
                    "SELECT * FROM teams WHERE organization_id = $1 ORDER BY name",
                    org_id
                )
                .fetch_all(pool)
                .await?;
                
                let teams = rows
                    .into_iter()
                    .map(|r| {
                        Ok(Team {
                            id: r.id,
                            organization_id: r.organization_id,
                            name: r.name,
                            description: r.description,
                            permissions: serde_json::from_value(r.permissions)?,
                            created_at: r.created_at,
                            updated_at: r.updated_at,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(teams)
            }
            TeamPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(vec![])
            }
        }
    }
    
    /// Get user's teams
    pub async fn get_user_teams(&self, user_id: Uuid) -> Result<Vec<(Team, TeamRole)>> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let rows = sqlx::query!(
                    r#"
                    SELECT t.*, tm.role
                    FROM teams t
                    JOIN team_members tm ON t.id = tm.team_id
                    WHERE tm.user_id = $1
                    ORDER BY t.name
                    "#,
                    user_id
                )
                .fetch_all(pool)
                .await?;
                
                let teams = rows
                    .into_iter()
                    .map(|r| {
                        let team = Team {
                            id: r.id,
                            organization_id: r.organization_id,
                            name: r.name,
                            description: r.description,
                            permissions: serde_json::from_value(r.permissions)?,
                            created_at: r.created_at,
                            updated_at: r.updated_at,
                        };
                        let role: TeamRole = serde_json::from_str(&format!("\"{}\"", r.role))?;
                        Ok((team, role))
                    })
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(teams)
            }
            TeamPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(vec![])
            }
        }
    }
    
    /// Add member to team
    pub async fn add_member(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        role: TeamRole,
    ) -> Result<()> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    INSERT INTO team_members (user_id, team_id, role, joined_at)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (user_id, team_id) 
                    DO UPDATE SET role = EXCLUDED.role
                    "#,
                    user_id,
                    team_id,
                    serde_json::to_string(&role)?.trim_matches('"'),
                    Utc::now()
                )
                .execute(pool)
                .await?;
            }
            TeamPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    INSERT OR REPLACE INTO team_members (user_id, team_id, role, joined_at)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    user_id.to_string(),
                    team_id.to_string(),
                    serde_json::to_string(&role)?.trim_matches('"'),
                    Utc::now().timestamp()
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Remove member from team
    pub async fn remove_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                sqlx::query!(
                    "DELETE FROM team_members WHERE team_id = $1 AND user_id = $2",
                    team_id,
                    user_id
                )
                .execute(pool)
                .await?;
            }
            TeamPool::Sqlite(pool) => {
                sqlx::query!(
                    "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
                    team_id.to_string(),
                    user_id.to_string()
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Get team members
    pub async fn get_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let rows = sqlx::query!(
                    "SELECT * FROM team_members WHERE team_id = $1",
                    team_id
                )
                .fetch_all(pool)
                .await?;
                
                let members = rows
                    .into_iter()
                    .map(|r| {
                        Ok(TeamMember {
                            user_id: r.user_id,
                            team_id: r.team_id,
                            role: serde_json::from_str(&format!("\"{}\"", r.role))?,
                            joined_at: r.joined_at,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(members)
            }
            TeamPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(vec![])
            }
        }
    }
    
    /// Check if user has permission in team
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        action: TeamAction,
    ) -> Result<bool> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let row = sqlx::query!(
                    "SELECT role FROM team_members WHERE user_id = $1 AND team_id = $2",
                    user_id,
                    team_id
                )
                .fetch_optional(pool)
                .await?;
                
                match row {
                    Some(r) => {
                        let role: TeamRole = serde_json::from_str(&format!("\"{}\"", r.role))?;
                        Ok(role.can_perform(action))
                    }
                    None => Ok(false),
                }
            }
            TeamPool::Sqlite(_pool) => {
                // SQLite implementation
                Ok(false)
            }
        }
    }
    
    /// Update team
    pub async fn update_team(&self, team: &Team) -> Result<()> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                sqlx::query!(
                    r#"
                    UPDATE teams 
                    SET name = $2, description = $3, permissions = $4, updated_at = $5
                    WHERE id = $1
                    "#,
                    team.id,
                    team.name,
                    team.description,
                    serde_json::to_value(&team.permissions)?,
                    Utc::now()
                )
                .execute(pool)
                .await?;
            }
            TeamPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"
                    UPDATE teams 
                    SET name = ?2, description = ?3, permissions = ?4, updated_at = ?5
                    WHERE id = ?1
                    "#,
                    team.id.to_string(),
                    team.name,
                    team.description,
                    serde_json::to_string(&team.permissions)?,
                    Utc::now().timestamp()
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    /// Delete team
    pub async fn delete_team(&self, team_id: Uuid) -> Result<()> {
        match &self.pool {
            TeamPool::Postgres(pool) => {
                let mut tx = pool.begin().await?;
                
                // Delete team members first
                sqlx::query!("DELETE FROM team_members WHERE team_id = $1", team_id)
                    .execute(&mut *tx)
                    .await?;
                
                // Delete team
                sqlx::query!("DELETE FROM teams WHERE id = $1", team_id)
                    .execute(&mut *tx)
                    .await?;
                
                tx.commit().await?;
            }
            TeamPool::Sqlite(pool) => {
                let mut tx = pool.begin().await?;
                
                sqlx::query!("DELETE FROM team_members WHERE team_id = ?1", team_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                
                sqlx::query!("DELETE FROM teams WHERE id = ?1", team_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                
                tx.commit().await?;
            }
        }
        Ok(())
    }
}

/// Team statistics
#[derive(Debug, Clone, Serialize)]
pub struct TeamStats {
    pub member_count: u32,
    pub neuron_count: u32,
    pub api_calls_today: u64,
    pub resource_usage: ResourceUsage,
}

/// Resource usage
#[derive(Debug, Clone, Serialize)]
pub struct ResourceUsage {
    pub cpu_cores: f32,
    pub memory_gb: f32,
    pub storage_gb: f32,
    pub concurrent_signals: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_role_permissions() {
        assert!(TeamRole::Owner.can_perform(TeamAction::DeleteTeam));
        assert!(TeamRole::Admin.can_perform(TeamAction::EditTeam));
        assert!(!TeamRole::Admin.can_perform(TeamAction::DeleteTeam));
        assert!(TeamRole::Member.can_perform(TeamAction::UseResources));
        assert!(!TeamRole::Member.can_perform(TeamAction::EditPermissions));
        assert!(TeamRole::ReadOnly.can_perform(TeamAction::ViewTeam));
        assert!(!TeamRole::ReadOnly.can_perform(TeamAction::UseResources));
    }
    
    #[test]
    fn test_team_permissions_default() {
        let perms = TeamPermissions::default();
        assert!(perms.neuron_creation);
        assert!(!perms.neuron_deletion);
        assert!(!perms.allow_browser_automation);
        assert!(perms.allow_code_generation);
    }
}