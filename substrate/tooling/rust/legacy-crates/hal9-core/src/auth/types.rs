//! Authentication types and errors

use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("API key not found")]
    ApiKeyNotFound,
    
    #[error("API key expired")]
    ApiKeyExpired,
    
    #[error("Password hash error: {0}")]
    PasswordHashError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("JWT error: {0}")]
    JwtError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// User permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // Neuron permissions
    CreateNeuron,
    DeleteNeuron,
    ViewNeuron,
    ModifyNeuron,
    
    // Signal permissions
    SendSignal,
    ViewSignals,
    
    // Memory permissions
    ViewMemory,
    ModifyMemory,
    
    // System permissions
    ViewMetrics,
    ManageUsers,
    ManageApiKeys,
    SystemAdmin,
    
    // Cost permissions
    ViewCosts,
    SetCostLimits,
}

/// Permission set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    permissions: HashSet<Permission>,
}

impl Permissions {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }
    
    pub fn with_permissions(perms: Vec<Permission>) -> Self {
        Self {
            permissions: perms.into_iter().collect(),
        }
    }
    
    pub fn add(&mut self, perm: Permission) {
        self.permissions.insert(perm);
    }
    
    pub fn remove(&mut self, perm: &Permission) {
        self.permissions.remove(perm);
    }
    
    pub fn has(&self, perm: &Permission) -> bool {
        self.permissions.contains(perm)
    }
    
    pub fn has_all(&self, perms: &[Permission]) -> bool {
        perms.iter().all(|p| self.has(p))
    }
    
    pub fn has_any(&self, perms: &[Permission]) -> bool {
        perms.iter().any(|p| self.has(p))
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new()
    }
}