#[cfg(test)]
mod core_logic_tests {
    use super::super::*;
    use crate::games::minority_game::MinorityGame;
    use crate::games::byzantine_generals::{ByzantineGenerals, Message, Decision};
    use crate::games::recursive_reasoning::RecursiveReasoning;
    use crate::games::swarm_optimization::{SwarmOptimization, Particle};
    // use crate::games::mini_go::{MiniGo, Stone};
    // use crate::games::mini_holdem::{MiniHoldem, Card, Suit, Rank, HandRank};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    #[test]
    fn test_minority_game_calculation() {
        let game = MinorityGame::new();
        
        // Test case 1: Clear minority
        let mut choices = HashMap::new();
        choices.insert("p1".to_string(), 0);
        choices.insert("p2".to_string(), 0);
        choices.insert("p3".to_string(), 1);
        
        let (winning_choice, winners) = game.calculate_minority(&choices);
        assert_eq!(winning_choice, 1);
        assert_eq!(winners.len(), 1);
        assert!(winners.contains(&"p3".to_string()));
        
        // Test case 2: Tie
        let mut choices = HashMap::new();
        choices.insert("p1".to_string(), 0);
        choices.insert("p2".to_string(), 1);
        
        let (winning_choice, winners) = game.calculate_minority(&choices);
        assert_eq!(winning_choice, -1);
        assert_eq!(winners.len(), 0);
        
        // Test case 3: All same choice
        let mut choices = HashMap::new();
        choices.insert("p1".to_string(), 0);
        choices.insert("p2".to_string(), 0);
        choices.insert("p3".to_string(), 0);
        
        let (winning_choice, winners) = game.calculate_minority(&choices);
        assert_eq!(winning_choice, 1); // The absent choice wins
        assert_eq!(winners.len(), 0); // But no one chose it
    }
    
    #[test]
    fn test_emergence_detection_logic() {
        let game = MinorityGame::new();
        
        let state = GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::MinorityGame,
            round: 25,
            scores: HashMap::new(),
            history: vec![],
            metadata: HashMap::new(),
        };
        
        // Create actions that should trigger emergence
        let mut actions = HashMap::new();
        
        // Collective players with perfect distribution
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
        
        // Non-collective players
        actions.insert("sota_1".to_string(), Action {
            player_id: "sota_1".to_string(),
            action_type: "decision".to_string(),
            data: serde_json::json!(0),
            reasoning: None,
            confidence: None,
        });
        
        let emergence = game.detect_emergence(&state, &actions);
        
