//! Authentication API endpoints

use axum::{
    extract::{State, Json, Path, Extension},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use hal9_core::auth::{
    User, UserManager, CreateUserRequest, UpdateUserRequest,
    JwtManager, TokenPair,
    ApiKeyManager, CreateApiKeyRequest, ApiKeyResponse, ApiKeyInfo,
    AuthError,
};
use crate::auth_middleware::AuthUser;

/// Authentication API state
pub struct AuthApiState {
    pub user_manager: Arc<UserManager>,
    pub jwt_manager: Arc<JwtManager>,
    pub api_key_manager: Arc<ApiKeyManager>,
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
    pub user: UserResponse,
    pub tokens: TokenPair,
}

/// User response (without sensitive data)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: i64,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
            is_active: user.is_active,
        }
    }
}

/// Register new user
pub async fn register(
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AuthErrorResponse> {
    let user = state.user_manager.create_user(request).await?;
    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Login user
pub async fn login(
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthErrorResponse> {
    let user = state.user_manager
        .authenticate(&request.username, &request.password)
        .await?;
    
    let tokens = state.jwt_manager
        .generate_token_pair(&user.id, &user.username, &user.role)?;
    
    Ok(Json(LoginResponse {
        user: user.into(),
        tokens,
    }))
}

/// Refresh token
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

pub async fn refresh_token(
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AuthErrorResponse> {
    let access_token = state.jwt_manager
        .refresh_access_token(&request.refresh_token)?;
    
    Ok(Json(RefreshResponse {
        access_token,
        expires_in: 900, // 15 minutes
    }))
}

/// Get current user profile
pub async fn get_profile(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Json<UserResponse>, AuthErrorResponse> {
    let user_data = state.user_manager.get_user(&user.user_id).await?;
    Ok(Json(user_data.into()))
}

/// Update user profile
pub async fn update_profile(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AuthErrorResponse> {
    let updated_user = state.user_manager
        .update_user(&user.user_id, request)
        .await?;
    Ok(Json(updated_user.into()))
}

/// Create API key
pub async fn create_api_key(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyResponse>), AuthErrorResponse> {
    let api_key = state.api_key_manager
        .create_api_key(&user.user_id, request)
        .await?;
    Ok((StatusCode::CREATED, Json(api_key)))
}

/// List API keys
pub async fn list_api_keys(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Json<Vec<ApiKeyInfo>>, AuthErrorResponse> {
    let keys = state.api_key_manager
        .list_user_api_keys(&user.user_id)
        .await?;
    
    let key_infos: Vec<ApiKeyInfo> = keys.into_iter()
        .map(|k| k.into())
        .collect();
    
    Ok(Json(key_infos))
}

/// Revoke API key
pub async fn revoke_api_key(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
    Path(key_id): Path<String>,
) -> Result<StatusCode, AuthErrorResponse> {
    state.api_key_manager
        .revoke_api_key(&user.user_id, &key_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Delete API key
pub async fn delete_api_key(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<AuthApiState>>,
    Path(key_id): Path<String>,
) -> Result<StatusCode, AuthErrorResponse> {
    state.api_key_manager
        .delete_api_key(&user.user_id, &key_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Error response for auth endpoints
#[derive(Debug)]
pub struct AuthErrorResponse(AuthError);

impl IntoResponse for AuthErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AuthError::ApiKeyNotFound => (StatusCode::NOT_FOUND, "API key not found"),
            AuthError::ApiKeyExpired => (StatusCode::UNAUTHORIZED, "API key expired"),
            AuthError::ValidationError(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };
        
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
            message: String,
        }
        
        let body = ErrorResponse {
            error: status.to_string(),
            message: message.to_string(),
        };
        
        (status, Json(body)).into_response()
    }
}

impl From<AuthError> for AuthErrorResponse {
    fn from(err: AuthError) -> Self {
        Self(err)
    }
}