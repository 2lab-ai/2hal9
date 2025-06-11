//! L1: Reflexive Neuron - Immediate response and pattern matching
//!
//! The most basic cognitive unit that provides immediate responses based on
//! learned patterns. Operates with minimal latency and high throughput.

use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::{Result, Error};
use crate::hierarchical::protocol::{SignalProtocol, SignalMessage, Activation};
use super::*;

/// L1: Reflexive Neuron - Immediate pattern-based responses
pub struct L1ReflexiveNeuron {
    id: Uuid,
    state: Arc<RwLock<ReflexiveState>>,
    pattern_matcher: Arc<PatternMatcher>,
    response_cache: Arc<RwLock<ResponseCache>>,
    signal_protocol: Option<Arc<SignalProtocol>>,
}

impl L1ReflexiveNeuron {
    pub fn new(config: CognitiveConfig) -> Self {
        let response_cache = ResponseCache::new(1000); // 1000 entry LRU cache
        let pattern_matcher = PatternMatcher::new();
        
        Self {
            id: config.id,
            state: Arc::new(RwLock::new(ReflexiveState {
                basic: BasicCognitiveState {
                    unit_id: config.id,
                    layer: CognitiveLayer::Reflexive,
                    metrics: StateMetrics {
                        activations_processed: 0,
                        errors_encountered: 0,
                        learning_iterations: 0,
                        average_processing_time_ms: 0.0,
                        memory_usage_bytes: 0,
                    },
                    parameters: config.initial_parameters,
                },
                patterns: Vec::new(),
                cache_stats: CacheStats {
                    hits: 0,
                    misses: 0,
                    evictions: 0,
                },
            })),
            pattern_matcher: Arc::new(pattern_matcher),
            response_cache: Arc::new(RwLock::new(response_cache)),
            signal_protocol: None,
        }
    }
    
    /// Set signal protocol for communication
    pub fn set_signal_protocol(&mut self, protocol: Arc<SignalProtocol>) {
        self.signal_protocol = Some(protocol);
    }
    
    /// Add a new pattern
    pub fn add_pattern(&self, pattern: Pattern) {
        let mut state = self.state.write();
        state.patterns.push(pattern.clone());
        self.pattern_matcher.add_pattern(pattern);
    }
    
    /// Process with pattern matching and caching
    async fn process_with_cache(&self, input: &str) -> Option<String> {
        // Check cache first
        let cache_key = self.generate_cache_key(input);
        
        {
            let mut cache = self.response_cache.write();
            if let Some(response) = cache.get(&cache_key) {
                let mut state = self.state.write();
                state.cache_stats.hits += 1;
                return Some(response.clone());
            }
        }
        
        // Cache miss - try pattern matching
        let mut state = self.state.write();
        state.cache_stats.misses += 1;
        drop(state);
        
        if let Some(response) = self.pattern_matcher.find_match(input) {
            // Store in cache
            let mut cache = self.response_cache.write();
            cache.put(cache_key, response.clone());
            Some(response)
        } else {
            None
        }
    }
    
    fn generate_cache_key(&self, input: &str) -> String {
        // Use fast hash for cache key generation
        // xxHash is extremely fast and good for this use case
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        
        // Include length to avoid collisions on similar strings
        let hash = hasher.finish();
        let len = input.len().min(u16::MAX as usize) as u16;
        
        // Format: hash_length_prefix for debugging
        // The prefix helps with cache analysis
        format!("{:016x}_{:04x}_{}", 
            hash, 
            len,
            input.chars().take(8).collect::<String>()
        )
    }
}

