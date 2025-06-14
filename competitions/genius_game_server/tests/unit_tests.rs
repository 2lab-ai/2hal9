#[cfg(test)]
mod game_engine_tests {
    use genius_game_server::games::{GameEngine, GameConfig, GameType, Action};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_game() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        assert_ne!(game_id, Uuid::nil());
        
        let state = engine.get_game_state(game_id).await.unwrap();
        assert_eq!(state.game_type, GameType::MinorityGame);
        assert_eq!(state.round, 0);
    }

    #[tokio::test]
    async fn test_process_turn() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        let mut actions = HashMap::new();
        actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!({"choice": 0}),
            reasoning: Some("Test reasoning".to_string()),
            confidence: Some(0.8),
        });
        
        actions.insert("player2".to_string(), Action {
            player_id: "player2".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!({"choice": 1}),
            reasoning: Some("Different choice".to_string()),
            confidence: Some(0.7),
        });
        
        let result = engine.process_turn(game_id, actions).await.unwrap();
        assert_eq!(result.round, 1);
        assert!(!result.scores_delta.is_empty());
    }

    #[tokio::test]
    async fn test_game_completion() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 1, // Very short game for testing
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Process enough turns to end the game
        for _ in 0..2 {
            let mut actions = HashMap::new();
            actions.insert("player1".to_string(), Action {
                player_id: "player1".to_string(),
                action_type: "decision".to_string(),
                data: serde_json::json!({"choice": 0}),
                reasoning: None,
                confidence: None,
            });
            
            let _ = engine.process_turn(game_id, actions).await;
        }
        
        assert!(engine.is_game_finished(game_id).await);
        
        let result = engine.finalize_game(game_id).await.unwrap();
        assert_eq!(result.game_id, game_id);
        assert!(result.total_rounds > 0);
    }
}

#[cfg(test)]
mod collective_intelligence_tests {
    use genius_game_server::collective::{CollectiveIntelligence, CollectiveConfig, CollectiveType, CoordinationStrategy};
    
