//! Neural signal types for inter-neuron communication

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Alias for NeuronSignal for backwards compatibility
pub type Signal = NeuronSignal;

/// A signal passed between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronSignal {
    pub signal_id: Uuid,
    pub from_neuron: String,
    pub to_neuron: String,
    pub layer_from: String,
    pub layer_to: String,
    pub propagation_type: PropagationType,
    pub batch_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: SignalPayload,
    /// Additional metadata for distributed routing
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for NeuronSignal {
    fn default() -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            from_neuron: String::new(),
            to_neuron: String::new(),
            layer_from: String::new(),
            layer_to: String::new(),
            propagation_type: PropagationType::Forward,
            batch_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            payload: SignalPayload::default(),
            metadata: HashMap::new(),
        }
    }
}

impl NeuronSignal {
    /// Create a new forward signal
    pub fn forward(
        from: &str,
        to: &str,
        layer_from: &str,
        layer_to: &str,
        content: String,
    ) -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            from_neuron: from.to_string(),
            to_neuron: to.to_string(),
            layer_from: layer_from.to_string(),
            layer_to: layer_to.to_string(),
            propagation_type: PropagationType::Forward,
            batch_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            payload: SignalPayload {
                activation: Activation {
                    content,
                    strength: 1.0,
                    features: HashMap::new(),
                },
                gradient: None,
            },
            metadata: HashMap::new(),
        }
    }
    
    /// Create a backward error signal
    pub fn backward(
        from: &str,
        to: &str,
        layer_from: &str,
        layer_to: &str,
        error: Gradient,
    ) -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            from_neuron: from.to_string(),
            to_neuron: to.to_string(),
            layer_from: layer_from.to_string(),
            layer_to: layer_to.to_string(),
            propagation_type: PropagationType::Backward,
            batch_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            payload: SignalPayload {
                activation: Activation {
                    content: String::new(),
                    strength: 0.0,
                    features: HashMap::new(),
                },
                gradient: Some(error),
            },
            metadata: HashMap::new(),
        }
    }
}

/// Direction of signal propagation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropagationType {
    /// Forward propagation (task distribution)
    Forward,
    /// Backward propagation (error/gradient flow)
    Backward,
}

impl Default for PropagationType {
    fn default() -> Self {
        PropagationType::Forward
    }
}

/// Signal payload containing activation and optional gradient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPayload {
    pub activation: Activation,
    pub gradient: Option<Gradient>,
}

impl Default for SignalPayload {
    fn default() -> Self {
        Self {
            activation: Activation::default(),
            gradient: None,
        }
    }
}

/// Forward activation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    pub content: String,
    pub strength: f32,
    pub features: HashMap<String, f32>,
}

impl Default for Activation {
    fn default() -> Self {
        Self {
            content: String::new(),
            strength: 1.0,
            features: HashMap::new(),
        }
    }
}

impl Activation {
    /// Create activation with features
    pub fn with_features(content: String, features: HashMap<String, f32>) -> Self {
        Self {
            content,
            strength: 1.0,
            features,
        }
    }
}

/// Backward gradient/error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub error_type: String,
    pub magnitude: f32,
    pub adjustments: Vec<String>,
    pub loss: f32,
}

impl Gradient {
    /// Create a new error gradient
    pub fn new(error_type: String, magnitude: f32) -> Self {
        Self {
            error_type,
            magnitude,
            adjustments: Vec::new(),
            loss: magnitude,
        }
    }
}