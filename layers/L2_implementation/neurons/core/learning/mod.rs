//! Learning and backward propagation system for HAL9

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;

pub mod gradient;
pub mod pattern;
pub mod adjuster;

pub use gradient::{ErrorGradient, ErrorContext, Adjustment, GradientCalculator};
pub use pattern::{ErrorPattern, PatternMatcher, Mitigation};
pub use adjuster::{PromptAdjuster, ConnectionWeightManager};

/// Types of errors that can occur in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ErrorType {
    /// Task-level errors
    TaskFailed { 
        reason: String 
    },
    IncorrectOutput { 
        expected: String, 
        actual: String 
    },
    
    /// Performance errors
    Timeout { 
        duration_ms: u64 
    },
    ResourceExhausted { 
        resource: String 
    },
    
    /// Quality errors
    LowQuality { 
        score: f32 
    },
    UserRejection { 
        feedback: String 
    },
    
    /// System errors
    ToolExecutionFailed { 
        tool: String, 
        error: String 
    },
    CommunicationError { 
        target: String 
    },
}

impl ErrorType {
    /// Get a signature string for pattern matching
    pub fn signature(&self) -> String {
        match self {
            ErrorType::TaskFailed { .. } => "task_failed".to_string(),
            ErrorType::IncorrectOutput { .. } => "incorrect_output".to_string(),
            ErrorType::Timeout { .. } => "timeout".to_string(),
            ErrorType::ResourceExhausted { resource } => format!("resource_exhausted:{}", resource),
            ErrorType::LowQuality { .. } => "low_quality".to_string(),
            ErrorType::UserRejection { .. } => "user_rejection".to_string(),
            ErrorType::ToolExecutionFailed { tool, .. } => format!("tool_failed:{}", tool),
            ErrorType::CommunicationError { target } => format!("comm_error:{}", target),
        }
    }
    
    /// Get the severity/magnitude of this error type
    pub fn default_magnitude(&self) -> f32 {
        match self {
            ErrorType::TaskFailed { .. } => 0.8,
            ErrorType::IncorrectOutput { .. } => 0.7,
            ErrorType::Timeout { .. } => 0.5,
            ErrorType::ResourceExhausted { .. } => 0.6,
            ErrorType::LowQuality { .. } => 0.6,
            ErrorType::UserRejection { .. } => 0.9,
            ErrorType::ToolExecutionFailed { .. } => 0.4,
            ErrorType::CommunicationError { .. } => 0.5,
        }
    }
}

/// Configuration for backward propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackwardPropagationConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    #[serde(default = "default_learning_rate")]
    pub learning_rate: f32,
    
    #[serde(default = "default_pattern_threshold")]
    pub pattern_threshold: usize,
    
    #[serde(default = "default_adjustment_decay")]
    pub adjustment_decay: f32,
    
    #[serde(default = "default_max_gradient_depth")]
    pub max_gradient_depth: usize,
    
    #[serde(default)]
    pub error_weights: HashMap<String, f32>,
}

impl Default for BackwardPropagationConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            learning_rate: default_learning_rate(),
            pattern_threshold: default_pattern_threshold(),
            adjustment_decay: default_adjustment_decay(),
            max_gradient_depth: default_max_gradient_depth(),
            error_weights: default_error_weights(),
        }
    }
}

// Default values
fn default_enabled() -> bool { true }
fn default_learning_rate() -> f32 { 0.1 }
fn default_pattern_threshold() -> usize { 3 }
fn default_adjustment_decay() -> f32 { 0.95 }
fn default_max_gradient_depth() -> usize { 3 }
fn default_error_weights() -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("task_failed".to_string(), 0.8);
    weights.insert("incorrect_output".to_string(), 0.7);
    weights.insert("timeout".to_string(), 0.5);
    weights.insert("low_quality".to_string(), 0.6);
    weights.insert("user_rejection".to_string(), 0.9);
    weights.insert("tool_execution_failed".to_string(), 0.4);
    weights
}

/// Result of processing that might contain errors
#[derive(Debug)]
pub struct ProcessingResult {
    pub success: bool,
    pub output: Option<String>,
    pub quality_score: Option<f32>,
    pub duration: Duration,
    pub error: Option<crate::Error>,
}

/// Learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub total_errors: u64,
    pub patterns_identified: u64,
    pub adjustments_made: u64,
    pub error_reduction_rate: f32,
    pub last_learning_event: Option<DateTime<Utc>>,
}

// Conversion from config module types
impl From<crate::config::BackwardPropagationConfig> for BackwardPropagationConfig {
    fn from(config: crate::config::BackwardPropagationConfig) -> Self {
        Self {
            enabled: config.enabled,
            learning_rate: config.learning_rate,
            pattern_threshold: config.pattern_threshold,
            adjustment_decay: config.adjustment_decay,
            max_gradient_depth: config.max_gradient_depth,
            error_weights: HashMap::new(), // Use defaults
        }
    }
}