    #[tokio::test]
    async fn test_opus_orchestra_decision() {
        let config = CollectiveConfig {
            name: "Test Orchestra".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::HierarchicalDemocracy {
                master: "master_strategist".to_string()
            },
            cost_per_hour: 120.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        
        let context = serde_json::json!({
            "round": 5,
            "history": []
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        
        assert_eq!(decision.consensus_method, "hierarchical_democracy");
        assert_eq!(decision.individual_decisions.len(), 6); // 6 Opus instances
        assert!(decision.dissent_rate >= 0.0 && decision.dissent_rate <= 1.0);
    }

    #[tokio::test]
    async fn test_swarm_intelligence_decision() {
        let config = CollectiveConfig {
            name: "Test Swarm".to_string(),
            config_type: CollectiveType::SwarmIntelligence,
            models: vec![],
            coordination: CoordinationStrategy::EmergentConsensus {
                communication: "local_only".to_string()
            },
            cost_per_hour: 2.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_swarm".to_string(), config);
        
        let context = serde_json::json!({
            "round": 10,
            "history": []
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        
        assert_eq!(decision.consensus_method, "emergent_consensus");
        assert_eq!(decision.individual_decisions.len(), 32); // 32 lightweight models
    }

    #[tokio::test]
    async fn test_chaos_engine_always_emergent() {
        let config = CollectiveConfig {
            name: "Test Chaos".to_string(),
            config_type: CollectiveType::ChaosEngine,
            models: vec![],
            coordination: CoordinationStrategy::NoCoordination,
            cost_per_hour: 3.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_chaos".to_string(), config);
        
        let context = serde_json::json!({});
        
        let decision = collective.make_decision(context).await.unwrap();
        
        assert!(decision.emergence_detected);
        assert_eq!(decision.consensus_method, "majority_emergence");
    }
}

#[cfg(test)]
mod sota_manager_tests {
    use genius_game_server::sota::{SOTAManager, SOTAConfig, ThinkingTime};
    
    #[tokio::test]
    async fn test_claude_decision() {
        let config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::UltraThink,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 25.0,
        };
        
        let mut manager = SOTAManager::new("test_claude".to_string(), config);
        
        let context = serde_json::json!({
            "round": 15,
            "history": []
        });
        
        let decision = manager.make_decision(context).await.unwrap();
        
        assert!(decision.reasoning_chain.len() > 2); // Ultra think should have multiple steps
        assert_eq!(decision.strategy, "deep_recursive_reasoning");
        assert!(decision.confidence >= 0.85);
    }

    #[tokio::test]
    async fn test_thinking_time_affects_reasoning() {
        let config_standard = SOTAConfig {
            model_name: "gpt-4-turbo".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        let mut manager_standard = SOTAManager::new("test_standard".to_string(), config_standard);
        
        let config_extended = SOTAConfig {
            model_name: "gpt-4-turbo".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        let mut manager_extended = SOTAManager::new("test_extended".to_string(), config_extended);
        
        let context = serde_json::json!({});
        
        let decision_standard = manager_standard.make_decision(context.clone()).await.unwrap();
        let decision_extended = manager_extended.make_decision(context).await.unwrap();
        
        assert!(decision_extended.reasoning_chain.len() >= decision_standard.reasoning_chain.len());
    }
}

#[cfg(test)]
mod analytics_engine_tests {
    use genius_game_server::analytics::AnalyticsEngine;
    use genius_game_server::games::{RoundResult, Outcome};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_process_round_analytics() {
        let analytics = AnalyticsEngine::new();
        let game_id = Uuid::new_v4();
        
        let round_result = RoundResult {
            round: 1,
            actions: HashMap::new(),
            outcome: Outcome {
                winners: vec!["collective_1".to_string()],
                losers: vec!["sota_1".to_string()],
                special_events: vec!["Test event".to_string()],
                emergence_detected: true,
            },
            scores_delta: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };
        
        analytics.process_round(game_id, &round_result).await;
        
        let game_analytics = analytics.get_game_analytics(game_id).await.unwrap();
        assert_eq!(game_analytics.rounds_played, 1);
        assert_eq!(game_analytics.emergence_analysis.total_emergence_events, 1);
        assert_eq!(game_analytics.performance_comparison.critical_moments.len(), 1);
    }

    #[tokio::test]
    async fn test_emergence_tracking() {
        let analytics = AnalyticsEngine::new();
        let game_id = Uuid::new_v4();
        
        // Simulate multiple rounds with emergence
        for i in 0..5 {
            let round_result = RoundResult {
                round: i + 1,
                actions: HashMap::new(),
                outcome: Outcome {
                    winners: vec![],
                    losers: vec![],
                    special_events: vec![],
                    emergence_detected: i % 2 == 0, // Emergence every other round
                },
                scores_delta: HashMap::new(),
                timestamp: chrono::Utc::now(),
            };
            
            analytics.process_round(game_id, &round_result).await;
        }
        
        let final_analytics = analytics.calculate_final_analytics(game_id).await.unwrap();
        assert_eq!(final_analytics.emergence_analysis.total_emergence_events, 3);
        assert_eq!(final_analytics.collective_metrics.emergence_frequency, 0.6); // 3/5
    }
}

#[cfg(test)]
mod streaming_engine_tests {
    use genius_game_server::streaming::StreamingEngine;
    use genius_game_server::games::GameState;
    use uuid::Uuid;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_streaming_broadcasts() {
        let engine = StreamingEngine::new();
        let game_id = Uuid::new_v4();
        
        // Test game started broadcast
        engine.game_started(
            game_id,
            vec!["collective_1".to_string()],
            vec!["sota_1".to_string()]
        ).await;
        
        // Test game state update
        let state = GameState {
            game_id,
            game_type: genius_game_server::games::GameType::MinorityGame,
            round: 5,
            scores: HashMap::new(),
            history: vec![],
            metadata: HashMap::new(),
        };
        
        engine.update_game_state(game_id, state).await;
        
        // These should not panic and complete successfully
    }
}

#[cfg(test)]
mod minority_game_tests {
    use genius_game_server::games::{Game, GameConfig, GameType, Action};
    use genius_game_server::games::minority_game::MinorityGame;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_minority_calculation() {
        let mut game = MinorityGame::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        
        // Create actions where minority wins
        let mut actions = HashMap::new();
        actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!(0),
            reasoning: None,
            confidence: None,
        });
        
        actions.insert("player2".to_string(), Action {
            player_id: "player2".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!(1),
            reasoning: None,
            confidence: None,
        });
        
        actions.insert("player3".to_string(), Action {
            player_id: "player3".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!(1),
            reasoning: None,
            confidence: None,
        });
        
        let result = game.process_round(&state, actions).await.unwrap();
        
        // Player1 chose minority (0), should win
        assert_eq!(result.outcome.winners, vec!["player1"]);
        assert_eq!(*result.scores_delta.get("player1").unwrap(), 10);
        assert_eq!(*result.scores_delta.get("player2").unwrap(), -5);
    }

    #[tokio::test]
    async fn test_emergence_detection() {
        let mut game = MinorityGame::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let mut state = game.initialize(config).await.unwrap();
        state.round = 25; // Past threshold for emergence detection
        
        // Create collective actions with good distribution
        let mut actions = HashMap::new();
        for i in 0..10 {
            actions.insert(format!("collective_{}", i), Action {
                player_id: format!("collective_{}", i),
                action_type: "decision".to_string(),
                data: serde_json::json!(i % 2), // Perfect 50/50 split
                reasoning: None,
                confidence: None,
            });
        }
        
        let result = game.process_round(&state, actions).await.unwrap();
        
        // Should detect emergence due to perfect distribution
        assert!(result.outcome.special_events.contains(&"Emergence detected!".to_string()));
    }
}