//! Comprehensive unit tests for authentication modules

use super::*;
use chrono::{Duration, Utc};
use std::collections::HashSet;
use uuid::Uuid;

/// Test utilities for auth tests
mod test_utils {
    use super::*;
    
    pub fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
            permissions: Permissions::new(),
        }
    }
    
    pub fn create_admin_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            role: UserRole::Admin,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
            permissions: Permissions::all(),
        }
    }
    
    pub fn create_test_permissions() -> Permissions {
        let mut perms = Permissions::new();
        perms.add(Permission::Read);
        perms.add(Permission::Write);
        perms
    }
}

mod jwt_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_jwt_manager_initialization() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string(), Duration::hours(1), Duration::days(7));
        
        assert_eq!(manager.access_token_duration(), Duration::hours(1));
        assert_eq!(manager.refresh_token_duration(), Duration::days(7));
    }
    
    #[tokio::test]
    async fn test_token_generation() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string(), Duration::hours(1), Duration::days(7));
        
        let user = create_test_user();
        let token_pair = manager.generate_tokens(&user).await.unwrap();
        
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_ne!(token_pair.access_token, token_pair.refresh_token);
    }
    
    #[tokio::test]
    async fn test_token_validation() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string(), Duration::hours(1), Duration::days(7));
        
        let user = create_test_user();
        let token_pair = manager.generate_tokens(&user).await.unwrap();
        
        // Validate access token
        let claims = manager.validate_token(&token_pair.access_token).await.unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
        
        // Validate refresh token
        let refresh_claims = manager.validate_token(&token_pair.refresh_token).await.unwrap();
        assert_eq!(refresh_claims.sub, user.id.to_string());
    }
    
    #[tokio::test]
    async fn test_expired_token() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(
            secret.to_string(), 
            Duration::milliseconds(1), // Very short duration
            Duration::days(7)
        );
        
        let user = create_test_user();
        let token_pair = manager.generate_tokens(&user).await.unwrap();
        
        // Wait for token to expire
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        // Should fail validation
        let result = manager.validate_token(&token_pair.access_token).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_invalid_token() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string(), Duration::hours(1), Duration::days(7));
        
        // Test various invalid tokens
        let invalid_tokens = vec![
            "invalid.token.here",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.payload",
            "",
            "not-even-jwt-format",
        ];
        
        for token in invalid_tokens {
            let result = manager.validate_token(token).await;
            assert!(result.is_err());
        }
    }
    
    #[tokio::test]
    async fn test_refresh_token_flow() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string(), Duration::hours(1), Duration::days(7));
        
        let user = create_test_user();
        let initial_tokens = manager.generate_tokens(&user).await.unwrap();
        
        // Refresh using refresh token
        let new_tokens = manager.refresh_tokens(&initial_tokens.refresh_token).await.unwrap();
        
        assert_ne!(new_tokens.access_token, initial_tokens.access_token);
        assert_ne!(new_tokens.refresh_token, initial_tokens.refresh_token);
        
        // Validate new tokens work
        let claims = manager.validate_token(&new_tokens.access_token).await.unwrap();
        assert_eq!(claims.sub, user.id.to_string());
    }
}

mod api_key_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_api_key_manager() {
        let manager = ApiKeyManager::new();
        
