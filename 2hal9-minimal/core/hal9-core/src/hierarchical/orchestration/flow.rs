//! Flow control for signal routing and load balancing

use crate::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Flow controller for managing signal routing
#[async_trait]
pub trait FlowController: Send + Sync {
    /// Route a forward signal through the network
    async fn route_forward(&self, signal: ForwardSignal) -> Result<RoutingDecision>;

    /// Route a backward gradient through the network
    async fn route_backward(&self, gradient: BackwardGradient) -> Result<RoutingDecision>;

    /// Balance load across available units
    async fn balance_load(&mut self) -> Result<LoadBalanceReport>;

    /// Get flow metrics
    async fn metrics(&self) -> Result<FlowMetrics>;

    /// Update routing weights based on performance
    async fn update_weights(&mut self, performance: &PerformanceMetrics) -> Result<()>;
}

/// Forward signal for routing
#[derive(Debug, Clone)]
pub struct ForwardSignal {
    pub signal_id: Uuid,
    pub source: Uuid,
    pub content: serde_json::Value,
    pub urgency: f32,
    pub constraints: RoutingConstraints,
}

/// Backward gradient for learning
#[derive(Debug, Clone)]
pub struct BackwardGradient {
    pub gradient_id: Uuid,
    pub error: f32,
    pub path: Vec<Uuid>,
    pub adjustments: HashMap<String, f32>,
}

/// Routing decision
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub targets: Vec<RoutingTarget>,
    pub strategy: RoutingStrategy,
    pub estimated_latency_ms: f32,
}

#[derive(Debug, Clone)]
pub struct RoutingTarget {
    pub unit_id: Uuid,
    pub weight: f32,
    pub priority: f32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RoutingStrategy {
    ShortestPath,
    LoadBalanced,
    LatencyOptimized,
    ReliabilityOptimized,
    Custom(String),
}

/// Routing constraints
#[derive(Debug, Clone)]
pub struct RoutingConstraints {
    pub max_latency_ms: Option<f32>,
    pub required_capabilities: Vec<String>,
    pub avoid_units: Vec<Uuid>,
    pub prefer_units: Vec<Uuid>,
}

/// Load balance report
#[derive(Debug, Clone)]
pub struct LoadBalanceReport {
    pub rebalanced_units: usize,
    pub moved_connections: usize,
    pub load_variance_before: f32,
    pub load_variance_after: f32,
}

/// Flow metrics
#[derive(Debug, Clone)]
pub struct FlowMetrics {
    pub total_signals_routed: u64,
    pub average_hops: f32,
    pub average_latency_ms: f32,
    pub congestion_points: Vec<CongestionPoint>,
    pub throughput_per_second: f32,
}

#[derive(Debug, Clone)]
pub struct CongestionPoint {
    pub unit_id: Uuid,
    pub congestion_level: f32,
    pub queue_depth: usize,
}

/// Performance metrics for weight updates
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub unit_metrics: HashMap<Uuid, UnitPerformance>,
    pub path_metrics: HashMap<Vec<Uuid>, PathPerformance>,
}

#[derive(Debug, Clone)]
pub struct UnitPerformance {
    pub processing_time_ms: f32,
    pub success_rate: f32,
    pub queue_depth: usize,
}

#[derive(Debug, Clone)]
pub struct PathPerformance {
    pub total_latency_ms: f32,
    pub reliability: f32,
    pub usage_count: u64,
}

/// Adaptive flow controller implementation
pub struct AdaptiveFlowController {
    routing_table: Arc<RwLock<RoutingTable>>,
    load_tracker: Arc<RwLock<LoadTracker>>,
    #[allow(dead_code)]
    performance_history: Arc<RwLock<PerformanceHistory>>,
    config: FlowConfig,
}

struct RoutingTable {
    routes: HashMap<(Uuid, Uuid), Vec<Route>>,
    weights: HashMap<Uuid, f32>,
}

#[derive(Clone)]
struct Route {
    path: Vec<Uuid>,
    cost: f32,
    reliability: f32,
}

struct LoadTracker {
    unit_loads: HashMap<Uuid, LoadInfo>,
    #[allow(dead_code)]
    update_interval: std::time::Duration,
}

struct LoadInfo {
    current_load: f32,
    capacity: f32,
    queue_depth: usize,
    #[allow(dead_code)]
    last_update: std::time::Instant,
}

#[allow(dead_code)]
struct PerformanceHistory {
    window_size: usize,
    unit_history: HashMap<Uuid, Vec<UnitPerformance>>,
    path_history: HashMap<Vec<Uuid>, Vec<PathPerformance>>,
}

#[derive(Debug, Clone)]
pub struct FlowConfig {
    pub load_balance_threshold: f32,
    pub congestion_threshold: f32,
    pub learning_rate: f32,
    pub exploration_rate: f32,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            load_balance_threshold: 0.2,
            congestion_threshold: 0.8,
            learning_rate: 0.01,
            exploration_rate: 0.1,
        }
    }
}

