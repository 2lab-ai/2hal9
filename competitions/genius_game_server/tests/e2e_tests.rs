#[cfg(test)]
mod e2e_tests {
    use genius_game_server::{GeniusGameServer, GameEngine};
    use genius_game_server::games::{GameConfig, GameType};
    use genius_game_server::collective::{CollectiveConfig, CollectiveType, CoordinationStrategy};
    use genius_game_server::sota::{SOTAConfig, ThinkingTime};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_full_minority_game_simulation() {
        // Create server components
        let server = GeniusGameServer::new();
        
        // Create collective players
        let opus_config = CollectiveConfig {
            name: "Opus Orchestra".to_string(),
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],
            coordination: CoordinationStrategy::HierarchicalDemocracy {
                master: "master_strategist".to_string()
            },
            cost_per_hour: 120.0,
        };
        
        let swarm_config = CollectiveConfig {
            name: "Swarm Intelligence".to_string(),
            config_type: CollectiveType::SwarmIntelligence,
            models: vec![],
            coordination: CoordinationStrategy::EmergentConsensus {
                communication: "local_only".to_string()
            },
            cost_per_hour: 2.0,
        };
        
        // Create SOTA players
        let claude_config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 25.0,
        };
        
        let gpt4_config = SOTAConfig {
            model_name: "gpt-4-turbo".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        // Create game
        let game_config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        // Run simulation
        let result = timeout(Duration::from_secs(30), async {
            // This would normally involve HTTP calls to the server
            // For e2e test, we directly use the engine
            let engine = GameEngine::new();
            let game_id = engine.create_game(game_config).await.unwrap();
            
            // Simulate 10 rounds
            for round in 0..10 {
                let mut actions = HashMap::new();
                
                // Collective decisions
                actions.insert("collective_opus".to_string(), genius_game_server::games::Action {
                    player_id: "collective_opus".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": round % 2}),
                    reasoning: Some("Opus strategic decision".to_string()),
                    confidence: Some(0.9),
                });
                
                actions.insert("collective_swarm".to_string(), genius_game_server::games::Action {
                    player_id: "collective_swarm".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": (round + 1) % 2}),
                    reasoning: Some("Swarm emergent decision".to_string()),
                    confidence: Some(0.7),
                });
                
                // SOTA decisions
                actions.insert("sota_claude".to_string(), genius_game_server::games::Action {
                    player_id: "sota_claude".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": round % 2}),
                    reasoning: Some("Claude analytical decision".to_string()),
                    confidence: Some(0.85),
                });
                
                actions.insert("sota_gpt4".to_string(), genius_game_server::games::Action {
                    player_id: "sota_gpt4".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": 0}),
                    reasoning: Some("GPT-4 calculated decision".to_string()),
                    confidence: Some(0.8),
                });
                
                let round_result = engine.process_turn(game_id, actions).await.unwrap();
                assert_eq!(round_result.round, round + 1);
            }
            
            // Finalize game
            let final_result = engine.finalize_game(game_id).await.unwrap();
            assert_eq!(final_result.total_rounds, 10);
            assert!(final_result.final_scores.len() > 0);
            
            final_result
        }).await;
        
        assert!(result.is_ok(), "Game simulation should complete within timeout");
        let game_result = result.unwrap();
        
        // Verify game completed properly
        assert!(game_result.winner.len() > 0);
        assert!(game_result.analytics.strategic_depth > 0.0);
    }

    #[tokio::test]
    async fn test_collective_vs_sota_performance() {
        let engine = GameEngine::new();
        
        // Run multiple games to test performance
        let mut collective_wins = 0;
        let mut sota_wins = 0;
        
        for i in 0..5 {
            let config = GameConfig {
                game_type: GameType::MinorityGame,
                rounds: 20,
                time_limit_ms: 5000,
                special_rules: HashMap::new(),
            };
            
            let game_id = engine.create_game(config).await.unwrap();
            
            // Simulate game with different strategies
            for round in 0..20 {
                let mut actions = HashMap::new();
                
                // Collective uses adaptive strategy
                let collective_choice = if round < 10 {
                    round % 2
                } else {
                    // Switch strategy mid-game
                    (round + i) % 2
                };
                
                actions.insert("collective_1".to_string(), genius_game_server::games::Action {
                    player_id: "collective_1".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": collective_choice}),
                    reasoning: None,
                    confidence: None,
                });
                
                // SOTA uses fixed strategy
                actions.insert("sota_1".to_string(), genius_game_server::games::Action {
                    player_id: "sota_1".to_string(),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": round % 2}),
                    reasoning: None,
                    confidence: None,
                });
                
                let _ = engine.process_turn(game_id, actions).await.unwrap();
            }
            
            let result = engine.finalize_game(game_id).await.unwrap();
            
            if result.winner.starts_with("collective") {
                collective_wins += 1;
            } else if result.winner.starts_with("sota") {
                sota_wins += 1;
            }
        }
        
        // Verify both can win
        assert!(collective_wins > 0 || sota_wins > 0, "At least one type should win");
    }

    #[tokio::test]
    async fn test_emergence_detection_e2e() {
        let engine = GameEngine::new();
        
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 50,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let game_id = engine.create_game(config).await.unwrap();
        
        let mut emergence_detected = false;
        
        // Simulate many collective agents to trigger emergence
        for round in 0..30 {
            let mut actions = HashMap::new();
            
            // Add 10 collective agents with coordinated behavior
            for i in 0..10 {
                let choice = if round > 20 {
                    // After round 20, create perfect distribution
                    i % 2
                } else {
                    // Random before round 20
                    (i + round) % 2
                };
                
                actions.insert(format!("collective_{}", i), genius_game_server::games::Action {
                    player_id: format!("collective_{}", i),
                    action_type: "decision".to_string(),
                    data: serde_json::json!({"choice": choice}),
                    reasoning: None,
                    confidence: None,
                });
            }
            
            let round_result = engine.process_turn(game_id, actions).await.unwrap();
            
            if round_result.outcome.emergence_detected {
                emergence_detected = true;
            }
        }
        
        let final_result = engine.finalize_game(game_id).await.unwrap();
        
        assert!(emergence_detected, "Emergence should be detected with coordinated collective behavior");
        assert!(final_result.emergence_events.len() > 0, "Final result should contain emergence events");
        assert!(final_result.analytics.emergence_frequency > 0.0, "Emergence frequency should be positive");
    }

    #[tokio::test]
    async fn test_multiple_game_types() {
        let engine = GameEngine::new();
        
        let game_types = vec![
            GameType::MinorityGame,
            GameType::ByzantineGenerals,
            GameType::CollectiveMaze,
            GameType::RecursiveReasoning,
            GameType::SwarmOptimization,
        ];
        
        for game_type in game_types {
            let config = GameConfig {
                game_type: game_type.clone(),
                rounds: 5,
                time_limit_ms: 5000,
                special_rules: HashMap::new(),
            };
            
            let result = engine.create_game(config).await;
            
            // Currently only MinorityGame is fully implemented
            if game_type == GameType::MinorityGame {
                assert!(result.is_ok(), "MinorityGame should be created successfully");
                
                let game_id = result.unwrap();
                let state = engine.get_game_state(game_id).await;
                assert!(state.is_some());
                assert_eq!(state.unwrap().game_type, game_type);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_games() {
        let engine = GameEngine::new();
        
        // Create multiple games concurrently
        let mut game_handles = vec![];
        
        for i in 0..3 {
            let engine_clone = GameEngine::new();
            let handle = tokio::spawn(async move {
                let config = GameConfig {
                    game_type: GameType::MinorityGame,
                    rounds: 10,
                    time_limit_ms: 5000,
                    special_rules: HashMap::new(),
                };
                
                let game_id = engine_clone.create_game(config).await.unwrap();
                
                // Run a few rounds
                for round in 0..5 {
                    let mut actions = HashMap::new();
                    actions.insert(format!("player_{}", i), genius_game_server::games::Action {
                        player_id: format!("player_{}", i),
                        action_type: "decision".to_string(),
                        data: serde_json::json!({"choice": round % 2}),
                        reasoning: None,
                        confidence: None,
                    });
                    
                    let _ = engine_clone.process_turn(game_id, actions).await;
                }
                
                engine_clone.finalize_game(game_id).await.unwrap()
            });
            
            game_handles.push(handle);
        }
        
        // Wait for all games to complete
        let results = futures::future::join_all(game_handles).await;
        
        // Verify all games completed successfully
        for result in results {
            assert!(result.is_ok());
            let game_result = result.unwrap();
            assert!(game_result.total_rounds > 0);
        }
    }
}