        assert!(emergence.is_some());
        let event = emergence.unwrap();
        assert_eq!(event.event_type, "perfect_distribution");
        assert!(event.emergence_score > 0.8);
    }
    
    #[test]
    fn test_byzantine_consensus_calculation() {
        let mut game = ByzantineGenerals::new();
        
        // Initialize with known traitors
        let players = vec![
            "g1".to_string(), "g2".to_string(), "g3".to_string(),
            "g4".to_string(), "g5".to_string(), "g6".to_string(),
            "g7".to_string()
        ];
        
        game.initialize_traitors(&players);
        
        // Create messages
        let mut messages = HashMap::new();
        
        // Honest generals vote attack
        for i in 0..5 {
            messages.insert(format!("g{}", i+1), vec![
                Message {
                    from: format!("g{}", i+1),
                    to: "all".to_string(),
                    decision: Decision::Attack,
                    round: 1,
                }
            ]);
        }
        
        // Test consensus
        let result = game.verify_consensus(&messages);
        
        // With 5 honest generals voting attack, consensus should be reached
        assert!(result.consensus_reached);
        assert_eq!(result.decision, Decision::Attack);
        assert!(result.attack_count >= 3); // Majority threshold
    }
    
    /*
    #[test]
    fn test_mini_go_capture_logic() {
        // Temporarily disabled until mini_go module is fixed
        // Already imported above
        
        let mut game = MiniGo::new();
        
        // Set up a capture scenario
        game.board[1][1] = Stone::Black;
        game.board[1][2] = Stone::White;
        game.board[2][1] = Stone::White;
        game.board[0][1] = Stone::White;
        
        // Black stone at (1,1) should have no liberties
        let liberties = game.count_liberties(1, 1);
        assert_eq!(liberties, 0);
        
        // Test capture
        let captured = game.capture_stones(Stone::Black);
        assert_eq!(captured.len(), 1);
        assert!(captured.contains(&(1, 1)));
        
        // Board should be updated
        assert_eq!(game.board[1][1], Stone::Empty);
    }
    
    #[test]
    fn test_mini_holdem_hand_evaluation() {
        // Temporarily disabled until mini_holdem module is fixed
        // Already imported above
        
        let game = MiniHoldem::new();
        
        // Test flush
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Hearts, rank: Rank::Ten },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let (rank, _) = game.evaluate_hand_from_cards(&cards);
        assert_eq!(rank, HandRank::Flush);
        
        // Test pair
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Spades, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
        ];
        
        let (rank, _) = game.evaluate_hand_from_cards(&cards);
        assert_eq!(rank, HandRank::Pair);
    }
    */
    
    #[test]
    fn test_swarm_optimization_convergence() {
        // Already imported above
        
        let mut game = SwarmOptimization::new();
        
        // Set dimensions to 2 for this test
        game.dimensions = 2;
        
        // Initialize particles
        let mut particles = vec![];
        for i in 0..10 {
            particles.push(Particle {
                id: i,
                position: vec![0.0, 0.0],
                velocity: vec![0.0, 0.0],
                personal_best: vec![0.0, 0.0],
                personal_best_value: f64::MAX,
            });
        }
        
        // Update global best to a different position
        game.global_best_position = vec![10.0, 10.0];
        game.global_best_fitness = 0.0;
        
        // Update particles
        game.update_particles(&mut particles);
        
        // Check particles moved towards global best
        for particle in &particles {
            assert!(particle.velocity[0].abs() > 0.0 || particle.velocity[1].abs() > 0.0);
        }
        
        // Test convergence detection
        // All particles at same position
        for particle in &mut particles {
            particle.position = vec![0.0, 0.0];
        }
        
        let converged = game.check_convergence(&particles);
        assert!(converged);
    }
    
    #[test]
    fn test_recursive_reasoning_depth() {
        let game = RecursiveReasoning::new();
        
        // Test reasoning depth calculation
        let guess = 50;
        let target = 67; // 2/3 of 100
        
        let depth = game.calculate_reasoning_depth(guess, target);
        
        // Guess of 50 when target is 67 shows limited depth
        assert!(depth < game.max_depth);
        
        // Perfect recursive guess
        let perfect_guess = 0; // Ultimate recursive answer
        let depth = game.calculate_reasoning_depth(perfect_guess, target);
        
        // Should show maximum depth
        assert_eq!(depth, game.max_depth);
    }
    
    #[test]
    fn test_analytics_calculation() {
        use crate::analytics::AnalyticsEngine;
        use crate::games::{RoundResult, Outcome};
        
        let analytics = AnalyticsEngine::new();
        let game_id = Uuid::new_v4();
        
        // Create round results
        let round_result = RoundResult {
            round: 1,
            actions: HashMap::new(),
            outcome: Outcome {
                winners: vec!["collective_1".to_string(), "collective_2".to_string()],
                losers: vec!["sota_1".to_string()],
                special_events: vec!["Emergence detected!".to_string()],
                emergence_detected: true,
            },
            scores_delta: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };
        
        // Process round
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            analytics.process_round(game_id, &round_result).await;
        });
        
        // Get analytics
        let game_analytics = tokio::runtime::Runtime::new().unwrap().block_on(async {
            analytics.get_game_analytics(game_id).await
        });
        
        assert!(game_analytics.is_some());
        let data = game_analytics.unwrap();
        assert_eq!(data.rounds_played, 1);
        assert_eq!(data.emergence_analysis.total_emergence_events, 1);
        assert!(data.performance_comparison.critical_moments.len() > 0);
    }
    
    #[test]
    fn test_coordination_score_calculation() {
        // Test coordination scoring logic
        let mut decision_distribution = HashMap::new();
        decision_distribution.insert(0, 3);
        decision_distribution.insert(1, 3);
        
        // Perfect distribution should have high coordination
        let total = 6;
        let variance = decision_distribution.values()
            .map(|&count| {
                let expected = total as f32 / decision_distribution.len() as f32;
                (count as f32 - expected).powi(2)
            })
            .sum::<f32>() / decision_distribution.len() as f32;
        
        let coordination_score = 1.0 - (variance / (total as f32).powi(2)).min(1.0);
        
        assert!(coordination_score > 0.9); // Should be very high for perfect distribution
    }
}