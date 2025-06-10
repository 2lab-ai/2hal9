//! Database abstraction layer supporting both SQLite and PostgreSQL

use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tracing::info;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database type (sqlite or postgres)
    pub database_type: DatabaseType,

    /// Connection URL
    pub url: String,

    /// Maximum connections in pool
    pub max_connections: u32,

    /// Minimum connections to maintain
    pub min_connections: u32,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Idle timeout before closing connection
    pub idle_timeout: Duration,

    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_type: DatabaseType::Sqlite,
            url: "sqlite:data/hal9.db?mode=rwc".to_string(),
            max_connections: 100,
            min_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// Database type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl DatabaseType {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" => Some(Self::Sqlite),
            "postgres" | "postgresql" => Some(Self::Postgres),
            _ => None,
        }
    }
}

/// Database connection pool abstraction
pub enum DatabasePool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        match config.database_type {
            DatabaseType::Sqlite => {
                info!("Connecting to SQLite database: {}", config.url);

                let pool = SqlitePoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(config.connection_timeout)
                    .idle_timeout(Some(config.idle_timeout))
                    .max_lifetime(Some(config.max_lifetime))
                    .connect(&config.url)
                    .await?;

                Ok(Self::Sqlite(pool))
            }
            DatabaseType::Postgres => {
                info!("Connecting to PostgreSQL database");

                let pool = PgPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(config.connection_timeout)
                    .idle_timeout(Some(config.idle_timeout))
                    .max_lifetime(Some(config.max_lifetime))
                    .connect(&config.url)
                    .await?;

                Ok(Self::Postgres(pool))
            }
        }
    }

    /// Run migrations
    pub async fn migrate(&self) -> Result<()> {
        match self {
            Self::Sqlite(pool) => {
                info!("Running SQLite migrations");
                sqlx::migrate!("./migrations/sqlite").run(pool).await?;
            }
            Self::Postgres(pool) => {
                info!("Running PostgreSQL migrations");
                sqlx::migrate!("./migrations/postgres").run(pool).await?;
            }
        }
        Ok(())
    }

    /// Get pool metrics
    pub fn metrics(&self) -> PoolMetrics {
        match self {
            Self::Sqlite(pool) => PoolMetrics {
                size: pool.size(),
                idle: pool.num_idle() as u32,
                max_size: pool.options().get_max_connections(),
            },
            Self::Postgres(pool) => PoolMetrics {
                size: pool.size(),
                idle: pool.num_idle() as u32,
                max_size: pool.options().get_max_connections(),
            },
        }
    }

    /// Close all connections
    pub async fn close(&self) {
        match self {
            Self::Sqlite(pool) => pool.close().await,
            Self::Postgres(pool) => pool.close().await,
        }
    }
}

/// Pool metrics
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub size: u32,
    pub idle: u32,
    pub max_size: u32,
}

/// Database operations trait
#[async_trait::async_trait]
pub trait DatabaseOperations {
    /// Insert a neuron signal
    async fn insert_signal(&self, signal: &NeuronSignal) -> Result<()>;

    /// Get signal by ID
    async fn get_signal(&self, id: &str) -> Result<Option<NeuronSignal>>;

    /// Update neuron state
    async fn update_neuron_state(&self, neuron_id: &str, state: &str) -> Result<()>;

    /// Get recent signals
    async fn get_recent_signals(&self, limit: i64) -> Result<Vec<NeuronSignal>>;

    /// Clean old data
    async fn cleanup_old_data(&self, days: i64) -> Result<u64>;
}

use hal9_core::NeuronSignal;

