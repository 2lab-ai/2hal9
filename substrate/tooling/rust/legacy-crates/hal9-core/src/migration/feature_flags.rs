//! Feature flag system for gradual rollout of hierarchical architecture
//!
//! This module provides a flexible feature flag system that allows:
//! - Gradual traffic routing between flat and hierarchical systems
//! - Per-feature toggles for specific capabilities
//! - User/request-based targeting
//! - Real-time configuration updates
//! - Automatic rollback on errors

use crate::{Error, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Main feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Master switch for hierarchical system
    pub hierarchical_enabled: bool,

    /// Percentage of traffic to route to hierarchical (0.0 - 100.0)
    pub hierarchical_traffic_percentage: f32,

    /// Enable automatic rollback on error threshold
    pub auto_rollback_on_error: bool,

    /// Individual feature toggles
    pub features: HashMap<String, FeatureConfig>,

    /// User targeting rules
    pub targeting_rules: Vec<TargetingRule>,

    /// Rollback configuration
    pub rollback_config: RollbackConfig,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            hierarchical_enabled: false,
            hierarchical_traffic_percentage: 0.0,
            auto_rollback_on_error: true,
            features: Self::default_features(),
            targeting_rules: Vec::new(),
            rollback_config: RollbackConfig::default(),
        }
    }
}

