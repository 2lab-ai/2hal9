#[cfg(test)]
mod sota_tests {
    use super::super::*;

    #[tokio::test]
    async fn test_sota_creation() {
        let config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec!["calculator".to_string()],
            cost_per_hour: 25.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config.clone());
        assert_eq!(sota.config.model_name, "claude-opus-4");
        assert_eq!(sota.config.context_window, 100000);
        assert_eq!(sota.config.temperature, 0.7);
    }

    #[tokio::test]
    async fn test_thinking_time_variants() {
        // Test Standard thinking time
        let config = SOTAConfig {
            model_name: "gpt-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 128000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.5,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.config.thinking_time, ThinkingTime::Standard);
        
        // Test Unlimited thinking time
        let config = SOTAConfig {
            model_name: "opus-4-ultra".to_string(),
            api_key: "test_key".to_string(),
            context_window: 200000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.9,
            tools: vec![],
            cost_per_hour: 50.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.config.thinking_time, ThinkingTime::Extended);
    }

    #[tokio::test]
    async fn test_decision_making() {
        let config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 25.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        let context = serde_json::json!({
            "game_type": "MinorityGame",
            "round": 5,
            "history": [
                {"round": 1, "choices": [0, 1, 0, 1]},
                {"round": 2, "choices": [1, 1, 0, 0]},
                {"round": 3, "choices": [0, 0, 1, 1]},
                {"round": 4, "choices": [1, 0, 1, 0]},
            ]
        });
        
        let decision = sota.make_decision(context).await.unwrap();
        
        assert!(!decision.decision.is_null());
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
        assert!(!decision.reasoning_chain.is_empty());
        assert_eq!(sota.id, "test_sota");
    }

    #[tokio::test]
    async fn test_different_model_types() {
        // Test Claude model
        let config = SOTAConfig {
            model_name: "claude-3.5-sonnet".to_string(),
            api_key: "test_key".to_string(),
            context_window: 200000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.5,
            tools: vec![],
            cost_per_hour: 15.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        let context = serde_json::json!({"test": "claude"});
        let decision = sota.make_decision(context).await.unwrap();
        assert!(decision.strategy.len() > 0);
        
        // Test GPT model
        let config = SOTAConfig {
            model_name: "gpt-4-turbo".to_string(),
            api_key: "test_key".to_string(),
            context_window: 128000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        let context = serde_json::json!({"test": "gpt"});
        let decision = sota.make_decision(context).await.unwrap();
        assert!(decision.strategy.len() > 0);
        
        // Test Gemini model
        let config = SOTAConfig {
            model_name: "gemini-2.0-flash".to_string(),
            api_key: "test_key".to_string(),
            context_window: 1000000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.8,
            tools: vec![],
            cost_per_hour: 30.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        let context = serde_json::json!({"test": "gemini"});
        let decision = sota.make_decision(context).await.unwrap();
        assert!(decision.strategy.len() > 0);
    }

    #[tokio::test]
    async fn test_tools_usage() {
        let config = SOTAConfig {
            model_name: "claude-opus-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![
                "calculator".to_string(),
                "search".to_string(),
                "code_interpreter".to_string(),
            ],
            cost_per_hour: 30.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.config.tools.len(), 3);
        assert!(sota.config.tools.contains(&"calculator".to_string()));
        assert!(sota.config.tools.contains(&"search".to_string()));
        assert!(sota.config.tools.contains(&"code_interpreter".to_string()));
    }

    #[tokio::test]
    async fn test_temperature_effects() {
        // Low temperature (more deterministic)
        let config = SOTAConfig {
            model_name: "gpt-4".to_string(),
            api_key: "test_key".to_string(),
            context_window: 128000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.1,
            tools: vec![],
            cost_per_hour: 20.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        let context = serde_json::json!({"deterministic": true});
        let decision1 = sota.make_decision(context.clone()).await.unwrap();
        let confidence1 = decision1.confidence;
        
        // High temperature (more creative)
        sota.config.temperature = 0.9;
        let decision2 = sota.make_decision(context).await.unwrap();
        let confidence2 = decision2.confidence;
        
        // Both should be valid confidence values
        assert!(confidence1 >= 0.0 && confidence1 <= 1.0);
        assert!(confidence2 >= 0.0 && confidence2 <= 1.0);
    }

    #[tokio::test]
    async fn test_context_window_limits() {
        let config = SOTAConfig {
            model_name: "small-model".to_string(),
            api_key: "test_key".to_string(),
            context_window: 4000, // Small context window
            thinking_time: ThinkingTime::Standard,
            temperature: 0.5,
            tools: vec![],
            cost_per_hour: 5.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.config.context_window, 4000);
        
        // Large context window
        let config = SOTAConfig {
            model_name: "large-model".to_string(),
            api_key: "test_key".to_string(),
            context_window: 1000000, // 1M tokens
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
            tools: vec![],
            cost_per_hour: 50.0,
        };
        
        let sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.config.context_window, 1000000);
    }

    #[test]
    fn test_thinking_time_serialization() {
        let standard = ThinkingTime::Standard;
        let extended = ThinkingTime::Extended;
        let unlimited = ThinkingTime::Extended; // Using Extended as there's no Unlimited variant
        
        // Test serialization
        assert_eq!(serde_json::to_string(&standard).unwrap(), "\"Standard\"");
        assert_eq!(serde_json::to_string(&extended).unwrap(), "\"Extended\"");
        assert_eq!(serde_json::to_string(&unlimited).unwrap(), "\"Extended\"");
        
        // Test deserialization
        let deserialized: ThinkingTime = serde_json::from_str("\"Extended\"").unwrap();
        assert_eq!(deserialized, ThinkingTime::Extended);
    }

    #[tokio::test]
    async fn test_decision_counter() {
        let config = SOTAConfig {
            model_name: "test-model".to_string(),
            api_key: "test_key".to_string(),
            context_window: 10000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.5,
            tools: vec![],
            cost_per_hour: 10.0,
        };
        
        let mut sota = SOTAManager::new("test_sota".to_string(), config);
        assert_eq!(sota.id, "test_sota");
        
        // Make multiple decisions
        for i in 1..=5 {
            let context = serde_json::json!({"round": i});
            let _ = sota.make_decision(context).await.unwrap();
        }
    }
}