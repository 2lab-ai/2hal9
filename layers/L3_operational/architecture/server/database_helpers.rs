//! Database helpers for handling cross-database compatibility

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use sqlx::{Postgres, Sqlite, Database};

/// Helper trait for handling JSON serialization across databases
pub trait JsonCompatible {
    type Output;
    
    fn to_db_json<T: Serialize>(value: &T) -> Result<Self::Output, serde_json::Error>;
    fn from_db_json<T: for<'de> Deserialize<'de>>(value: Self::Output) -> Result<T, serde_json::Error>;
}

/// PostgreSQL JSON handling
pub struct PostgresJson;

impl JsonCompatible for PostgresJson {
    type Output = JsonValue;
    
    fn to_db_json<T: Serialize>(value: &T) -> Result<Self::Output, serde_json::Error> {
        serde_json::to_value(value)
    }
    
    fn from_db_json<T: for<'de> Deserialize<'de>>(value: Self::Output) -> Result<T, serde_json::Error> {
        serde_json::from_value(value)
    }
}

/// SQLite JSON handling (stored as TEXT)
pub struct SqliteJson;

impl JsonCompatible for SqliteJson {
    type Output = String;
    
    fn to_db_json<T: Serialize>(value: &T) -> Result<Self::Output, serde_json::Error> {
        serde_json::to_string(value)
    }
    
    fn from_db_json<T: for<'de> Deserialize<'de>>(value: Self::Output) -> Result<T, serde_json::Error> {
        serde_json::from_str(&value)
    }
}

/// Macro for handling database-specific queries
#[macro_export]
macro_rules! db_query {
    (postgres: $pg_query:expr, sqlite: $sq_query:expr) => {
        match std::env::var("DATABASE_URL").ok().as_deref() {
            Some(url) if url.starts_with("postgres") => $pg_query,
            _ => $sq_query,
        }
    };
}

/// Macro for binding JSON values
#[macro_export]
macro_rules! bind_json {
    ($query:expr, $value:expr, postgres) => {
        $query.bind(sqlx::types::Json($value))
    };
    ($query:expr, $value:expr, sqlite) => {
        $query.bind(serde_json::to_string($value).unwrap())
    };
}