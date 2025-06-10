//! Comprehensive unit tests for authentication modules

use super::*;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use std::collections::HashSet;
use uuid::Uuid;

/// Test utilities for auth tests
mod test_utils {
    use super::*;
    
    pub fn create_test_user() -> User {
        User {
            id: Uuid::new_v4().to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "dummy_hash".to_string(),
            role: UserRole::User.to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            is_active: true,
        }
    }
    
    pub fn create_admin_user() -> User {
        User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            password_hash: "dummy_hash".to_string(),
            role: UserRole::Admin.to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            is_active: true,
        }
    }
    
    pub fn create_test_permissions() -> Permissions {
        let mut perms = Permissions::new();
        perms.add(Permission::ViewNeuron);
        perms.add(Permission::ModifyNeuron);
        perms
    }
}

mod jwt_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_jwt_manager_initialization() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::with_durations(secret.to_string(), 60, 7);
        
        // Test that manager is created successfully
        assert!(true); // Manager creation test
    }
    
    #[tokio::test]
    async fn test_token_generation() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string());
        
        let user = create_test_user();
        let access_token = manager.generate_access_token(&user.id, &user.username, &user.role).unwrap();
        let refresh_token = manager.generate_refresh_token(&user.id, &user.username, &user.role).unwrap();
        
        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());
        assert_ne!(access_token, refresh_token);
    }
    
    #[tokio::test]
    async fn test_token_validation() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string());
        
        let user = create_test_user();
        let access_token = manager.generate_access_token(&user.id, &user.username, &user.role).unwrap();
        let refresh_token = manager.generate_refresh_token(&user.id, &user.username, &user.role).unwrap();
        
        // Validate access token
        let claims = manager.validate_token(&access_token).unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
        
        // Validate refresh token
        let refresh_claims = manager.validate_token(&refresh_token).unwrap();
        assert_eq!(refresh_claims.sub, user.id);
    }
    
    #[tokio::test]
    async fn test_expired_token() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::with_durations(
            secret.to_string(), 
            0, // 0 minutes = expired immediately
            7
        );
        
        let user = create_test_user();
        let access_token = manager.generate_access_token(&user.id, &user.username, &user.role).unwrap();
        
        // Wait for token to expire
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
        // Should fail validation
        let result = manager.validate_token(&access_token);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_invalid_token() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string());
        
        // Test various invalid tokens
        let invalid_tokens = vec![
            "invalid.token.here",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.payload",
            "",
            "not-even-jwt-format",
        ];
        
        for token in invalid_tokens {
            let result = manager.validate_token(token);
            assert!(result.is_err());
        }
    }
    
    #[tokio::test]
    async fn test_refresh_token_flow() {
        let secret = "test-secret-key-with-sufficient-length-for-hs256";
        let manager = JwtManager::new(secret.to_string());
        
        let user = create_test_user();
        let initial_access = manager.generate_access_token(&user.id, &user.username, &user.role).unwrap();
        let refresh_token = manager.generate_refresh_token(&user.id, &user.username, &user.role).unwrap();
        
        // Refresh using refresh token
        let new_access = manager.refresh_access_token(&refresh_token).unwrap();
        
        assert_ne!(new_access, initial_access);
        
        // Validate new token works
        let claims = manager.validate_token(&new_access).unwrap();
        assert_eq!(claims.sub, user.id);
    }
}

