//! Authentication middleware for HAL9 server

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use hal9_core::auth::{ApiKeyManager, AuthError, JwtManager, Permissions};
use std::sync::Arc;

/// Authentication state
#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
    pub api_key_manager: Arc<ApiKeyManager>,
}

/// Authenticated user info
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub permissions: Permissions,
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// Extract API key from X-API-Key header
fn extract_api_key(req: &Request) -> Option<String> {
    req.headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string())
}

/// Authentication middleware
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try JWT token first
    if let Some(token) = extract_bearer_token(&req) {
        match auth_state.jwt_manager.validate_access_token(&token) {
            Ok(claims) => {
                let user = AuthUser {
                    user_id: claims.sub.clone(),
                    username: claims.username.clone(),
                    role: claims.role.clone(),
                    permissions: get_role_permissions(&claims.role),
                };
                req.extensions_mut().insert(user);
                req.extensions_mut().insert(claims);
                return Ok(next.run(req).await);
            }
            Err(AuthError::TokenExpired) => return Err(StatusCode::UNAUTHORIZED),
            Err(_) => {} // Try API key next
        }
    }

    // Try API key
    if let Some(api_key) = extract_api_key(&req) {
        match auth_state.api_key_manager.validate_api_key(&api_key).await {
            Ok((key_info, permissions)) => {
                let user = AuthUser {
                    user_id: key_info.user_id.clone(),
                    username: format!("api_key_{}", key_info.name),
                    role: "api_key".to_string(),
                    permissions,
                };
                req.extensions_mut().insert(user);
                return Ok(next.run(req).await);
            }
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Optional authentication middleware (doesn't require auth but adds user info if available)
pub async fn optional_auth_middleware(
    State(auth_state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try JWT token
    if let Some(token) = extract_bearer_token(&req) {
        if let Ok(claims) = auth_state.jwt_manager.validate_access_token(&token) {
            let user = AuthUser {
                user_id: claims.sub.clone(),
                username: claims.username.clone(),
                role: claims.role.clone(),
                permissions: get_role_permissions(&claims.role),
            };
            req.extensions_mut().insert(user);
            req.extensions_mut().insert(claims);
        }
    } else if let Some(api_key) = extract_api_key(&req) {
        // Try API key
        if let Ok((key_info, permissions)) =
            auth_state.api_key_manager.validate_api_key(&api_key).await
        {
            let user = AuthUser {
                user_id: key_info.user_id.clone(),
                username: format!("api_key_{}", key_info.name),
                role: "api_key".to_string(),
                permissions,
            };
            req.extensions_mut().insert(user);
        }
    }

    Ok(next.run(req).await)
}

/// Permission check middleware factory
pub fn require_permission(
    permission: hal9_core::auth::Permission,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>,
> + Clone {
    move |req: Request, next: Next| {
        let permission = permission.clone();
        Box::pin(async move {
            match req.extensions().get::<AuthUser>() {
                Some(user) if user.permissions.has(&permission) => Ok(next.run(req).await),
                Some(_) => Err(StatusCode::FORBIDDEN),
                None => Err(StatusCode::UNAUTHORIZED),
            }
        })
    }
}

/// Get permissions for a role
fn get_role_permissions(role: &str) -> Permissions {
    use hal9_core::auth::UserRole;

    match role {
        "admin" => UserRole::Admin.default_permissions(),
        "user" => UserRole::User.default_permissions(),
        "guest" => UserRole::Guest.default_permissions(),
        _ => Permissions::new(),
    }
}
