//! Authentication and authorization module for HAL9

pub mod user;
pub mod jwt;
pub mod api_key;
pub mod types;

pub use user::{User, UserManager, UserRole, CreateUserRequest, UpdateUserRequest};
pub use jwt::{JwtClaims, JwtManager, TokenPair};
pub use api_key::{ApiKey, ApiKeyManager, CreateApiKeyRequest, ApiKeyResponse, ApiKeyInfo};
pub use types::{AuthError, AuthResult, Permissions, Permission};