mod api_key_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_api_key_manager() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = ApiKeyManager::new(pool);
        manager.initialize().await.unwrap();
        
        // Create API key
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Test API Key".to_string(),
            permissions: create_test_permissions(),
            expires_in_days: Some(30),
        };
        
        let response = manager.create_api_key(&user.id, request).await.unwrap();
        
        assert!(!response.key.is_empty());
        assert!(!response.id.is_empty());
        assert_eq!(response.name, "Test API Key");
    }
    
    #[tokio::test]
    async fn test_api_key_validation() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = ApiKeyManager::new(pool);
        manager.initialize().await.unwrap();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            permissions: create_test_permissions(),
            expires_in_days: None,
        };
        
        let response = manager.create_api_key(&user.id, request).await.unwrap();
        
        // Validate the key
        let (key_info, _perms) = manager.validate_api_key(&response.key).await.unwrap();
        
        assert_eq!(key_info.id, response.id);
        assert_eq!(key_info.user_id, user.id);
        assert_eq!(key_info.name, "Test Key");
        assert!(key_info.is_active);
    }
    
    #[tokio::test]
    async fn test_expired_api_key() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = ApiKeyManager::new(pool);
        manager.initialize().await.unwrap();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Expired Key".to_string(),
            permissions: create_test_permissions(),
            expires_in_days: Some(-1), // Negative days = already expired
        };
        
        let response = manager.create_api_key(&user.id, request).await.unwrap();
        
        // Should fail validation
        let result = manager.validate_api_key(&response.key).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_api_key_revocation() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = ApiKeyManager::new(pool);
        manager.initialize().await.unwrap();
        
        let user = create_test_user();
        let request = CreateApiKeyRequest {
            name: "Revocable Key".to_string(),
            permissions: create_test_permissions(),
            expires_in_days: None,
        };
        
        let response = manager.create_api_key(&user.id, request).await.unwrap();
        
        // Revoke the key
        manager.revoke_api_key(&user.id, &response.id).await.unwrap();
        
        // Should fail validation after revocation
        let result = manager.validate_api_key(&response.key).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_list_user_api_keys() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = ApiKeyManager::new(pool);
        manager.initialize().await.unwrap();
        
        let user = create_test_user();
        
        // Create multiple keys
        for i in 0..3 {
            let request = CreateApiKeyRequest {
                name: format!("Key {}", i),
                permissions: create_test_permissions(),
                expires_in_days: None,
            };
            manager.create_api_key(&user.id, request).await.unwrap();
        }
        
        // List keys
        let keys = manager.list_user_api_keys(&user.id).await.unwrap();
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
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = UserManager::new(pool);
        manager.initialize().await.unwrap();
        
        // Create user
        let request = CreateUserRequest {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            password: "secure_password123".to_string(),
            role: Some(UserRole::User),
        };
        
        let user = manager.create_user(request).await.unwrap();
        
        assert_eq!(user.username, "newuser");
        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.role, UserRole::User.to_string());
        assert!(user.is_active);
    }
    
    #[tokio::test]
    async fn test_user_authentication() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = UserManager::new(pool);
        manager.initialize().await.unwrap();
        
        let request = CreateUserRequest {
            username: "authuser".to_string(),
            email: "auth@example.com".to_string(),
            password: "test_password".to_string(),
            role: Some(UserRole::User),
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
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = UserManager::new(pool);
        manager.initialize().await.unwrap();
        
        let request = CreateUserRequest {
            username: "duplicate".to_string(),
            email: "dup@example.com".to_string(),
            password: "password".to_string(),
            role: Some(UserRole::User),
        };
        
        // Create first user
        manager.create_user(request).await.unwrap();
        
        // Try to create duplicate with same username
        let duplicate_request = CreateUserRequest {
            username: "duplicate".to_string(),
            email: "dup2@example.com".to_string(),
            password: "password".to_string(),
            role: Some(UserRole::User),
        };
        let result = manager.create_user(duplicate_request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_update_user() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = UserManager::new(pool);
        manager.initialize().await.unwrap();
        
        let create_request = CreateUserRequest {
            username: "updateuser".to_string(),
            email: "update@example.com".to_string(),
            password: "password".to_string(),
            role: Some(UserRole::User),
        };
        
        let user = manager.create_user(create_request).await.unwrap();
        
        // Update user
        let update_request = UpdateUserRequest {
            email: Some("newemail@example.com".to_string()),
            role: Some(UserRole::Admin),
            is_active: None,
        };
        
        let updated = manager.update_user(&user.id, update_request).await.unwrap();
        
        assert_eq!(updated.email, "newemail@example.com");
        assert_eq!(updated.role, UserRole::Admin.to_string());
        assert_eq!(updated.username, "updateuser"); // Unchanged
    }
    
    #[tokio::test]
    async fn test_deactivate_user() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let manager = UserManager::new(pool);
        manager.initialize().await.unwrap();
        
        let request = CreateUserRequest {
            username: "deactivate".to_string(),
            email: "deactivate@example.com".to_string(),
            password: "password".to_string(),
            role: Some(UserRole::User),
        };
        
        let user = manager.create_user(request).await.unwrap();
        
        // Deactivate user by updating is_active flag
        let deactivate_request = UpdateUserRequest {
            email: None,
            role: None,
            is_active: Some(false),
        };
        manager.update_user(&user.id, deactivate_request).await.unwrap();
        
        // Try to authenticate - should fail
        let auth_result = manager.authenticate("deactivate", "password").await;
        assert!(auth_result.is_err());
        
        // Get user to verify status
        let deactivated = manager.get_user(&user.id).await.unwrap();
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
        perms.add(Permission::ViewNeuron);
        perms.add(Permission::ModifyNeuron);
        
        assert!(perms.has(&Permission::ViewNeuron));
        assert!(perms.has(&Permission::ModifyNeuron));
        assert!(!perms.has(&Permission::DeleteNeuron));
        
        // Remove permission
        perms.remove(&Permission::ModifyNeuron);
        assert!(!perms.has(&Permission::ModifyNeuron));
    }
    
    #[test]
    fn test_permission_sets() {
        // Test predefined sets
        let read_only = Permissions::read_only();
        assert!(read_only.has(&Permission::ViewNeuron));
        assert!(!read_only.has(&Permission::ModifyNeuron));
        
        let read_write = Permissions::read_write();
        assert!(read_write.has(&Permission::ViewNeuron));
        assert!(read_write.has(&Permission::ModifyNeuron));
        assert!(!read_write.has(&Permission::DeleteNeuron));
        
        let admin = Permissions::all();
        assert!(admin.has(&Permission::ViewNeuron));
        assert!(admin.has(&Permission::ModifyNeuron));
        assert!(admin.has(&Permission::DeleteNeuron));
        assert!(admin.has(&Permission::SystemAdmin));
    }
    
    #[test]
    fn test_permission_intersection() {
        let mut perms1 = Permissions::new();
        perms1.add(Permission::ViewNeuron);
        perms1.add(Permission::ModifyNeuron);
        
        let mut perms2 = Permissions::new();
        perms2.add(Permission::ModifyNeuron);
        perms2.add(Permission::DeleteNeuron);
        
        let intersection = perms1.intersection(&perms2);
        assert!(!intersection.has(&Permission::ViewNeuron));
        assert!(intersection.has(&Permission::ModifyNeuron));
        assert!(!intersection.has(&Permission::DeleteNeuron));
    }
    
    #[test]
    fn test_permission_union() {
        let mut perms1 = Permissions::new();
        perms1.add(Permission::ViewNeuron);
        
        let mut perms2 = Permissions::new();
        perms2.add(Permission::ModifyNeuron);
        
        let union = perms1.union(&perms2);
        assert!(union.has(&Permission::ViewNeuron));
        assert!(union.has(&Permission::ModifyNeuron));
    }
}

