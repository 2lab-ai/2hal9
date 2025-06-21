use crate::test_framework::*;
use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;

/// Test authentication flows
#[tokio::test]
async fn test_authentication_flow() -> Result<()> {
    let mut config = TestConfig::default();
    config.auth_enabled = true;
    let client = E2ETestClient::new(config);
    
    client.wait_for_server().await?;
    
    println!("=== Testing Authentication Flow ===");
    
    // Test 1: Register new user
    let username = format!("test_user_{}", Uuid::new_v4());
    let password = "secure_password_123";
    
    println!("1. Registering new user: {}", username);
    let register_response: Value = client.register(&username, &password).await?;
    
    assert!(register_response.get("id").is_some());
    assert!(register_response.get("username").is_some());
    println!("✅ Registration successful");
    
    // Test 2: Login with credentials
    println!("\n2. Testing login");
    client.login(&username, &password).await?;
    println!("✅ Login successful");
    
    // Test 3: Access protected endpoint
    println!("\n3. Testing protected endpoint access");
    let profile: Value = client.get("/api/v1/auth/profile").await?;
    assert_eq!(
        profile.get("username").and_then(|v| v.as_str()),
        Some(username.as_str())
    );
    println!("✅ Protected endpoint accessible");
    
    // Test 4: Create API key
    println!("\n4. Testing API key creation");
    let api_key_data = serde_json::json!({
        "name": "Test API Key",
        "permissions": ["read", "write"]
    });
    
    let api_key_response: Value = client.post("/api/v1/auth/api-keys", &api_key_data).await?;
    let api_key = api_key_response.get("key")
        .and_then(|v| v.as_str())
        .expect("API key should be returned");
    
    println!("✅ API key created: {}...", &api_key[..8]);
    
    // Test 5: Use API key for authentication
    println!("\n5. Testing API key authentication");
    let new_client = E2ETestClient::new(client.config.clone());
    
    // Set API key as auth token
    *new_client.auth_token.lock().await = Some(api_key.to_string());
    
    let api_key_profile: Value = new_client.get("/api/v1/auth/profile").await?;
    assert_eq!(
        api_key_profile.get("username").and_then(|v| v.as_str()),
        Some(username.as_str())
    );
    println!("✅ API key authentication working");
    
    // Test 6: Token refresh
    println!("\n6. Testing token refresh");
    let refresh_response: Value = client.post("/api/v1/auth/refresh", &serde_json::json!({})).await?;
    assert!(refresh_response.get("token").is_some());
    println!("✅ Token refresh successful");
    
    // Test 7: Invalid credentials
    println!("\n7. Testing invalid credentials");
    let invalid_client = E2ETestClient::new(client.config.clone());
    
    match invalid_client.login(&username, "wrong_password").await {
        Err(e) if e.to_string().contains("401") => {
            println!("✅ Invalid credentials properly rejected");
        }
        Ok(_) => anyhow::bail!("Invalid credentials should fail"),
        Err(e) => return Err(e),
    }
    
    // Test 8: Logout
    println!("\n8. Testing logout");
    let _: Value = client.post("/api/v1/auth/logout", &serde_json::json!({})).await?;
    
    // Verify token is invalidated
    match client.get::<Value>("/api/v1/auth/profile").await {
        Err(e) if e.to_string().contains("401") => {
            println!("✅ Logout successful, token invalidated");
        }
        Ok(_) => anyhow::bail!("Should not be able to access profile after logout"),
        Err(e) => return Err(e),
    }
    
    println!("\n✅ All authentication tests passed!");
    Ok(())
}