#[async_trait::async_trait]
impl DatabaseOperations for DatabasePool {
    async fn insert_signal(&self, signal: &NeuronSignal) -> Result<()> {
        match self {
            Self::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#
                )
                .bind(signal.signal_id.to_string())
                .bind(&signal.from_neuron)
                .bind(&signal.to_neuron)
                .bind(&signal.layer_from)
                .bind(&signal.layer_to)
                .bind(&signal.payload.activation.content)
                .bind(signal.timestamp.timestamp())
                .execute(pool)
                .await?;
            }
            Self::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#
                )
                .bind(signal.signal_id)
                .bind(&signal.from_neuron)
                .bind(&signal.to_neuron)
                .bind(&signal.layer_from)
                .bind(&signal.layer_to)
                .bind(&signal.payload.activation.content)
                .bind(signal.timestamp)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn get_signal(&self, _id: &str) -> Result<Option<NeuronSignal>> {
        // Implementation would deserialize from database
        // For now, return None
        Ok(None)
    }

    async fn update_neuron_state(&self, neuron_id: &str, state: &str) -> Result<()> {
        match self {
            Self::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE neurons SET state = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                )
                .bind(state)
                .bind(neuron_id)
                .execute(pool)
                .await?;
            }
            Self::Postgres(pool) => {
                sqlx::query("UPDATE neurons SET state = $1, updated_at = NOW() WHERE id = $2")
                    .bind(state)
                    .bind(neuron_id)
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }

    async fn get_recent_signals(&self, _limit: i64) -> Result<Vec<NeuronSignal>> {
        // Implementation would query and deserialize
        Ok(vec![])
    }

    async fn cleanup_old_data(&self, days: i64) -> Result<u64> {
        let count = match self {
            Self::Sqlite(pool) => {
                let result = sqlx::query(
                    "DELETE FROM signals WHERE timestamp < datetime('now', '-' || ?1 || ' days')",
                )
                .bind(days)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
            Self::Postgres(pool) => {
                let result = sqlx::query(
                    "DELETE FROM signals WHERE timestamp < NOW() - make_interval(days => $1)",
                )
                .bind(days as i32)
                .execute(pool)
                .await?;
                result.rows_affected()
            }
        };
        Ok(count)
    }
}

/// Batch operations for performance
pub struct BatchOperations<'a> {
    pool: &'a DatabasePool,
    batch_size: usize,
}

impl<'a> BatchOperations<'a> {
    pub fn new(pool: &'a DatabasePool, batch_size: usize) -> Self {
        Self { pool, batch_size }
    }

    /// Batch insert signals
    pub async fn insert_signals(&self, signals: Vec<NeuronSignal>) -> Result<()> {
        match self.pool {
            DatabasePool::Postgres(pool) => {
                // Use COPY for PostgreSQL
                let mut tx = pool.begin().await?;

                // TODO: Implement COPY protocol
                // For now, use batch inserts
                for chunk in signals.chunks(self.batch_size) {
                    for signal in chunk {
                        sqlx::query(
                            r#"
                            INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp)
                            VALUES ($1, $2, $3, $4, $5, $6, $7)
                            "#
                        )
                        .bind(signal.signal_id)
                        .bind(&signal.from_neuron)
                        .bind(&signal.to_neuron)
                        .bind(&signal.layer_from)
                        .bind(&signal.layer_to)
                        .bind(&signal.payload.activation.content)
                        .bind(signal.timestamp)
                        .execute(&mut *tx)
                        .await?;
                    }
                }

                tx.commit().await?;
            }
            DatabasePool::Sqlite(pool) => {
                // SQLite doesn't support COPY, use transactions
                let mut tx = pool.begin().await?;

                for signal in signals {
                    sqlx::query(
                        r#"
                        INSERT INTO signals (id, from_neuron, to_neuron, layer_from, layer_to, content, timestamp)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                        "#
                    )
                    .bind(signal.signal_id.to_string())
                    .bind(&signal.from_neuron)
                    .bind(&signal.to_neuron)
                    .bind(&signal.layer_from)
                    .bind(&signal.layer_to)
                    .bind(&signal.payload.activation.content)
                    .bind(signal.timestamp.timestamp())
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_parsing() {
        assert_eq!(DatabaseType::from_str("sqlite"), Some(DatabaseType::Sqlite));
        assert_eq!(
            DatabaseType::from_str("postgres"),
            Some(DatabaseType::Postgres)
        );
        assert_eq!(
            DatabaseType::from_str("postgresql"),
            Some(DatabaseType::Postgres)
        );
        assert_eq!(DatabaseType::from_str("mysql"), None);
    }

    #[tokio::test]
    async fn test_sqlite_connection() {
        let config = DatabaseConfig {
            database_type: DatabaseType::Sqlite,
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = DatabasePool::new(&config).await.unwrap();
        let metrics = pool.metrics();
        assert!(metrics.max_size > 0);
    }
}