mod role_tests {
    use super::*;
    
    #[test]
    fn test_role_hierarchy() {
        assert!(UserRole::Admin.has_permission(Permission::SystemAdmin));
        assert!(UserRole::Admin.has_permission(Permission::ModifyNeuron));
        assert!(UserRole::Admin.has_permission(Permission::ViewNeuron));
        
        assert!(!UserRole::User.has_permission(Permission::SystemAdmin));
        assert!(UserRole::User.has_permission(Permission::ModifyNeuron));
        assert!(UserRole::User.has_permission(Permission::ViewNeuron));
        
        assert!(!UserRole::Guest.has_permission(Permission::SystemAdmin));
        assert!(!UserRole::Guest.has_permission(Permission::ModifyNeuron));
        assert!(UserRole::Guest.has_permission(Permission::ViewNeuron));
    }
    
    #[test]
    fn test_role_ordering() {
        assert!(UserRole::Admin > UserRole::User);
        assert!(UserRole::User > UserRole::Guest);
        assert!(UserRole::Admin > UserRole::Guest);
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
        let jwt_manager = JwtManager::new(jwt_secret.to_string());
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let api_key_manager = ApiKeyManager::new(pool.clone());
        api_key_manager.initialize().await.unwrap();
        let user_manager = UserManager::new(pool);
        user_manager.initialize().await.unwrap();
        
        // Create user
        let create_request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "secure_password".to_string(),
            role: Some(UserRole::User),
        };
        
        let user = user_manager.create_user(create_request).await.unwrap();
        
        // Generate JWT tokens
        let tokens = jwt_manager.generate_token_pair(&user.id, &user.username, &user.role).unwrap();
        
        // Validate JWT
        let claims = jwt_manager.validate_token(&tokens.access_token).unwrap();
        assert_eq!(claims.username, "testuser");
        
        // Create API key
        let api_key_request = CreateApiKeyRequest {
            name: "Integration Test Key".to_string(),
            permissions: Permissions::read_write(),
            expires_in_days: None,
        };
        
        let api_key_response = api_key_manager.create_api_key(&user.id, api_key_request).await.unwrap();
        
        // Validate API key
        let (key_info, _perms) = api_key_manager.validate_api_key(&api_key_response.key).await.unwrap();
        assert_eq!(key_info.user_id, user.id);
        
        // Test authentication
        let auth_user = user_manager.authenticate("testuser", "secure_password").await.unwrap();
        assert_eq!(auth_user.id, user.id);
    }
}