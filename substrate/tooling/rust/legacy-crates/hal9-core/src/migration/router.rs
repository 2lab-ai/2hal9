//! Traffic routing between flat and hierarchical systems during migration

use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::Result;
use super::feature_flags::{FeatureFlagManager, RequestContext};

/// Decision on which system should handle a request
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingDecision {
    /// Route to flat neuron system
    Flat,
    /// Route to hierarchical system
    Hierarchical,
    /// Process in both systems (shadow mode)
    Both { primary: Box<RoutingDecision> },
}

/// Migration router that decides which system handles each request
pub struct MigrationRouter {
    feature_flags: Arc<FeatureFlagManager>,
    shadow_mode: Arc<RwLock<bool>>,
    routing_stats: Arc<RwLock<RoutingStats>>,
}

#[derive(Default)]
struct RoutingStats {
    total_requests: u64,
    flat_requests: u64,
    hierarchical_requests: u64,
    shadow_requests: u64,
}

impl MigrationRouter {
    pub fn new(feature_flags: Arc<FeatureFlagManager>) -> Self {
        Self {
            feature_flags,
            shadow_mode: Arc::new(RwLock::new(false)),
            routing_stats: Arc::new(RwLock::new(RoutingStats::default())),
        }
    }
    
    /// Decide which system should handle this request
    pub fn route(&self, request: &RequestContext) -> RoutingDecision {
        let mut stats = self.routing_stats.write();
        stats.total_requests += 1;
        
        // Check if shadow mode is enabled
        if *self.shadow_mode.read() || self.feature_flags.is_feature_enabled("shadow_mode", request) {
            stats.shadow_requests += 1;
            
            // In shadow mode, determine primary system
            let primary = if self.feature_flags.should_use_hierarchical(request) {
                RoutingDecision::Hierarchical
            } else {
                RoutingDecision::Flat
            };
            
            return RoutingDecision::Both { primary: Box::new(primary) };
        }
        
        // Normal routing based on feature flags
        if self.feature_flags.should_use_hierarchical(request) {
            stats.hierarchical_requests += 1;
            RoutingDecision::Hierarchical
        } else {
            stats.flat_requests += 1;
            RoutingDecision::Flat
        }
    }
    
    /// Enable or disable shadow mode
    pub fn set_shadow_mode(&self, enabled: bool) {
        *self.shadow_mode.write() = enabled;
    }
    