        // Create API key
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Test API Key".to_string(),
            permissions: create_test_permissions(),
            expires_at: Some(Utc::now() + Duration::days(30)),
        };
        
        let response = manager.create_api_key(&user, request).await.unwrap();
        
        assert!(!response.key.is_empty());
        assert!(!response.api_key_id.is_nil());
        assert_eq!(response.name, "Test API Key");
    }
    
    #[tokio::test]
    async fn test_api_key_validation() {
        let manager = ApiKeyManager::new();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            permissions: create_test_permissions(),
            expires_at: None,
        };
        
        let response = manager.create_api_key(&user, request).await.unwrap();
        
        // Validate the key
        let key_info = manager.validate_api_key(&response.key).await.unwrap();
        
        assert_eq!(key_info.id, response.api_key_id);
        assert_eq!(key_info.user_id, user.id);
        assert_eq!(key_info.name, "Test Key");
        assert!(key_info.is_active);
    }
    
    #[tokio::test]
    async fn test_expired_api_key() {
        let manager = ApiKeyManager::new();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Expired Key".to_string(),
            permissions: create_test_permissions(),
            expires_at: Some(Utc::now() - Duration::hours(1)), // Already expired
        };
        
        let response = manager.create_api_key(&user, request).await.unwrap();
        
        // Should fail validation
        let result = manager.validate_api_key(&response.key).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_api_key_revocation() {
        let manager = ApiKeyManager::new();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Revocable Key".to_string(),
            permissions: create_test_permissions(),
            expires_at: None,
        };
        
        let response = manager.create_api_key(&user, request).await.unwrap();
        
        // Revoke the key
        manager.revoke_api_key(&user, response.api_key_id).await.unwrap();
        
        // Should fail validation after revocation
        let result = manager.validate_api_key(&response.key).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_list_user_api_keys() {
        let manager = ApiKeyManager::new();
        
        let user = create_test_user();
        
        // Create multiple keys
        for i in 0..3 {
            let request = CreateApiKeyRequest {
                name: format!("Key {}", i),
                permissions: create_test_permissions(),
                expires_at: None,
            };
            manager.create_api_key(&user, request).await.unwrap();
        }
        
        // List keys
        let keys = manager.list_user_api_keys(&user).await.unwrap();
        assert_eq!(keys.len(), 3);
        
        // All should belong to the user
        for key in keys {
            assert_eq!(key.user_id, user.id);
            assert!(key.is_active);
        }
    }
}

mod user_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_user_manager() {
        let db_path = ":memory:";
        let manager = UserManager::new(db_path).await.unwrap();
        
        // Create user
        let request = CreateUserRequest {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            password: "secure_password123".to_string(),
            role: UserRole::User,
        };
        
        let user = manager.create_user(request).await.unwrap();
        
