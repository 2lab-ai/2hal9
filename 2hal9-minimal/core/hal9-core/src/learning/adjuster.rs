//! Adjustment mechanisms for learning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Adjustment, ErrorType};
use crate::Result;

/// Manages prompt adjustments for neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAdjuster {
    neuron_id: String,
    base_prompt: String,
    adjustments: Vec<PromptAdjustment>,
    current_prompt: String,
    learning_rate: f32,
}

/// Individual prompt adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAdjustment {
    pub timestamp: DateTime<Utc>,
    pub trigger_error: ErrorType,
    pub adjustment_type: AdjustmentType,
    pub content: String,
    pub effectiveness: Option<f32>,
}

/// Types of prompt adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    PrependContext(String),
    AppendGuideline(String),
    ReplaceSection { old: String, new: String },
    AddConstraint(String),
    RemoveConstraint(String),
}

impl PromptAdjuster {
    pub fn new(neuron_id: String, base_prompt: String, learning_rate: f32) -> Self {
        Self {
            neuron_id,
            current_prompt: base_prompt.clone(),
            base_prompt,
            adjustments: Vec::new(),
            learning_rate,
        }
    }

    /// Apply an adjustment based on error feedback
    pub fn apply_adjustment(&mut self, error: &ErrorType, adjustment: &Adjustment) -> Result<()> {
        let adj_type = self.determine_adjustment_type(error, adjustment)?;

        let prompt_adjustment = PromptAdjustment {
            timestamp: Utc::now(),
            trigger_error: error.clone(),
            adjustment_type: adj_type.clone(),
            content: adjustment
                .suggested_value
                .as_str()
                .unwrap_or("")
                .to_string(),
            effectiveness: None,
        };

        // Apply to current prompt
        self.current_prompt = self.apply_adjustment_to_prompt(&self.current_prompt, &adj_type)?;
        self.adjustments.push(prompt_adjustment);

        Ok(())
    }

    /// Determine adjustment type based on error and suggestion
    fn determine_adjustment_type(
        &self,
        error: &ErrorType,
        adjustment: &Adjustment,
    ) -> Result<AdjustmentType> {
        match &adjustment.parameter[..] {
            "add_context" => Ok(AdjustmentType::PrependContext(
                adjustment
                    .suggested_value
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            )),
            "add_guideline" => Ok(AdjustmentType::AppendGuideline(
                adjustment
                    .suggested_value
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            )),
            "add_constraint" => Ok(AdjustmentType::AddConstraint(
                adjustment
                    .suggested_value
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            )),
            _ => {
                // Default: add guideline based on error
                let guideline = match error {
                    ErrorType::Timeout { .. } => {
                        "Focus on efficiency and avoid complex operations."
                    }
                    ErrorType::ToolExecutionFailed { tool, .. } => {
                        &format!("Validate parameters before using the {} tool.", tool)
                    }
                    ErrorType::IncorrectOutput { .. } => {
                        "Double-check output format and content accuracy."
                    }
                    _ => "Be more careful to avoid errors.",
                };
                Ok(AdjustmentType::AppendGuideline(guideline.to_string()))
            }
        }
    }

    /// Apply adjustment to a prompt string
    fn apply_adjustment_to_prompt(
        &self,
        prompt: &str,
        adjustment: &AdjustmentType,
    ) -> Result<String> {
        match adjustment {
            AdjustmentType::PrependContext(context) => Ok(format!("{}\n\n{}", context, prompt)),
            AdjustmentType::AppendGuideline(guideline) => {
                Ok(format!("{}\n\nAdditional guideline: {}", prompt, guideline))
            }
            AdjustmentType::ReplaceSection { old, new } => Ok(prompt.replace(old, new)),
            AdjustmentType::AddConstraint(constraint) => {
                Ok(format!("{}\n\nConstraint: {}", prompt, constraint))
            }
            AdjustmentType::RemoveConstraint(constraint) => {
                Ok(prompt.replace(&format!("Constraint: {}", constraint), ""))
            }
        }
    }

    /// Get the current adjusted prompt
    pub fn get_current_prompt(&self) -> &str {
        &self.current_prompt
    }

