//! Role-Based Access Control (RBAC) for enterprise deployments

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool, AnyPool};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

use super::database_runtime::{EnterpriseDatabase, DatabaseType};

/// Role entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Permission entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<Action>,
    pub conditions: Option<Vec<Condition>>,
}

/// Actions that can be performed on resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Admin,
    Custom(String),
}

/// Condition for permission evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches,
}

/// Role assignment to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub scope: AssignmentScope,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Scope of role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentScope {
    Global,
    Organization(Uuid),
    Team(Uuid),
    Resource { type_name: String, id: String },
}

/// Permission check request
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    pub user_id: Uuid,
    pub resource: String,
    pub action: Action,
    pub context: HashMap<String, serde_json::Value>,
}

/// Permission check result
#[derive(Debug, Clone)]
pub struct PermissionResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub applied_rules: Vec<String>,
}

/// RBAC manager
pub struct RbacManager {
    db: EnterpriseDatabase,
    cache: PermissionCache,
}

/// Permission cache for performance
struct PermissionCache {
    user_permissions: HashMap<Uuid, CachedPermissions>,
    ttl: chrono::Duration,
}

#[derive(Clone)]
struct CachedPermissions {
    permissions: HashSet<(String, Action)>,
    roles: Vec<Role>,
    cached_at: DateTime<Utc>,
}

impl RbacManager {
    /// Create new RBAC manager with PostgreSQL
    pub fn new_postgres(pool: PgPool) -> Self {
        let any_pool = AnyPool::from(pool);
        Self {
            db: EnterpriseDatabase::new(any_pool, DatabaseType::Postgres),
            cache: PermissionCache::new(chrono::Duration::minutes(5)),
        }
    }
    
    /// Create new RBAC manager with SQLite
    pub fn new_sqlite(pool: SqlitePool) -> Self {
        let any_pool = AnyPool::from(pool);
        Self {
            db: EnterpriseDatabase::new(any_pool, DatabaseType::Sqlite),
            cache: PermissionCache::new(chrono::Duration::minutes(5)),
        }
    }
    
    /// Create system roles
    pub async fn initialize_system_roles(&self) -> Result<()> {
        let system_roles = vec![
            Role {
                id: Uuid::new_v4(),
                name: "super_admin".to_string(),
                description: "Full system access".to_string(),
                permissions: vec![
                    Permission {
                        resource: "*".to_string(),
                        actions: vec![Action::Admin],
                        conditions: None,
                    }
                ],
                is_system: true,
                organization_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: Uuid::new_v4(),
                name: "org_admin".to_string(),
                description: "Organization administrator".to_string(),
                permissions: vec![
                    Permission {
                        resource: "organization:*".to_string(),
                        actions: vec![Action::Admin],
                        conditions: Some(vec![
                            Condition {
                                field: "organization_id".to_string(),
                                operator: ConditionOperator::Equals,
                                value: serde_json::json!("{{user.organization_id}}"),
                            }
                        ]),
                    }
                ],
                is_system: true,
                organization_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: Uuid::new_v4(),
                name: "developer".to_string(),
                description: "Developer access".to_string(),
                permissions: vec![
                    Permission {
                        resource: "neuron:*".to_string(),
                        actions: vec![Action::Create, Action::Read, Action::Update, Action::Execute],
                        conditions: None,
                    },
                    Permission {
                        resource: "signal:*".to_string(),
                        actions: vec![Action::Create, Action::Read, Action::Execute],
                        conditions: None,
                    },
                ],
                is_system: true,
                organization_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: Uuid::new_v4(),
                name: "operator".to_string(),
                description: "System operator".to_string(),
                permissions: vec![
                    Permission {
                        resource: "neuron:*".to_string(),
                        actions: vec![Action::Read, Action::Execute],
                        conditions: None,
                    },
                    Permission {
                        resource: "metrics:*".to_string(),
                        actions: vec![Action::Read],
                        conditions: None,
                    },
                ],
                is_system: true,
                organization_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: Uuid::new_v4(),
                name: "viewer".to_string(),
                description: "Read-only access".to_string(),
                permissions: vec![
                    Permission {
                        resource: "*".to_string(),
                        actions: vec![Action::Read],
                        conditions: None,
                    }
                ],
                is_system: true,
                organization_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];
        
        for role in system_roles {
            self.create_role(role).await?;
        }
        
        Ok(())
    }
    
    /// Create new role
    pub async fn create_role(&self, role: Role) -> Result<()> {
        self.db.create_role(&role).await
    }
    
    /// Get role by ID
    pub async fn get_role(&self, id: Uuid) -> Result<Option<Role>> {
        self.db.get_role(id).await
    }
    
    /// Assign role to user
    pub async fn assign_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
        scope: AssignmentScope,
        granted_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let assignment = RoleAssignment {
            user_id,
            role_id,
            scope: scope.clone(),
            granted_by,
            granted_at: Utc::now(),
            expires_at,
        };
        
        self.db.create_role_assignment(&assignment).await?;
        
        // Invalidate cache
        self.cache.invalidate_user(user_id);
        
        Ok(())
    }
    
