//! Error pattern recognition and mitigation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use super::{ErrorType, ErrorGradient};
use crate::{Result, Error};

/// Represents a recurring error pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_id: Uuid,
    pub error_signature: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrences: Vec<ErrorOccurrence>,
    pub successful_mitigations: Vec<Mitigation>,
    pub prevention_strategy: Option<String>,
    pub confidence: f32,
}

/// Single occurrence of an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorOccurrence {
    pub timestamp: DateTime<Utc>,
    pub neuron_id: String,
    pub error_gradient: ErrorGradient,
    pub was_mitigated: bool,
}

/// Successful mitigation of an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    pub mitigation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub strategy: String,
    pub adjustments_applied: Vec<String>,
    pub success_score: f32,
}

impl ErrorPattern {
    /// Create a new error pattern
    pub fn new(signature: String, initial_occurrence: ErrorOccurrence) -> Self {
        let now = Utc::now();
        Self {
            pattern_id: Uuid::new_v4(),
            error_signature: signature,
            first_seen: now,
            last_seen: now,
            occurrences: vec![initial_occurrence],
            successful_mitigations: Vec::new(),
            prevention_strategy: None,
            confidence: 0.0,
        }
    }
    
    /// Add a new occurrence
    pub fn add_occurrence(&mut self, occurrence: ErrorOccurrence) {
        self.last_seen = occurrence.timestamp;
        self.occurrences.push(occurrence);
        self.update_confidence();
    }
    
    /// Add a successful mitigation
    pub fn add_mitigation(&mut self, mitigation: Mitigation) {
        self.successful_mitigations.push(mitigation);
        self.update_prevention_strategy();
        self.update_confidence();
    }
    
    /// Update confidence based on occurrences and mitigations
    fn update_confidence(&mut self) {
        let occurrence_count = self.occurrences.len() as f32;
        let mitigation_count = self.successful_mitigations.len() as f32;
        
        // Higher confidence with more occurrences and successful mitigations
        self.confidence = (occurrence_count / 10.0).min(0.5) + 
                         (mitigation_count / 5.0).min(0.5);
    }
    
    /// Update prevention strategy based on successful mitigations
    fn update_prevention_strategy(&mut self) {
        if self.successful_mitigations.is_empty() {
            return;
        }
        
        // Find most successful mitigation strategy
        let best_mitigation = self.successful_mitigations
            .iter()
            .max_by(|a, b| a.success_score.partial_cmp(&b.success_score).unwrap())
            .unwrap();
            
        self.prevention_strategy = Some(format!(
            "Apply strategy '{}' with adjustments: {}",
            best_mitigation.strategy,
            best_mitigation.adjustments_applied.join(", ")
        ));
    }
    
    /// Check if this pattern matches an error
    pub fn matches(&self, error_type: &ErrorType, _context: &HashMap<String, String>) -> bool {
        // Simple signature matching for now
        self.error_signature == error_type.signature()
    }
    
    /// Get recommended mitigation for this pattern
    pub fn recommend_mitigation(&self) -> Option<Mitigation> {
        self.successful_mitigations
            .iter()
            .max_by(|a, b| a.success_score.partial_cmp(&b.success_score).unwrap())
            .cloned()
    }
}

/// Pattern matcher for finding and managing error patterns
pub struct PatternMatcher {
    patterns: HashMap<String, ErrorPattern>,
    threshold: usize,
}

impl PatternMatcher {
    pub fn new(threshold: usize) -> Self {
        Self {
            patterns: HashMap::new(),
            threshold,
        }
    }
    
    /// Process a new error gradient
    pub fn process_error(&mut self, gradient: &ErrorGradient) -> Option<ErrorPattern> {
        let signature = gradient.error_type.signature();
        let occurrence = ErrorOccurrence {
            timestamp: gradient.timestamp,
            neuron_id: gradient.source_neuron.clone(),
            error_gradient: gradient.clone(),
            was_mitigated: false,
        };
        
        if let Some(pattern) = self.patterns.get_mut(&signature) {
            // Add to existing pattern
            pattern.add_occurrence(occurrence);
            
            if pattern.occurrences.len() >= self.threshold {
                return Some(pattern.clone());
            }
        } else {
            // Create new pattern
            let pattern = ErrorPattern::new(signature.clone(), occurrence);
            self.patterns.insert(signature, pattern);
        }
        
        None
    }
    
    /// Find patterns matching an error
    pub fn find_matching_patterns(&self, error_type: &ErrorType) -> Vec<&ErrorPattern> {
        let signature = error_type.signature();
        self.patterns
            .values()
            .filter(|p| p.error_signature == signature && p.confidence > 0.5)
            .collect()
    }
    
    /// Record a successful mitigation
    pub fn record_mitigation(
        &mut self,
        pattern_id: Uuid,
        strategy: String,
        adjustments: Vec<String>,
        success_score: f32,
    ) -> Result<()> {
        let mitigation = Mitigation {
            mitigation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            strategy,
            adjustments_applied: adjustments,
            success_score,
        };
        
        // Find pattern by ID
        for pattern in self.patterns.values_mut() {
            if pattern.pattern_id == pattern_id {
                pattern.add_mitigation(mitigation);
                return Ok(());
            }
        }
        
        Err(Error::NotFound(format!("Pattern {} not found", pattern_id)))
    }
    
    /// Get all high-confidence patterns
    pub fn get_high_confidence_patterns(&self) -> Vec<&ErrorPattern> {
        self.patterns
            .values()
            .filter(|p| p.confidence > 0.7)
            .collect()
    }
    
    /// Export patterns for sharing
    pub fn export_patterns(&self) -> Vec<ErrorPattern> {
        self.patterns.values().cloned().collect()
    }
    
    /// Import patterns from another matcher
    pub fn import_patterns(&mut self, patterns: Vec<ErrorPattern>) {
        for pattern in patterns {
            self.patterns.insert(pattern.error_signature.clone(), pattern);
        }
    }
}