use async_graphql::{Context, FieldResult, ID};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    auth::{AuthService, User},
    enterprise::{Organization, OrganizationService, Team, TeamService},
    error::HAL9Error,
    memory_manager::MemoryManager,
    metrics::Metrics,
    neuron::{NeuronManager, NeuronState},
    router::Router,
    signal::Signal,
};

use super::schema::*;

// ============ Context Extensions ============

pub struct GraphQLContext {
    pub db: PgPool,
    pub auth_service: Arc<AuthService>,
    pub org_service: Arc<OrganizationService>,
    pub team_service: Arc<TeamService>,
    pub neuron_manager: Arc<RwLock<NeuronManager>>,
    pub router: Arc<Router>,
    pub memory_manager: Arc<MemoryManager>,
    pub metrics: Arc<Metrics>,
}

// ============ Query Resolvers ============

impl QueryRoot {
    pub async fn resolve_send_signal(
        &self,
        ctx: &Context<'_>,
        input: SignalInput,
    ) -> FieldResult<SignalResponse> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let user = ctx.data::<Arc<User>>()?;

        // Create signal
        let signal = Signal {
            id: Uuid::new_v4(),
            content: input.content,
            source: format!("user:{}", user.id),
            target: Some(input.layer.clone()),
            signal_type: "user_query".to_string(),
            priority: input.priority.unwrap_or(5),
            metadata: input.metadata,
            created_at: Utc::now(),
            processed_at: None,
        };

        // Route signal through system
        let result = context.router.route_signal(signal.clone()).await?;

        // Convert to GraphQL response
        Ok(SignalResponse {
            id: ID(signal.id.to_string()),
            signal_id: signal.id,
            content: signal.content,
            layer: input.layer,
            priority: signal.priority,
            status: "completed".to_string(),
            created_at: signal.created_at,
            processed_at: Some(Utc::now()),
            result: Some(result),
        })
    }

    pub async fn resolve_signal(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> FieldResult<Option<SignalResponse>> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let signal_id = Uuid::parse_str(&id.0)?;

        // Query database for signal
        let signal = sqlx::query_as!(
            SignalRecord,
            r#"
            SELECT id, content, source, target, signal_type, priority,
                   metadata, created_at, processed_at, result
            FROM signals
            WHERE id = $1
            "#,
            signal_id
        )
        .fetch_optional(&context.db)
        .await?;

        Ok(signal.map(|s| SignalResponse {
            id: ID(s.id.to_string()),
            signal_id: s.id,
            content: s.content,
            layer: s.target.unwrap_or_default(),
            priority: s.priority,
            status: if s.processed_at.is_some() {
                "completed"
            } else {
                "pending"
            }
            .to_string(),
            created_at: s.created_at,
            processed_at: s.processed_at,
            result: s.result,
        }))
    }

    pub async fn resolve_neurons(
        &self,
        ctx: &Context<'_>,
        layer: Option<String>,
        state: Option<String>,
        pagination: Option<PaginationInput>,
    ) -> FieldResult<NeuronConnection> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let limit = pagination
            .as_ref()
            .and_then(|p| p.limit)
            .unwrap_or(20)
            .min(100);
        let offset = pagination.as_ref().and_then(|p| p.offset).unwrap_or(0);

        let neuron_manager = context.neuron_manager.read().await;
        let neurons = neuron_manager.list_neurons();

        // Filter neurons
        let filtered: Vec<_> = neurons
            .into_iter()
            .filter(|n| layer.as_ref().map_or(true, |l| &n.layer == l))
            .filter(|n| state.as_ref().map_or(true, |s| n.state.to_string() == *s))
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        let total_count = filtered.len() as i32;
        let has_next_page = total_count > limit;

        let edges: Vec<NeuronEdge> = filtered
            .into_iter()
            .enumerate()
            .map(|(idx, neuron)| {
                let metrics = neuron_manager
                    .get_neuron_metrics(&neuron.id)
                    .unwrap_or_default();

                NeuronEdge {
                    cursor: base64::encode(format!("neuron:{}", offset + idx as i32)),
                    node: NeuronInfo {
                        id: ID(neuron.id.to_string()),
                        neuron_id: neuron.id,
                        name: neuron.name.clone(),
                        neuron_type: neuron.neuron_type.clone(),
                        layer: neuron.layer.clone(),
                        state: neuron.state.to_string(),
                        config: neuron.config.clone(),
                        metrics: NeuronMetrics {
                            processed_count: metrics.processed_count,
                            error_count: metrics.error_count,
                            average_latency_ms: metrics.average_latency_ms,
                            success_rate: metrics.success_rate,
                            last_activity: metrics.last_activity,
                        },
                        created_at: neuron.created_at,
                        updated_at: neuron.updated_at,
                    },
                }
            })
            .collect();

        Ok(NeuronConnection {
            edges,
            page_info: PageInfo {
                has_next_page,
                has_previous_page: offset > 0,
                start_cursor: edges.first().map(|e| e.cursor.clone()),
                end_cursor: edges.last().map(|e| e.cursor.clone()),
                total_count: Some(total_count),
            },
        })
    }

    pub async fn resolve_system_metrics(&self, ctx: &Context<'_>) -> FieldResult<SystemMetrics> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let metrics = context.metrics.get_system_metrics().await?;

        Ok(SystemMetrics {
            total_neurons: metrics.total_neurons,
            active_neurons: metrics.active_neurons,
            signals_processed: metrics.signals_processed,
            average_response_time_ms: metrics.average_response_time_ms,
            uptime_seconds: metrics.uptime_seconds,
            memory_usage_mb: metrics.memory_usage_mb,
            cpu_usage_percent: metrics.cpu_usage_percent,
        })
    }

    pub async fn resolve_search_memory(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> FieldResult<Vec<MemoryEntry>> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let results = context
            .memory_manager
            .search(&query, limit.unwrap_or(10) as usize)
            .await?;

        Ok(results
            .into_iter()
            .map(|entry| MemoryEntry {
                id: ID(entry.id.to_string()),
                key: entry.key,
                content: entry.content,
                embedding_similarity: entry.similarity,
                access_count: entry.access_count,
                created_at: entry.created_at,
                last_accessed: entry.last_accessed,
            })
            .collect())
    }
}

