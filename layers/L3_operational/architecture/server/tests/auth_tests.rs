//! Authentication integration tests

use hal9_core::auth::{JwtManager, UserManager, CreateUserRequest, ApiKeyManager, CreateApiKeyRequest, Permission, UserRole, Permissions};
use sqlx::SqlitePool;
use anyhow::Result;

#[tokio::test]
async fn test_jwt_authentication_flow() -> Result<()> {
    // Create temporary database
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Initialize user manager
    let user_manager = UserManager::new(pool.clone());
    user_manager.initialize().await?;
    
    // Create JWT manager
    let jwt_manager = JwtManager::new("test-secret-key".to_string());
    
    // Create a test user
    let create_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        role: Some(UserRole::User),
    };
    
    let user = user_manager.create_user(create_request).await?;
    assert_eq!(user.username, "testuser");
    
    // Test authentication
    let authenticated_user = user_manager
        .authenticate("testuser", "password123")
        .await?;
    assert_eq!(authenticated_user.id, user.id);
    
    // Generate JWT tokens
    let token_pair = jwt_manager.generate_token_pair(
        &user.id,
        &user.username,
        &user.role
    )?;
    
    // Validate access token
    let claims = jwt_manager.validate_access_token(&token_pair.access_token)?;
    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.username, user.username);
    assert_eq!(claims.role, user.role);
    
    // Test refresh token
    let new_access_token = jwt_manager.refresh_access_token(&token_pair.refresh_token)?;
    let new_claims = jwt_manager.validate_access_token(&new_access_token)?;
    assert_eq!(new_claims.sub, user.id);
    
    // Test invalid password
    let auth_result = user_manager.authenticate("testuser", "wrongpassword").await;
    assert!(auth_result.is_err());
    
    // Test invalid token
    let invalid_result = jwt_manager.validate_access_token("invalid-token");
    assert!(invalid_result.is_err());
    
    println!("✅ JWT authentication test passed!");
    Ok(())
}

#[tokio::test]
async fn test_api_key_authentication() -> Result<()> {
    // Create temporary database
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Initialize managers
    let user_manager = UserManager::new(pool.clone());
    user_manager.initialize().await?;
    
    let api_key_manager = ApiKeyManager::new(pool.clone());
    api_key_manager.initialize().await?;
    
    // Create a test user
    let create_request = CreateUserRequest {
        username: "apiuser".to_string(),
        email: "api@example.com".to_string(),
        password: "password123".to_string(),
        role: Some(UserRole::User),
    };
    
    let user = user_manager.create_user(create_request).await?;
    
    // Create API key
    let api_key_request = CreateApiKeyRequest {
        name: "test-key".to_string(),
        permissions: Permissions::with_permissions(vec![Permission::ViewNeuron, Permission::SendSignal]),
        expires_in_days: None,
    };
    
    let api_key_response = api_key_manager
        .create_api_key(&user.id, api_key_request)
        .await?;
    
    // Validate API key
    let (key_info, permissions) = api_key_manager
        .validate_api_key(&api_key_response.key)
        .await?;
    
    assert_eq!(key_info.user_id, user.id);
    assert_eq!(key_info.name, "test-key");
    assert!(permissions.has(&Permission::ViewNeuron));
    assert!(permissions.has(&Permission::SendSignal));
    
    // List user's API keys
    let keys = api_key_manager.list_user_api_keys(&user.id).await?;
    assert_eq!(keys.len(), 1);
    
    // Revoke API key
    api_key_manager.revoke_api_key(&user.id, &keys[0].id).await?;
    
    // Validate revoked key should fail
    let revoked_result = api_key_manager.validate_api_key(&api_key_response.key).await;
    assert!(revoked_result.is_err());
    
    println!("✅ API key authentication test passed!");
    Ok(())
}

#[tokio::test]
async fn test_user_roles_and_permissions() -> Result<()> {
    
    // Test admin permissions
    let admin_perms = UserRole::Admin.default_permissions();
    assert!(admin_perms.has(&Permission::ViewNeuron));
    assert!(admin_perms.has(&Permission::ModifyNeuron));
    assert!(admin_perms.has(&Permission::DeleteNeuron));
    assert!(admin_perms.has(&Permission::SystemAdmin));
    
    // Test user permissions
    let user_perms = UserRole::User.default_permissions();
    assert!(user_perms.has(&Permission::ViewNeuron));
    assert!(user_perms.has(&Permission::SendSignal));
    assert!(!user_perms.has(&Permission::DeleteNeuron));
    assert!(!user_perms.has(&Permission::SystemAdmin));
    
    // Test guest permissions
    let guest_perms = UserRole::Guest.default_permissions();
    assert!(guest_perms.has(&Permission::ViewNeuron));
    assert!(!guest_perms.has(&Permission::SendSignal));
    assert!(!guest_perms.has(&Permission::DeleteNeuron));
    assert!(!guest_perms.has(&Permission::SystemAdmin));
    
    println!("✅ Roles and permissions test passed!");
    Ok(())
}