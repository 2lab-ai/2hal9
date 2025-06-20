// Test Redis connection approach
use redis::{Client, AsyncCommands, cmd};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test 1: Direct connection
    println!("Testing Redis connection approaches...\n");
    
    let client = Client::open("redis://127.0.0.1:6379")?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    
    // Test AsyncCommands
    println!("Test 1: AsyncCommands");
    let _: () = conn.set("test_key", "test_value").await?;
    let value: String = conn.get("test_key").await?;
    println!("✓ AsyncCommands work: {}", value);
    
    // Test cmd approach
    println!("\nTest 2: cmd approach");
    let _: () = cmd("SET")
        .arg("test_key2")
        .arg("test_value2")
        .query_async(&mut conn)
        .await?;
    let value2: String = cmd("GET")
        .arg("test_key2")
        .query_async(&mut conn)
        .await?;
    println!("✓ cmd approach works: {}", value2);
    
    // Clean up
    let _: () = conn.del(&["test_key", "test_key2"]).await?;
    
    println!("\nAll Redis connection tests passed!");
    Ok(())
}