// ============ Mutation Resolvers ============

impl MutationRoot {
    pub async fn resolve_create_neuron(
        &self,
        ctx: &Context<'_>,
        input: CreateNeuronInput,
    ) -> FieldResult<NeuronInfo> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let user = ctx.data::<Arc<User>>()?;

        // Check permissions
        if !user.has_permission("neurons.create") {
            return Err("Insufficient permissions".into());
        }

        let mut neuron_manager = context.neuron_manager.write().await;
        let neuron = neuron_manager
            .create_neuron(input.name, input.neuron_type, input.layer, input.config)
            .await?;

        let metrics = neuron_manager
            .get_neuron_metrics(&neuron.id)
            .unwrap_or_default();

        Ok(NeuronInfo {
            id: ID(neuron.id.to_string()),
            neuron_id: neuron.id,
            name: neuron.name,
            neuron_type: neuron.neuron_type,
            layer: neuron.layer,
            state: neuron.state.to_string(),
            config: neuron.config,
            metrics: NeuronMetrics {
                processed_count: metrics.processed_count,
                error_count: metrics.error_count,
                average_latency_ms: metrics.average_latency_ms,
                success_rate: metrics.success_rate,
                last_activity: metrics.last_activity,
            },
            created_at: neuron.created_at,
            updated_at: neuron.updated_at,
        })
    }

    pub async fn resolve_update_neuron(
        &self,
        ctx: &Context<'_>,
        input: UpdateNeuronInput,
    ) -> FieldResult<NeuronInfo> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let user = ctx.data::<Arc<User>>()?;

        // Check permissions
        if !user.has_permission("neurons.update") {
            return Err("Insufficient permissions".into());
        }

        let neuron_id = Uuid::parse_str(&input.id.0)?;
        let mut neuron_manager = context.neuron_manager.write().await;

        let neuron = neuron_manager
            .update_neuron(neuron_id, input.name, input.config, input.enabled)
            .await?;

        let metrics = neuron_manager
            .get_neuron_metrics(&neuron.id)
            .unwrap_or_default();

        Ok(NeuronInfo {
            id: ID(neuron.id.to_string()),
            neuron_id: neuron.id,
            name: neuron.name,
            neuron_type: neuron.neuron_type,
            layer: neuron.layer,
            state: neuron.state.to_string(),
            config: neuron.config,
            metrics: NeuronMetrics {
                processed_count: metrics.processed_count,
                error_count: metrics.error_count,
                average_latency_ms: metrics.average_latency_ms,
                success_rate: metrics.success_rate,
                last_activity: metrics.last_activity,
            },
            created_at: neuron.created_at,
            updated_at: neuron.updated_at,
        })
    }

    pub async fn resolve_trigger_learning(
        &self,
        ctx: &Context<'_>,
        layer: Option<String>,
    ) -> FieldResult<bool> {
        let context = ctx.data::<Arc<GraphQLContext>>()?;
        let user = ctx.data::<Arc<User>>()?;

        // Check permissions
        if !user.has_permission("learning.trigger") {
            return Err("Insufficient permissions".into());
        }

        // Trigger learning cycle
        context
            .neuron_manager
            .write()
            .await
            .trigger_learning(layer)
            .await?;

        Ok(true)
    }
}

// ============ Helper Types ============

#[derive(Debug, Clone)]
struct SignalRecord {
    id: Uuid,
    content: String,
    source: String,
    target: Option<String>,
    signal_type: String,
    priority: i32,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
    result: Option<serde_json::Value>,
}

#[derive(Debug, Default)]
struct NeuronMetricsData {
    processed_count: i64,
    error_count: i64,
    average_latency_ms: f64,
    success_rate: f64,
    last_activity: Option<DateTime<Utc>>,
}
