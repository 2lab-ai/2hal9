// HAL9 Feature Flag System for Enterprise Features
// Enables gradual rollout and safe deployment of complex features

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: f32,  // 0.0 to 100.0
    pub whitelist: Vec<String>,   // User/org IDs
    pub blacklist: Vec<String>,   // User/org IDs
    pub dependencies: Vec<String>, // Other required flags
}

#[derive(Clone)]
pub struct FeatureFlagSystem {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl FeatureFlagSystem {
    pub fn new() -> Self {
        let mut system = Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize default enterprise feature flags
        system.init_default_flags();
        system
    }
    
    fn init_default_flags(&mut self) {
        // Phase 1: JWT Authentication
        self.add_flag(FeatureFlag {
            name: "jwt_auth".to_string(),
            enabled: true,
            rollout_percentage: 100.0,
            whitelist: vec![],
            blacklist: vec![],
            dependencies: vec![],
        });
        
        // Phase 2: SSO Integration
        self.add_flag(FeatureFlag {
            name: "sso_integration".to_string(),
            enabled: false,
            rollout_percentage: 0.0,
            whitelist: vec!["enterprise_pilot_org".to_string()],
            blacklist: vec![],
            dependencies: vec!["jwt_auth".to_string()],
        });
        
        // Phase 3: RBAC System
        self.add_flag(FeatureFlag {
            name: "rbac_system".to_string(),
            enabled: false,
            rollout_percentage: 0.0,
            whitelist: vec![],
            blacklist: vec![],
            dependencies: vec!["jwt_auth".to_string(), "sso_integration".to_string()],
        });
        
        // Additional enterprise features
        self.add_flag(FeatureFlag {
            name: "audit_logging".to_string(),
            enabled: false,
            rollout_percentage: 0.0,
            whitelist: vec![],
            blacklist: vec![],
            dependencies: vec!["jwt_auth".to_string()],
        });
        
        self.add_flag(FeatureFlag {
            name: "multi_tenancy".to_string(),
            enabled: false,
            rollout_percentage: 0.0,
            whitelist: vec![],
            blacklist: vec![],
            dependencies: vec!["rbac_system".to_string()],
        });
    }
    
    fn add_flag(&self, flag: FeatureFlag) {
        let flags_clone = self.flags.clone();
        tokio::spawn(async move {
            let mut flags = flags_clone.write().await;
            flags.insert(flag.name.clone(), flag);
        });
    }
    
    pub async fn is_enabled(&self, flag_name: &str, context: &FeatureContext) -> bool {
        let flags = self.flags.read().await;
        
        if let Some(flag) = flags.get(flag_name) {
            // Check dependencies first
            for dep in &flag.dependencies {
                if !self.is_enabled(dep, context).await {
                    return false;
                }
            }
            
            // Check blacklist
            if flag.blacklist.contains(&context.user_id) ||
               flag.blacklist.contains(&context.org_id) {
                return false;
            }
            
            // Check whitelist (overrides rollout)
            if flag.whitelist.contains(&context.user_id) ||
               flag.whitelist.contains(&context.org_id) {
                return true;
            }
            
            // Check if globally enabled
            if !flag.enabled {
                return false;
            }
            
            // Check rollout percentage
            if flag.rollout_percentage >= 100.0 {
                return true;
            }
            
            // Use consistent hashing for gradual rollout
            let hash = self.hash_context(context);
            let threshold = (flag.rollout_percentage / 100.0 * u64::MAX as f32) as u64;
            
            hash <= threshold
        } else {
            false
        }
    }
    
    fn hash_context(&self, context: &FeatureContext) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        context.user_id.hash(&mut hasher);
        context.org_id.hash(&mut hasher);
        hasher.finish()
    }
    
    pub async fn update_flag(&self, name: &str, enabled: bool, rollout: f32) {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(name) {
            flag.enabled = enabled;
            flag.rollout_percentage = rollout.clamp(0.0, 100.0);
        }
    }
    
    pub async fn add_to_whitelist(&self, flag_name: &str, id: String) {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(flag_name) {
            if !flag.whitelist.contains(&id) {
                flag.whitelist.push(id);
            }
        }
    }
    
    pub async fn get_enabled_features(&self, context: &FeatureContext) -> Vec<String> {
        let mut enabled = Vec::new();
        let flags = self.flags.read().await;
        
        for (name, _) in flags.iter() {
            if self.is_enabled(name, context).await {
                enabled.push(name.clone());
            }
        }
        
        enabled
    }
}

#[derive(Debug, Clone)]
pub struct FeatureContext {
    pub user_id: String,
    pub org_id: String,
    pub attributes: HashMap<String, String>,
}

// Integration with HAL9 components
impl super::super::enterprise::EnterpriseFeatures {
    pub async fn check_feature(&self, feature: &str) -> bool {
        let context = self.get_current_context();
        self.feature_flags.is_enabled(feature, &context).await
    }
    
    pub async fn with_feature<F, R>(&self, feature: &str, f: F) -> Result<R, &'static str>
    where
        F: FnOnce() -> R,
    {
        if self.check_feature(feature).await {
            Ok(f())
        } else {
            Err("Feature not enabled")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature_flag_dependencies() {
        let system = FeatureFlagSystem::new();
        let context = FeatureContext {
            user_id: "test_user".to_string(),
            org_id: "test_org".to_string(),
            attributes: HashMap::new(),
        };
        
        // RBAC depends on SSO which depends on JWT
        // If JWT is disabled, RBAC should be disabled
        system.update_flag("jwt_auth", false, 100.0).await;
        assert!(!system.is_enabled("rbac_system", &context).await);
        
        // Enable JWT but not SSO
        system.update_flag("jwt_auth", true, 100.0).await;
        system.update_flag("sso_integration", false, 100.0).await;
        assert!(!system.is_enabled("rbac_system", &context).await);
    }
    
    #[tokio::test]
    async fn test_gradual_rollout() {
        let system = FeatureFlagSystem::new();
        
        // Test with different rollout percentages
        system.update_flag("sso_integration", true, 50.0).await;
        
        let mut enabled_count = 0;
        for i in 0..1000 {
            let context = FeatureContext {
                user_id: format!("user_{}", i),
                org_id: "test_org".to_string(),
                attributes: HashMap::new(),
            };
            
            if system.is_enabled("sso_integration", &context).await {
                enabled_count += 1;
            }
        }
        
        // Should be roughly 50% (allowing for some variance)
        assert!(enabled_count > 400 && enabled_count < 600);
    }
}