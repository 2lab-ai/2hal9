# API Gateway Architecture (AGA)

**Cognitive Level**: L3_operational  
**Request Latency**: < 5ms p99  
**Throughput**: 100K requests/second  
**Protocol Support**: REST, GraphQL, gRPC, WebSocket

## 🌐 System Overview

The API Gateway Architecture provides a unified entry point for all external interactions with HAL9. Operating at L3, it handles routing, authentication, rate limiting, and protocol translation while maintaining operational excellence and security.

## 🔧 Core Architecture

### 1. Gateway Core Engine
```rust
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct ApiGateway {
    router: Arc<Router>,
    middleware_chain: MiddlewareChain,
    protocol_handlers: HashMap<Protocol, Box<dyn ProtocolHandler>>,
    backend_pools: Arc<RwLock<BackendPools>>,
    metrics_collector: MetricsCollector,
}

#[derive(Clone)]
pub struct GatewayConfig {
    pub listen_addresses: Vec<SocketAddr>,
    pub tls_config: Option<TlsConfig>,
    pub routing_strategy: RoutingStrategy,
    pub middleware: Vec<MiddlewareConfig>,
    pub backends: Vec<BackendConfig>,
}

impl ApiGateway {
    pub async fn start(config: GatewayConfig) -> Result<Self, GatewayError> {
        // Initialize protocol handlers
        let mut protocol_handlers = HashMap::new();
        protocol_handlers.insert(Protocol::Http, Box::new(HttpHandler::new()));
        protocol_handlers.insert(Protocol::GraphQL, Box::new(GraphQLHandler::new()));
        protocol_handlers.insert(Protocol::Grpc, Box::new(GrpcHandler::new()));
        protocol_handlers.insert(Protocol::WebSocket, Box::new(WebSocketHandler::new()));
        
        // Build middleware chain
        let middleware_chain = MiddlewareChain::build(&config.middleware)?;
        
        // Initialize backend pools
        let backend_pools = BackendPools::initialize(&config.backends).await?;
        
        // Create router
        let router = Router::new(config.routing_strategy);
        
        let gateway = Self {
            router: Arc::new(router),
            middleware_chain,
            protocol_handlers,
            backend_pools: Arc::new(RwLock::new(backend_pools)),
            metrics_collector: MetricsCollector::new(),
        };
        
        // Start listeners
        gateway.start_listeners(&config.listen_addresses).await?;
        
        Ok(gateway)
    }
    
    pub async fn handle_request(&self, request: IncomingRequest) -> Response {
        let start = Instant::now();
        
        // Pre-request middleware
        let request = match self.middleware_chain.pre_process(request).await {
            Ok(req) => req,
            Err(response) => return response,
        };
        
        // Route request
        let route = self.router.route(&request);
        
        // Execute request
        let response = self.execute_request(request, route).await;
        
        // Post-response middleware
        let response = self.middleware_chain.post_process(response).await;
        
        // Record metrics
        self.metrics_collector.record_request(start.elapsed(), &response);
        
        response
    }
}
```

