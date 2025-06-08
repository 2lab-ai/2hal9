//! JWT token management

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::types::{AuthError, AuthResult};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Not before
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Token type (access or refresh)
    pub token_type: String,
}

/// JWT manager for token operations
pub struct JwtManager {
    secret: String,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_token_duration: Duration::minutes(15),
            refresh_token_duration: Duration::days(7),
        }
    }
    
    /// Create with custom durations
    pub fn with_durations(
        secret: String,
        access_minutes: i64,
        refresh_days: i64,
    ) -> Self {
        Self {
            secret,
            access_token_duration: Duration::minutes(access_minutes),
            refresh_token_duration: Duration::days(refresh_days),
        }
    }
    
    /// Generate access token
    pub fn generate_access_token(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + self.access_token_duration;
        
        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };
        
        self.encode_token(&claims)
    }
    
    /// Generate refresh token
    pub fn generate_refresh_token(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + self.refresh_token_duration;
        
        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
        };
        
        self.encode_token(&claims)
    }
    
    /// Validate and decode token
    pub fn validate_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let validation = Validation::new(Algorithm::HS256);
        
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        })
    }
    
    /// Validate access token specifically
    pub fn validate_access_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let claims = self.validate_token(token)?;
        
        if claims.token_type != "access" {
            return Err(AuthError::InvalidToken);
        }
        
        Ok(claims)
    }
    
    /// Validate refresh token specifically
    pub fn validate_refresh_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let claims = self.validate_token(token)?;
        
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }
        
        Ok(claims)
    }
    
    /// Refresh access token using refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> AuthResult<String> {
        let claims = self.validate_refresh_token(refresh_token)?;
        
        self.generate_access_token(&claims.sub, &claims.username, &claims.role)
    }
    
    /// Encode token
    fn encode_token(&self, claims: &JwtClaims) -> AuthResult<String> {
        encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::JwtError(e.to_string()))
    }
}

/// Token pair returned on login
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

impl JwtManager {
    /// Generate both access and refresh tokens
    pub fn generate_token_pair(
        &self,
        user_id: &str,
        username: &str,
        role: &str,
    ) -> AuthResult<TokenPair> {
        let access_token = self.generate_access_token(user_id, username, role)?;
        let refresh_token = self.generate_refresh_token(user_id, username, role)?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_token_duration.num_seconds(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_generation_and_validation() {
        let manager = JwtManager::new("test_secret".to_string());
        
        let token = manager.generate_access_token("user123", "testuser", "user")
            .expect("Failed to generate token");
        
        let claims = manager.validate_access_token(&token)
            .expect("Failed to validate token");
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.token_type, "access");
    }
    
    #[test]
    fn test_refresh_token() {
        let manager = JwtManager::new("test_secret".to_string());
        
        let refresh = manager.generate_refresh_token("user123", "testuser", "user")
            .expect("Failed to generate refresh token");
        
        let new_access = manager.refresh_access_token(&refresh)
            .expect("Failed to refresh token");
        
        let claims = manager.validate_access_token(&new_access)
            .expect("Failed to validate new access token");
        
        assert_eq!(claims.sub, "user123");
    }
}