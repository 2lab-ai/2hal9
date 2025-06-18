//! JWT Authentication for AI Genius Game
//! 
//! This module provides secure authentication using JWT tokens
//! with refresh token support and role-based access control.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use chrono::{Duration, Utc};

/// JWT secret key - In production, this should come from environment variables
static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "hal9-secret-key-change-in-production".to_string());
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// User roles
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    Admin,
    Player,
    Guest,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Player => write!(f, "player"),
            Role::Guest => write!(f, "guest"),
        }
    }
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub username: String,
    pub role: Role,
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
}

impl Claims {
    pub fn new(user_id: String, username: String, role: Role) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username,
            role,
            iat: now.timestamp(),
            exp: (now + Duration::hours(24)).timestamp(), // 24 hour expiration
        }
    }

    /// Create a refresh token claim (7 days expiration)
    pub fn new_refresh(user_id: String, username: String, role: Role) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username,
            role,
            iat: now.timestamp(),
            exp: (now + Duration::days(7)).timestamp(),
        }
    }
}

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    TokenCreation,
    InvalidToken,
    ExpiredToken,
    MissingCredentials,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
        };
        let body = Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

/// Create JWT token
pub fn create_jwt(claims: &Claims) -> Result<String, AuthError> {
    encode(&Header::default(), claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)
}

/// Validate JWT token
pub fn validate_jwt(token: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(token, &KEYS.decoding, &Validation::default())
        .map(|data| data.claims)
        .map_err(|err| {
            eprintln!("JWT validation error: {:?}", err);
            match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
                _ => AuthError::InvalidToken,
            }
        })
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

/// User struct for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// User info (without sensitive data)
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: Role,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
        }
    }
}

/// Register request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Extractor for JWT claims
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingCredentials)?;
        
        // Decode the user data
        validate_jwt(bearer.token())
    }
}

/// Auth middleware for protected routes
pub async fn require_auth(claims: Claims) -> Result<Claims, AuthError> {
    Ok(claims)
}

/// Auth middleware for admin-only routes
pub async fn require_admin(claims: Claims) -> Result<Claims, AuthError> {
    if claims.role != Role::Admin {
        return Err(AuthError::WrongCredentials);
    }
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_creation_and_validation() {
        let claims = Claims::new("user123".to_string(), "testuser".to_string(), Role::Player);
        let token = create_jwt(&claims).unwrap();
        let decoded = validate_jwt(&token).unwrap();
        assert_eq!(decoded.sub, "user123");
        assert_eq!(decoded.username, "testuser");
        assert_eq!(decoded.role, Role::Player);
    }
}