    /// Record effectiveness of recent adjustments
    pub fn record_effectiveness(&mut self, success_rate: f32) {
        // Update effectiveness of recent adjustments
        let recent_count = 3;
        let len = self.adjustments.len();
        if len > 0 {
            let start = len.saturating_sub(recent_count);
            for adj in &mut self.adjustments[start..] {
                adj.effectiveness = Some(success_rate);
            }
        }
    }

    /// Rollback ineffective adjustments
    pub fn rollback_ineffective(&mut self, threshold: f32) {
        self.adjustments
            .retain(|adj| adj.effectiveness.map(|e| e >= threshold).unwrap_or(true));

        // Rebuild prompt from base + effective adjustments
        self.current_prompt = self.base_prompt.clone();
        for adj in &self.adjustments {
            if let Ok(new_prompt) =
                self.apply_adjustment_to_prompt(&self.current_prompt, &adj.adjustment_type)
            {
                self.current_prompt = new_prompt;
            }
        }
    }
}

/// Manages connection weights between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionWeightManager {
    weights: HashMap<String, ConnectionWeight>,
    decay_factor: f32,
}

/// Weight for a connection between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionWeight {
    pub from_neuron: String,
    pub to_neuron: String,
    pub weight: f32,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_adjusted: DateTime<Utc>,
}

impl ConnectionWeightManager {
    pub fn new(decay_factor: f32) -> Self {
        Self {
            weights: HashMap::new(),
            decay_factor,
        }
    }

    /// Get connection key
    fn get_key(from: &str, to: &str) -> String {
        format!("{}->{}", from, to)
    }

    /// Get or create weight for connection
    pub fn get_weight(&mut self, from: &str, to: &str) -> f32 {
        let key = Self::get_key(from, to);
        self.weights.get(&key).map(|w| w.weight).unwrap_or(0.5) // Default weight
    }

    /// Record successful signal routing
    pub fn record_success(&mut self, from: &str, to: &str) {
        let key = Self::get_key(from, to);
        let weight = self.weights.entry(key).or_insert(ConnectionWeight {
            from_neuron: from.to_string(),
            to_neuron: to.to_string(),
            weight: 0.5,
            success_count: 0,
            failure_count: 0,
            last_adjusted: Utc::now(),
        });

        weight.success_count += 1;
        weight.last_adjusted = Utc::now();

        // Increase weight (up to 1.0)
        weight.weight = (weight.weight + 0.1 * self.decay_factor).min(1.0);
    }

    /// Record failed signal routing
    pub fn record_failure(&mut self, from: &str, to: &str) {
        let key = Self::get_key(from, to);
        let weight = self.weights.entry(key).or_insert(ConnectionWeight {
            from_neuron: from.to_string(),
            to_neuron: to.to_string(),
            weight: 0.5,
            success_count: 0,
            failure_count: 0,
            last_adjusted: Utc::now(),
        });

        weight.failure_count += 1;
        weight.last_adjusted = Utc::now();

        // Decrease weight (down to 0.0)
        weight.weight = (weight.weight - 0.2 * self.decay_factor).max(0.0);
    }

    /// Get best connection for routing
    pub fn get_best_connection(&self, from: &str, targets: &[String]) -> Option<String> {
        targets
            .iter()
            .map(|to| {
                let key = Self::get_key(from, to);
                let weight = self.weights.get(&key).map(|w| w.weight).unwrap_or(0.5);
                (to.clone(), weight)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(target, _)| target)
    }

    /// Apply time-based decay to all weights
    pub fn apply_decay(&mut self) {
        let now = Utc::now();
        for weight in self.weights.values_mut() {
            let days_elapsed = (now - weight.last_adjusted).num_days() as f32;
            if days_elapsed > 0.0 {
                // Decay towards 0.5 (neutral)
                let decay = self.decay_factor.powf(days_elapsed);
                weight.weight = weight.weight * decay + 0.5 * (1.0 - decay);
            }
        }
    }

    /// Get statistics for a connection
    pub fn get_stats(&self, from: &str, to: &str) -> Option<&ConnectionWeight> {
        let key = Self::get_key(from, to);
        self.weights.get(&key)
    }
}
