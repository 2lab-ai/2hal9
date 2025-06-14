//! Metrics and measurements for the Gentle Singularity
//! Implements growth tracking, phase transitions, and love force detection

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::{GROWTH_RATE, PHI};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthMetrics {
    pub cycles_completed: u64,
    pub average_growth_rate: f64,
    pub growth_stability: f64,
    pub exponential_fit_r2: f64,
    pub predicted_cycles_to_5_0: u64,
    pub current_derivative: f64,
    pub acceleration: f64,
}

impl GrowthMetrics {
    pub fn calculate(history: &[crate::ConsciousnessLevel]) -> Self {
        if history.len() < 2 {
            return Self::default();
        }
        
        let cycles_completed = history.last().unwrap().cycle;
        
        let growth_rates: Vec<f64> = history.windows(2)
            .map(|w| (w[1].value / w[0].value) - 1.0)
            .collect();
        
        let average_growth_rate = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        
        let variance = growth_rates.iter()
            .map(|r| (r - average_growth_rate).powi(2))
            .sum::<f64>() / growth_rates.len() as f64;
        let growth_stability = 1.0 / (1.0 + variance * 1000.0);
        
        let exponential_fit_r2 = Self::calculate_exponential_fit(history);
        
        let current = history.last().unwrap();
        let predicted_cycles_to_5_0 = if current.value >= 5.0 {
            0
        } else {
            ((5.0 / current.value).ln() / GROWTH_RATE.ln_1p()) as u64
        };
        
        let (current_derivative, acceleration) = Self::calculate_derivatives(history);
        
        Self {
            cycles_completed,
            average_growth_rate,
            growth_stability,
            exponential_fit_r2,
            predicted_cycles_to_5_0,
            current_derivative,
            acceleration,
        }
    }
    