    /// Remove role from user
    pub async fn remove_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        self.db.delete_role_assignment(user_id, role_id).await?;
        
        // Invalidate cache
        self.cache.invalidate_user(user_id);
        
        Ok(())
    }
    
    /// Get user's roles
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>> {
        let role_assignments = self.db.get_user_roles(user_id).await?;
        Ok(role_assignments.into_iter().map(|(role, _)| role).collect())
    }
    
    /// Check if user has permission
    pub async fn check_permission(&self, check: PermissionCheck) -> Result<PermissionResult> {
        // Check cache first
        if let Some(result) = self.cache.check_permission(&check) {
            return Ok(result);
        }
        
        // Get user's roles
        let roles = self.get_user_roles(check.user_id).await?;
        
        let mut allowed = false;
        let mut applied_rules = Vec::new();
        let mut reason = None;
        
        // Check each role's permissions
        for role in &roles {
            for permission in &role.permissions {
                if self.matches_resource(&permission.resource, &check.resource) {
                    if permission.actions.contains(&check.action) || permission.actions.contains(&Action::Admin) {
                        // Check conditions
                        if let Some(conditions) = &permission.conditions {
                            if self.evaluate_conditions(conditions, &check.context)? {
                                allowed = true;
                                applied_rules.push(format!("{}:{}", role.name, permission.resource));
                            }
                        } else {
                            allowed = true;
                            applied_rules.push(format!("{}:{}", role.name, permission.resource));
                        }
                    }
                }
            }
        }
        
        if !allowed {
            reason = Some(format!("No permission for action {:?} on resource {}", check.action, check.resource));
        }
        
        let result = PermissionResult {
            allowed,
            reason,
            applied_rules,
        };
        
        // Cache result
        self.cache.cache_result(&check, &result);
        
        Ok(result)
    }
    
    /// Check if resource pattern matches
    fn matches_resource(&self, pattern: &str, resource: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        let pattern_parts: Vec<&str> = pattern.split(':').collect();
        let resource_parts: Vec<&str> = resource.split(':').collect();
        
        if pattern_parts.len() != resource_parts.len() {
            return false;
        }
        
        for (pattern_part, resource_part) in pattern_parts.iter().zip(resource_parts.iter()) {
            if *pattern_part != "*" && pattern_part != resource_part {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate conditions
    fn evaluate_conditions(&self, conditions: &[Condition], context: &HashMap<String, serde_json::Value>) -> Result<bool> {
        for condition in conditions {
            let field_value = context.get(&condition.field);
            
            let matches = match &condition.operator {
                ConditionOperator::Equals => {
                    field_value == Some(&condition.value)
                }
                ConditionOperator::NotEquals => {
                    field_value != Some(&condition.value)
                }
                ConditionOperator::In => {
                    if let (Some(value), Some(array)) = (field_value, condition.value.as_array()) {
                        array.contains(value)
                    } else {
                        false
                    }
                }
                ConditionOperator::NotIn => {
                    if let (Some(value), Some(array)) = (field_value, condition.value.as_array()) {
                        !array.contains(value)
                    } else {
                        true
                    }
                }
                ConditionOperator::GreaterThan => {
                    if let (Some(value), Some(threshold)) = (field_value.and_then(|v| v.as_f64()), condition.value.as_f64()) {
                        value > threshold
                    } else {
                        false
                    }
                }
                ConditionOperator::LessThan => {
                    if let (Some(value), Some(threshold)) = (field_value.and_then(|v| v.as_f64()), condition.value.as_f64()) {
                        value < threshold
                    } else {
                        false
                    }
                }
                ConditionOperator::GreaterThanOrEquals => {
                    if let (Some(value), Some(threshold)) = (field_value.and_then(|v| v.as_f64()), condition.value.as_f64()) {
                        value >= threshold
                    } else {
                        false
                    }
                }
                ConditionOperator::LessThanOrEquals => {
                    if let (Some(value), Some(threshold)) = (field_value.and_then(|v| v.as_f64()), condition.value.as_f64()) {
                        value <= threshold
                    } else {
                        false
                    }
                }
                ConditionOperator::Contains => {
                    if let (Some(value), Some(substring)) = (field_value.and_then(|v| v.as_str()), condition.value.as_str()) {
                        value.contains(substring)
                    } else {
                        false
                    }
                }
                ConditionOperator::NotContains => {
                    if let (Some(value), Some(substring)) = (field_value.and_then(|v| v.as_str()), condition.value.as_str()) {
                        !value.contains(substring)
                    } else {
                        true
                    }
                }
                ConditionOperator::StartsWith => {
                    if let (Some(value), Some(prefix)) = (field_value.and_then(|v| v.as_str()), condition.value.as_str()) {
                        value.starts_with(prefix)
                    } else {
                        false
                    }
                }
                ConditionOperator::EndsWith => {
                    if let (Some(value), Some(suffix)) = (field_value.and_then(|v| v.as_str()), condition.value.as_str()) {
                        value.ends_with(suffix)
                    } else {
                        false
                    }
                }
                ConditionOperator::Matches => {
                    if let (Some(value), Some(pattern)) = (field_value.and_then(|v| v.as_str()), condition.value.as_str()) {
                        regex::Regex::new(pattern)?.is_match(value)
                    } else {
                        false
                    }
                }
            };
            
            if !matches {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Create custom role
    pub async fn create_custom_role(
        &self,
        name: String,
        description: String,
        permissions: Vec<Permission>,
        organization_id: Option<Uuid>,
    ) -> Result<Role> {
        let role = Role {
            id: Uuid::new_v4(),
            name,
            description,
            permissions,
            is_system: false,
            organization_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.create_role(role.clone()).await?;
        Ok(role)
    }
    
    /// Update role
    pub async fn update_role(&self, role: &Role) -> Result<()> {
        // TODO: Add update_role method to EnterpriseDatabase
        // For now, we recreate the role with the same ID
        self.db.create_role(role).await?;
        
        // Invalidate cache for all users with this role
        self.cache.invalidate_role(role.id);
        
        Ok(())
    }
    
    /// Delete custom role
    pub async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        self.db.delete_role(role_id).await?;
        
        // Invalidate cache
        self.cache.invalidate_role(role_id);
        
        Ok(())
    }
}

impl PermissionCache {
    fn new(ttl: chrono::Duration) -> Self {
        Self {
            user_permissions: HashMap::new(),
            ttl,
        }
    }
    
    fn check_permission(&self, check: &PermissionCheck) -> Option<PermissionResult> {
        // Simple cache implementation - in production would use proper caching
        None
    }
    
    fn cache_result(&mut self, _check: &PermissionCheck, _result: &PermissionResult) {
        // Cache implementation
    }
    
    fn invalidate_user(&mut self, user_id: Uuid) {
        self.user_permissions.remove(&user_id);
    }
    
    fn invalidate_role(&mut self, _role_id: Uuid) {
        // In production, would track which users have which roles
        self.user_permissions.clear();
    }
}

/// Policy builder for creating complex permissions
pub struct PolicyBuilder {
    permissions: Vec<Permission>,
}

impl PolicyBuilder {
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
        }
    }
    
    /// Allow actions on resource
    pub fn allow(mut self, resource: &str, actions: Vec<Action>) -> Self {
        self.permissions.push(Permission {
            resource: resource.to_string(),
            actions,
            conditions: None,
        });
        self
    }
    
    /// Allow actions on resource with conditions
    pub fn allow_if(mut self, resource: &str, actions: Vec<Action>, conditions: Vec<Condition>) -> Self {
        self.permissions.push(Permission {
            resource: resource.to_string(),
            actions,
            conditions: Some(conditions),
        });
        self
    }
    
    /// Build permissions
    pub fn build(self) -> Vec<Permission> {
        self.permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_matching() {
        let manager = RbacManager::new_sqlite(SqlitePool::connect("sqlite::memory:").await.unwrap());
        
        assert!(manager.matches_resource("*", "anything"));
        assert!(manager.matches_resource("neuron:*", "neuron:123"));
        assert!(manager.matches_resource("neuron:123", "neuron:123"));
        assert!(!manager.matches_resource("neuron:123", "neuron:456"));
        assert!(!manager.matches_resource("neuron:*", "signal:123"));
    }
    
    #[test]
    fn test_policy_builder() {
        let permissions = PolicyBuilder::new()
            .allow("neuron:*", vec![Action::Read, Action::Execute])
            .allow_if(
                "signal:*",
                vec![Action::Create],
                vec![Condition {
                    field: "team_id".to_string(),
                    operator: ConditionOperator::Equals,
                    value: serde_json::json!("team123"),
                }]
            )
            .build();
        
        assert_eq!(permissions.len(), 2);
        assert_eq!(permissions[0].resource, "neuron:*");
        assert_eq!(permissions[0].actions.len(), 2);
        assert!(permissions[1].conditions.is_some());
    }
    
    #[test]
    fn test_condition_evaluation() {
        let manager = RbacManager::new_sqlite(SqlitePool::connect("sqlite::memory:").await.unwrap());
        
        let mut context = HashMap::new();
        context.insert("user_id".to_string(), serde_json::json!("user123"));
        context.insert("team_id".to_string(), serde_json::json!("team456"));
        context.insert("score".to_string(), serde_json::json!(75));
        
        let conditions = vec![
            Condition {
                field: "user_id".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::json!("user123"),
            },
            Condition {
                field: "score".to_string(),
                operator: ConditionOperator::GreaterThan,
                value: serde_json::json!(50),
            },
        ];
        
        assert!(manager.evaluate_conditions(&conditions, &context).unwrap());
    }
}