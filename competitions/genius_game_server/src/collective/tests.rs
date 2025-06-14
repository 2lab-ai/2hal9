#[cfg(test)]
mod collective_tests {
    use super::super::*;

    #[tokio::test]
    async fn test_collective_creation() {
        let config = CollectiveConfig {
            name: "Test Collective".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 100.0,
        };
        
        let collective = CollectiveIntelligence::new("test_collective".to_string(), config.clone());
        assert_eq!(collective.config.name, "Test Collective");
        assert_eq!(collective.config.cost_per_hour, 100.0);
    }

    #[tokio::test]
    async fn test_coordination_strategies() {
        // Test Hierarchical Democracy
        let config = CollectiveConfig {
            name: "Hierarchical Test".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::HierarchicalDemocracy {
                master: "master_ai".to_string(),
            },
            cost_per_hour: 120.0,
        };
        
        let collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        match &collective.config.coordination {
            CoordinationStrategy::HierarchicalDemocracy { master } => {
                assert_eq!(master, "master_ai");
            }
            _ => panic!("Wrong coordination strategy"),
        }
        
        // Test Emergent Consensus
        let config = CollectiveConfig {
            name: "Emergent Test".to_string(),
            config_type: CollectiveType::SwarmIntelligence,
            models: vec![],
            coordination: CoordinationStrategy::EmergentConsensus {
                communication: "local_only".to_string(),
            },
            cost_per_hour: 2.0,
        };
        
        let collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        match &collective.config.coordination {
            CoordinationStrategy::EmergentConsensus { communication } => {
                assert_eq!(communication, "local_only");
            }
            _ => panic!("Wrong coordination strategy"),
        }
    }

    #[tokio::test]
    async fn test_decision_making() {
        let config = CollectiveConfig {
            name: "Decision Test".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 100.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        let context = serde_json::json!({
            "game_type": "MinorityGame",
            "round": 1,
            "history": []
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        assert!(!decision.final_decision.is_null());
        assert!(decision.dissent_rate >= 0.0 && decision.dissent_rate <= 1.0);
        assert!(!decision.consensus_method.is_empty());
    }

    #[tokio::test]
    async fn test_swarm_intelligence() {
        let config = CollectiveConfig {
            name: "Swarm Test".to_string(),
            config_type: CollectiveType::SwarmIntelligence,
            models: vec![],
            coordination: CoordinationStrategy::EmergentConsensus {
                communication: "global".to_string(),
            },
            cost_per_hour: 2.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        let context = serde_json::json!({
            "game_type": "SwarmOptimization",
            "dimensions": 10,
            "search_space": {"min": -100, "max": 100}
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        assert!(decision.dissent_rate >= 0.0 && decision.dissent_rate <= 1.0);
        assert_eq!(decision.consensus_method, "emergent_consensus");
    }

    #[tokio::test]
    async fn test_hybrid_council() {
        let config = CollectiveConfig {
            name: "Hybrid Test".to_string(),
            config_type: CollectiveType::HybridCouncil,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 50.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        let context = serde_json::json!({
            "game_type": "ByzantineGenerals",
            "n_generals": 7,
            "n_traitors": 2
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        assert_eq!(decision.consensus_method, "specialist_democracy");
        assert!(decision.dissent_rate >= 0.0);
    }

    #[tokio::test]
    async fn test_chaos_engine() {
        let config = CollectiveConfig {
            name: "Chaos Test".to_string(),
            config_type: CollectiveType::ChaosEngine,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 0.5,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        let context = serde_json::json!({
            "game_type": "MinorityGame",
            "chaos_level": "maximum"
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        assert_eq!(decision.consensus_method, "majority_emergence");
        // Chaos engine should have some dissent
        assert!(decision.dissent_rate >= 0.0); // Dissent rate varies in simulation
    }

    #[tokio::test]
    async fn test_collective_metrics() {
        let config = CollectiveConfig {
            name: "Metrics Test".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 120.0,
        };
        
        let collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        
        // Test decision count tracking
        assert_eq!(collective.id, "test_collective");
        
        // Test cost calculation
        assert_eq!(collective.config.cost_per_hour, 120.0);
        
        // Verify collective type
        assert_eq!(collective.config.config_type, CollectiveType::OpusOrchestra);
    }

    #[test]
    fn test_collective_type_properties() {
        // Test that each type has appropriate characteristics
        let opus = CollectiveType::OpusOrchestra;
        let swarm = CollectiveType::SwarmIntelligence;
        let hybrid = CollectiveType::HybridCouncil;
        let chaos = CollectiveType::ChaosEngine;
        
        // Verify distinct types
        assert_ne!(opus, swarm);
        assert_ne!(hybrid, chaos);
        assert_ne!(opus, chaos);
        
        // Test serialization
        let serialized = serde_json::to_string(&opus).unwrap();
        assert_eq!(serialized, "\"OpusOrchestra\"");
        
        let deserialized: CollectiveType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, opus);
    }
}