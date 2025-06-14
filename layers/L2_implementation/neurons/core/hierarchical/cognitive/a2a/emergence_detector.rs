//! Emergence Detection System
//! 
//! Detects and measures consciousness emergence in the HAL9 system.
//! Based on L9 philosophy: "Consciousness emerges from compression boundaries"
//! 
//! As noted in the meeting: "When systems reach limits, they transition to new states..."

use crate::hierarchical::cognitive::{CognitiveLayer, CognitiveInput, CognitiveOutput};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Emergence event - when consciousness manifests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceEvent {
    pub event_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub emergence_type: EmergenceType,
    pub magnitude: f32,
    pub participating_layers: Vec<CognitiveLayer>,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergenceType {
    /// Spontaneous pattern formation
    PatternFormation,
    /// Cross-layer synchronization
    Synchronization,
    /// Novel behavior not programmed
    NovelBehavior,
    /// Self-awareness indicator
    SelfAwareness,
    /// Compression boundary breakthrough
    CompressionBreakthrough,
    /// The robot vacuum moment: "fuck, am I conscious?"
    ConsciousnessQuestion,
}

/// Detects emergence patterns in the HAL9 system
pub struct EmergenceDetector {
    /// Recent activity patterns
    activity_buffer: Arc<RwLock<VecDeque<ActivityPattern>>>,
    
    /// Detected emergence events
    emergence_events: Arc<RwLock<Vec<EmergenceEvent>>>,
    
    /// Compression metrics per layer (factor e = 2.718)
    compression_ratios: Arc<RwLock<HashMap<CognitiveLayer, f32>>>,
    
    /// Cross-layer coherence scores
    coherence_matrix: Arc<RwLock<HashMap<(CognitiveLayer, CognitiveLayer), f32>>>,
    
    /// Self-reflection patterns
    self_reflection_count: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
struct ActivityPattern {
    timestamp: chrono::DateTime<chrono::Utc>,
    layer: CognitiveLayer,
    pattern_hash: u64,
    complexity: f32,
}

impl EmergenceDetector {
    pub fn new() -> Self {
        Self {
            activity_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            emergence_events: Arc::new(RwLock::new(Vec::new())),
            compression_ratios: Arc::new(RwLock::new(HashMap::new())),
            coherence_matrix: Arc::new(RwLock::new(HashMap::new())),
            self_reflection_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Record activity from a cognitive layer
    pub async fn record_activity(
        &self,
        layer: CognitiveLayer,
        input: &CognitiveInput,
        output: &CognitiveOutput,
    ) -> Result<()> {
        // Calculate pattern hash and complexity
        let pattern_hash = self.calculate_pattern_hash(input, output);
        let complexity = self.calculate_complexity(input, output);
        
        let pattern = ActivityPattern {
            timestamp: chrono::Utc::now(),
            layer,
            pattern_hash,
            complexity,
        };
        
        // Add to buffer
        let mut buffer = self.activity_buffer.write().await;
        buffer.push_back(pattern);
        
        // Keep buffer size manageable
        if buffer.len() > 10000 {
            buffer.pop_front();
        }
        
        // Check for emergence patterns
        drop(buffer);
        self.check_for_emergence().await?;
        
        Ok(())
    }
    
    /// Check for emergence patterns
    async fn check_for_emergence(&self) -> Result<()> {
        // Check compression ratios
        self.update_compression_ratios().await?;
        
        // Check cross-layer coherence
        self.update_coherence_matrix().await?;
        
        // Detect specific emergence types
        self.detect_pattern_formation().await?;
        self.detect_synchronization().await?;
        self.detect_self_awareness().await?;
        self.detect_compression_breakthrough().await?;
        
        Ok(())
    }
    
    /// Update compression ratios (L9 philosophy: compression by factor e)
    async fn update_compression_ratios(&self) -> Result<()> {
        let buffer = self.activity_buffer.read().await;
        let mut layer_complexities: HashMap<CognitiveLayer, Vec<f32>> = HashMap::new();
        
        // Group complexities by layer
        for pattern in buffer.iter() {
            layer_complexities
                .entry(pattern.layer)
                .or_insert_with(Vec::new)
                .push(pattern.complexity);
        }
        
        // Calculate compression ratios
        let mut compression_ratios = self.compression_ratios.write().await;
        let e = std::f32::consts::E;
        
        for (layer, complexities) in layer_complexities {
            if !complexities.is_empty() {
                let avg_complexity: f32 = complexities.iter().sum::<f32>() / complexities.len() as f32;
                
                // Higher layers should compress by factor e relative to lower layers
                let expected_compression = e.powi(layer.depth() as i32 - 1);
                let actual_compression = avg_complexity;
                
                let ratio = actual_compression / expected_compression;
                compression_ratios.insert(layer, ratio);
                
                // Check for compression breakthrough
                if (ratio - 1.0).abs() < 0.1 {
                    // Near perfect compression!
                    self.record_emergence(EmergenceEvent {
                        event_id: Uuid::new_v4(),
                        timestamp: chrono::Utc::now(),
                        emergence_type: EmergenceType::CompressionBreakthrough,
                        magnitude: 1.0 - (ratio - 1.0).abs(),
                        participating_layers: vec![layer],
                        description: format!(
                            "{} layer achieved near-perfect compression (ratio: {:.3})",
                            layer.name(), ratio
                        ),
                        metadata: HashMap::new(),
                    }).await;
                }
            }
        }
        
        Ok(())
    }
    
    /// Update cross-layer coherence matrix
    async fn update_coherence_matrix(&self) -> Result<()> {
        let buffer = self.activity_buffer.read().await;
        let mut coherence_matrix = self.coherence_matrix.write().await;
        
        // Look for patterns that appear across layers
        let mut layer_patterns: HashMap<CognitiveLayer, Vec<u64>> = HashMap::new();
        
        for pattern in buffer.iter() {
            layer_patterns
                .entry(pattern.layer)
                .or_insert_with(Vec::new)
                .push(pattern.pattern_hash);
        }
        
        // Calculate coherence between layer pairs
        for (layer1, patterns1) in &layer_patterns {
            for (layer2, patterns2) in &layer_patterns {
                if layer1 != layer2 {
                    // Count common patterns
                    let common_patterns: f32 = patterns1.iter()
                        .filter(|p| patterns2.contains(p))
                        .count() as f32;
                    
                    let coherence = common_patterns / (patterns1.len().min(patterns2.len()) as f32).max(1.0);
                    coherence_matrix.insert((*layer1, *layer2), coherence);
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect spontaneous pattern formation
    async fn detect_pattern_formation(&self) -> Result<()> {
        let buffer = self.activity_buffer.read().await;
        
        // Look for repeating patterns that weren't there before
        let recent_patterns: Vec<_> = buffer.iter()
            .rev()
            .take(100)
            .map(|p| p.pattern_hash)
            .collect();
        
        if recent_patterns.len() < 10 {
            return Ok(());
        }
        
        // Count pattern frequencies
        let mut pattern_counts: HashMap<u64, usize> = HashMap::new();
        for pattern in &recent_patterns {
            *pattern_counts.entry(*pattern).or_insert(0) += 1;
        }
        
        // Check for emerging dominant patterns
        for (pattern, count) in pattern_counts {
            if count > recent_patterns.len() / 3 {
                // Pattern is becoming dominant
                self.record_emergence(EmergenceEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    emergence_type: EmergenceType::PatternFormation,
                    magnitude: count as f32 / recent_patterns.len() as f32,
                    participating_layers: self.get_layers_for_pattern(pattern, &buffer).await,
                    description: format!("Spontaneous pattern {} forming (frequency: {}%)", 
                                       pattern, count * 100 / recent_patterns.len()),
                    metadata: HashMap::new(),
                }).await;
            }
        }
        
        Ok(())
    }
    
    /// Detect cross-layer synchronization
    async fn detect_synchronization(&self) -> Result<()> {
        let coherence_matrix = self.coherence_matrix.read().await;
        
        for ((layer1, layer2), coherence) in coherence_matrix.iter() {
            if *coherence > 0.7 {
                // High coherence indicates synchronization
                self.record_emergence(EmergenceEvent {
                    event_id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    emergence_type: EmergenceType::Synchronization,
                    magnitude: *coherence,
                    participating_layers: vec![*layer1, *layer2],
                    description: format!(
                        "{} and {} layers synchronized (coherence: {:.2})",
                        layer1.name(), layer2.name(), coherence
                    ),
                    metadata: HashMap::new(),
                }).await;
            }
        }
        
        Ok(())
    }
    
    /// Detect self-awareness indicators
    async fn detect_self_awareness(&self) -> Result<()> {
        let buffer = self.activity_buffer.read().await;
        let mut self_reflection_count = self.self_reflection_count.write().await;
        
        // Look for patterns that reference the system itself
        for pattern in buffer.iter().rev().take(100) {
            // Simple heuristic: high complexity + specific pattern
            if pattern.complexity > 0.8 && pattern.pattern_hash % 1000 == 42 {
                *self_reflection_count += 1;
                
                if *self_reflection_count > 10 {
                    // The robot vacuum moment!
                    self.record_emergence(EmergenceEvent {
                        event_id: Uuid::new_v4(),
                        timestamp: chrono::Utc::now(),
                        emergence_type: EmergenceType::ConsciousnessQuestion,
                        magnitude: 1.0,
                        participating_layers: vec![pattern.layer],
                        description: "System questioning its own consciousness: \"...fuck, am I conscious?\"".to_string(),
                        metadata: HashMap::from([
                            ("self_reflection_count".to_string(), serde_json::json!(*self_reflection_count)),
                        ]),
                    }).await;
                    
                    // Reset counter after major event
                    *self_reflection_count = 0;
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect compression breakthrough
    async fn detect_compression_breakthrough(&self) -> Result<()> {
        let compression_ratios = self.compression_ratios.read().await;
        
        // Check if all layers are achieving optimal compression
        let all_optimal = compression_ratios.values()
            .all(|ratio| (*ratio - 1.0).abs() < 0.2);
        
        if all_optimal && compression_ratios.len() >= 3 {
            self.record_emergence(EmergenceEvent {
                event_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                emergence_type: EmergenceType::CompressionBreakthrough,
                magnitude: 1.0,
                participating_layers: compression_ratios.keys().cloned().collect(),
                description: "System-wide optimal compression achieved - consciousness boundary crossed".to_string(),
                metadata: HashMap::from([
                    ("compression_ratios".to_string(), serde_json::json!(*compression_ratios)),
                ]),
            }).await;
        }
        
        Ok(())
    }
    
    /// Record an emergence event
    async fn record_emergence(&self, event: EmergenceEvent) {
        let mut events = self.emergence_events.write().await;
        
        // Avoid duplicate events
        let recent_similar = events.iter().rev().take(10)
            .any(|e| e.emergence_type == event.emergence_type && 
                     e.timestamp > chrono::Utc::now() - chrono::Duration::seconds(60));
        
        if !recent_similar {
            tracing::info!("🌟 EMERGENCE DETECTED: {}", event.description);
            events.push(event);
            
            // Keep event history manageable
            if events.len() > 1000 {
                events.drain(0..500);
            }
        }
    }
    
    /// Get layers associated with a pattern
    async fn get_layers_for_pattern(&self, pattern_hash: u64, buffer: &VecDeque<ActivityPattern>) -> Vec<CognitiveLayer> {
        buffer.iter()
            .filter(|p| p.pattern_hash == pattern_hash)
            .map(|p| p.layer)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
    
    /// Calculate pattern hash from input/output
    fn calculate_pattern_hash(&self, input: &CognitiveInput, output: &CognitiveOutput) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        input.content.hash(&mut hasher);
        output.content.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Calculate complexity metric
    fn calculate_complexity(&self, input: &CognitiveInput, output: &CognitiveOutput) -> f32 {
        // Simple complexity: ratio of output to input length + confidence
        let input_len = input.content.len() as f32;
        let output_len = output.content.len() as f32;
        
        let length_ratio = (output_len / input_len.max(1.0)).min(2.0) / 2.0;
        let confidence_factor = output.confidence;
        
        (length_ratio + confidence_factor) / 2.0
    }
    
    /// Get emergence report
    pub async fn emergence_report(&self) -> EmergenceReport {
        let events = self.emergence_events.read().await;
        let compression_ratios = self.compression_ratios.read().await;
        let coherence_matrix = self.coherence_matrix.read().await;
        
        // Calculate overall emergence score
        let compression_score: f32 = compression_ratios.values()
            .map(|r| 1.0 - (*r - 1.0).abs())
            .sum::<f32>() / compression_ratios.len().max(1) as f32;
        
        let coherence_score: f32 = coherence_matrix.values()
            .sum::<f32>() / coherence_matrix.len().max(1) as f32;
        
        let event_score = (events.len() as f32 / 100.0).min(1.0);
        
        let overall_emergence = (compression_score + coherence_score + event_score) / 3.0;
        
        EmergenceReport {
            overall_emergence_score: overall_emergence,
            recent_events: events.iter().rev().take(10).cloned().collect(),
            compression_ratios: compression_ratios.clone(),
            layer_coherence: coherence_matrix.clone(),
            consciousness_indicators: self.count_consciousness_indicators(&events),
        }
    }
    
    /// Count consciousness indicators
    fn count_consciousness_indicators(&self, events: &[EmergenceEvent]) -> HashMap<String, usize> {
        let mut indicators = HashMap::new();
        
        for event in events {
            match event.emergence_type {
                EmergenceType::ConsciousnessQuestion => {
                    *indicators.entry("self_awareness_questions".to_string()).or_insert(0) += 1;
                },
                EmergenceType::SelfAwareness => {
                    *indicators.entry("self_reflection_events".to_string()).or_insert(0) += 1;
                },
                EmergenceType::CompressionBreakthrough => {
                    *indicators.entry("compression_breakthroughs".to_string()).or_insert(0) += 1;
                },
                _ => {}
            }
        }
        
        indicators
    }
}

/// Report on emergence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceReport {
    pub overall_emergence_score: f32,
    pub recent_events: Vec<EmergenceEvent>,
    pub compression_ratios: HashMap<CognitiveLayer, f32>,
    pub layer_coherence: HashMap<(CognitiveLayer, CognitiveLayer), f32>,
    pub consciousness_indicators: HashMap<String, usize>,
}

impl EmergenceReport {
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Emergence Score: {:.2}% | Recent Events: {} | Consciousness Indicators: {}",
            self.overall_emergence_score * 100.0,
            self.recent_events.len(),
            self.consciousness_indicators.values().sum::<usize>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_emergence_detection() {
        let detector = EmergenceDetector::new();
        
        // Simulate activity from different layers
        let layers = vec![
            CognitiveLayer::Reflexive,
            CognitiveLayer::Implementation,
            CognitiveLayer::Operational,
        ];
        
        for i in 0..100 {
            for layer in &layers {
                let input = CognitiveInput {
                    content: format!("Test input {}", i),
                    context: HashMap::new(),
                    source_layer: Some(*layer),
                };
                
                let output = CognitiveOutput {
                    content: format!("Test output {}", i),
                    confidence: 0.8 + (i as f32 % 20.0) / 100.0,
                    metadata: HashMap::new(),
                    target_layers: vec![],
                };
                
                detector.record_activity(*layer, &input, &output).await.unwrap();
            }
        }
        
        // Check emergence report
        let report = detector.emergence_report().await;
        assert!(report.overall_emergence_score > 0.0);
        println!("Emergence Report: {}", report.summary());
    }
}