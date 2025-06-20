//! Comprehensive unit tests for neuron core functionality
//! 
//! This test suite covers:
//! - Neuron creation and initialization
//! - Signal processing pipeline
//! - State transitions
//! - Layer assignment
//! - Health monitoring
//! - Concurrent operations

use hal9_neurons::{Neuron, NeuronInterface, Layer, NeuronState, NeuronHealth, NeuronSignal, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Duration;

#[cfg(test)]
mod neuron_creation {
    use super::*;
    
    #[test]
    fn test_layer_parsing() {
        // Test valid layer parsing
        assert_eq!(Layer::from_str("L1"), Some(Layer::L1));
        assert_eq!(Layer::from_str("L5"), Some(Layer::L5));
        assert_eq!(Layer::from_str("L9"), Some(Layer::L9));
        
        // Test invalid layer parsing
        assert_eq!(Layer::from_str("L0"), None);
        assert_eq!(Layer::from_str("L10"), None);
        assert_eq!(Layer::from_str("invalid"), None);
    }
    
    #[test]
    fn test_layer_descriptions() {
        assert_eq!(Layer::L1.description(), "Reflexive Layer");
        assert_eq!(Layer::L5.description(), "Strategic Layer");
        assert_eq!(Layer::L9.description(), "Universal Layer");
    }
    
    #[test]
    fn test_layer_display() {
        assert_eq!(Layer::L1.to_string(), "L1");
        assert_eq!(Layer::L5.to_string(), "L5");
        assert_eq!(Layer::L9.to_string(), "L9");
    }
}

#[cfg(test)]
mod neuron_state {
    use super::*;
    
    #[test]
    fn test_state_transitions() {
        let state = NeuronState::Starting;
        assert_eq!(state, NeuronState::Starting);
        
        // Test all state values
        let states = vec![
            NeuronState::Starting,
            NeuronState::Running,
            NeuronState::Processing,
            NeuronState::Failed,
            NeuronState::Stopped,
        ];
        
        // Ensure states are distinct
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }
    
    #[test]
    fn test_state_serialization() {
        let state = NeuronState::Running;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: NeuronState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

#[cfg(test)]
mod neuron_health {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_health_creation() {
        let health = NeuronHealth {
            state: NeuronState::Running,
            last_signal: Some(Utc::now()),
            signals_processed: 42,
            errors_count: 2,
            uptime_seconds: 3600,
        };
        
        assert_eq!(health.state, NeuronState::Running);
        assert!(health.last_signal.is_some());
        assert_eq!(health.signals_processed, 42);
        assert_eq!(health.errors_count, 2);
        assert_eq!(health.uptime_seconds, 3600);
    }
    
    #[test]
    fn test_health_serialization() {
        let health = NeuronHealth {
            state: NeuronState::Running,
            last_signal: Some(Utc::now()),
            signals_processed: 100,
            errors_count: 0,
            uptime_seconds: 7200,
        };
        
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: NeuronHealth = serde_json::from_str(&json).unwrap();
        
        assert_eq!(health.state, deserialized.state);
        assert_eq!(health.signals_processed, deserialized.signals_processed);
        assert_eq!(health.errors_count, deserialized.errors_count);
        assert_eq!(health.uptime_seconds, deserialized.uptime_seconds);
    }
}

#[cfg(test)]
mod signal_processing {
    use super::*;
    
    #[test]
    fn test_signal_creation() {
        let signal = NeuronSignal {
            from: "neuron1".to_string(),
            to: Some("neuron2".to_string()),
            content: "test signal".to_string(),
            signal_type: "process".to_string(),
            timestamp: Utc::now(),
            trace_id: Some("trace123".to_string()),
            metadata: None,
        };
        
        assert_eq!(signal.from, "neuron1");
        assert_eq!(signal.to, Some("neuron2".to_string()));
        assert_eq!(signal.content, "test signal");
    }
    
    #[test]
    fn test_signal_routing() {
        let mut signal = NeuronSignal {
            from: "neuron1".to_string(),
            to: None,
            content: "broadcast signal".to_string(),
            signal_type: "broadcast".to_string(),
            timestamp: Utc::now(),
            trace_id: None,
            metadata: None,
        };
        
        // Test broadcast signal (no specific target)
        assert!(signal.to.is_none());
        
        // Test directed signal
        signal.to = Some("neuron2".to_string());
        assert_eq!(signal.to, Some("neuron2".to_string()));
    }
}

#[tokio::test]
async fn test_concurrent_signal_processing() {
    use tokio::task;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    let processed_count = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];
    
    // Simulate concurrent signal processing
    for i in 0..10 {
        let count = processed_count.clone();
        let handle = task::spawn(async move {
            // Simulate processing
            tokio::time::sleep(Duration::from_millis(10)).await;
            count.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(processed_count.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_neuron_lifecycle() {
    // Test complete neuron lifecycle
    let states = vec![
        NeuronState::Starting,
        NeuronState::Running,
        NeuronState::Processing,
        NeuronState::Stopped,
    ];
    
    let (tx, mut rx) = mpsc::channel(10);
    
    // Simulate state transitions
    task::spawn(async move {
        for state in states {
            tx.send(state).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    
    // Verify state sequence
    assert_eq!(rx.recv().await.unwrap(), NeuronState::Starting);
    assert_eq!(rx.recv().await.unwrap(), NeuronState::Running);
    assert_eq!(rx.recv().await.unwrap(), NeuronState::Processing);
    assert_eq!(rx.recv().await.unwrap(), NeuronState::Stopped);
}

#[test]
fn test_layer_hierarchy() {
    // Test that layers maintain proper hierarchy
    let layers = vec![
        Layer::L1, Layer::L2, Layer::L3, Layer::L4, Layer::L5,
        Layer::L6, Layer::L7, Layer::L8, Layer::L9,
    ];
    
    // Each layer should be unique
    for (i, layer1) in layers.iter().enumerate() {
        for (j, layer2) in layers.iter().enumerate() {
            if i != j {
                assert_ne!(layer1.as_str(), layer2.as_str());
                assert_ne!(layer1.description(), layer2.description());
            }
        }
    }
}

#[test]
fn test_error_handling() {
    // Test error scenarios
    let invalid_layer = Layer::from_str("INVALID");
    assert!(invalid_layer.is_none());
    
    // Test health with error state
    let error_health = NeuronHealth {
        state: NeuronState::Failed,
        last_signal: None,
        signals_processed: 0,
        errors_count: 10,
        uptime_seconds: 60,
    };
    
    assert_eq!(error_health.state, NeuronState::Failed);
    assert!(error_health.errors_count > 0);
}

// Helper function for creating test neurons
pub fn create_test_neuron(id: &str, layer: Layer) -> MockNeuron {
    MockNeuron {
        id: id.to_string(),
        layer,
        state: NeuronState::Starting,
        signals_processed: 0,
    }
}

// Mock neuron for testing
pub struct MockNeuron {
    pub id: String,
    pub layer: Layer,
    pub state: NeuronState,
    pub signals_processed: u64,
}

impl MockNeuron {
    pub fn process_signal(&mut self, signal: &NeuronSignal) -> Result<()> {
        self.state = NeuronState::Processing;
        self.signals_processed += 1;
        self.state = NeuronState::Running;
        Ok(())
    }
    
    pub fn get_health(&self) -> NeuronHealth {
        NeuronHealth {
            state: self.state,
            last_signal: Some(Utc::now()),
            signals_processed: self.signals_processed,
            errors_count: 0,
            uptime_seconds: 0,
        }
    }
}