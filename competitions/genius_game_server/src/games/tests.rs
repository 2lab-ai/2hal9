#[cfg(test)]
mod game_tests {
    use super::super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_byzantine_generals_initialization() {
        let mut game = byzantine_generals::ByzantineGenerals::new();
        let config = GameConfig {
            game_type: GameType::ByzantineGenerals,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        assert_eq!(state.game_type, GameType::ByzantineGenerals);
        assert_eq!(state.round, 0);
        assert!(state.scores.is_empty());
    }

    #[tokio::test]
    async fn test_byzantine_generals_traitor_mechanics() {
        let mut game = byzantine_generals::ByzantineGenerals::new();
        let config = GameConfig {
            game_type: GameType::ByzantineGenerals,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: {
                let mut rules = HashMap::new();
                rules.insert("n_generals".to_string(), "7".to_string());
                rules
            },
        };
        
        let state = game.initialize(config).await.unwrap();
        
        // Create actions for 7 generals
        let mut actions = HashMap::new();
        for i in 0..7 {
            let general_id = format!("general_{}", i);
            actions.insert(general_id.clone(), Action {
                player_id: general_id,
                action_type: "message".to_string(),
                data: serde_json::json!({
                    "messages": [
                        {"to": "general_0", "decision": "attack"},
                        {"to": "general_1", "decision": "attack"},
                    ]
                }),
                reasoning: None,
                confidence: None,
            });
        }
        
        let result = game.process_round(&state, actions).await.unwrap();
        assert_eq!(result.round, 1);
        assert!(!result.scores_delta.is_empty());
    }

    #[tokio::test]
    async fn test_collective_maze_movement() {
        let mut game = collective_maze::CollectiveMaze::new();
        let config = GameConfig {
            game_type: GameType::CollectiveMaze,
            rounds: 50,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let mut state = game.initialize(config).await.unwrap();
        
        // Test movement - try multiple directions to ensure at least one succeeds
        let mut actions = HashMap::new();
        actions.insert("agent_1".to_string(), Action {
            player_id: "agent_1".to_string(),
            action_type: "move".to_string(),
            data: serde_json::json!({"move": "north"}),
            reasoning: None,
            confidence: None,
        });
        
        // First round places the agents
        let result = game.process_round(&state, actions.clone()).await.unwrap();
        state.round = result.round;
        
        // Try different directions to ensure we get a successful move
        let directions = ["north", "south", "east", "west"];
        let mut moved = false;
        
        for dir in &directions {
            actions.get_mut("agent_1").unwrap().data = serde_json::json!({"move": dir});
            let result2 = game.process_round(&state, actions.clone()).await.unwrap();
            
            if !result2.scores_delta.is_empty() {
                moved = true;
                assert_eq!(result2.round, 2);
                break;
            }
        }
        
        assert!(moved, "Agent should have been able to move in at least one direction");
    }

    #[tokio::test]
    async fn test_recursive_reasoning_depth() {
        let mut game = recursive_reasoning::RecursiveReasoning::new();
        let config = GameConfig {
            game_type: GameType::RecursiveReasoning,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        
        // Test different reasoning depths
        let mut actions = HashMap::new();
        for i in 0..5 {
            let agent_id = format!("agent_{}", i);
            actions.insert(agent_id.clone(), Action {
                player_id: agent_id,
                action_type: "guess".to_string(),
                data: serde_json::json!({"guess": 50 + i * 10}),
                reasoning: Some("Multi-level reasoning".to_string()),
                confidence: Some(0.8),
            });
        }
        
        let result = game.process_round(&state, actions).await.unwrap();
        assert!(!result.scores_delta.is_empty());
        assert!(result.outcome.winners.len() > 0);
    }

    #[tokio::test]
    async fn test_swarm_optimization_convergence() {
        let mut game = swarm_optimization::SwarmOptimization::new();
        let config = GameConfig {
            game_type: GameType::SwarmOptimization,
            rounds: 50,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let mut state = game.initialize(config).await.unwrap();
        
        // Test swarm movement
        let mut actions = HashMap::new();
        for i in 0..5 {
            let agent_id = format!("agent_{}", i);
            actions.insert(agent_id.clone(), Action {
                player_id: agent_id,
                action_type: "move".to_string(),
                data: serde_json::json!({
                    "moves": {
                        "0": 1.0,
                        "1": -0.5,
                        "2": 0.3
                    }
                }),
                reasoning: None,
                confidence: None,
            });
        }
        
        let result = game.process_round(&state, actions.clone()).await.unwrap();
        state.round = result.round;
        
        // Test multiple rounds to check convergence
        for _ in 1..10 {
            let result = game.process_round(&state, actions.clone()).await.unwrap();
            state.round = result.round;
            state.history.push(result);
        }
        
        assert!(state.round > 0);
        let final_result = game.calculate_final_result(&state).await;
        assert!(final_result.analytics.decision_diversity_index >= 0.0);
    }

    #[tokio::test]
    async fn test_minority_game_emergence() {
        let mut game = minority_game::MinorityGame::new();
        let config = GameConfig {
            game_type: GameType::MinorityGame,
            rounds: 30,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let mut state = game.initialize(config).await.unwrap();
        
        // Create many collective agents to trigger emergence
        let mut actions = HashMap::new();
        for i in 0..10 {
            let agent_id = format!("collective_{}", i);
            actions.insert(agent_id.clone(), Action {
                player_id: agent_id,
                action_type: "decision".to_string(),
                data: serde_json::json!(i % 2),
                reasoning: None,
                confidence: None,
            });
        }
        
        // Run multiple rounds
        for round in 0..25 {
            // After round 20, create perfect distribution
            if round > 20 {
                actions.clear();
                for i in 0..10 {
                    let agent_id = format!("collective_{}", i);
                    actions.insert(agent_id.clone(), Action {
                        player_id: agent_id,
                        action_type: "decision".to_string(),
                        data: serde_json::json!(i % 2), // Direct value, not object
                        reasoning: None,
                        confidence: None,
                    });
                }
            }
            
            let result = game.process_round(&state, actions.clone()).await.unwrap();
            state.round = result.round;
            state.history.push(result.clone());
            
            for (player, delta) in &result.scores_delta {
                *state.scores.entry(player.clone()).or_insert(0) += delta;
            }
        }
        
        let final_result = game.calculate_final_result(&state).await;
        assert!(final_result.emergence_events.len() > 0);
        assert!(final_result.analytics.emergence_frequency > 0.0);
    }

    #[tokio::test]
    async fn test_game_over_conditions() {
        // Test Byzantine Generals game over
        let game = byzantine_generals::ByzantineGenerals::new();
        let mut state = GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::ByzantineGenerals,
            round: 11,
            scores: HashMap::new(),
            history: vec![],
            metadata: HashMap::new(),
        };
        assert!(game.is_game_over(&state).await);
        
        // Test with score condition
        state.round = 5;
        state.scores.insert("player1".to_string(), 101);
        assert!(game.is_game_over(&state).await);
        
        // Test Collective Maze game over
        let game = collective_maze::CollectiveMaze::new();
        state.game_type = GameType::CollectiveMaze;
        state.round = 101;
        assert!(game.is_game_over(&state).await);
        
        // Test Swarm Optimization game over
        let game = swarm_optimization::SwarmOptimization::new();
        state.game_type = GameType::SwarmOptimization;
        state.round = 51;
        assert!(game.is_game_over(&state).await);
    }

    #[tokio::test]
    async fn test_analytics_calculation() {
        let game = recursive_reasoning::RecursiveReasoning::new();
        let state = GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::RecursiveReasoning,
            round: 10,
            scores: {
                let mut scores = HashMap::new();
                scores.insert("agent_1".to_string(), 500);
                scores.insert("agent_2".to_string(), 300);
                scores
            },
            history: vec![],
            metadata: HashMap::new(),
        };
        
        let result = game.calculate_final_result(&state).await;
        assert_eq!(result.winner, "agent_1");
        assert_eq!(result.total_rounds, 10);
        assert!(result.analytics.strategic_depth >= 0.0);
        assert!(result.analytics.strategic_depth <= 1.0);
    }
}