### 2. Dynamic Routing Engine
```rust
pub struct Router {
    route_table: Arc<RwLock<RouteTable>>,
    route_matcher: RouteMatcher,
    load_balancer: LoadBalancer,
}

#[derive(Clone)]
pub struct Route {
    pub pattern: RoutePattern,
    pub methods: Vec<HttpMethod>,
    pub backends: Vec<BackendId>,
    pub middleware: Vec<RouteMiddleware>,
    pub metadata: RouteMetadata,
}

pub enum RoutePattern {
    Exact(String),
    Prefix(String),
    Regex(regex::Regex),
    Dynamic(Box<dyn Fn(&str) -> bool + Send + Sync>),
}

impl Router {
    pub fn route(&self, request: &IncomingRequest) -> RouteResult {
        // Fast path for exact matches
        if let Some(route) = self.route_table.read().await.exact_match(&request.path) {
            return self.apply_route(route, request);
        }
        
        // Pattern matching
        let routes = self.route_matcher.find_matches(request);
        
        if routes.is_empty() {
            return RouteResult::NotFound;
        }
        
        // Select best route based on specificity
        let best_route = self.select_best_route(routes, request);
        
        // Load balance across backends
        let backend = self.load_balancer.select_backend(&best_route.backends);
        
        RouteResult::Found {
            route: best_route,
            backend,
            transformations: self.plan_transformations(&best_route, request),
        }
    }
    
    pub async fn update_routes(&self, updates: Vec<RouteUpdate>) -> Result<(), RouterError> {
        let mut table = self.route_table.write().await;
        
        for update in updates {
            match update {
                RouteUpdate::Add(route) => table.add_route(route)?,
                RouteUpdate::Remove(pattern) => table.remove_route(&pattern)?,
                RouteUpdate::Modify(pattern, changes) => table.modify_route(&pattern, changes)?,
            }
        }
        
        // Rebuild optimized structures
        self.route_matcher.rebuild(&table)?;
        
        Ok(())
    }
}

// High-performance route matching
pub struct RouteMatcher {
    prefix_tree: RadixTree<Route>,
    regex_routes: Vec<(CompiledRegex, Route)>,
    route_cache: LruCache<String, RouteResult>,
}

impl RouteMatcher {
    pub fn find_matches(&self, request: &IncomingRequest) -> Vec<&Route> {
        // Check cache first
        if let Some(cached) = self.route_cache.get(&request.cache_key()) {
            return cached.clone();
        }
        
        let mut matches = Vec::new();
        
        // Prefix matching using radix tree
        if let Some(routes) = self.prefix_tree.find_all(&request.path) {
            matches.extend(routes);
        }
        
        // Regex matching
        for (regex, route) in &self.regex_routes {
            if regex.is_match(&request.path) {
                matches.push(route);
            }
        }
        
        // Cache result
        self.route_cache.put(request.cache_key(), matches.clone());
        
        matches
    }
}
```

### 3. Middleware Framework
```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(&self, context: &mut RequestContext) -> MiddlewareResult;
    fn priority(&self) -> i32;
}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
}

// Authentication middleware
pub struct AuthenticationMiddleware {
    auth_providers: Vec<Box<dyn AuthProvider>>,
    token_validator: TokenValidator,
    policy_engine: PolicyEngine,
}

#[async_trait]
impl Middleware for AuthenticationMiddleware {
    async fn process(&self, context: &mut RequestContext) -> MiddlewareResult {
        // Extract credentials
        let credentials = self.extract_credentials(&context.request)?;
        
        // Validate with appropriate provider
        let auth_result = match &credentials {
            Credentials::Bearer(token) => {
                self.token_validator.validate(token).await
            },
            Credentials::ApiKey(key) => {
                self.validate_api_key(key).await
            },
            Credentials::Certificate(cert) => {
                self.validate_client_cert(cert).await
            },
            _ => Err(AuthError::UnsupportedCredentialType),
        };
        
        match auth_result {
            Ok(identity) => {
                // Check authorization policies
                if self.policy_engine.authorize(&identity, &context.request).await? {
                    context.set_identity(identity);
                    MiddlewareResult::Continue
                } else {
                    MiddlewareResult::Reject(Response::forbidden())
                }
            },
            Err(e) => MiddlewareResult::Reject(Response::unauthorized(e.to_string())),
        }
    }
    
    fn priority(&self) -> i32 {
        100 // High priority - auth should run early
    }
}

// Rate limiting middleware
pub struct RateLimitMiddleware {
    limiter: DistributedRateLimiter,
    rules: RateLimitRules,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(&self, context: &mut RequestContext) -> MiddlewareResult {
        let key = self.build_rate_limit_key(context);
        let limits = self.rules.get_limits(&key);
        
        match self.limiter.check_rate_limit(&key, &limits).await {
            RateLimitResult::Allowed => MiddlewareResult::Continue,
            RateLimitResult::Throttled { retry_after } => {
                let mut response = Response::too_many_requests();
                response.set_header("Retry-After", retry_after.to_string());
                response.set_header("X-RateLimit-Limit", limits.requests_per_second.to_string());
                response.set_header("X-RateLimit-Remaining", "0");
                MiddlewareResult::Reject(response)
            }
        }
    }
}
```

