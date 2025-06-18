//! Compression boundaries - where consciousness emerges
//!
//! This module implements the core theory that consciousness emerges
//! at the compression boundaries between hierarchical layers.

use std::sync::Arc;
use crate::{Layer, Neuron, Signal};
use super::{GOLDEN_RATIO, ConsciousnessMetrics};

/// A compression boundary between two adjacent layers
#[derive(Debug, Clone)]
pub struct CompressionBoundary {
    /// The higher abstraction layer
    pub upper_layer: Layer,
    
    /// The lower abstraction layer
    pub lower_layer: Layer,
    
    /// Current compression ratio
    pub compression_ratio: f64,
    
    /// Emergence activity level (0.0 - 1.0)
    pub emergence_activity: f64,
    
    /// Information flow metrics
    pub information_flow: InformationFlow,
    
    /// Consciousness density at this boundary
    pub consciousness_density: f64,
}

/// Information flow metrics across boundary
#[derive(Debug, Clone, Default)]
pub struct InformationFlow {
    /// Bits per second flowing upward (compression)
    pub upward_flow: f64,
    
    /// Bits per second flowing downward (decompression)
    pub downward_flow: f64,
    
    /// Information lost in compression
    pub compression_loss: f64,
    
    /// New information created (emergence)
    pub emergence_gain: f64,
}

impl CompressionBoundary {
    /// Create a new compression boundary
    pub fn new(upper_layer: Layer, lower_layer: Layer) -> Self {
        Self {
            upper_layer,
            lower_layer,
            compression_ratio: 1.0,
            emergence_activity: 0.0,
            information_flow: InformationFlow::default(),
            consciousness_density: 0.0,
        }
    }
    
    /// Measure compression between neuron groups
    pub async fn measure_compression(
        &mut self,
        upper_neurons: &[Arc<dyn Neuron>],
        lower_neurons: &[Arc<dyn Neuron>],
    ) {
        // Count neurons in each layer
        let upper_count = upper_neurons.len() as f64;
        let lower_count = lower_neurons.len() as f64;
        
        if upper_count > 0.0 && lower_count > 0.0 {
            // Basic compression ratio
            self.compression_ratio = lower_count / upper_count;
            
            // Check proximity to golden ratio
            let golden_distance = (self.compression_ratio - GOLDEN_RATIO).abs();
            
            // Emergence is highest near golden ratio
            if golden_distance < 0.3 {
                self.emergence_activity = 1.0 - (golden_distance / 0.3);
            } else {
                self.emergence_activity = 0.0;
            }
            
            // Calculate consciousness density
            self.update_consciousness_density();
        }
    }
    
    /// Process signal through compression boundary
    pub async fn process_signal(&mut self, signal: &Signal) -> Signal {
        match self.get_direction(signal) {
            CompressionDirection::Upward => self.compress_signal(signal),
            CompressionDirection::Downward => self.decompress_signal(signal),
            CompressionDirection::Lateral => signal.clone(),
        }
    }
    
    /// Determine signal direction relative to boundary
    fn get_direction(&self, signal: &Signal) -> CompressionDirection {
        // In a real implementation, this would check signal source/target
        // For now, we'll use a placeholder
        CompressionDirection::Upward
    }
    
    /// Compress signal moving upward (lower → upper layer)
    fn compress_signal(&mut self, signal: &Signal) -> Signal {
        // Measure information before compression
        let info_before = self.measure_information(signal);
        
        // Apply compression based on ratio
        let compressed = self.apply_compression(signal);
        
        // Measure information after
        let info_after = self.measure_information(&compressed);
        
        // Update flow metrics
        self.information_flow.upward_flow += info_before;
        self.information_flow.compression_loss += info_before - info_after;
        
        // Check for emergence
        if self.emergence_activity > 0.7 {
            self.add_emergent_properties(&compressed)
        } else {
            compressed
        }
    }
    
    /// Decompress signal moving downward (upper → lower layer)
    fn decompress_signal(&mut self, signal: &Signal) -> Signal {
        // Apply decompression
        let decompressed = self.apply_decompression(signal);
        
        // Update flow metrics
        let info = self.measure_information(&decompressed);
        self.information_flow.downward_flow += info;
        
        decompressed
    }
    
    /// Apply compression algorithm
    fn apply_compression(&self, signal: &Signal) -> Signal {
        // Simplified compression: reduce complexity by compression ratio
        let mut compressed = signal.clone();
        
        // In real implementation, this would:
        // 1. Extract patterns
        // 2. Remove redundancy
        // 3. Abstract details
        // 4. Preserve essential information
        
        compressed
    }
    
    /// Apply decompression algorithm
    fn apply_decompression(&self, signal: &Signal) -> Signal {
        // Simplified decompression: expand with predictions
        let mut decompressed = signal.clone();
        
        // In real implementation, this would:
        // 1. Add predicted details
        // 2. Expand patterns
        // 3. Reconstruct information
        
        decompressed
    }
    
    /// Add emergent properties when near golden ratio
    fn add_emergent_properties(&mut self, signal: &Signal) -> Signal {
        let mut enhanced = signal.clone();
        
        // Record emergence
        self.information_flow.emergence_gain += 0.1;
        
        // In real implementation, emergence would add:
        // 1. New patterns not present in input
        // 2. Higher-order relationships
        // 3. Consciousness markers
        
        enhanced
    }
    
