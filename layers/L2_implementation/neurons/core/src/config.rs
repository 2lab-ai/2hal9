//! HAL9 Configuration Management
//! 
//! Handles environment-based configuration with validation
//! and type-safe access to configuration values.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// Main configuration structure for HAL9
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub claude: ClaudeConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub monitoring: MonitoringConfig,
    pub game: GameConfig,
    pub features: FeatureFlags,
    pub storage: StorageConfig,
    pub backup: BackupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expiry: String,
    pub refresh_token_expiry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub max_connections: u32,
    pub ssl_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: Option<String>,
    pub mode: ClaudeMode,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaudeMode {
    Real,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<PathBuf>,
    pub max_size: Option<String>,
    pub max_backups: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_origins: Vec<String>,
    pub session_secret: String,
    pub rate_limit_per_minute: u32,
    pub max_request_size: String,
    pub enable_cors: bool,
    pub cors_max_age: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: u32,
    pub neuron_pool_size: u32,
    pub compression_enabled: bool,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_interval: String,
    pub enable_profiling: bool,
    pub prometheus_enabled: Option<bool>,
    pub grafana_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub max_games_per_user: u32,
    pub max_players_per_game: u32,
    pub game_timeout_minutes: u32,
    pub default_rounds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_consciousness_module: bool,
    pub enable_self_organization: bool,
    pub enable_a2a_protocol: bool,
    pub enable_quantum_simulations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub upload_dir: PathBuf,
    pub max_upload_size: String,
    pub temp_dir: PathBuf,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub schedule: String,
    pub retention_days: u32,
    pub s3_bucket: Option<String>,
    pub encryption_key: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();
        
        // Determine environment
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
        
        let config = Config {
            server: ServerConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3456".to_string())
                    .parse()
                    .context("Invalid PORT")?,
                host: env::var("HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                environment: match environment.as_str() {
                    "production" => Environment::Production,
                    "staging" => Environment::Staging,
                    _ => Environment::Development,
                },
            },
            auth: AuthConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-key".to_string()),
                access_token_expiry: env::var("JWT_ACCESS_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "24h".to_string()),
                refresh_token_expiry: env::var("JWT_REFRESH_TOKEN_EXPIRY")
                    .unwrap_or_else(|_| "7d".to_string()),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://localhost/hal9_dev".to_string()),
                pool_size: env::var("DATABASE_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .context("Invalid DATABASE_POOL_SIZE")?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .context("Invalid DATABASE_MAX_CONNECTIONS")?,
                ssl_mode: env::var("DATABASE_SSL_MODE").ok(),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .context("Invalid REDIS_POOL_SIZE")?,
                password: env::var("REDIS_PASSWORD").ok(),
            },
            claude: ClaudeConfig {
                api_key: env::var("CLAUDE_API_KEY").ok(),
                mode: match env::var("CLAUDE_MODE")
                    .unwrap_or_else(|_| "mock".to_string())
                    .as_str() {
                    "real" => ClaudeMode::Real,
                    _ => ClaudeMode::Mock,
                },
                model: env::var("CLAUDE_MODEL")
                    .unwrap_or_else(|_| "claude-3-opus-20240229".to_string()),
                max_tokens: env::var("CLAUDE_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".to_string())
                    .parse()
                    .context("Invalid CLAUDE_MAX_TOKENS")?,
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                format: match env::var("LOG_FORMAT")
                    .unwrap_or_else(|_| "pretty".to_string())
                    .as_str() {
                    "json" => LogFormat::Json,
                    _ => LogFormat::Pretty,
                },
                file: env::var("LOG_FILE").ok().map(PathBuf::from),
                max_size: env::var("LOG_MAX_SIZE").ok(),
                max_backups: env::var("LOG_MAX_BACKUPS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            security: SecurityConfig {
                allowed_origins: env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                session_secret: env::var("SESSION_SECRET")
                    .unwrap_or_else(|_| "dev-session-secret".to_string()),
                rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .context("Invalid RATE_LIMIT_PER_MINUTE")?,
                max_request_size: env::var("MAX_REQUEST_SIZE")
                    .unwrap_or_else(|_| "10mb".to_string()),
                enable_cors: env::var("ENABLE_CORS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid ENABLE_CORS")?,
                cors_max_age: env::var("CORS_MAX_AGE")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            performance: PerformanceConfig {
                worker_threads: env::var("WORKER_THREADS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .context("Invalid WORKER_THREADS")?,
                neuron_pool_size: env::var("NEURON_POOL_SIZE")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()
                    .context("Invalid NEURON_POOL_SIZE")?,
                compression_enabled: env::var("COMPRESSION_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid COMPRESSION_ENABLED")?,
                cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .context("Invalid CACHE_TTL_SECONDS")?,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: env::var("METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid METRICS_ENABLED")?,
                metrics_port: env::var("METRICS_PORT")
                    .unwrap_or_else(|_| "9090".to_string())
                    .parse()
                    .context("Invalid METRICS_PORT")?,
                health_check_interval: env::var("HEALTH_CHECK_INTERVAL")
                    .unwrap_or_else(|_| "30s".to_string()),
                enable_profiling: env::var("ENABLE_PROFILING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .context("Invalid ENABLE_PROFILING")?,
                prometheus_enabled: env::var("PROMETHEUS_ENABLED")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                grafana_api_key: env::var("GRAFANA_API_KEY").ok(),
            },
            game: GameConfig {
                max_games_per_user: env::var("MAX_GAMES_PER_USER")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .context("Invalid MAX_GAMES_PER_USER")?,
                max_players_per_game: env::var("MAX_PLAYERS_PER_GAME")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .context("Invalid MAX_PLAYERS_PER_GAME")?,
                game_timeout_minutes: env::var("GAME_TIMEOUT_MINUTES")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .context("Invalid GAME_TIMEOUT_MINUTES")?,
                default_rounds: env::var("DEFAULT_ROUNDS")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()
                    .context("Invalid DEFAULT_ROUNDS")?,
            },
            features: FeatureFlags {
                enable_consciousness_module: env::var("ENABLE_CONSCIOUSNESS_MODULE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid ENABLE_CONSCIOUSNESS_MODULE")?,
                enable_self_organization: env::var("ENABLE_SELF_ORGANIZATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid ENABLE_SELF_ORGANIZATION")?,
                enable_a2a_protocol: env::var("ENABLE_A2A_PROTOCOL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .context("Invalid ENABLE_A2A_PROTOCOL")?,
                enable_quantum_simulations: env::var("ENABLE_QUANTUM_SIMULATIONS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .context("Invalid ENABLE_QUANTUM_SIMULATIONS")?,
            },
            storage: StorageConfig {
                upload_dir: env::var("UPLOAD_DIR")
                    .unwrap_or_else(|_| "/tmp/hal9/uploads".to_string())
                    .into(),
                max_upload_size: env::var("MAX_UPLOAD_SIZE")
                    .unwrap_or_else(|_| "50mb".to_string()),
                temp_dir: env::var("TEMP_DIR")
                    .unwrap_or_else(|_| "/tmp/hal9".to_string())
                    .into(),
                s3_bucket: env::var("S3_BUCKET").ok(),
                s3_region: env::var("S3_REGION").ok(),
                s3_access_key: env::var("S3_ACCESS_KEY").ok(),
                s3_secret_key: env::var("S3_SECRET_KEY").ok(),
            },
            backup: BackupConfig {
                enabled: env::var("BACKUP_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .context("Invalid BACKUP_ENABLED")?,
                schedule: env::var("BACKUP_SCHEDULE")
                    .unwrap_or_else(|_| "0 2 * * *".to_string()),
                retention_days: env::var("BACKUP_RETENTION_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .context("Invalid BACKUP_RETENTION_DAYS")?,
                s3_bucket: env::var("BACKUP_S3_BUCKET").ok(),
                encryption_key: env::var("BACKUP_ENCRYPTION_KEY").ok(),
            },
        };
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate JWT secret in production
        if matches!(self.server.environment, Environment::Production) {
            if self.auth.jwt_secret == "dev-secret-key" {
                anyhow::bail!("JWT_SECRET must be set in production");
            }
            if self.security.session_secret == "dev-session-secret" {
                anyhow::bail!("SESSION_SECRET must be set in production");
            }
        }
        
        // Validate port range
        if self.server.port == 0 {
            anyhow::bail!("PORT must be greater than 0");
        }
        
        // Validate worker threads
        if self.performance.worker_threads > 256 {
            anyhow::bail!("WORKER_THREADS too high (max 256)");
        }
        
        Ok(())
    }
    
    /// Check if running in production
    pub fn is_production(&self) -> bool {
        matches!(self.server.environment, Environment::Production)
    }
    
    /// Check if running in development
    pub fn is_development(&self) -> bool {
        matches!(self.server.environment, Environment::Development)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        // Clear env vars
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("DATABASE_URL");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.server.port, 3456);
        assert_eq!(config.auth.jwt_secret, "dev-secret-key");
        assert!(config.is_development());
    }
    
    #[test]
    fn test_production_validation() {
        std::env::set_var("ENVIRONMENT", "production");
        std::env::set_var("JWT_SECRET", "dev-secret-key");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        // Clean up
        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("JWT_SECRET");
    }
}