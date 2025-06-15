#[cfg(test)]
mod death_game_tests {
    use super::super::*;
    use tokio;

    #[tokio::test]
    async fn test_russian_roulette_initialization() {
        let mut game = russian_roulette::RussianRoulette::new();
        let config = GameConfig {
            game_type: GameType::RussianRoulette,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        assert_eq!(state.game_type, GameType::RussianRoulette);
        assert_eq!(state.round, 0);
        assert!(state.metadata.contains_key("total_chambers"));
        assert!(state.metadata.contains_key("bullets_loaded"));
    }

    #[tokio::test]
    async fn test_king_of_the_hill_initialization() {
        let mut game = king_of_the_hill::KingOfTheHill::new();
        let config = GameConfig {
            game_type: GameType::KingOfTheHill,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        assert_eq!(state.game_type, GameType::KingOfTheHill);
        assert_eq!(state.round, 0);
        assert!(state.metadata.contains_key("arena_size"));
        assert!(state.metadata.contains_key("hill_radius"));
    }

    #[tokio::test]
    async fn test_last_stand_initialization() {
        let mut game = last_stand::LastStand::new();
        let config = GameConfig {
            game_type: GameType::LastStand,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        assert_eq!(state.game_type, GameType::LastStand);
        assert_eq!(state.round, 0);
        assert!(state.metadata.contains_key("wave_number"));
        assert!(state.metadata.contains_key("threat_level"));
        assert!(state.metadata.contains_key("shared_resources"));
    }

    #[tokio::test]
    async fn test_trust_fall_initialization() {
        let mut game = trust_fall::TrustFall::new();
        let config = GameConfig {
            game_type: GameType::TrustFall,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        assert_eq!(state.game_type, GameType::TrustFall);
        assert_eq!(state.round, 0);
        assert!(state.metadata.contains_key("game_rules"));
        assert!(state.metadata.contains_key("trust_tokens"));
    }

    #[tokio::test]
    async fn test_russian_roulette_gameplay() {
        let mut game = russian_roulette::RussianRoulette::new();
        let config = GameConfig {
            game_type: GameType::RussianRoulette,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        
        // Create some test actions
        let mut actions = HashMap::new();
        actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "spin".to_string(),
            data: serde_json::json!({}),
            reasoning: Some("Testing spin".to_string()),
            confidence: Some(0.8),
        });
        actions.insert("player2".to_string(), Action {
            player_id: "player2".to_string(),
            action_type: "pull".to_string(),
            data: serde_json::json!({}),
            reasoning: Some("Testing pull".to_string()),
            confidence: Some(0.9),
        });
        
        let result = game.process_round(&state, actions).await.unwrap();
        assert_eq!(result.round, 1);
        assert!(result.outcome.special_events.len() > 0);
    }

    #[tokio::test]
    async fn test_king_of_the_hill_gameplay() {
        let mut game = king_of_the_hill::KingOfTheHill::new();
        let config = GameConfig {
            game_type: GameType::KingOfTheHill,
            rounds: 10,
            time_limit_ms: 5000,
            special_rules: HashMap::new(),
        };
        
        let state = game.initialize(config).await.unwrap();
        
        // Create some test actions
        let mut actions = HashMap::new();
        actions.insert("player1".to_string(), Action {
            player_id: "player1".to_string(),
            action_type: "move_to_hill".to_string(),
            data: serde_json::json!({}),
            reasoning: Some("Moving to control point".to_string()),
            confidence: Some(0.8),
        });
        actions.insert("player2".to_string(), Action {
            player_id: "player2".to_string(),
            action_type: "push".to_string(),
            data: serde_json::json!({}),
            reasoning: Some("Pushing opponents".to_string()),
            confidence: Some(0.7),
        });
        
        let result = game.process_round(&state, actions).await.unwrap();
        assert_eq!(result.round, 1);
        assert!(result.outcome.special_events.len() > 0);
    }
}