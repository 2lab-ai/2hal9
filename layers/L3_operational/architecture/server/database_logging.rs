//! Database query logging and instrumentation
//!
//! This module provides wrappers for database queries to add
//! consistent logging and performance tracking.

use anyhow::Result;
use sqlx::{Database, Executor, IntoArguments};
use std::time::Instant;
use tracing::{debug, warn, instrument};
use crate::{log_performance, logging::db_span};

/// Trait for logged database queries
#[allow(async_fn_in_trait)]
pub trait LoggedQuery<'q, DB: Database> {
    /// Execute a query with logging
    async fn execute_logged<'e, E>(self, executor: E, table: &str) -> Result<DB::QueryResult>
    where
        E: Executor<'e, Database = DB>,
        'q: 'e;
    
    /// Fetch one result with logging
    async fn fetch_one_logged<'e, E, O>(self, executor: E, table: &str) -> Result<O>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e;
    
    /// Fetch all results with logging
    async fn fetch_all_logged<'e, E, O>(self, executor: E, table: &str) -> Result<Vec<O>>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e;
    
    /// Fetch optional result with logging
    async fn fetch_optional_logged<'e, E, O>(self, executor: E, table: &str) -> Result<Option<O>>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e;
}

/// Extension trait to add logging to sqlx::Query
impl<'q, DB, A> LoggedQuery<'q, DB> for sqlx::query::Query<'q, DB, A>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
{
    #[instrument(skip(self, executor), fields(table = %table))]
    async fn execute_logged<'e, E>(self, executor: E, table: &str) -> Result<DB::QueryResult>
    where
        E: Executor<'e, Database = DB>,
        'q: 'e,
    {
        let span = db_span("execute", table);
        let _enter = span.enter();
        let start = Instant::now();
        
        debug!(
            target: "db.query",
            table = %table,
            query_type = "execute",
            "Executing database query"
        );
        
        match self.execute(executor).await {
            Ok(result) => {
                let duration = start.elapsed();
                log_performance!(
                    "db_query",
                    duration,
                    true,
                    "table" => table,
                    "query_type" => "execute"
                );
                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();
                warn!(
                    target: "db.error",
                    table = %table,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Database query failed"
                );
                log_performance!(
                    "db_query",
                    duration,
                    false,
                    "table" => table,
                    "query_type" => "execute",
                    "error" => e.to_string()
                );
                Err(e.into())
            }
        }
    }
    
    #[instrument(skip(self, executor), fields(table = %table))]
    async fn fetch_one_logged<'e, E, O>(self, executor: E, table: &str) -> Result<O>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e,
    {
        let span = db_span("fetch_one", table);
        let _enter = span.enter();
        let start = Instant::now();
        
        debug!(
            target: "db.query",
            table = %table,
            query_type = "fetch_one",
            "Fetching single row from database"
        );
        
        match self.fetch_one(executor).await {
            Ok(row) => {
                let duration = start.elapsed();
                log_performance!(
                    "db_query",
                    duration,
                    true,
                    "table" => table,
                    "query_type" => "fetch_one"
                );
                // Convert row to O using FromRow trait
                O::from_row(&row).map_err(|e| e.into())
            }
            Err(e) => {
                let duration = start.elapsed();
                warn!(
                    target: "db.error",
                    table = %table,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Failed to fetch row"
                );
                log_performance!(
                    "db_query",
                    duration,
                    false,
                    "table" => table,
                    "query_type" => "fetch_one",
                    "error" => e.to_string()
                );
                Err(e.into())
            }
        }
    }
    