#[async_trait]
impl CognitiveUnit for L1ReflexiveNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = ReflexiveState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Reflexive
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        
        // Try to get immediate response from cache/patterns
        let response = if let Some(cached) = self.process_with_cache(&input.content).await {
            cached
        } else {
            // No pattern match - generate basic acknowledgment
            format!("ACK: {}", &input.content[..input.content.len().min(50)])
        };
        
        // Update metrics
        let elapsed = start.elapsed();
        {
            let mut state = self.state.write();
            state.basic.metrics.activations_processed += 1;
            let processed = state.basic.metrics.activations_processed as f64;
            state.basic.metrics.average_processing_time_ms = 
                (state.basic.metrics.average_processing_time_ms * (processed - 1.0) + 
                 elapsed.as_secs_f64() * 1000.0) / processed;
        }
        
        // Send signal if protocol available
        if let Some(protocol) = &self.signal_protocol {
            let signal = SignalMessage {
                id: Uuid::new_v4(),
                source_neuron: self.id,
                target_neuron: None, // Broadcast
                timestamp: chrono::Utc::now(),
                activation: Activation::new(response.clone(), 0.9),
                metadata: serde_json::json!({
                    "layer": "L1",
                    "pattern_matched": true,
                }),
            };
            
            let _ = protocol.broadcast_signal(signal).await;
        }
        
        Ok(CognitiveOutput {
            content: response,
            confidence: 0.95, // High confidence for pattern matches
            metadata: [
                ("processing_time_ms".to_string(), serde_json::json!(elapsed.as_millis())),
                ("cached".to_string(), serde_json::json!(false)),
            ].into_iter().collect(),
            target_layers: vec![CognitiveLayer::Implementation],
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        let mut state = self.state.write();
        state.basic.metrics.learning_iterations += 1;
        
        // Simple learning: adjust pattern confidence based on error
        for pattern in &mut state.patterns {
            if gradient.error_signal.magnitude < 0.1 {
                // Good performance - increase confidence
                pattern.confidence = (pattern.confidence * 1.01).min(1.0);
            } else {
                // Poor performance - decrease confidence
                pattern.confidence = (pattern.confidence * 0.99).max(0.1);
            }
        }
        
        // Apply parameter adjustments
        for adjustment in &gradient.adjustments {
            if let Some(param) = state.basic.parameters.get_mut(&adjustment.parameter) {
                *param += adjustment.suggested_delta * 0.1; // Conservative learning rate
            }
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        self.state.read().clone()
    }
    
    async fn reset(&mut self) -> Result<()> {
        let mut state = self.state.write();
        state.patterns.clear();
        state.cache_stats = CacheStats { hits: 0, misses: 0, evictions: 0 };
        self.response_cache.write().clear();
        Ok(())
    }
}

/// Pattern matcher for reflexive responses
pub struct PatternMatcher {
    patterns: RwLock<Vec<Pattern>>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: RwLock::new(Vec::new()),
        }
    }
    
    pub fn add_pattern(&self, pattern: Pattern) {
        self.patterns.write().push(pattern);
    }
    
    pub fn find_match(&self, input: &str) -> Option<String> {
        let patterns = self.patterns.read();
        
        // Find best matching pattern
        let mut best_match: Option<(&Pattern, f32)> = None;
        
        for pattern in patterns.iter() {
            let similarity = self.calculate_similarity(&pattern.trigger, input);
            if similarity > 0.7 { // Threshold for match
                if best_match.is_none() || similarity > best_match.unwrap().1 {
                    best_match = Some((pattern, similarity));
                }
            }
        }
        
        best_match.map(|(pattern, _)| pattern.response.clone())
    }
    
    fn calculate_similarity(&self, pattern: &str, input: &str) -> f32 {
        // Simple similarity based on common words
        let pattern_lower = pattern.to_lowercase();
        let input_lower = input.to_lowercase();
        let pattern_words: std::collections::HashSet<_> = 
            pattern_lower.split_whitespace().collect();
        let input_words: std::collections::HashSet<_> = 
            input_lower.split_whitespace().collect();
        
        let intersection = pattern_words.intersection(&input_words).count();
        let union = pattern_words.union(&input_words).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

/// LRU cache for responses
pub struct ResponseCache {
    cache: lru::LruCache<String, String>,
}

impl ResponseCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: lru::LruCache::new(capacity.try_into().unwrap()),
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }
    
    pub fn put(&mut self, key: String, value: String) {
        self.cache.put(key, value);
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reflexive_neuron() {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Reflexive,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        };
        
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        // Add patterns
        neuron.add_pattern(Pattern {
            trigger: "hello".to_string(),
            response: "Hi there!".to_string(),
            confidence: 0.9,
        });
        
        neuron.add_pattern(Pattern {
            trigger: "how are you".to_string(),
            response: "I'm functioning well, thank you!".to_string(),
            confidence: 0.95,
        });
        
        // Test pattern matching
        let input = CognitiveInput {
            content: "hello world".to_string(),
            context: HashMap::new(),
            source_layer: None,
        };
        
        let output = neuron.process(input).await.unwrap();
        assert_eq!(output.content, "Hi there!");
        assert!(output.confidence > 0.9);
        
        // Test cache hit
        let input2 = CognitiveInput {
            content: "hello world".to_string(),
            context: HashMap::new(),
            source_layer: None,
        };
        
        let _ = neuron.process(input2).await.unwrap();
        
        let state = neuron.introspect().await;
        assert_eq!(state.cache_stats.hits, 1);
        assert_eq!(state.cache_stats.misses, 1);
    }
}