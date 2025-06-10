//! Configuration management for HAL9 codegen CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub server_url: Option<String>,
    pub api_key: Option<String>,
    pub default_language: Option<String>,
    pub default_framework: Option<String>,
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;

            serde_json::from_str(&content).context("Failed to parse config file")
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;

        Ok(config_dir.join("hal9-codegen").join("config.json"))
    }
}
