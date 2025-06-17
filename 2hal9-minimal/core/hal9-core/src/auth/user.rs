//! User management module

use crate::auth::types::{AuthError, AuthResult, Permission, Permissions};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::Guest => write!(f, "guest"),
        }
    }
}

impl UserRole {
    /// Get default permissions for a role
    pub fn default_permissions(&self) -> Permissions {
        match self {
            UserRole::Admin => Permissions::with_permissions(vec![
                Permission::CreateNeuron,
                Permission::DeleteNeuron,
                Permission::ViewNeuron,
                Permission::ModifyNeuron,
                Permission::SendSignal,
                Permission::ViewSignals,
                Permission::ViewMemory,
                Permission::ModifyMemory,
                Permission::ViewMetrics,
                Permission::ManageUsers,
                Permission::ManageApiKeys,
                Permission::SystemAdmin,
                Permission::ViewCosts,
                Permission::SetCostLimits,
            ]),
            UserRole::User => Permissions::with_permissions(vec![
                Permission::CreateNeuron,
                Permission::ViewNeuron,
                Permission::ModifyNeuron,
                Permission::SendSignal,
                Permission::ViewSignals,
                Permission::ViewMemory,
                Permission::ViewMetrics,
                Permission::ViewCosts,
            ]),
            UserRole::Guest => Permissions::with_permissions(vec![
                Permission::ViewNeuron,
                Permission::ViewSignals,
                Permission::ViewMetrics,
            ]),
        }
    }
    
    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.default_permissions().has(&permission)
    }
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (UserRole::Admin, UserRole::Admin) => std::cmp::Ordering::Equal,
            (UserRole::Admin, _) => std::cmp::Ordering::Greater,
            (_, UserRole::Admin) => std::cmp::Ordering::Less,
            (UserRole::User, UserRole::User) => std::cmp::Ordering::Equal,
            (UserRole::User, UserRole::Guest) => std::cmp::Ordering::Greater,
            (UserRole::Guest, UserRole::User) => std::cmp::Ordering::Less,
            (UserRole::Guest, UserRole::Guest) => std::cmp::Ordering::Equal,
        }
    }
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_active: bool,
}

/// User creation request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
}

/// User update request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

/// User manager for database operations
pub struct UserManager {
    pool: SqlitePool,
}

impl UserManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize user tables
    pub async fn initialize(&self) -> AuthResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_active BOOLEAN DEFAULT TRUE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> AuthResult<User> {
        // Validate input
        if request.username.is_empty() || request.email.is_empty() {
            return Err(AuthError::ValidationError(
                "Username and email are required".to_string(),
            ));
        }

        if request.password.len() < 8 {
            return Err(AuthError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        // Hash password
        let password_hash = hash(request.password.as_bytes(), DEFAULT_COST)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: request.username,
            email: request.email,
            password_hash,
            role: request.role.unwrap_or(UserRole::User).to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            is_active: true,
        };

        // Insert user
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                AuthError::UserAlreadyExists
            } else {
                AuthError::DatabaseError(e.to_string())
            }
        })?;

        Ok(user)
    }

    /// Authenticate user with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> AuthResult<User> {
        let user = self.get_user_by_username(username).await?;

        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // Verify password
        if !verify(password.as_bytes(), &user.password_hash)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?
        {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> AuthResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::UserNotFound)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> AuthResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::UserNotFound)
    }

    /// Update user
    pub async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> AuthResult<User> {
        let mut user = self.get_user(user_id).await?;

        if let Some(email) = request.email {
            user.email = email;
        }

        if let Some(role) = request.role {
            user.role = role.to_string();
        }

        if let Some(is_active) = request.is_active {
            user.is_active = is_active;
        }

        user.updated_at = Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE users 
            SET email = $2, role = $3, is_active = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.role)
        .bind(user.is_active)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// Delete user
    pub async fn delete_user(&self, user_id: &str) -> AuthResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// List all users
    pub async fn list_users(&self) -> AuthResult<Vec<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }
}