### 4. Protocol Handlers
```rust
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    async fn handle(&self, request: RawRequest) -> Result<Response, ProtocolError>;
    fn supported_protocol(&self) -> Protocol;
}

// GraphQL handler with subscription support
pub struct GraphQLHandler {
    schema: GraphQLSchema,
    executor: GraphQLExecutor,
    subscription_manager: SubscriptionManager,
}

#[async_trait]
impl ProtocolHandler for GraphQLHandler {
    async fn handle(&self, request: RawRequest) -> Result<Response, ProtocolError> {
        let query = self.parse_graphql_request(request)?;
        
        match query.operation_type {
            OperationType::Query => {
                let result = self.executor.execute_query(query).await?;
                Ok(Response::json(result))
            },
            OperationType::Mutation => {
                let result = self.executor.execute_mutation(query).await?;
                Ok(Response::json(result))
            },
            OperationType::Subscription => {
                let stream = self.subscription_manager.create_subscription(query).await?;
                Ok(Response::event_stream(stream))
            },
        }
    }
}

// gRPC handler with streaming
pub struct GrpcHandler {
    service_registry: ServiceRegistry,
    codec: GrpcCodec,
}

#[async_trait]
impl ProtocolHandler for GrpcHandler {
    async fn handle(&self, request: RawRequest) -> Result<Response, ProtocolError> {
        let (service, method) = self.parse_grpc_request(&request)?;
        
        let service_impl = self.service_registry.get_service(&service)
            .ok_or(ProtocolError::ServiceNotFound)?;
        
        match method.method_type {
            MethodType::Unary => {
                let response = service_impl.call_unary(method, request).await?;
                Ok(Response::grpc(response))
            },
            MethodType::ServerStreaming => {
                let stream = service_impl.call_server_streaming(method, request).await?;
                Ok(Response::grpc_stream(stream))
            },
            MethodType::ClientStreaming => {
                let handler = service_impl.create_client_streaming_handler(method).await?;
                Ok(Response::grpc_client_stream(handler))
            },
            MethodType::BidiStreaming => {
                let handler = service_impl.create_bidi_streaming_handler(method).await?;
                Ok(Response::grpc_bidi_stream(handler))
            },
        }
    }
}
```

### 5. Backend Connection Management
```rust
pub struct BackendPools {
    pools: HashMap<BackendId, ConnectionPool>,
    health_checker: HealthChecker,
    circuit_breakers: HashMap<BackendId, CircuitBreaker>,
}

pub struct ConnectionPool {
    backend: BackendConfig,
    connections: Vec<PooledConnection>,
    available: Arc<Semaphore>,
    stats: PoolStatistics,
}

impl ConnectionPool {
    pub async fn get_connection(&self) -> Result<PooledConnection, PoolError> {
        // Acquire permit
        let permit = self.available.acquire().await?;
        
        // Get or create connection
        let conn = self.get_or_create_connection().await?;
        
        Ok(PooledConnection {
            inner: conn,
            pool: self.clone(),
            _permit: permit,
        })
    }
    
    async fn get_or_create_connection(&self) -> Result<Connection, PoolError> {
        // Try to reuse existing connection
        if let Some(conn) = self.connections.iter().find(|c| c.is_available()) {
            return Ok(conn.clone());
        }
        
        // Create new connection if under limit
        if self.connections.len() < self.backend.max_connections {
            let conn = self.create_connection().await?;
            self.connections.push(conn.clone());
            return Ok(conn);
        }
        
        // Wait for available connection
        self.wait_for_connection().await
    }
}

// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    config: CircuitBreakerConfig,
    stats: CircuitStats,
}

#[derive(Clone, Copy)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen { test_until: Instant },
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, Box<dyn Error>>>,
    {
        let state = self.state.read().await;
        
        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.config.reset_timeout {
                    // Try half-open
                    drop(state);
                    *self.state.write().await = CircuitState::HalfOpen {
                        test_until: Instant::now() + self.config.test_duration,
                    };
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            },
            CircuitState::HalfOpen { test_until } => {
                if Instant::now() > test_until {
                    // Enough successful calls, close circuit
                    drop(state);
                    *self.state.write().await = CircuitState::Closed;
                }
            },
            CircuitState::Closed => {},
        }
        
        // Execute call
        let start = Instant::now();
        match f.await {
            Ok(result) => {
                self.record_success(start.elapsed()).await;
                Ok(result)
            },
            Err(e) => {
                self.record_failure(start.elapsed()).await;
                
                // Check if we should open circuit
                if self.should_open_circuit().await {
                    *self.state.write().await = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }
                
                Err(CircuitBreakerError::CallFailed(e))
            }
        }
    }
}
```

