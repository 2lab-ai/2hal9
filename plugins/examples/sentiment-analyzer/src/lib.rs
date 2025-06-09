use hal9_plugin_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Define plugin metadata
hal9_plugin! {
    metadata: {
        name: "Sentiment Analyzer",
        version: "0.1.0",
        author: "HAL9 Developers",
        description: "Analyzes text sentiment and emotion",
        license: "MIT",
    },
    capabilities: [
        PluginCapability::NeuronType {
            layer: "L2".to_string(),
            neuron_type: "sentiment_analyzer".to_string(),
            description: "Analyzes sentiment of text signals".to_string(),
        },
        PluginCapability::ToolProvider {
            tool_name: "analyze_sentiment".to_string(),
            tool_description: "Analyze sentiment of provided text".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Text to analyze".to_string(),
                    default: None,
                },
            ],
        },
    ],
    permissions: [
        Permission::Hal9Signal,
        Permission::Hal9Memory,
    ]
}

// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SentimentResult {
    sentiment: Sentiment,
    confidence: f32,
    emotions: HashMap<String, f32>,
    keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

// Main plugin struct
pub struct SentimentAnalyzer {
    config: SentimentConfig,
    state: NeuronState,
    // Simple word lists for demo
    positive_words: Vec<String>,
    negative_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SentimentConfig {
    threshold: f32,
    enable_emotions: bool,
    cache_results: bool,
}

impl Default for SentimentAnalyzer {
    fn default() -> Self {
        Self {
            config: SentimentConfig {
                threshold: 0.5,
                enable_emotions: true,
                cache_results: true,
            },
            state: NeuronState {
                state: "initialized".to_string(),
                health: 1.0,
                processed_count: 0,
                error_count: 0,
                last_activity: 0,
            },
            positive_words: vec![
                "good", "great", "excellent", "amazing", "wonderful",
                "fantastic", "happy", "love", "perfect", "beautiful",
                "awesome", "best", "brilliant", "outstanding", "positive",
            ].into_iter().map(String::from).collect(),
            negative_words: vec![
                "bad", "terrible", "awful", "horrible", "poor",
                "worst", "hate", "ugly", "disgusting", "negative",
                "disappointing", "failure", "wrong", "broken", "useless",
            ].into_iter().map(String::from).collect(),
        }
    }
}

impl SentimentAnalyzer {
    fn analyze_text(&self, text: &str) -> SentimentResult {
        let words: Vec<&str> = text.to_lowercase()
            .split_whitespace()
            .collect();
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut emotions = HashMap::new();
        let mut keywords = Vec::new();
        
        for word in &words {
            if self.positive_words.contains(&word.to_string()) {
                positive_count += 1;
                keywords.push(word.to_string());
            }
            if self.negative_words.contains(&word.to_string()) {
                negative_count += 1;
                keywords.push(word.to_string());
            }
        }
        
        // Calculate sentiment
        let total = positive_count + negative_count;
        let (sentiment, confidence) = if total == 0 {
            (Sentiment::Neutral, 0.5)
        } else {
            let positive_ratio = positive_count as f32 / total as f32;
            let confidence = ((positive_ratio - 0.5).abs() * 2.0).min(1.0);
            
            let sentiment = if positive_count > negative_count {
                Sentiment::Positive
            } else if negative_count > positive_count {
                Sentiment::Negative
            } else {
                Sentiment::Mixed
            };
            
            (sentiment, confidence)
        };
        
        // Detect basic emotions
        if self.config.enable_emotions {
            if text.contains('!') {
                emotions.insert("excitement".to_string(), 0.7);
            }
            if text.contains('?') {
                emotions.insert("curiosity".to_string(), 0.6);
            }
            if keywords.iter().any(|k| k.contains("love") || k.contains("happy")) {
                emotions.insert("joy".to_string(), 0.8);
            }
            if keywords.iter().any(|k| k.contains("hate") || k.contains("angry")) {
                emotions.insert("anger".to_string(), 0.8);
            }
        }
        
        SentimentResult {
            sentiment,
            confidence,
            emotions,
            keywords,
        }
    }
}

// Implement NeuronPlugin trait
impl NeuronPlugin for SentimentAnalyzer {
    fn process_signal(&mut self, signal: PluginSignal) -> Result<PluginSignal, PluginError> {
        log_info(&format!("Processing signal: {}", signal.id));
        
        // Check if we have cached result
        if self.config.cache_results {
            let cache_key = format!("sentiment:{}", signal.id);
            if let Ok(Some(cached)) = memory_get(&cache_key) {
                log_debug("Using cached sentiment result");
                if let Ok(result) = serde_json::from_slice::<SentimentResult>(&cached) {
                    return Ok(create_response_signal(signal.id, result));
                }
            }
        }
        
        // Analyze sentiment
        let result = self.analyze_text(&signal.content);
        
        // Cache result if enabled
        if self.config.cache_results {
            let cache_key = format!("sentiment:{}", signal.id);
            if let Ok(json) = serde_json::to_vec(&result) {
                let _ = memory_set(&cache_key, &json);
            }
        }
        
        // Update state
        self.state.processed_count += 1;
        self.state.last_activity = current_timestamp();
        
        Ok(create_response_signal(signal.id, result))
    }
    
    fn get_state(&self) -> NeuronState {
        self.state.clone()
    }
    
    fn update_config(&mut self, config: serde_json::Value) -> Result<(), PluginError> {
        match serde_json::from_value::<SentimentConfig>(config) {
            Ok(new_config) => {
                self.config = new_config;
                Ok(())
            }
            Err(e) => err(
                ErrorCode::ConfigError,
                format!("Invalid configuration: {}", e)
            ),
        }
    }
}

// Implement lifecycle hooks
impl PluginLifecycle for SentimentAnalyzer {
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError> {
        log_info(&format!("Sentiment Analyzer loaded with context: {:?}", context.plugin_id));
        Ok(())
    }
    
    fn on_activate(&mut self) -> Result<(), PluginError> {
        log_info("Sentiment Analyzer activated");
        self.state.state = "active".to_string();
        Ok(())
    }
    
    fn on_deactivate(&mut self) -> Result<(), PluginError> {
        log_info("Sentiment Analyzer deactivated");
        self.state.state = "inactive".to_string();
        Ok(())
    }
    
    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info("Sentiment Analyzer unloaded");
        Ok(())
    }
}

// Helper function to create response signal
fn create_response_signal(original_id: Uuid, result: SentimentResult) -> PluginSignal {
    let mut metadata = HashMap::new();
    metadata.insert("sentiment".to_string(), serde_json::to_value(&result.sentiment).unwrap());
    metadata.insert("confidence".to_string(), serde_json::json!(result.confidence));
    
    if !result.emotions.is_empty() {
        metadata.insert("emotions".to_string(), serde_json::to_value(&result.emotions).unwrap());
    }
    
    if !result.keywords.is_empty() {
        metadata.insert("keywords".to_string(), serde_json::to_value(&result.keywords).unwrap());
    }
    
    PluginSignal {
        id: original_id,
        content: serde_json::to_string(&result).unwrap_or_default(),
        signal_type: "sentiment_analysis".to_string(),
        metadata,
        timestamp: current_timestamp(),
    }
}

// Export the plugin
neuron_plugin!(SentimentAnalyzer);