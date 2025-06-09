use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use super::api::{PluginMetadata, PluginCapability};

// ============ Plugin Registry ============

pub struct PluginRegistry {
    db: Option<PgPool>,
    cache: HashMap<Uuid, PluginPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackage {
    pub metadata: PluginMetadata,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_url: String,
    pub download_count: i64,
    pub rating: f32,
    pub verified: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchQuery {
    pub query: Option<String>,
    pub layer: Option<String>,
    pub capability_type: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub min_rating: Option<f32>,
    pub verified_only: bool,
    pub sort_by: SortBy,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    Recent,
    Name,
}

impl Default for PluginSearchQuery {
    fn default() -> Self {
        Self {
            query: None,
            layer: None,
            capability_type: None,
            author: None,
            tags: Vec::new(),
            min_rating: None,
            verified_only: false,
            sort_by: SortBy::Relevance,
            limit: 20,
            offset: 0,
        }
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            db: None,
            cache: HashMap::new(),
        }
    }
    
    pub fn with_database(db: PgPool) -> Self {
        Self {
            db: Some(db),
            cache: HashMap::new(),
        }
    }
    
    /// Register a new plugin
    pub async fn register_plugin(&self, plugin_id: Uuid) -> Result<()> {
        if let Some(db) = &self.db {
            sqlx::query!(
                r#"
                INSERT INTO plugin_registry (id, registered_at)
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET
                    last_seen = EXCLUDED.registered_at
                "#,
                plugin_id,
                Utc::now()
            )
            .execute(db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Unregister a plugin
    pub async fn unregister_plugin(&self, plugin_id: Uuid) -> Result<()> {
        if let Some(db) = &self.db {
            sqlx::query!(
                "UPDATE plugin_registry SET active = false WHERE id = $1",
                plugin_id
            )
            .execute(db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Publish a plugin package
    pub async fn publish_package(&mut self, package: PluginPackage) -> Result<()> {
        let plugin_id = package.metadata.id;
        
        if let Some(db) = &self.db {
            sqlx::query!(
                r#"
                INSERT INTO plugin_packages (
                    id, metadata, published_at, updated_at,
                    download_url, download_count, rating, verified, tags
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE SET
                    metadata = EXCLUDED.metadata,
                    updated_at = EXCLUDED.updated_at,
                    download_url = EXCLUDED.download_url,
                    tags = EXCLUDED.tags
                "#,
                plugin_id,
                serde_json::to_value(&package.metadata)?,
                package.published_at,
                package.updated_at,
                package.download_url,
                package.download_count,
                package.rating,
                package.verified,
                &package.tags
            )
            .execute(db)
            .await?;
        }
        
        // Update cache
        self.cache.insert(plugin_id, package);
        
        Ok(())
    }
    
    /// Search for plugins
    pub async fn search_plugins(&self, query: PluginSearchQuery) -> Result<Vec<PluginPackage>> {
        if let Some(db) = &self.db {
            // Build SQL query dynamically
            let mut sql = String::from(
                "SELECT * FROM plugin_packages WHERE active = true"
            );
            let mut conditions = Vec::new();
            
            if let Some(q) = &query.query {
                conditions.push(format!(
                    "(metadata->>'name' ILIKE '%{}%' OR metadata->>'description' ILIKE '%{}%')",
                    q, q
                ));
            }
            
            if let Some(layer) = &query.layer {
                conditions.push(format!(
                    "metadata->'capabilities' @> '[{{\"layer\": \"{}\"}}]'",
                    layer
                ));
            }
            
            if let Some(author) = &query.author {
                conditions.push(format!(
                    "metadata->>'author' = '{}'",
                    author
                ));
            }
            
            if !query.tags.is_empty() {
                conditions.push(format!(
                    "tags && ARRAY[{}]",
                    query.tags.iter()
                        .map(|t| format!("'{}'", t))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            
            if let Some(min_rating) = query.min_rating {
                conditions.push(format!("rating >= {}", min_rating));
            }
            
            if query.verified_only {
                conditions.push("verified = true".to_string());
            }
            
            if !conditions.is_empty() {
                sql.push_str(" AND ");
                sql.push_str(&conditions.join(" AND "));
            }
            
            // Add sorting
            sql.push_str(" ORDER BY ");
            match query.sort_by {
                SortBy::Downloads => sql.push_str("download_count DESC"),
                SortBy::Rating => sql.push_str("rating DESC"),
                SortBy::Recent => sql.push_str("updated_at DESC"),
                SortBy::Name => sql.push_str("metadata->>'name' ASC"),
                SortBy::Relevance => {
                    if query.query.is_some() {
                        // Simple relevance based on name match
                        sql.push_str("CASE WHEN metadata->>'name' ILIKE $1 THEN 0 ELSE 1 END, download_count DESC");
                    } else {
                        sql.push_str("download_count DESC");
                    }
                }
            }
            
            sql.push_str(&format!(" LIMIT {} OFFSET {}", query.limit, query.offset));
            
            // Execute query
            // Note: This is a simplified version. In production, use parameterized queries
            let rows = sqlx::query_as::<_, PluginPackageRow>(&sql)
                .fetch_all(db)
                .await?;
            
            Ok(rows.into_iter().map(|r| r.into()).collect())
        } else {
            // Fallback to cache search
            let mut results: Vec<_> = self.cache.values()
                .filter(|p| {
                    if let Some(q) = &query.query {
                        let q = q.to_lowercase();
                        p.metadata.name.to_lowercase().contains(&q) ||
                        p.metadata.description.to_lowercase().contains(&q)
                    } else {
                        true
                    }
                })
                .filter(|p| {
                    if let Some(layer) = &query.layer {
                        p.metadata.capabilities.iter().any(|c| match c {
                            PluginCapability::NeuronType { layer: l, .. } => l == layer,
                            _ => false,
                        })
                    } else {
                        true
                    }
                })
                .filter(|p| {
                    if let Some(author) = &query.author {
                        &p.metadata.author == author
                    } else {
                        true
                    }
                })
                .filter(|p| {
                    if let Some(min_rating) = query.min_rating {
                        p.rating >= min_rating
                    } else {
                        true
                    }
                })
                .filter(|p| !query.verified_only || p.verified)
                .cloned()
                .collect();
            
            // Sort results
            match query.sort_by {
                SortBy::Downloads => results.sort_by(|a, b| b.download_count.cmp(&a.download_count)),
                SortBy::Rating => results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
                SortBy::Recent => results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
                SortBy::Name => results.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name)),
                SortBy::Relevance => results.sort_by(|a, b| b.download_count.cmp(&a.download_count)),
            }
            
            // Apply pagination
            Ok(results.into_iter()
                .skip(query.offset)
                .take(query.limit)
                .collect())
        }
    }
    
    /// Get a specific plugin package
    pub async fn get_package(&self, plugin_id: Uuid) -> Result<Option<PluginPackage>> {
        if let Some(package) = self.cache.get(&plugin_id) {
            return Ok(Some(package.clone()));
        }
        
        if let Some(db) = &self.db {
            let row = sqlx::query_as::<_, PluginPackageRow>(
                "SELECT * FROM plugin_packages WHERE id = $1 AND active = true"
            )
            .bind(plugin_id)
            .fetch_optional(db)
            .await?;
            
            Ok(row.map(|r| r.into()))
        } else {
            Ok(None)
        }
    }
    
    /// Increment download count
    pub async fn increment_downloads(&self, plugin_id: Uuid) -> Result<()> {
        if let Some(db) = &self.db {
            sqlx::query!(
                "UPDATE plugin_packages SET download_count = download_count + 1 WHERE id = $1",
                plugin_id
            )
            .execute(db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Update plugin rating
    pub async fn update_rating(&self, plugin_id: Uuid, rating: f32) -> Result<()> {
        if let Some(db) = &self.db {
            // This is simplified. In production, you'd want to track individual ratings
            sqlx::query!(
                "UPDATE plugin_packages SET rating = $1 WHERE id = $2",
                rating,
                plugin_id
            )
            .execute(db)
            .await?;
        }
        
        Ok(())
    }
}

// ============ Database Row Types ============

#[derive(sqlx::FromRow)]
struct PluginPackageRow {
    id: Uuid,
    metadata: serde_json::Value,
    published_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    download_url: String,
    download_count: i64,
    rating: f32,
    verified: bool,
    tags: Vec<String>,
}

impl From<PluginPackageRow> for PluginPackage {
    fn from(row: PluginPackageRow) -> Self {
        Self {
            metadata: serde_json::from_value(row.metadata).unwrap_or_else(|_| {
                PluginMetadata {
                    id: row.id,
                    name: "Unknown".to_string(),
                    version: "0.0.0".to_string(),
                    author: "Unknown".to_string(),
                    description: String::new(),
                    license: String::new(),
                    repository: None,
                    homepage: None,
                    capabilities: Vec::new(),
                    requirements: Default::default(),
                }
            }),
            published_at: row.published_at,
            updated_at: row.updated_at,
            download_url: row.download_url,
            download_count: row.download_count,
            rating: row.rating,
            verified: row.verified,
            tags: row.tags,
        }
    }
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            version: "0.1.0".to_string(),
            author: String::new(),
            description: String::new(),
            license: "MIT".to_string(),
            repository: None,
            homepage: None,
            capabilities: Vec::new(),
            requirements: Default::default(),
        }
    }
}

impl Default for super::api::PluginRequirements {
    fn default() -> Self {
        Self {
            min_hal9_version: "0.1.0".to_string(),
            max_memory_mb: 64,
            required_permissions: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}