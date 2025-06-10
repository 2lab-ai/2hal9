use anyhow::Result;
use ha_prompter::{HAPrompter, HARequest, HALevel};
use serde_json::json;
use std::io::{self, BufRead, Write};
use tracing::{info, debug};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("ha_prompter=debug")
        .init();

    info!("HA Prompter MCP Server starting...");

    let prompter = HAPrompter::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // MCP protocol: send capabilities
    let capabilities = json!({
        "name": "ha-prompter",
        "version": "0.1.0",
        "description": "Hierarchical Abstraction prompt generator for L1-L9 cognitive levels",
        "tools": [
            {
                "name": "compress",
                "description": "Compress content to a higher HA level",
                "parameters": {
                    "content": "string",
                    "data_type": "string",
                    "target_level": "number (1-9)",
                    "current_level": "number (1-9, optional)"
                }
            },
            {
                "name": "expand",
                "description": "Expand content from one HA level to another",
                "parameters": {
                    "content": "string",
                    "data_type": "string", 
                    "from_level": "number (1-9)",
                    "to_level": "number (1-9)"
                }
            },
            {
                "name": "cascade_down",
                "description": "Explain content from L9 down to L1",
                "parameters": {
                    "content": "string",
                    "data_type": "string"
                }
            },
            {
                "name": "cascade_up",
                "description": "Explain content from L1 up to L9",
                "parameters": {
                    "content": "string",
                    "data_type": "string"
                }
            },
            {
                "name": "analyze",
                "description": "Analyze content to determine its HA level",
                "parameters": {
                    "content": "string",
                    "data_type": "string"
                }
            }
        ]
    });

    writeln!(stdout, "{}", serde_json::to_string(&capabilities)?)?;
    stdout.flush()?;

    // Main loop - process requests
    for line in stdin.lock().lines() {
        let line = line?;
        debug!("Received: {}", line);

        if let Ok(request_json) = serde_json::from_str::<serde_json::Value>(&line) {
            let response = match request_json["tool"].as_str() {
                Some("compress") => {
                    let content = request_json["parameters"]["content"].as_str().unwrap_or("").to_string();
                    let data_type = request_json["parameters"]["data_type"].as_str().unwrap_or("text").to_string();
                    let target_level = request_json["parameters"]["target_level"].as_u64().unwrap_or(9) as u8;
                    let current_level = request_json["parameters"]["current_level"].as_u64().map(|n| n as u8);

                    if let Some(target) = HALevel::from_int(target_level) {
                        let current = current_level.and_then(HALevel::from_int);
                        let request = HARequest::Compress { content, data_type, target_level: target, current_level: current };
                        prompter.process_request(request)
                    } else {
                        continue;
                    }
                },
                Some("expand") => {
                    let content = request_json["parameters"]["content"].as_str().unwrap_or("").to_string();
                    let data_type = request_json["parameters"]["data_type"].as_str().unwrap_or("text").to_string();
                    let from_level = request_json["parameters"]["from_level"].as_u64().unwrap_or(9) as u8;
                    let to_level = request_json["parameters"]["to_level"].as_u64().unwrap_or(1) as u8;

                    if let (Some(from), Some(to)) = (HALevel::from_int(from_level), HALevel::from_int(to_level)) {
                        let request = HARequest::Expand { content, data_type, from_level: from, to_level: to };
                        prompter.process_request(request)
                    } else {
                        continue;
                    }
                },
                Some("cascade_down") => {
                    let content = request_json["parameters"]["content"].as_str().unwrap_or("").to_string();
                    let data_type = request_json["parameters"]["data_type"].as_str().unwrap_or("text").to_string();
                    let request = HARequest::CascadeDown { content, data_type };
                    prompter.process_request(request)
                },
                Some("cascade_up") => {
                    let content = request_json["parameters"]["content"].as_str().unwrap_or("").to_string();
                    let data_type = request_json["parameters"]["data_type"].as_str().unwrap_or("text").to_string();
                    let request = HARequest::CascadeUp { content, data_type };
                    prompter.process_request(request)
                },
                Some("analyze") => {
                    let content = request_json["parameters"]["content"].as_str().unwrap_or("").to_string();
                    let data_type = request_json["parameters"]["data_type"].as_str().unwrap_or("text").to_string();
                    let request = HARequest::Analyze { content, data_type };
                    prompter.process_request(request)
                },
                _ => continue,
            };

            let response_json = json!({
                "tool": request_json["tool"],
                "result": {
                    "prompt": response.prompt,
                    "metadata": response.metadata
                }
            });

            writeln!(stdout, "{}", serde_json::to_string(&response_json)?)?;
            stdout.flush()?;
        }
    }

    Ok(())
}