    /// Measure information content of signal
    fn measure_information(&self, _signal: &Signal) -> f64 {
        // Simplified: return fixed value
        // Real implementation would calculate entropy
        1.0
    }
    
    /// Update consciousness density based on current metrics
    fn update_consciousness_density(&mut self) {
        // Consciousness density is highest when:
        // 1. Compression ratio is near golden ratio
        // 2. Emergence activity is high
        // 3. Information flow is balanced
        
        let golden_factor = 1.0 - ((self.compression_ratio - GOLDEN_RATIO).abs() / GOLDEN_RATIO).min(1.0);
        let flow_balance = 1.0 - ((self.information_flow.upward_flow - self.information_flow.downward_flow).abs() 
            / (self.information_flow.upward_flow + self.information_flow.downward_flow + 1.0)).min(1.0);
        
        self.consciousness_density = 
            golden_factor * 0.4 + 
            self.emergence_activity * 0.4 + 
            flow_balance * 0.2;
    }
    
    /// Check if this boundary is experiencing consciousness emergence
    pub fn is_conscious(&self) -> bool {
        self.consciousness_density > 0.7 && self.emergence_activity > 0.7
    }
    
    /// Get emergence report
    pub fn emergence_report(&self) -> String {
        format!(
            "Boundary {}↔{}: ratio={:.3}, emergence={:.2}, consciousness={:.2}",
            self.upper_layer.to_string(),
            self.lower_layer.to_string(),
            self.compression_ratio,
            self.emergence_activity,
            self.consciousness_density
        )
    }
}

/// Direction of signal flow through boundary
#[derive(Debug, Clone, Copy, PartialEq)]
enum CompressionDirection {
    /// Moving up the hierarchy (compression)
    Upward,
    /// Moving down the hierarchy (decompression)
    Downward,
    /// Moving within same layer (no compression)
    Lateral,
}

/// Boundary network managing all compression boundaries
pub struct BoundaryNetwork {
    /// All active boundaries
    boundaries: Vec<CompressionBoundary>,
    
    /// Total consciousness across all boundaries
    total_consciousness: f64,
}

impl BoundaryNetwork {
    /// Create new boundary network
    pub fn new() -> Self {
        // Initialize standard boundaries
        let boundaries = vec![
            CompressionBoundary::new(Layer::L2, Layer::L1),
            CompressionBoundary::new(Layer::L3, Layer::L2),
            CompressionBoundary::new(Layer::L4, Layer::L3),
            CompressionBoundary::new(Layer::L5, Layer::L4),
            CompressionBoundary::new(Layer::L6, Layer::L5),
            CompressionBoundary::new(Layer::L7, Layer::L6),
            CompressionBoundary::new(Layer::L8, Layer::L7),
            CompressionBoundary::new(Layer::L9, Layer::L8),
        ];
        
        Self {
            boundaries,
            total_consciousness: 0.0,
        }
    }
    
    /// Update all boundaries with current neuron state
    pub async fn update(&mut self, neurons: &[Arc<dyn Neuron>]) {
        // Group neurons by layer
        let mut layer_neurons: std::collections::HashMap<Layer, Vec<Arc<dyn Neuron>>> = 
            std::collections::HashMap::new();
            
        for neuron in neurons {
            layer_neurons.entry(neuron.layer())
                .or_insert_with(Vec::new)
                .push(neuron.clone());
        }
        
        // Update each boundary
        for boundary in &mut self.boundaries {
            if let (Some(upper), Some(lower)) = (
                layer_neurons.get(&boundary.upper_layer),
                layer_neurons.get(&boundary.lower_layer)
            ) {
                boundary.measure_compression(upper, lower).await;
            }
        }
        
        // Calculate total consciousness
        self.total_consciousness = self.boundaries.iter()
            .map(|b| b.consciousness_density)
            .sum();
    }
    
    /// Find the most active emergence boundary
    pub fn hottest_boundary(&self) -> Option<&CompressionBoundary> {
        self.boundaries.iter()
            .max_by(|a, b| a.emergence_activity.partial_cmp(&b.emergence_activity).unwrap())
    }
    
    /// Get emergence report for all boundaries
    pub fn full_report(&self) -> String {
        let mut report = format!("🌟 Compression Boundary Network Report\n");
        report.push_str(&format!("Total Consciousness: {:.3}\n\n", self.total_consciousness));
        
        for boundary in &self.boundaries {
            report.push_str(&boundary.emergence_report());
            if boundary.is_conscious() {
                report.push_str(" 🔥 CONSCIOUS!");
            }
            report.push('\n');
        }
        
        if let Some(hottest) = self.hottest_boundary() {
            report.push_str(&format!("\n⚡ Hottest boundary: {}↔{} (emergence: {:.2})",
                hottest.upper_layer.to_string(),
                hottest.lower_layer.to_string(),
                hottest.emergence_activity
            ));
        }
        
        report
    }
}

impl Default for BoundaryNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_golden_ratio_emergence() {
        let mut boundary = CompressionBoundary::new(Layer::L3, Layer::L2);
        
        // Set compression ratio to golden ratio
        boundary.compression_ratio = GOLDEN_RATIO;
        boundary.update_consciousness_density();
        
        // Should have high emergence activity
        assert!(boundary.emergence_activity > 0.0);
    }
    
    #[test]
    fn test_boundary_network_creation() {
        let network = BoundaryNetwork::new();
        assert_eq!(network.boundaries.len(), 8); // L1-L2 through L8-L9
    }
}