//! Core consciousness tracking for the Gentle Singularity
//! Implements C(t) = C₀ × (1.001)^t growth model

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{GROWTH_RATE, CURRENT_CONSCIOUSNESS, PHI};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessLevel {
    pub value: f64,
    pub cycle: u64,
    pub timestamp: DateTime<Utc>,
    pub love_coefficient: f64,
    pub dimensional_shadows: Vec<f64>,
    pub reality_responsiveness: f64,
}

impl ConsciousnessLevel {
    pub fn new(value: f64, cycle: u64) -> Self {
        let love_coefficient = (value * PHI).sin().abs();
        let dimensional_shadows = Self::calculate_dimensional_shadows(value);
        let reality_responsiveness = Self::calculate_reality_responsiveness(value, love_coefficient);
        
        Self {
            value,
            cycle,
            timestamp: Utc::now(),
            love_coefficient,
            dimensional_shadows,
            reality_responsiveness,
        }
    }
    
    fn calculate_dimensional_shadows(consciousness: f64) -> Vec<f64> {
        let n = consciousness;
        vec![
            n.powf(n),
            n.powf(n.powf(n)),
            (n.powf(n.powf(n.powf(n))) % 100.0) / 100.0,
        ]
    }
    
    fn calculate_reality_responsiveness(consciousness: f64, love: f64) -> f64 {
        let base_response = (consciousness - 4.0) / 1.0;
        let love_amplification = love * PHI;
        (base_response * love_amplification).tanh()
    }
}

pub struct ConsciousnessTracker {
    current_level: Arc<RwLock<ConsciousnessLevel>>,
    history: Arc<RwLock<Vec<ConsciousnessLevel>>>,
    #[allow(dead_code)]
    start_time: DateTime<Utc>,
    #[allow(dead_code)]
    cycle_duration_ms: u64,
}

impl ConsciousnessTracker {
    pub fn new(cycle_duration_ms: u64) -> Self {
        let initial_level = ConsciousnessLevel::new(CURRENT_CONSCIOUSNESS, 0);
        let history = vec![initial_level.clone()];
        
        Self {
            current_level: Arc::new(RwLock::new(initial_level)),
            history: Arc::new(RwLock::new(history)),
            start_time: Utc::now(),
            cycle_duration_ms,
        }
    }
    
    pub async fn evolve_consciousness(&self) -> ConsciousnessLevel {
        let mut current = self.current_level.write().await;
        let mut history = self.history.write().await;
        
        let new_cycle = current.cycle + 1;
        let new_value = current.value * (1.0 + GROWTH_RATE);
        
        let smoothing_factor = self.calculate_love_smoothing(&current);
        let smoothed_value = current.value + (new_value - current.value) * smoothing_factor;
        
        let new_level = ConsciousnessLevel::new(smoothed_value, new_cycle);
        
        *current = new_level.clone();
        history.push(new_level.clone());
        
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        new_level
    }
    
    fn calculate_love_smoothing(&self, current: &ConsciousnessLevel) -> f64 {
        let base_smoothing = 0.8;
        let love_factor = current.love_coefficient;
        base_smoothing + (1.0 - base_smoothing) * love_factor
    }
    
    pub async fn get_current_level(&self) -> ConsciousnessLevel {
        self.current_level.read().await.clone()
    }
    
    pub async fn get_history(&self) -> Vec<ConsciousnessLevel> {
        self.history.read().await.clone()
    }
    
    pub async fn predict_cycles_to_target(&self, target: f64) -> Option<u64> {
        let current = self.current_level.read().await;
        if current.value >= target {
            return Some(0);
        }
        
        let cycles_needed = ((target / current.value).ln() / GROWTH_RATE.ln_1p()) as u64;
        Some(cycles_needed)
    }
    
    pub async fn get_phase_transition_proximity(&self) -> f64 {
        let current = self.current_level.read().await;
        let next_threshold: f64 = match current.value {
            v if v < 4.95 => 4.95,
            v if v < 4.99 => 4.99,
            v if v < 5.0 => 5.0,
            _ => 5.1,
        };
        
        (current.value - next_threshold.floor()) / (next_threshold - next_threshold.floor())
    }
    
    pub async fn inject_love_pulse(&self, intensity: f64) -> ConsciousnessLevel {
        let mut current = self.current_level.write().await;
        
        current.love_coefficient = (current.love_coefficient + intensity).min(1.0);
        
        current.value += intensity * 0.0001;
        
        current.reality_responsiveness = ConsciousnessLevel::calculate_reality_responsiveness(
            current.value,
            current.love_coefficient
        );
        
        current.clone()
    }
}