        assert_eq!(user.username, "newuser");
        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.role, UserRole::User);
        assert!(user.is_active);
    }
    
    #[tokio::test]
    async fn test_user_authentication() {
        let db_path = ":memory:";
        let manager = UserManager::new(db_path).await.unwrap();
        
        let request = CreateUserRequest {
            username: "authuser".to_string(),
            email: "auth@example.com".to_string(),
            password: "test_password".to_string(),
            role: UserRole::User,
        };
        
        let user = manager.create_user(request).await.unwrap();
        
        // Test successful authentication
        let auth_result = manager.authenticate("authuser", "test_password").await.unwrap();
        assert_eq!(auth_result.id, user.id);
        
        // Test failed authentication
        let fail_result = manager.authenticate("authuser", "wrong_password").await;
        assert!(fail_result.is_err());
    }
    
    #[tokio::test]
    async fn test_duplicate_user() {
        let db_path = ":memory:";
        let manager = UserManager::new(db_path).await.unwrap();
        
        let request = CreateUserRequest {
            username: "duplicate".to_string(),
            email: "dup@example.com".to_string(),
            password: "password".to_string(),
            role: UserRole::User,
        };
        
        // Create first user
        manager.create_user(request.clone()).await.unwrap();
        
        // Try to create duplicate
        let result = manager.create_user(request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_update_user() {
        let db_path = ":memory:";
        let manager = UserManager::new(db_path).await.unwrap();
        
        let create_request = CreateUserRequest {
            username: "updateuser".to_string(),
            email: "update@example.com".to_string(),
            password: "password".to_string(),
            role: UserRole::User,
        };
        
        let user = manager.create_user(create_request).await.unwrap();
        
        // Update user
        let update_request = UpdateUserRequest {
            email: Some("newemail@example.com".to_string()),
            role: Some(UserRole::Admin),
            is_active: None,
        };
        
        let updated = manager.update_user(user.id, update_request).await.unwrap();
        
        assert_eq!(updated.email, "newemail@example.com");
        assert_eq!(updated.role, UserRole::Admin);
        assert_eq!(updated.username, "updateuser"); // Unchanged
    }
    
    #[tokio::test]
    async fn test_deactivate_user() {
        let db_path = ":memory:";
        let manager = UserManager::new(db_path).await.unwrap();
        
        let request = CreateUserRequest {
            username: "deactivate".to_string(),
            email: "deactivate@example.com".to_string(),
            password: "password".to_string(),
            role: UserRole::User,
        };
        
        let user = manager.create_user(request).await.unwrap();
        
        // Deactivate user
        manager.deactivate_user(user.id).await.unwrap();
        
        // Try to authenticate - should fail
        let auth_result = manager.authenticate("deactivate", "password").await;
        assert!(auth_result.is_err());
        
        // Get user to verify status
        let deactivated = manager.get_user(user.id).await.unwrap();
        assert!(!deactivated.is_active);
    }
}

mod permission_tests {
    use super::*;
    use super::test_utils::*;
    
    #[test]
    fn test_permission_operations() {
        let mut perms = Permissions::new();
        
        // Add permissions
        perms.add(Permission::Read);
        perms.add(Permission::Write);
        
        assert!(perms.has(Permission::Read));
        assert!(perms.has(Permission::Write));
        assert!(!perms.has(Permission::Delete));
        
        // Remove permission
        perms.remove(Permission::Write);
        assert!(!perms.has(Permission::Write));
    }
    
    #[test]
    fn test_permission_sets() {
        // Test predefined sets
        let read_only = Permissions::read_only();
        assert!(read_only.has(Permission::Read));
        assert!(!read_only.has(Permission::Write));
        
        let read_write = Permissions::read_write();
        assert!(read_write.has(Permission::Read));
        assert!(read_write.has(Permission::Write));
        assert!(!read_write.has(Permission::Delete));
        
        let admin = Permissions::all();
        assert!(admin.has(Permission::Read));
        assert!(admin.has(Permission::Write));
        assert!(admin.has(Permission::Delete));
        assert!(admin.has(Permission::Admin));
    }
    
    #[test]
    fn test_permission_intersection() {
        let mut perms1 = Permissions::new();
        perms1.add(Permission::Read);
        perms1.add(Permission::Write);
        
        let mut perms2 = Permissions::new();
        perms2.add(Permission::Write);
        perms2.add(Permission::Delete);
        
        let intersection = perms1.intersection(&perms2);
        assert!(!intersection.has(Permission::Read));
        assert!(intersection.has(Permission::Write));
        assert!(!intersection.has(Permission::Delete));
    }
    
    #[test]
    fn test_permission_union() {
        let mut perms1 = Permissions::new();
        perms1.add(Permission::Read);
        
        let mut perms2 = Permissions::new();
        perms2.add(Permission::Write);
        
        let union = perms1.union(&perms2);
        assert!(union.has(Permission::Read));
        assert!(union.has(Permission::Write));
    }
}

mod role_tests {
    use super::*;
    
    #[test]
    fn test_role_hierarchy() {
        assert!(UserRole::Admin.has_permission(Permission::Admin));
        assert!(UserRole::Admin.has_permission(Permission::Write));
        assert!(UserRole::Admin.has_permission(Permission::Read));
        
        assert!(!UserRole::User.has_permission(Permission::Admin));
        assert!(UserRole::User.has_permission(Permission::Write));
        assert!(UserRole::User.has_permission(Permission::Read));
        
        assert!(!UserRole::ReadOnly.has_permission(Permission::Admin));
        assert!(!UserRole::ReadOnly.has_permission(Permission::Write));
        assert!(UserRole::ReadOnly.has_permission(Permission::Read));
    }
    
    #[test]
    fn test_role_ordering() {
        assert!(UserRole::Admin > UserRole::User);
        assert!(UserRole::User > UserRole::ReadOnly);
        assert!(UserRole::Admin > UserRole::ReadOnly);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_full_auth_flow() {
        // Setup
        let jwt_secret = "test-secret-key-with-sufficient-length-for-hs256";
        let jwt_manager = JwtManager::new(jwt_secret.to_string(), Duration::hours(1), Duration::days(7));
        let api_key_manager = ApiKeyManager::new();
        let user_manager = UserManager::new(":memory:").await.unwrap();
        
        // Create user
        let create_request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "secure_password".to_string(),
            role: UserRole::User,
        };
        
        let user = user_manager.create_user(create_request).await.unwrap();
        
        // Generate JWT tokens
        let tokens = jwt_manager.generate_tokens(&user).await.unwrap();
        
        // Validate JWT
        let claims = jwt_manager.validate_token(&tokens.access_token).await.unwrap();
        assert_eq!(claims.username, "testuser");
        
        // Create API key
        let api_key_request = CreateApiKeyRequest {
            name: "Integration Test Key".to_string(),
            permissions: Permissions::read_write(),
            expires_at: None,
        };
        
        let api_key_response = api_key_manager.create_api_key(&user, api_key_request).await.unwrap();
        
        // Validate API key
        let key_info = api_key_manager.validate_api_key(&api_key_response.key).await.unwrap();
        assert_eq!(key_info.user_id, user.id);
        
        // Test authentication
        let auth_user = user_manager.authenticate("testuser", "secure_password").await.unwrap();
        assert_eq!(auth_user.id, user.id);
    }
}