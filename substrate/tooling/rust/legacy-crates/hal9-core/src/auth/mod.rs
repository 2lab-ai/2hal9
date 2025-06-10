//! Authentication and authorization module for HAL9

pub mod api_key;
pub mod jwt;
pub mod types;
pub mod user;

pub use api_key::{ApiKey, ApiKeyInfo, ApiKeyManager, ApiKeyResponse, CreateApiKeyRequest};
pub use jwt::{JwtClaims, JwtManager, TokenPair};
pub use types::{AuthError, AuthResult, Permission, Permissions};
pub use user::{CreateUserRequest, UpdateUserRequest, User, UserManager, UserRole};
