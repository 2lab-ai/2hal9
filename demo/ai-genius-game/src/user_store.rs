//! Simple in-memory user store for AI Genius Game
//! In production, this should be replaced with a proper database (PostgreSQL)

use crate::auth::{User, Role, hash_password};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, User>>>,
    username_index: Arc<RwLock<HashMap<String, String>>>, // username -> user_id
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize with default admin user
    pub async fn init_default_admin(&self) {
        let admin_user = User {
            id: "admin".to_string(),
            username: "admin".to_string(),
            email: "admin@hal9.ai".to_string(),
            password_hash: hash_password("admin123").unwrap_or_default(),
            role: Role::Admin,
            created_at: Utc::now(),
        };
        
        self.users.write().await.insert(admin_user.id.clone(), admin_user.clone());
        self.username_index.write().await.insert(admin_user.username.clone(), admin_user.id.clone());
        
        tracing::info!("Default admin user created (username: admin, password: admin123)");
    }
    
    /// Create a new user
    pub async fn create_user(&self, username: String, email: String, password: String) -> Result<User, String> {
        // Check if username already exists
        if self.username_index.read().await.contains_key(&username) {
            return Err("Username already exists".to_string());
        }
        
        // Hash password
        let password_hash = hash_password(&password)
            .map_err(|_| "Failed to hash password")?;
        
        // Create user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email,
            password_hash,
            role: Role::Player, // Default role for new users
            created_at: Utc::now(),
        };
        
        // Store user
        self.users.write().await.insert(user.id.clone(), user.clone());
        self.username_index.write().await.insert(username, user.id.clone());
        
        Ok(user)
    }
    
    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.read().await.get(user_id).cloned()
    }
    
    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let users = self.users.read().await;
        let username_index = self.username_index.read().await;
        
        username_index.get(username)
            .and_then(|user_id| users.get(user_id))
            .cloned()
    }
    
    /// Update user role (admin only)
    pub async fn update_user_role(&self, user_id: &str, new_role: Role) -> Result<(), String> {
        let mut users = self.users.write().await;
        
        match users.get_mut(user_id) {
            Some(user) => {
                user.role = new_role;
                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
    
    /// List all users (admin only)
    pub async fn list_users(&self) -> Vec<User> {
        self.users.read().await.values().cloned().collect()
    }
    
    /// Delete user (admin only)
    pub async fn delete_user(&self, user_id: &str) -> Result<(), String> {
        let mut users = self.users.write().await;
        let mut username_index = self.username_index.write().await;
        
        match users.remove(user_id) {
            Some(user) => {
                username_index.remove(&user.username);
                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}