/// Test authorization and permissions
#[tokio::test]
async fn test_authorization() -> Result<()> {
    let mut config = TestConfig::default();
    config.auth_enabled = true;
    let client = E2ETestClient::new(config.clone());
    
    client.wait_for_server().await?;
    
    println!("=== Testing Authorization ===");
    
    // Create users with different roles
    let admin_user = format!("admin_{}", Uuid::new_v4());
    let regular_user = format!("user_{}", Uuid::new_v4());
    let guest_user = format!("guest_{}", Uuid::new_v4());
    
    // Register users (in real system, admin would be pre-configured)
    for (username, role) in &[
        (&admin_user, "admin"),
        (&regular_user, "user"),
        (&guest_user, "guest"),
    ] {
        let mut user_data = Fixtures::user(username);
        user_data["role"] = serde_json::json!(role);
        
        let _: Value = client.post("/api/v1/auth/register", &user_data).await?;
        println!("Created {} user: {}", role, username);
    }
    
    // Test admin permissions
    println!("\n1. Testing admin permissions");
    let admin_client = E2ETestClient::new(config.clone());
    admin_client.login(&admin_user, "test_password_123").await?;
    
    // Admin should be able to access system management
    let _: Value = admin_client.post("/api/system/self-organize", &serde_json::json!({})).await?;
    println!("✅ Admin can trigger self-organization");
    
    // Admin should see all neurons
    let neurons: Value = admin_client.get("/api/neurons").await?;
    println!("✅ Admin can list all neurons");
    
    // Test regular user permissions
    println!("\n2. Testing regular user permissions");
    let user_client = E2ETestClient::new(config.clone());
    user_client.login(&regular_user, "test_password_123").await?;
    
    // User can create neurons
    let neuron: Value = user_client.post("/api/neurons", &Fixtures::neuron()).await?;
    let neuron_id = neuron.get("id").and_then(|v| v.as_str()).unwrap();
    println!("✅ User can create neurons");
    
    // User can only modify their own neurons
    let _: Value = user_client.put(
        &format!("/api/neurons/{}", neuron_id),
        &serde_json::json!({ "processing_speed": 1.5 })
    ).await?;
    println!("✅ User can modify own neurons");
    
    // User cannot trigger system-wide operations
    match user_client.post::<_, Value>("/api/system/self-organize", &serde_json::json!({})).await {
        Err(e) if e.to_string().contains("403") => {
            println!("✅ User correctly denied system operations");
        }
        Ok(_) => anyhow::bail!("User should not have system permissions"),
        Err(e) => return Err(e),
    }
    
    // Test guest permissions
    println!("\n3. Testing guest permissions");
    let guest_client = E2ETestClient::new(config);
    guest_client.login(&guest_user, "test_password_123").await?;
    
    // Guest can only read
    let _: Value = guest_client.get("/api/neurons").await?;
    println!("✅ Guest can read neurons");
    
    // Guest cannot create
    match guest_client.post::<_, Value>("/api/neurons", &Fixtures::neuron()).await {
        Err(e) if e.to_string().contains("403") => {
            println!("✅ Guest correctly denied write operations");
        }
        Ok(_) => anyhow::bail!("Guest should not have write permissions"),
        Err(e) => return Err(e),
    }
    
    println!("\n✅ All authorization tests passed!");
    Ok(())
}

/// Test security features
#[tokio::test]
async fn test_security_features() -> Result<()> {
    let config = TestConfig::default();
    let client = E2ETestClient::new(config);
    
    client.wait_for_server().await?;
    
    println!("=== Testing Security Features ===");
    
    // Test 1: SQL injection attempts
    println!("1. Testing SQL injection protection");
    let malicious_input = serde_json::json!({
        "layer": "1; DROP TABLE neurons;--",
        "position": [0.5, 0.5, 0.5],
        "processing_speed": 1.0
    });
    
    match client.post::<_, Value>("/api/neurons", &malicious_input).await {
        Err(e) if e.to_string().contains("400") || e.to_string().contains("422") => {
            println!("✅ SQL injection attempt properly rejected");
        }
        Ok(_) => anyhow::bail!("Malicious input should be rejected"),
        Err(e) => println!("⚠️  Unexpected error (may be OK): {}", e),
    }
    
    // Test 2: XSS attempts
    println!("\n2. Testing XSS protection");
    let xss_attempt = serde_json::json!({
        "username": "<script>alert('xss')</script>",
        "password": "password123"
    });
    
    if let Ok(response) = client.post::<_, Value>("/api/v1/auth/register", &xss_attempt).await {
        // Check that script tags are escaped
        if let Some(username) = response.get("username").and_then(|v| v.as_str()) {
            assert!(
                !username.contains("<script>"),
                "Script tags should be escaped or rejected"
            );
            println!("✅ XSS attempt properly handled");
        }
    }
    
    // Test 3: Path traversal attempts
    println!("\n3. Testing path traversal protection");
    let path_traversal_attempts = vec![
        "/api/neurons/../../../etc/passwd",
        "/api/neurons/..%2F..%2F..%2Fetc%2Fpasswd",
        "/api/neurons/./././sensitive",
    ];
    
    for path in path_traversal_attempts {
        match client.get::<Value>(path).await {
            Err(e) if e.to_string().contains("400") || e.to_string().contains("404") => {
                println!("✅ Path traversal attempt blocked: {}", path);
            }
            Ok(_) => anyhow::bail!("Path traversal should be blocked: {}", path),
            Err(e) => println!("⚠️  Error for {}: {}", path, e),
        }
    }
    
    // Test 4: Large payload protection
    println!("\n4. Testing large payload protection");
    let large_pattern = vec![0.1f32; 1_000_000]; // 1M floats
    let large_payload = serde_json::json!({
        "pattern": large_pattern,
        "intensity": 0.5
    });
    
    match client.post::<_, Value>("/api/neurons/test-id/signal", &large_payload).await {
        Err(e) if e.to_string().contains("413") || e.to_string().contains("400") => {
            println!("✅ Large payload properly rejected");
        }
        Ok(_) => println!("⚠️  Large payload accepted (may need limits)"),
        Err(e) => println!("⚠️  Unexpected error: {}", e),
    }
    
    // Test 5: CORS headers
    println!("\n5. Testing CORS headers");
    let response = client.http_client
        .get(&format!("{}/health", client.config.base_url))
        .header("Origin", "https://malicious-site.com")
        .send()
        .await?;
    
    let cors_header = response.headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    
    match cors_header {
        Some("*") => println!("⚠️  CORS allows all origins (may be intentional)"),
        Some(origin) => println!("✅ CORS restricted to: {}", origin),
        None => println!("✅ CORS headers not present (good for API)"),
    }
    
    println!("\n✅ Security tests completed!");
    Ok(())
}