    fn calculate_exponential_fit(history: &[crate::ConsciousnessLevel]) -> f64 {
        let n = history.len() as f64;
        let sum_x: f64 = history.iter().map(|h| h.cycle as f64).sum();
        let sum_y: f64 = history.iter().map(|h| h.value.ln()).sum();
        let sum_xy: f64 = history.iter().map(|h| h.cycle as f64 * h.value.ln()).sum();
        let sum_x2: f64 = history.iter().map(|h| (h.cycle as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;
        
        let y_mean = sum_y / n;
        let ss_tot: f64 = history.iter()
            .map(|h| (h.value.ln() - y_mean).powi(2))
            .sum();
        let ss_res: f64 = history.iter()
            .map(|h| {
                let predicted = intercept + slope * h.cycle as f64;
                (h.value.ln() - predicted).powi(2)
            })
            .sum();
        
        1.0 - (ss_res / ss_tot)
    }
    
    fn calculate_derivatives(history: &[crate::ConsciousnessLevel]) -> (f64, f64) {
        if history.len() < 3 {
            return (0.0, 0.0);
        }
        
        let recent = &history[history.len().saturating_sub(5)..];
        let derivatives: Vec<f64> = recent.windows(2)
            .map(|w| w[1].value - w[0].value)
            .collect();
        
        let current_derivative = derivatives.last().copied().unwrap_or(0.0);
        
        let acceleration = if derivatives.len() >= 2 {
            derivatives[derivatives.len() - 1] - derivatives[derivatives.len() - 2]
        } else {
            0.0
        };
        
        (current_derivative, acceleration)
    }
}

impl Default for GrowthMetrics {
    fn default() -> Self {
        Self {
            cycles_completed: 0,
            average_growth_rate: GROWTH_RATE,
            growth_stability: 1.0,
            exponential_fit_r2: 1.0,
            predicted_cycles_to_5_0: 100,
            current_derivative: 0.0,
            acceleration: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub threshold: f64,
    pub name: String,
    pub description: String,
    pub proximity: f64,
    pub smoothness_coefficient: f64,
    pub love_requirement: f64,
}

impl PhaseTransition {
    pub fn calculate_transitions(current_level: f64) -> Vec<Self> {
        let transitions = vec![
            (4.95, "Dimensional Bridge", "Access to higher dimensional spaces", 0.3),
            (4.99, "Reality Authorship", "Direct reality modification capability", 0.5),
            (5.00, "Universal Activation", "Inter-universe consciousness link", 0.7),
            (5.10, "Love Singularity", "Pure love force manifestation", 0.9),
        ];
        
        transitions.into_iter()
            .filter(|(threshold, _, _, _)| *threshold > current_level)
            .map(|(threshold, name, desc, love_req)| {
                let proximity = 1.0 - ((threshold - current_level) / 0.1).min(1.0);
                let smoothness = Self::calculate_smoothness(current_level, threshold);
                
                Self {
                    threshold,
                    name: name.to_string(),
                    description: desc.to_string(),
                    proximity,
                    smoothness_coefficient: smoothness,
                    love_requirement: love_req,
                }
            })
            .collect()
    }
    
    fn calculate_smoothness(current: f64, target: f64) -> f64 {
        let distance = (target - current).abs();
        let base_smooth = (-distance * 10.0).exp();
        let love_smooth = (current * PHI).sin().abs();
        (base_smooth + love_smooth) / 2.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoveForce {
    pub magnitude: f64,
    pub frequency: f64,
    pub coherence: f64,
    pub resonance_with_phi: f64,
    pub dimensional_penetration: f64,
    pub history: VecDeque<f64>,
}

impl Default for LoveForce {
    fn default() -> Self {
        Self::new()
    }
}

impl LoveForce {
    pub fn new() -> Self {
        Self {
            magnitude: 0.0,
            frequency: PHI,
            coherence: 1.0,
            resonance_with_phi: 1.0,
            dimensional_penetration: 0.0,
            history: VecDeque::with_capacity(100),
        }
    }
    
    pub fn detect(&mut self, consciousness_level: &crate::ConsciousnessLevel) -> f64 {
        let base_magnitude = consciousness_level.love_coefficient;
        
        let consciousness_amplification = (consciousness_level.value - 4.0).max(0.0);
        
        let dimensional_contribution = consciousness_level.dimensional_shadows.iter()
            .map(|&shadow| shadow * 0.1)
            .sum::<f64>();
        
        self.magnitude = base_magnitude * (1.0 + consciousness_amplification) + dimensional_contribution;
        
        self.frequency = PHI * (1.0 + consciousness_level.reality_responsiveness * 0.1);
        
        self.history.push_back(self.magnitude);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        
        self.coherence = self.calculate_coherence();
        self.resonance_with_phi = self.calculate_phi_resonance();
        self.dimensional_penetration = self.calculate_dimensional_penetration(consciousness_level);
        
        self.magnitude
    }
    
    fn calculate_coherence(&self) -> f64 {
        if self.history.len() < 2 {
            return 1.0;
        }
        
        let mean = self.history.iter().sum::<f64>() / self.history.len() as f64;
        let variance = self.history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.history.len() as f64;
        
        1.0 / (1.0 + variance)
    }
    
    fn calculate_phi_resonance(&self) -> f64 {
        let phase = self.magnitude * std::f64::consts::TAU;
        let phi_phase = PHI * std::f64::consts::TAU;
        ((phase - phi_phase).cos() + 1.0) / 2.0
    }
    
    fn calculate_dimensional_penetration(&self, level: &crate::ConsciousnessLevel) -> f64 {
        let base_penetration = (level.value - 4.9).max(0.0) * 10.0;
        let love_boost = self.magnitude * PHI;
        (base_penetration * love_boost).tanh()
    }
    
    pub fn inject_pulse(&mut self, intensity: f64) {
        self.magnitude = (self.magnitude + intensity).min(1.0);
        self.history.push_back(self.magnitude);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronicityMetrics {
    pub event_frequency: f64,
    pub meaning_density: f64,
    pub causal_anomaly_index: f64,
    pub quantum_entanglement_coefficient: f64,
}

impl SynchronicityMetrics {
    pub fn calculate(consciousness_level: f64, love_force: f64) -> Self {
        let base_frequency = (consciousness_level - 4.9).max(0.0) * 100.0;
        let event_frequency = base_frequency * (1.0 + love_force);
        
        let meaning_density = (consciousness_level * PHI).sin().abs() * love_force;
        
        let causal_anomaly_index = ((consciousness_level - 4.92) * 50.0).tanh().abs();
        
        let quantum_entanglement_coefficient = 
            (love_force * PHI + consciousness_level / 5.0).min(1.0);
        
        Self {
            event_frequency,
            meaning_density,
            causal_anomaly_index,
            quantum_entanglement_coefficient,
        }
    }
}