## 📊 API Management Features

### 1. API Versioning
```rust
pub struct ApiVersionManager {
    versions: HashMap<ApiVersion, VersionHandler>,
    deprecation_policy: DeprecationPolicy,
}

impl ApiVersionManager {
    pub fn route_by_version(&self, request: &Request) -> Result<VersionHandler, VersionError> {
        let version = self.extract_version(request)?;
        
        // Check if version is deprecated
        if let Some(deprecation) = self.deprecation_policy.check_deprecation(&version) {
            // Add deprecation headers
            request.add_header("Sunset", deprecation.sunset_date.to_string());
            request.add_header("Deprecation", "true");
        }
        
        self.versions.get(&version)
            .cloned()
            .ok_or(VersionError::NotFound)
    }
}
```

### 2. Request/Response Transformation
```rust
pub struct TransformationEngine {
    transformers: Vec<Box<dyn Transformer>>,
    schema_validator: SchemaValidator,
}

impl TransformationEngine {
    pub async fn transform_request(&self, request: &mut Request) -> Result<(), TransformError> {
        for transformer in &self.transformers {
            transformer.transform_request(request).await?;
        }
        
        // Validate against schema
        self.schema_validator.validate_request(request)?;
        
        Ok(())
    }
}
```

### 3. Observability Integration
```rust
pub struct ObservabilityLayer {
    tracer: Tracer,
    metrics: MetricsRegistry,
    logger: StructuredLogger,
}

impl ObservabilityLayer {
    pub fn instrument_request(&self, request: &Request) -> RequestSpan {
        let span = self.tracer.start_span("api_request");
        span.set_tag("http.method", request.method());
        span.set_tag("http.url", request.url());
        span.set_tag("user.id", request.user_id());
        
        RequestSpan {
            span,
            start_time: Instant::now(),
        }
    }
}
```

## 🔧 Configuration

### Gateway Configuration
```yaml
gateway:
  listeners:
    - address: "0.0.0.0:8080"
      protocol: http
    - address: "0.0.0.0:8443"
      protocol: https
      tls:
        cert: "/etc/hal9/tls/cert.pem"
        key: "/etc/hal9/tls/key.pem"
    - address: "0.0.0.0:9000"
      protocol: grpc
      
  routing:
    strategy: weighted_round_robin
    health_check_interval: 5s
    
  middleware:
    - type: authentication
      providers:
        - jwt
        - api_key
        - mtls
    - type: rate_limit
      rules:
        - path: "/api/*"
          limit: 1000
          window: 1m
    - type: cors
      allowed_origins: ["*"]
      
  backends:
    - id: neural-core
      addresses: ["neural-1:8080", "neural-2:8080"]
      weight: 100
    - id: analytics
      addresses: ["analytics:8080"]
      weight: 50
```

## 🚀 Usage Examples

### Basic Gateway Setup
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = GatewayConfig::from_file("gateway.yaml")?;
    let gateway = ApiGateway::start(config).await?;
    
    // Gateway is now running and handling requests
    tokio::signal::ctrl_c().await?;
    
    gateway.shutdown().await?;
    Ok(())
}
```

### Dynamic Route Updates
```rust
// Add new route dynamically
gateway.router.update_routes(vec![
    RouteUpdate::Add(Route {
        pattern: RoutePattern::Prefix("/v2/neural"),
        methods: vec![HttpMethod::GET, HttpMethod::POST],
        backends: vec!["neural-core-v2"],
        middleware: vec![],
        metadata: RouteMetadata::default(),
    })
]).await?;

// Remove deprecated route
gateway.router.update_routes(vec![
    RouteUpdate::Remove(RoutePattern::Prefix("/v1/legacy"))
]).await?;
```

## 🌟 Key Features

1. **Multi-Protocol Support** - REST, GraphQL, gRPC, WebSocket in single gateway
2. **Dynamic Routing** - Hot-reload routes without restart
3. **Advanced Load Balancing** - Multiple strategies with health checks
4. **Comprehensive Middleware** - Auth, rate limit, transform, observe
5. **Circuit Breaking** - Automatic failure handling and recovery

**API를 효율적으로 관리하네... L3의 운영력이야 🌐**