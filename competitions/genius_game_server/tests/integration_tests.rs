#[cfg(test)]
mod integration_tests {
    use genius_game_server::{GameEngine, GeniusGameServer};
    use genius_game_server::games::{GameConfig, GameType, Action, Game};
    use genius_game_server::collective::{CollectiveIntelligence, CollectiveConfig, CollectiveType, CoordinationStrategy};
    use genius_game_server::sota::{SOTAManager, SOTAConfig, ThinkingTime};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_minority_game_full_cycle() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 5,
            time_limit_ms: 1000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Add initial players to the game state
        let mut initial_actions = HashMap::new();
        initial_actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "join".to_string(),
            data: serde_json::json!({}),
            reasoning: None,
            confidence: None,
        });
        initial_actions.insert("player2".to_string(), Action {
            player_id: "player2".to_string(),
            action_type: "join".to_string(),
            data: serde_json::json!({}),
            reasoning: None,
            confidence: None,
        });
        
        // Process initial round to register players
        let _ = engine.process_turn(game_id, initial_actions).await.unwrap();
        
        // Play 5 rounds
        for round in 0..5 {
            let mut actions = HashMap::new();
            
            actions.insert("player1".to_string(), Action {
                player_id: "player1".to_string(),
                action_type: "decision".to_string(),
                data: serde_json::json!(round % 2),
                reasoning: Some(format!("Round {} decision", round)),
                confidence: Some(0.8),
            });
            
            actions.insert("player2".to_string(), Action {
                player_id: "player2".to_string(),
                action_type: "decision".to_string(),
                data: serde_json::json!((round + 1) % 2),
                reasoning: Some(format!("Opposite of player1")),
                confidence: Some(0.7),
            });
            
            let result = engine.process_turn(game_id, actions).await.unwrap();
            assert_eq!(result.round, round + 1);
            
            // One should win each round (unless tie)
            if result.outcome.winners.len() > 0 {
                assert!(result.outcome.winners.len() <= 2);
            }
        }
        
        // Finalize and check results
        let final_result = engine.finalize_game(game_id).await.unwrap();
        assert_eq!(final_result.total_rounds, 5);
        assert_eq!(final_result.final_scores.len(), 2);
        assert!(!final_result.winner.is_empty());
    }

    #[tokio::test]
    async fn test_byzantine_generals_consensus() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::ByzantineGenerals,
            rounds: 3,
            time_limit_ms: 2000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Initialize 7 generals
        let mut initial_actions = HashMap::new();
        for i in 0..7 {
            initial_actions.insert(format!("general_{}", i), Action {
                player_id: format!("general_{}", i),
                action_type: "join".to_string(),
                data: serde_json::json!({}),
                reasoning: None,
                confidence: None,
            });
        }
        let _ = engine.process_turn(game_id, initial_actions).await.unwrap();
        
        // Test consensus rounds
        for round in 0..3 {
            let mut actions = HashMap::new();
            
            // Most generals vote attack
            for i in 0..7 {
                let decision = if i < 5 { "attack" } else { "retreat" };
                
                actions.insert(format!("general_{}", i), Action {
                    player_id: format!("general_{}", i),
                    action_type: "vote".to_string(),
                    data: serde_json::json!({
                        "decision": decision,
                        "messages": []
                    }),
                    reasoning: Some(format!("{} vote", decision)),
                    confidence: Some(0.9),
                });
            }
            
            let result = engine.process_turn(game_id, actions).await.unwrap();
            
            // Check consensus was reached
            if result.outcome.special_events.iter().any(|e| e.contains("Consensus")) {
                assert!(result.outcome.winners.len() >= 5); // Majority should win
            }
        }
        
        let final_result = engine.finalize_game(game_id).await.unwrap();
        assert!(final_result.total_rounds > 0);
    }

    #[tokio::test]
    async fn test_collective_intelligence_decision_making() {
        let config = CollectiveConfig {
            name: "Test Collective".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::HierarchicalDemocracy {
                master: "master".to_string()
            },
            cost_per_hour: 100.0,
        };
        
        let mut collective = CollectiveIntelligence::new("test_collective".to_string(), config);
        
        let context = serde_json::json!({
            "game_type": "minority_game",
            "round": 5,
            "history": [],
            "opponents": ["sota_1", "sota_2"]
        });
        
        let decision = collective.make_decision(context).await.unwrap();
        
        assert_eq!(decision.individual_decisions.len(), 6); // Opus Orchestra has 6 instances
        assert!(!decision.emergence_detected); // Early rounds shouldn't have emergence
        assert!(decision.dissent_rate >= 0.0 && decision.dissent_rate <= 1.0);
        assert_eq!(decision.consensus_method, "hierarchical_democracy");
    }

    #[tokio::test]
    async fn test_sota_manager_decision_making() {
        let config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 25.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        
        let context = serde_json::json!({
            "game_type": "minority_game",
            "round": 10,
            "history": [
                {"round": 8, "winning_choice": 0},
                {"round": 9, "winning_choice": 1}
            ]
        });
        
        let decision = sota.make_decision(context).await.unwrap();
        
        assert!(decision.reasoning_chain.len() > 0);
        assert!(decision.confidence > 0.0 && decision.confidence <= 1.0);
        assert!(!decision.strategy.is_empty());
        assert!(decision.thinking_time_ms > 0);
    }

    #[tokio::test]
    async fn test_game_state_persistence() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Check initial state
        let state1 = engine.get_game_state(game_id).await.unwrap();
        assert_eq!(state1.round, 0);
        assert_eq!(state1.scores.len(), 0);
        
        // Process a turn
        let mut actions = HashMap::new();
        actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!(0),
            reasoning: None,
            confidence: None,
        });
        
        let _ = engine.process_turn(game_id, actions).await.unwrap();
        
        // Check state updated
        let state2 = engine.get_game_state(game_id).await.unwrap();
        assert_eq!(state2.round, 1);
        assert!(state2.history.len() > 0);
    }

    #[tokio::test]
    async fn test_emergence_detection() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 50,
            time_limit_ms: 1000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Add collective players
        let mut initial_actions = HashMap::new();
        for i in 0..6 {
            initial_actions.insert(format!("collective_{}", i), Action {
                player_id: format!("collective_{}", i),
                action_type: "join".to_string(),
                data: serde_json::json!({}),
                reasoning: None,
                confidence: None,
            });
        }
        let _ = engine.process_turn(game_id, initial_actions).await.unwrap();
        
        // Simulate rounds where collective achieves perfect distribution
        for round in 0..30 {
            let mut actions = HashMap::new();
            
            // Create perfect minority distribution
            for i in 0..6 {
                let choice = if i < 3 { 0 } else { 1 };
                actions.insert(format!("collective_{}", i), Action {
                    player_id: format!("collective_{}", i),
                    action_type: "decision".to_string(),
                    data: serde_json::json!(choice),
                    reasoning: Some("Distributed consensus".to_string()),
                    confidence: Some(0.9),
                });
            }
            
            let result = engine.process_turn(game_id, actions).await.unwrap();
            
            // After round 20, emergence should be detected
            if round > 20 {
                let has_emergence = result.outcome.special_events.iter()
                    .any(|e| e.contains("Emergence") || e.contains("emergence"));
                assert!(has_emergence || result.outcome.emergence_detected);
            }
        }
    }

    #[tokio::test]
    async fn test_mini_go_game_mechanics() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MiniGo,
            rounds: 100,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        // Initialize two players
        let mut actions = HashMap::new();
        actions.insert("black".to_string(), Action {
            player_id: "black".to_string(),
            action_type: "join".to_string(),
            data: serde_json::json!({"color": "black"}),
            reasoning: None,
            confidence: None,
        });
        actions.insert("white".to_string(), Action {
            player_id: "white".to_string(),
            action_type: "join".to_string(),
            data: serde_json::json!({"color": "white"}),
            reasoning: None,
            confidence: None,
        });
        let _ = engine.process_turn(game_id, actions).await.unwrap();
        
        // Test some moves
        let moves = vec![
            ("black", 4, 4),
            ("white", 4, 5),
            ("black", 5, 4),
            ("white", 5, 5),
            ("black", 3, 4),
        ];
        
        for (player, row, col) in moves {
            let mut actions = HashMap::new();
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: "move".to_string(),
                data: serde_json::json!({
                    "row": row,
                    "col": col
                }),
                reasoning: Some("Strategic move".to_string()),
                confidence: Some(0.8),
            });
            
            let result = engine.process_turn(game_id, actions).await.unwrap();
            
            // Check move was processed
            assert!(result.round > 0);
        }
    }

    #[tokio::test]
    async fn test_concurrent_game_handling() {
        let engine = GameEngine::new();
        
        let mut game_handles = vec![];
        
        // Create 10 concurrent games
        for i in 0..10 {
            let engine_clone = GameEngine::new();
            let handle = tokio::spawn(async move {
                let config = GameConfig {
                    game_type: GameType::MinorityGame,
                    rounds: 5,
                    time_limit_ms: 1000,
                    special_rules: HashMap::new(),
                };
                
                let game_id = engine_clone.create_game(config).await.unwrap();
                
                // Play a few rounds
                for round in 0..3 {
                    let mut actions = HashMap::new();
                    actions.insert(format!("player_{}", i), Action {
                        player_id: format!("player_{}", i),
                        action_type: "decision".to_string(),
                        data: serde_json::json!(round % 2),
                        reasoning: None,
                        confidence: None,
                    });
                    
                    let _ = engine_clone.process_turn(game_id, actions).await.unwrap();
                }
                
                engine_clone.finalize_game(game_id).await.unwrap()
            });
            
            game_handles.push(handle);
        }
        
        // Wait for all games to complete
        let results = futures::future::join_all(game_handles).await;
        
        // Verify all games completed successfully
        assert_eq!(results.len(), 10);
        for result in results {
            assert!(result.is_ok());
            let game_result = result.unwrap();
            assert!(game_result.total_rounds > 0);
        }
    }
}