    #[instrument(skip(self, executor), fields(table = %table))]
    async fn fetch_all_logged<'e, E, O>(self, executor: E, table: &str) -> Result<Vec<O>>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e,
    {
        let span = db_span("fetch_all", table);
        let _enter = span.enter();
        let start = Instant::now();
        
        debug!(
            target: "db.query",
            table = %table,
            query_type = "fetch_all",
            "Fetching all rows from database"
        );
        
        match self.fetch_all(executor).await {
            Ok(rows) => {
                let duration = start.elapsed();
                // Convert rows to Vec<O>
                let results: Result<Vec<O>> = rows.iter()
                    .map(|row| O::from_row(row))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.into());
                
                match results {
                    Ok(results) => {
                        debug!(
                            target: "db.query",
                            table = %table,
                            row_count = results.len(),
                            duration_ms = duration.as_millis(),
                            "Fetched rows successfully"
                        );
                        log_performance!(
                            "db_query",
                            duration,
                            true,
                            "table" => table,
                            "query_type" => "fetch_all",
                            "row_count" => results.len()
                        );
                        Ok(results)
                    }
                    Err(e) => Err(e)
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                warn!(
                    target: "db.error",
                    table = %table,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Failed to fetch rows"
                );
                log_performance!(
                    "db_query",
                    duration,
                    false,
                    "table" => table,
                    "query_type" => "fetch_all",
                    "error" => e.to_string()
                );
                Err(e.into())
            }
        }
    }
    
    #[instrument(skip(self, executor), fields(table = %table))]
    async fn fetch_optional_logged<'e, E, O>(self, executor: E, table: &str) -> Result<Option<O>>
    where
        E: Executor<'e, Database = DB>,
        O: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        'q: 'e,
    {
        let span = db_span("fetch_optional", table);
        let _enter = span.enter();
        let start = Instant::now();
        
        debug!(
            target: "db.query",
            table = %table,
            query_type = "fetch_optional",
            "Fetching optional row from database"
        );
        
        match self.fetch_optional(executor).await {
            Ok(maybe_row) => {
                let duration = start.elapsed();
                // Convert Option<Row> to Option<O>
                let result = match maybe_row {
                    Some(row) => {
                        let parsed: Result<O, sqlx::Error> = O::from_row(&row);
                        Some(parsed?)
                    },
                    None => None,
                };
                debug!(
                    target: "db.query",
                    table = %table,
                    found = result.is_some(),
                    duration_ms = duration.as_millis(),
                    "Optional fetch completed"
                );
                log_performance!(
                    "db_query",
                    duration,
                    true,
                    "table" => table,
                    "query_type" => "fetch_optional",
                    "found" => result.is_some()
                );
                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();
                warn!(
                    target: "db.error",
                    table = %table,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Failed to fetch optional row"
                );
                log_performance!(
                    "db_query",
                    duration,
                    false,
                    "table" => table,
                    "query_type" => "fetch_optional",
                    "error" => e.to_string()
                );
                Err(e.into())
            }
        }
    }
}

/// Helper functions for common database operations with logging
pub mod helpers {
    use super::*;
    use sqlx::Pool;
    
    /// Log connection pool metrics
    pub fn log_pool_metrics<DB: Database>(pool: &Pool<DB>, pool_name: &str) {
        debug!(
            target: "db.pool",
            pool_name = %pool_name,
            size = pool.size(),
            idle = pool.num_idle(),
            max_size = pool.options().get_max_connections(),
            "Database pool metrics"
        );
    }
    
    /// Log transaction start
    pub fn log_transaction_start(tx_id: &str) {
        debug!(
            target: "db.transaction",
            tx_id = %tx_id,
            "Starting database transaction"
        );
    }
    
    /// Log transaction commit
    pub fn log_transaction_commit(tx_id: &str, duration: std::time::Duration) {
        debug!(
            target: "db.transaction",
            tx_id = %tx_id,
            duration_ms = duration.as_millis(),
            "Transaction committed successfully"
        );
        log_performance!(
            "db_transaction",
            duration,
            true,
            "tx_id" => tx_id,
            "operation" => "commit"
        );
    }
    
    /// Log transaction rollback
    pub fn log_transaction_rollback(tx_id: &str, duration: std::time::Duration, reason: &str) {
        warn!(
            target: "db.transaction",
            tx_id = %tx_id,
            duration_ms = duration.as_millis(),
            reason = %reason,
            "Transaction rolled back"
        );
        log_performance!(
            "db_transaction",
            duration,
            false,
            "tx_id" => tx_id,
            "operation" => "rollback",
            "reason" => reason
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_db_span_creation() {
        let span = db_span("select", "users");
        assert_eq!(span.metadata().unwrap().name(), "db_query");
    }
}