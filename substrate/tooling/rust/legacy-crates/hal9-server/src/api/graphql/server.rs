use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::{Extension, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    auth::{AuthService, User},
    enterprise::{OrganizationService, TeamService},
    error::HAL9Error,
    memory_manager::MemoryManager,
    metrics::Metrics,
    neuron::NeuronManager,
    router::Router as SignalRouter,
};

use super::{
    resolvers::GraphQLContext,
    schema::{build_schema, HAL9Schema, MutationRoot, QueryRoot, SubscriptionRoot},
    subscriptions::EventBus,
};

// ============ GraphQL Handler ============

pub async fn graphql_handler(
    Extension(schema): Extension<HAL9Schema>,
    Extension(user): Extension<Option<Arc<User>>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, StatusCode> {
    let mut request = req.into_inner();

    // Add user context if authenticated
    if let Some(user) = user {
        request = request.data(user);
    }

    // Add request headers for tracing
    request = request.data(headers);

    Ok(schema.execute(request).await.into())
}

// ============ GraphQL Subscription Handler ============

pub async fn graphql_subscription_handler(
    Extension(schema): Extension<HAL9Schema>,
    Extension(user): Extension<Option<Arc<User>>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        GraphQLSubscription::new(socket, schema)
            .with_data(user)
            .serve()
    })
}

// ============ GraphQL Playground ============

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws")
            .title("HAL9 GraphQL Playground"),
    ))
}

// ============ Schema Builder with Context ============

pub fn create_graphql_schema(
    db: PgPool,
    auth_service: Arc<AuthService>,
    org_service: Arc<OrganizationService>,
    team_service: Arc<TeamService>,
    neuron_manager: Arc<RwLock<NeuronManager>>,
    router: Arc<SignalRouter>,
    memory_manager: Arc<MemoryManager>,
    metrics: Arc<Metrics>,
    event_bus: Arc<EventBus>,
) -> HAL9Schema {
    let context = Arc::new(GraphQLContext {
        db,
        auth_service,
        org_service,
        team_service,
        neuron_manager,
        router,
        memory_manager,
        metrics,
    });

    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(context)
        .data(event_bus)
        .enable_federation()
        .enable_subscription_in_federation()
        .limit_complexity(1000)
        .limit_depth(10)
        .finish()
}

// ============ GraphQL Router ============

pub fn graphql_routes(schema: HAL9Schema) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql/ws", get(graphql_subscription_handler))
        .route("/graphql/playground", get(graphql_playground))
        .layer(Extension(schema))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

// ============ GraphQL Schema Documentation ============

/// GraphQL API Documentation
///
/// # Authentication
///
/// All GraphQL requests require authentication via JWT token in the Authorization header:
/// ```
/// Authorization: Bearer <token>
/// ```
///
/// # Query Examples
///
/// ## Send a signal
/// ```graphql
/// mutation SendSignal($input: SignalInput!) {
///   sendSignal(input: $input) {
///     id
///     content
///     status
///     result
///   }
/// }
/// ```
///
/// ## List neurons
/// ```graphql
/// query ListNeurons($layer: String) {
///   neurons(layer: $layer) {
///     edges {
///       node {
///         id
///         name
///         state
///         metrics {
///           processedCount
///           errorCount
///           successRate
///         }
///       }
///     }
///     pageInfo {
///       hasNextPage
///       totalCount
///     }
///   }
/// }
/// ```
///
/// ## Subscribe to updates
/// ```graphql
/// subscription SignalUpdates($signalId: ID) {
///   signalUpdates(signalId: $signalId) {
///     signalId
///     status
///     progress
///     message
///   }
/// }
/// ```
///
/// # Error Handling
///
/// Errors are returned in the standard GraphQL error format:
/// ```json
/// {
///   "errors": [{
///     "message": "Error message",
///     "extensions": {
///       "code": "ERROR_CODE"
///     }
///   }]
/// }
/// ```
pub struct GraphQLDocs;

// ============ Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{Request, Response};

    #[tokio::test]
    async fn test_schema_creation() {
        let schema = build_schema();

        let query = r#"
            {
                __schema {
                    queryType {
                        name
                    }
                    mutationType {
                        name
                    }
                    subscriptionType {
                        name
                    }
                }
            }
        "#;

        let response: Response = schema.execute(Request::new(query)).await;
        assert!(response.is_ok());
    }
}