impl FeatureFlags {
    fn default_features() -> HashMap<String, FeatureConfig> {
        let mut features = HashMap::new();

        // Core hierarchical features
        features.insert(
            "hierarchical_routing".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features.insert(
            "cognitive_layers".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features.insert(
            "protocol_layer".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features.insert(
            "substrate_layer".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features.insert(
            "intelligence_layer".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        // Migration features
        features.insert(
            "shadow_mode".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features.insert(
            "state_migration".to_string(),
            FeatureConfig {
                enabled: false,
                percentage: 0.0,
                conditions: vec![],
            },
        );

        features
    }
}

/// Configuration for individual features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Is this feature enabled
    pub enabled: bool,

    /// Percentage rollout (0.0 - 100.0)
    pub percentage: f32,

    /// Additional conditions for enabling
    pub conditions: Vec<FeatureCondition>,
}

/// Conditions for feature activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureCondition {
    /// Time-based condition
    After(chrono::DateTime<chrono::Utc>),
    Before(chrono::DateTime<chrono::Utc>),

    /// User-based condition
    UserInList(Vec<Uuid>),
    UserMatchesPattern(String),

    /// Request-based condition
    HeaderPresent(String, String),
    QueryParamPresent(String, String),

    /// System-based condition
    LoadBelow(f32),
    ErrorRateBelow(f32),
}

/// Targeting rules for specific users or requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingRule {
    pub name: String,
    pub priority: i32,
    pub conditions: Vec<TargetCondition>,
    pub action: TargetAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetCondition {
    UserAttribute(String, String),
    RequestPath(String),
    TimeWindow(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    RandomPercentage(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetAction {
    ForceHierarchical,
    ForceFlat,
    UsePercentage(f32),
}

/// Rollback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    /// Error rate threshold for automatic rollback (0.0 - 1.0)
    pub error_threshold: f32,

    /// Time window for error rate calculation
    pub error_window: std::time::Duration,

    /// Cooldown period after rollback
    pub cooldown_period: std::time::Duration,

    /// Maximum rollback attempts
    pub max_rollback_attempts: u32,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            error_threshold: 0.05, // 5% error rate
            error_window: std::time::Duration::from_secs(60),
            cooldown_period: std::time::Duration::from_secs(300),
            max_rollback_attempts: 3,
        }
    }
}

/// Feature flag manager
pub struct FeatureFlagManager {
    flags: Arc<RwLock<FeatureFlags>>,
    evaluator: Arc<FeatureEvaluator>,
    updater: Arc<ConfigUpdater>,
    monitor: Arc<FeatureMonitor>,
}

impl FeatureFlagManager {
    pub fn new(initial_flags: FeatureFlags) -> Self {
        let flags = Arc::new(RwLock::new(initial_flags));

        Self {
            flags: flags.clone(),
            evaluator: Arc::new(FeatureEvaluator::new(flags.clone())),
            updater: Arc::new(ConfigUpdater::new(flags.clone())),
            monitor: Arc::new(FeatureMonitor::new(flags.clone())),
        }
    }

    /// Check if hierarchical system should handle this request
    pub fn should_use_hierarchical(&self, context: &RequestContext) -> bool {
        self.evaluator.evaluate(context)
    }

    /// Check if a specific feature is enabled
    pub fn is_feature_enabled(&self, feature: &str, context: &RequestContext) -> bool {
        self.evaluator.is_feature_enabled(feature, context)
    }

    /// Update feature flags configuration
    pub async fn update_flags(&self, new_flags: FeatureFlags) -> Result<()> {
        self.updater.update(new_flags).await
    }

    /// Get current configuration
    pub fn get_flags(&self) -> FeatureFlags {
        self.flags.read().clone()
    }

    /// Start monitoring for automatic rollback
    pub fn start_monitoring(&self) {
        self.monitor.start();
    }

    /// Record an error for rollback monitoring
    pub fn record_error(&self, is_hierarchical: bool) {
        self.monitor.record_error(is_hierarchical);
    }

    /// Record a success for monitoring
    pub fn record_success(&self, is_hierarchical: bool) {
        self.monitor.record_success(is_hierarchical);
    }
}

/// Request context for feature evaluation
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_id: Option<Uuid>,
    pub request_id: Uuid,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub attributes: HashMap<String, String>,
}

/// Feature evaluator
struct FeatureEvaluator {
    flags: Arc<RwLock<FeatureFlags>>,
    hasher: std::collections::hash_map::DefaultHasher,
}

impl FeatureEvaluator {
    fn new(flags: Arc<RwLock<FeatureFlags>>) -> Self {
        Self {
            flags,
            hasher: std::collections::hash_map::DefaultHasher::new(),
        }
    }

    fn evaluate(&self, context: &RequestContext) -> bool {
        let flags = self.flags.read();

        // Master switch
        if !flags.hierarchical_enabled {
            return false;
        }

        // Check targeting rules first
        for rule in &flags.targeting_rules {
            if self.matches_rule(rule, context) {
                return match rule.action {
                    TargetAction::ForceHierarchical => true,
                    TargetAction::ForceFlat => false,
                    TargetAction::UsePercentage(pct) => self.in_percentage(context, pct),
                };
            }
        }

        // Use global percentage
        self.in_percentage(context, flags.hierarchical_traffic_percentage)
    }

    fn is_feature_enabled(&self, feature: &str, context: &RequestContext) -> bool {
        let flags = self.flags.read();

        if let Some(config) = flags.features.get(feature) {
            if !config.enabled {
                return false;
            }

            // Check conditions
            for condition in &config.conditions {
                if !self.matches_condition(condition, context) {
                    return false;
                }
            }

            // Check percentage
            self.in_percentage(context, config.percentage)
        } else {
            false
        }
    }

    fn matches_rule(&self, rule: &TargetingRule, context: &RequestContext) -> bool {
        rule.conditions
            .iter()
            .all(|cond| self.matches_target_condition(cond, context))
    }

    fn matches_target_condition(
        &self,
        condition: &TargetCondition,
        context: &RequestContext,
    ) -> bool {
        match condition {
            TargetCondition::UserAttribute(key, value) => {
                context.attributes.get(key) == Some(value)
            }
            TargetCondition::RequestPath(pattern) => context.path.contains(pattern),
            TargetCondition::TimeWindow(start, end) => {
                let now = chrono::Utc::now();
                now >= *start && now <= *end
            }
            TargetCondition::RandomPercentage(pct) => self.in_percentage(context, *pct),
        }
    }

    fn matches_condition(&self, condition: &FeatureCondition, context: &RequestContext) -> bool {
        match condition {
            FeatureCondition::After(time) => chrono::Utc::now() >= *time,
            FeatureCondition::Before(time) => chrono::Utc::now() <= *time,
            FeatureCondition::UserInList(users) => {
                context.user_id.is_some_and(|id| users.contains(&id))
            }
            FeatureCondition::UserMatchesPattern(pattern) => context
                .user_id
                .is_some_and(|id| id.to_string().contains(pattern)),
            FeatureCondition::HeaderPresent(key, value) => context.headers.get(key) == Some(value),
            FeatureCondition::QueryParamPresent(key, value) => {
                context.query_params.get(key) == Some(value)
            }
            FeatureCondition::LoadBelow(_) => true, // Would check actual system load
            FeatureCondition::ErrorRateBelow(_) => true, // Would check actual error rate
        }
    }

    fn in_percentage(&self, context: &RequestContext, percentage: f32) -> bool {
        use std::hash::{Hash, Hasher};

        let mut hasher = self.hasher.clone();
        context.request_id.hash(&mut hasher);

        let hash_value = hasher.finish();
        let normalized = (hash_value % 10000) as f32 / 100.0;

        normalized < percentage
    }
}

/// Configuration updater
struct ConfigUpdater {
    flags: Arc<RwLock<FeatureFlags>>,
}

impl ConfigUpdater {
    fn new(flags: Arc<RwLock<FeatureFlags>>) -> Self {
        Self { flags }
    }

    async fn update(&self, new_flags: FeatureFlags) -> Result<()> {
        // Validate new configuration
        self.validate(&new_flags)?;

        // Update atomically
        *self.flags.write() = new_flags;

        Ok(())
    }

    fn validate(&self, flags: &FeatureFlags) -> Result<()> {
        // Validate percentage ranges
        if flags.hierarchical_traffic_percentage < 0.0
            || flags.hierarchical_traffic_percentage > 100.0
        {
            return Err(Error::Configuration(
                "Traffic percentage must be between 0 and 100".to_string(),
            ));
        }

        // Validate feature percentages
        for (name, config) in &flags.features {
            if config.percentage < 0.0 || config.percentage > 100.0 {
                return Err(Error::Configuration(format!(
                    "Feature {} percentage must be between 0 and 100",
                    name
                )));
            }
        }

        Ok(())
    }
}

/// Feature monitoring for automatic rollback
struct FeatureMonitor {
    flags: Arc<RwLock<FeatureFlags>>,
    metrics: Arc<RwLock<MonitoringMetrics>>,
}

struct MonitoringMetrics {
    hierarchical_errors: u64,
    hierarchical_successes: u64,
    flat_errors: u64,
    flat_successes: u64,
    last_reset: std::time::Instant,
    rollback_count: u32,
}

impl Default for MonitoringMetrics {
    fn default() -> Self {
        Self {
            hierarchical_errors: 0,
            hierarchical_successes: 0,
            flat_errors: 0,
            flat_successes: 0,
            last_reset: std::time::Instant::now(),
            rollback_count: 0,
        }
    }
}

impl FeatureMonitor {
    fn new(flags: Arc<RwLock<FeatureFlags>>) -> Self {
        Self {
            flags,
            metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
        }
    }

    fn start(&self) {
        let flags = self.flags.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                let should_rollback = {
                    let flags = flags.read();
                    let mut metrics = metrics.write();

                    if !flags.auto_rollback_on_error {
                        false
                    } else {
                        let elapsed = metrics.last_reset.elapsed();
                        if elapsed >= flags.rollback_config.error_window {
                            // Calculate error rates
                            let hier_total =
                                metrics.hierarchical_errors + metrics.hierarchical_successes;
                            let hier_error_rate = if hier_total > 0 {
                                metrics.hierarchical_errors as f32 / hier_total as f32
                            } else {
                                0.0
                            };

                            // Reset metrics
                            metrics.hierarchical_errors = 0;
                            metrics.hierarchical_successes = 0;
                            metrics.flat_errors = 0;
                            metrics.flat_successes = 0;
                            metrics.last_reset = std::time::Instant::now();

                            // Check if rollback needed
                            hier_error_rate > flags.rollback_config.error_threshold
                                && metrics.rollback_count
                                    < flags.rollback_config.max_rollback_attempts
                        } else {
                            false
                        }
                    }
                };

                if should_rollback {
                    Self::perform_rollback(&flags, &metrics).await;
                }
            }
        });
    }

