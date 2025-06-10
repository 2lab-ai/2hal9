//! Error gradient calculation and propagation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;

use super::ErrorType;
use crate::{Error, Gradient};

/// Error gradient for backward propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGradient {
    pub id: Uuid,
    pub error_type: ErrorType,
    pub magnitude: f32,
    pub source_neuron: String,
    pub target_neuron: String,
    pub timestamp: DateTime<Utc>,
    pub context: ErrorContext,
    pub suggested_adjustments: Vec<Adjustment>,
    pub propagation_depth: usize,
}

/// Context surrounding an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub original_task: String,
    pub attempted_solution: String,
    pub failure_point: String,
    pub environmental_factors: HashMap<String, Value>,
}

/// Suggested adjustment to prevent future errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    pub parameter: String,
    pub current_value: Value,
    pub suggested_value: Value,
    pub confidence: f32,
    pub rationale: String,
}

impl ErrorGradient {
    /// Create a new error gradient
    pub fn new(
        error_type: ErrorType,
        source_neuron: String,
        target_neuron: String,
        context: ErrorContext,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            error_type: error_type.clone(),
            magnitude: error_type.default_magnitude(),
            source_neuron,
            target_neuron,
            timestamp: Utc::now(),
            context,
            suggested_adjustments: Vec::new(),
            propagation_depth: 0,
        }
    }
    
    /// Add a suggested adjustment
    pub fn add_adjustment(&mut self, adjustment: Adjustment) {
        self.suggested_adjustments.push(adjustment);
    }
    
    /// Create a propagated gradient (for sending upstream)
    pub fn propagate(&self, new_target: String) -> Self {
        let mut propagated = self.clone();
        propagated.id = Uuid::new_v4();
        propagated.source_neuron = self.target_neuron.clone();
        propagated.target_neuron = new_target;
        propagated.propagation_depth += 1;
        propagated.magnitude *= 0.9; // Decay magnitude as we go up
        propagated.timestamp = Utc::now();
        propagated
    }
    
    /// Convert to signal gradient format
    pub fn to_signal_gradient(&self) -> Gradient {
        let adjustments = self.suggested_adjustments.iter()
            .map(|a| format!("{}: {} -> {}", a.parameter, a.current_value, a.suggested_value))
            .collect();
            
        Gradient {
            error_type: format!("{:?}", self.error_type),
            magnitude: self.magnitude,
            adjustments,
            loss: self.magnitude, // Simplified loss calculation
        }
    }
}

/// Gradient calculator for analyzing errors
pub struct GradientCalculator {
    _config: super::BackwardPropagationConfig,
}

impl GradientCalculator {
    pub fn new(config: super::BackwardPropagationConfig) -> Self {
        Self { _config: config }
    }
    
    /// Calculate gradient from a processing error
    pub fn calculate_from_error(
        &self,
        error: &Error,
        source_neuron: &str,
        target_neuron: &str,
        task: &str,
        attempted_solution: &str,
    ) -> ErrorGradient {
        let error_type = self.classify_error(error);
        
        let context = ErrorContext {
            original_task: task.to_string(),
            attempted_solution: attempted_solution.to_string(),
            failure_point: error.to_string(),
            environmental_factors: HashMap::new(),
        };
        
        let mut gradient = ErrorGradient::new(
            error_type,
            source_neuron.to_string(),
            target_neuron.to_string(),
            context,
        );
        
        // Add adjustments based on error type
        self.suggest_adjustments(&mut gradient, error);
        
        gradient
    }
    
    /// Classify system error into error type
    fn classify_error(&self, error: &Error) -> ErrorType {
        match error {
            Error::Timeout(duration) => ErrorType::Timeout { 
                duration_ms: duration * 1000 
            },
            Error::CostLimit { reason } => ErrorType::ResourceExhausted { 
                resource: format!("cost: {}", reason) 
            },
            Error::ToolExecution(msg) => {
                let tool = msg.split(':').next().unwrap_or("unknown");
                ErrorType::ToolExecutionFailed { 
                    tool: tool.to_string(), 
                    error: msg.clone() 
                }
            },
            Error::Communication(msg) | Error::Network(msg) => {
                ErrorType::CommunicationError { 
                    target: msg.clone() 
                }
            },
            _ => ErrorType::TaskFailed { 
                reason: error.to_string() 
            },
        }
    }
    
    /// Suggest adjustments based on error type
    fn suggest_adjustments(&self, gradient: &mut ErrorGradient, _error: &Error) {
        match &gradient.error_type {
            ErrorType::Timeout { duration_ms } => {
                gradient.add_adjustment(Adjustment {
                    parameter: "processing_timeout".to_string(),
                    current_value: json!(*duration_ms),
                    suggested_value: json!(duration_ms * 2),
                    confidence: 0.8,
                    rationale: "Double timeout to prevent future timeouts".to_string(),
                });
                
                gradient.add_adjustment(Adjustment {
                    parameter: "task_complexity_limit".to_string(),
                    current_value: json!(null),
                    suggested_value: json!("medium"),
                    confidence: 0.6,
                    rationale: "Limit task complexity to reduce processing time".to_string(),
                });
            }
            
            ErrorType::ToolExecutionFailed { tool, .. } => {
                gradient.add_adjustment(Adjustment {
                    parameter: format!("tool_{}_validation", tool),
                    current_value: json!(false),
                    suggested_value: json!(true),
                    confidence: 0.9,
                    rationale: format!("Enable validation for {} tool", tool),
                });
            }
            
            ErrorType::ResourceExhausted { resource } => {
                if resource.contains("cost") {
                    gradient.add_adjustment(Adjustment {
                        parameter: "use_mock_mode".to_string(),
                        current_value: json!(false),
                        suggested_value: json!(true),
                        confidence: 0.95,
                        rationale: "Switch to mock mode when approaching cost limits".to_string(),
                    });
                }
            }
            
            _ => {
                // Generic adjustment for other errors
                gradient.add_adjustment(Adjustment {
                    parameter: "error_handling_verbosity".to_string(),
                    current_value: json!("normal"),
                    suggested_value: json!("detailed"),
                    confidence: 0.5,
                    rationale: "Increase verbosity to better diagnose errors".to_string(),
                });
            }
        }
    }
}

use serde_json::json;