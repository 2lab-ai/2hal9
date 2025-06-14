//! MCP Tools Demo - Shows how neurons can use external tools

use hal9_core::mcp::{
    ToolRegistry, Tool, FilesystemReadTool, FilesystemWriteTool, 
    ShellTool, WebFetchTool
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HAL9 MCP Tools Demo");
    println!("==================\n");
    
    // Create tool registry
    let mut registry = ToolRegistry::new();
    
    // Register filesystem read tool (allow access to current directory)
    let fs_read = FilesystemReadTool::new(vec![
        ".".to_string(),
        "examples".to_string(),
    ]);
    registry.register(Box::new(fs_read));
    
    // Register filesystem write tool (allow writing to temp directory)
    let fs_write = FilesystemWriteTool::new(vec![
        "/tmp".to_string(),
        "./temp".to_string(),
    ]);
    registry.register(Box::new(fs_write));
    
    // Register shell tool (only allow safe commands)
    let shell = ShellTool::new(vec![
        "ls".to_string(),
        "echo".to_string(),
        "date".to_string(),
        "pwd".to_string(),
    ]);
    registry.register(Box::new(shell));
    
    // Register web fetch tool (no domain restrictions for demo)
    let web = WebFetchTool::new(None);
    registry.register(Box::new(web));
    
    // Show available tools
    println!("Available Tools:");
    for def in registry.definitions() {
        println!("- {}: {}", def.name, def.description);
    }
    println!();
    
    // Demo 1: Read a file
    println!("Demo 1: Reading a file");
    println!("----------------------");
    let result = registry.execute("filesystem_read", json!({
        "path": "Cargo.toml"
    })).await?;
    
    if let Some(content) = result.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")) {
        let preview = content.as_str().unwrap_or("").lines().take(5).collect::<Vec<_>>().join("\n");
        println!("First 5 lines of Cargo.toml:");
        println!("{}", preview);
    }
    println!();
    
    // Demo 2: Execute a shell command
    println!("Demo 2: Execute shell command");
    println!("-----------------------------");
    let result = registry.execute("shell_execute", json!({
        "command": "date"
    })).await?;
    
    if let Some(content) = result.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")) {
        println!("Current date: {}", content.as_str().unwrap_or("").trim());
    }
    println!();
    
    // Demo 3: Write a file
    println!("Demo 3: Write a file");
    println!("--------------------");
    let result = registry.execute("filesystem_write", json!({
        "path": "/tmp/hal9-demo.txt",
        "content": "Hello from HAL9 MCP Tools!\nThis file was created by a neuron using tools."
    })).await?;
    
    if let Some(content) = result.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")) {
        println!("{}", content.as_str().unwrap_or(""));
    }
    println!();
    
    // Demo 4: Fetch web content
    println!("Demo 4: Fetch web content");
    println!("-------------------------");
    let result = registry.execute("web_fetch", json!({
        "url": "https://api.github.com/repos/anthropics/claude-mcp/commits?per_page=1",
        "headers": {
            "User-Agent": "HAL9-Demo"
        }
    })).await?;
    
    if let Some(status) = result.get("metadata").and_then(|m| m.get("status")) {
        println!("HTTP Status: {}", status);
    }
    if let Some(content) = result.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")) {
        let parsed: serde_json::Value = serde_json::from_str(content.as_str().unwrap_or("{}")).unwrap_or_default();
        if let Some(commit) = parsed.get(0) {
            if let Some(message) = commit.get("commit").and_then(|c| c.get("message")) {
                println!("Latest commit: {}", message.as_str().unwrap_or("").lines().next().unwrap_or(""));
            }
        }
    }
    println!();
    
    // Demo 5: Error handling
    println!("Demo 5: Error handling");
    println!("----------------------");
    
    // Try to read a file outside allowed paths
    match registry.execute("filesystem_read", json!({
        "path": "/etc/passwd"
    })).await {
        Ok(_) => println!("Unexpectedly succeeded!"),
        Err(e) => println!("✓ Security check worked: {}", e),
    }
    
    // Try to execute a disallowed command
    match registry.execute("shell_execute", json!({
        "command": "rm",
        "args": ["-rf", "/"]
    })).await {
        Ok(_) => println!("Unexpectedly succeeded!"),
        Err(e) => println!("✓ Command restriction worked: {}", e),
    }
    
    println!("\nDemo complete!");
    
    Ok(())
}

// Example of how a neuron would use tools in its processing
pub async fn neuron_with_tools_example() -> Result<(), Box<dyn std::error::Error>> {
    // In a real neuron, the tool registry would be injected
    let mut registry = ToolRegistry::new();
    
    // Register tools based on neuron's layer and permissions
    registry.register(Box::new(FilesystemReadTool::new(vec!["./data".to_string()])));
    registry.register(Box::new(WebFetchTool::new(Some(vec!["api.github.com".to_string()]))));
    
    // Process a task that requires tools
    let task = "Analyze the README.md file and fetch the latest release info from GitHub";
    
    // Step 1: Read local file
    let readme_result = registry.execute("filesystem_read", json!({
        "path": "./data/README.md"
    })).await?;
    
    // Step 2: Fetch web data
    let github_result = registry.execute("web_fetch", json!({
        "url": "https://api.github.com/repos/owner/repo/releases/latest"
    })).await?;
    
    // Step 3: Process and combine results
    println!("Task completed: {}", task);
    println!("- Read README.md");
    println!("- Fetched latest release from GitHub");
    
    Ok(())
}