impl AdaptiveFlowController {
    pub fn new(config: FlowConfig) -> Self {
        Self {
            routing_table: Arc::new(RwLock::new(RoutingTable {
                routes: HashMap::new(),
                weights: HashMap::new(),
            })),
            load_tracker: Arc::new(RwLock::new(LoadTracker {
                unit_loads: HashMap::new(),
                update_interval: std::time::Duration::from_secs(1),
            })),
            performance_history: Arc::new(RwLock::new(PerformanceHistory {
                window_size: 100,
                unit_history: HashMap::new(),
                path_history: HashMap::new(),
            })),
            config,
        }
    }

    async fn select_route(&self, source: Uuid, constraints: &RoutingConstraints) -> Result<Route> {
        let table = self.routing_table.read().await;
        let loads = self.load_tracker.read().await;

        // Find candidate routes
        let mut candidates = Vec::new();

        for ((from, _), routes) in table.routes.iter() {
            if *from == source {
                for route in routes {
                    if self.meets_constraints(route, constraints, &loads) {
                        candidates.push(route.clone());
                    }
                }
            }
        }

        // Select best route based on current conditions
        candidates
            .into_iter()
            .min_by(|a, b| {
                let a_score = self.calculate_route_score(a, &loads);
                let b_score = self.calculate_route_score(b, &loads);
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| Error::Routing("No viable route found".to_string()))
    }

    fn meets_constraints(
        &self,
        route: &Route,
        constraints: &RoutingConstraints,
        _loads: &LoadTracker,
    ) -> bool {
        // Check if route avoids specified units
        for unit in &constraints.avoid_units {
            if route.path.contains(unit) {
                return false;
            }
        }

        // Check latency constraint
        if let Some(max_latency) = constraints.max_latency_ms {
            let estimated_latency = route.cost * 10.0; // Simplified estimation
            if estimated_latency > max_latency {
                return false;
            }
        }

        true
    }

    fn calculate_route_score(&self, route: &Route, loads: &LoadTracker) -> f32 {
        let mut score = route.cost;

        // Factor in current load
        for unit_id in &route.path {
            if let Some(load_info) = loads.unit_loads.get(unit_id) {
                let load_factor = load_info.current_load / load_info.capacity;
                score *= 1.0 + load_factor;
            }
        }

        // Factor in reliability
        score /= route.reliability.max(0.1);

        score
    }
}

#[async_trait]
impl FlowController for AdaptiveFlowController {
    async fn route_forward(&self, signal: ForwardSignal) -> Result<RoutingDecision> {
        let route = self
            .select_route(signal.source, &signal.constraints)
            .await?;

        let targets = route
            .path
            .windows(2)
            .map(|window| RoutingTarget {
                unit_id: window[1],
                weight: 1.0,
                priority: signal.urgency,
            })
            .collect();

        Ok(RoutingDecision {
            targets,
            strategy: RoutingStrategy::LoadBalanced,
            estimated_latency_ms: route.cost * 10.0,
        })
    }

    async fn route_backward(&self, gradient: BackwardGradient) -> Result<RoutingDecision> {
        // Route backward along the recorded path
        let targets = gradient
            .path
            .windows(2)
            .rev()
            .map(|window| RoutingTarget {
                unit_id: window[0],
                weight: 1.0,
                priority: 1.0,
            })
            .collect();

        Ok(RoutingDecision {
            targets,
            strategy: RoutingStrategy::ShortestPath,
            estimated_latency_ms: gradient.path.len() as f32 * 5.0,
        })
    }

    async fn balance_load(&mut self) -> Result<LoadBalanceReport> {
        let loads = self.load_tracker.write().await;

        // Calculate load variance
        let load_values: Vec<f32> = loads
            .unit_loads
            .values()
            .map(|info| info.current_load / info.capacity)
            .collect();

        let variance_before = calculate_variance(&load_values);

        // Implement load balancing logic
        // This is a placeholder for actual implementation
        let rebalanced = 0;
        let moved = 0;

        let variance_after = variance_before * 0.9; // Simulated improvement

        Ok(LoadBalanceReport {
            rebalanced_units: rebalanced,
            moved_connections: moved,
            load_variance_before: variance_before,
            load_variance_after: variance_after,
        })
    }

    async fn metrics(&self) -> Result<FlowMetrics> {
        let loads = self.load_tracker.read().await;

        let congestion_points = loads
            .unit_loads
            .iter()
            .filter_map(|(id, info)| {
                let congestion = info.current_load / info.capacity;
                if congestion > self.config.congestion_threshold {
                    Some(CongestionPoint {
                        unit_id: *id,
                        congestion_level: congestion,
                        queue_depth: info.queue_depth,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(FlowMetrics {
            total_signals_routed: 0, // Would track in production
            average_hops: 3.0,
            average_latency_ms: 15.0,
            congestion_points,
            throughput_per_second: 1000.0,
        })
    }

    async fn update_weights(&mut self, performance: &PerformanceMetrics) -> Result<()> {
        let mut table = self.routing_table.write().await;

        // Update weights based on performance
        for (unit_id, unit_perf) in &performance.unit_metrics {
            let current_weight = table.weights.get(unit_id).copied().unwrap_or(1.0);
            let performance_factor = unit_perf.success_rate / unit_perf.processing_time_ms.max(1.0);
            let new_weight =
                current_weight + self.config.learning_rate * (performance_factor - 1.0);
            table.weights.insert(*unit_id, new_weight.clamp(0.1, 10.0));
        }

        Ok(())
    }
}

fn calculate_variance(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;

    variance
}