    fn record_error(&self, is_hierarchical: bool) {
        let mut metrics = self.metrics.write();
        if is_hierarchical {
            metrics.hierarchical_errors += 1;
        } else {
            metrics.flat_errors += 1;
        }
    }

    fn record_success(&self, is_hierarchical: bool) {
        let mut metrics = self.metrics.write();
        if is_hierarchical {
            metrics.hierarchical_successes += 1;
        } else {
            metrics.flat_successes += 1;
        }
    }

    async fn perform_rollback(
        flags: &Arc<RwLock<FeatureFlags>>,
        metrics: &Arc<RwLock<MonitoringMetrics>>,
    ) {
        tracing::error!("Performing automatic rollback due to error threshold exceeded");

        // Disable hierarchical system
        {
            let mut flags = flags.write();
            flags.hierarchical_enabled = false;
            flags.hierarchical_traffic_percentage = 0.0;
        }

        // Increment rollback count
        metrics.write().rollback_count += 1;

        // TODO: Send alerts, record metrics, etc.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_default() {
        let flags = FeatureFlags::default();
        assert!(!flags.hierarchical_enabled);
        assert_eq!(flags.hierarchical_traffic_percentage, 0.0);
        assert!(flags.auto_rollback_on_error);
    }

    #[test]
    fn test_percentage_evaluation() {
        let flags = FeatureFlags {
            hierarchical_enabled: true,
            hierarchical_traffic_percentage: 50.0,
            ..Default::default()
        };

        let manager = FeatureFlagManager::new(flags);

        let mut hierarchical_count = 0;
        let total_requests = 1000;

        for i in 0..total_requests {
            let context = RequestContext {
                user_id: None,
                request_id: Uuid::new_v4(),
                path: "/test".to_string(),
                headers: HashMap::new(),
                query_params: HashMap::new(),
                attributes: HashMap::new(),
            };

            if manager.should_use_hierarchical(&context) {
                hierarchical_count += 1;
            }
        }

        // Should be roughly 50% (within 10% margin)
        let percentage = (hierarchical_count as f32 / total_requests as f32) * 100.0;
        assert!(percentage > 40.0 && percentage < 60.0);
    }

    #[test]
    fn test_targeting_rules() {
        let mut flags = FeatureFlags::default();
        flags.hierarchical_enabled = true;
        flags.hierarchical_traffic_percentage = 0.0; // Default to flat

        // Add targeting rule for specific users
        let user_id = Uuid::new_v4();
        flags.targeting_rules.push(TargetingRule {
            name: "test_users".to_string(),
            priority: 1,
            conditions: vec![TargetCondition::UserAttribute(
                "role".to_string(),
                "tester".to_string(),
            )],
            action: TargetAction::ForceHierarchical,
        });

        let manager = FeatureFlagManager::new(flags);

        // Test with matching user
        let mut context = RequestContext {
            user_id: Some(user_id),
            request_id: Uuid::new_v4(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            attributes: HashMap::new(),
        };
        context
            .attributes
            .insert("role".to_string(), "tester".to_string());

        assert!(manager.should_use_hierarchical(&context));

        // Test with non-matching user
        context
            .attributes
            .insert("role".to_string(), "user".to_string());
        assert!(!manager.should_use_hierarchical(&context));
    }
}