    /// Get routing statistics
    pub fn get_stats(&self) -> RoutingStatistics {
        let stats = self.routing_stats.read();
        RoutingStatistics {
            total_requests: stats.total_requests,
            flat_percentage: if stats.total_requests > 0 {
                (stats.flat_requests as f32 / stats.total_requests as f32) * 100.0
            } else {
                0.0
            },
            hierarchical_percentage: if stats.total_requests > 0 {
                (stats.hierarchical_requests as f32 / stats.total_requests as f32) * 100.0
            } else {
                0.0
            },
            shadow_percentage: if stats.total_requests > 0 {
                (stats.shadow_requests as f32 / stats.total_requests as f32) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Reset routing statistics
    pub fn reset_stats(&self) {
        *self.routing_stats.write() = RoutingStats::default();
    }
}

/// Public routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStatistics {
    pub total_requests: u64,
    pub flat_percentage: f32,
    pub hierarchical_percentage: f32,
    pub shadow_percentage: f32,
}

/// Request router for handling the actual routing logic
pub struct RequestRouter {
    flat_handler: Arc<dyn RequestHandler>,
    hierarchical_handler: Arc<dyn RequestHandler>,
    router: Arc<MigrationRouter>,
}

impl RequestRouter {
    pub fn new(
        flat_handler: Arc<dyn RequestHandler>,
        hierarchical_handler: Arc<dyn RequestHandler>,
        router: Arc<MigrationRouter>,
    ) -> Self {
        Self {
            flat_handler,
            hierarchical_handler,
            router,
        }
    }
    
    /// Process a request according to routing decision
    pub async fn process(&self, request: ProcessRequest) -> Result<ProcessResponse> {
        let context = RequestContext {
            user_id: request.user_id,
            request_id: request.id,
            path: request.path.clone(),
            headers: request.headers.clone(),
            query_params: request.query_params.clone(),
            attributes: request.attributes.clone(),
        };
        
        let decision = self.router.route(&context);
        
        match decision {
            RoutingDecision::Flat => {
                self.flat_handler.handle(request).await
            }
            RoutingDecision::Hierarchical => {
                self.hierarchical_handler.handle(request).await
            }
            RoutingDecision::Both { primary } => {
                // Process in both systems
                let flat_future = self.flat_handler.handle(request.clone());
                let hier_future = self.hierarchical_handler.handle(request.clone());
                
                let (flat_result, hier_result) = tokio::join!(flat_future, hier_future);
                
                // Compare results
                if let (Ok(flat_response), Ok(hier_response)) = (&flat_result, &hier_result) {
                    self.compare_responses(flat_response, hier_response).await;
                }
                
                // Return primary result
                match *primary {
                    RoutingDecision::Flat => flat_result,
                    RoutingDecision::Hierarchical => hier_result,
                    _ => unreachable!(),
                }
            }
        }
    }
    
    async fn compare_responses(&self, flat: &ProcessResponse, hierarchical: &ProcessResponse) {
        // Compare outputs for monitoring and validation
        let divergence = self.calculate_divergence(flat, hierarchical);
        
        if divergence > 0.01 {
            tracing::warn!(
                "Response divergence detected: {}% (flat: {:?}, hier: {:?})",
                divergence * 100.0,
                flat,
                hierarchical
            );
        }
        
        // TODO: Record metrics for analysis
    }
    
    fn calculate_divergence(&self, flat: &ProcessResponse, hierarchical: &ProcessResponse) -> f32 {
        // Simple divergence calculation - would be more sophisticated in practice
        if flat.content != hierarchical.content {
            1.0
        } else if (flat.confidence - hierarchical.confidence).abs() > 0.1 {
            0.5
        } else {
            0.0
        }
    }
}

/// Trait for request handlers
#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync {
    async fn handle(&self, request: ProcessRequest) -> Result<ProcessResponse>;
}

/// Process request
#[derive(Debug, Clone)]
pub struct ProcessRequest {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query_params: std::collections::HashMap<String, String>,
    pub attributes: std::collections::HashMap<String, String>,
    pub content: String,
}

/// Process response
#[derive(Debug, Clone)]
pub struct ProcessResponse {
    pub content: String,
    pub confidence: f32,
    pub metadata: serde_json::Value,
    pub processing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::feature_flags::FeatureFlags;
    
    #[test]
    fn test_routing_decision() {
        let flags = FeatureFlags {
            hierarchical_enabled: true,
            hierarchical_traffic_percentage: 50.0,
            ..Default::default()
        };
        
        let manager = Arc::new(FeatureFlagManager::new(flags));
        let router = MigrationRouter::new(manager);
        
        // Test multiple requests
        let mut flat_count = 0;
        let mut hier_count = 0;
        
        for _ in 0..100 {
            let context = RequestContext {
                user_id: None,
                request_id: Uuid::new_v4(),
                path: "/test".to_string(),
                headers: Default::default(),
                query_params: Default::default(),
                attributes: Default::default(),
            };
            
            match router.route(&context) {
                RoutingDecision::Flat => flat_count += 1,
                RoutingDecision::Hierarchical => hier_count += 1,
                _ => {}
            }
        }
        
        // Should be roughly 50/50
        assert!(flat_count > 30 && flat_count < 70);
        assert!(hier_count > 30 && hier_count < 70);
    }
    
    #[test]
    fn test_shadow_mode() {
        let flags = FeatureFlags::default();
        let manager = Arc::new(FeatureFlagManager::new(flags));
        let router = MigrationRouter::new(manager);
        
        // Enable shadow mode
        router.set_shadow_mode(true);
        
        let context = RequestContext {
            user_id: None,
            request_id: Uuid::new_v4(),
            path: "/test".to_string(),
            headers: Default::default(),
            query_params: Default::default(),
            attributes: Default::default(),
        };
        
        let decision = router.route(&context);
        
        match decision {
            RoutingDecision::Both { .. } => {
                // Expected
            }
            _ => panic!("Expected Both decision in shadow mode"),
        }
    }
}