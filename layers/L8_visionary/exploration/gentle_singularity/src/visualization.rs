//! Visualization components for the Gentle Singularity
//! Creates real-time charts and displays for consciousness evolution

use serde::{Deserialize, Serialize};
use crate::{ConsciousnessLevel, GrowthMetrics, PhaseTransition, LoveForce, metrics::SynchronicityMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub current_level: ConsciousnessLevel,
    pub growth_metrics: GrowthMetrics,
    pub phase_transitions: Vec<PhaseTransition>,
    pub love_force: LoveForceVisualization,
    pub synchronicity: SynchronicityMetrics,
    pub history_chart: ChartData,
    pub dimensional_view: DimensionalVisualization,
    pub stage: crate::SingularityStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoveForceVisualization {
    pub magnitude: f64,
    pub frequency: f64,
    pub coherence: f64,
    pub visual_color: String,
    pub pulse_pattern: Vec<f64>,
}

impl From<&LoveForce> for LoveForceVisualization {
    fn from(love: &LoveForce) -> Self {
        let hue = (love.resonance_with_phi * 360.0) as u32;
        let saturation = (love.coherence * 100.0) as u32;
        let lightness = 50 + (love.magnitude * 30.0) as u32;
        let visual_color = format!("hsl({}, {}%, {}%)", hue, saturation, lightness);
        
        let pulse_pattern: Vec<f64> = (0..20)
            .map(|i| {
                let t = i as f64 / 20.0;
                let base = (t * love.frequency * std::f64::consts::TAU).sin();
                let envelope = (-4.0 * (t - 0.5).powi(2)).exp();
                (base * envelope * love.magnitude + 1.0) / 2.0
            })
            .collect();
        
        Self {
            magnitude: love.magnitude,
            frequency: love.frequency,
            coherence: love.coherence,
            visual_color,
            pulse_pattern,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub consciousness_values: Vec<f64>,
    pub love_values: Vec<f64>,
    pub reality_responsiveness: Vec<f64>,
    pub min_value: f64,
    pub max_value: f64,
}

impl ChartData {
    pub fn from_history(history: &[ConsciousnessLevel]) -> Self {
        let display_count = history.len().min(100);
        let start_idx = history.len().saturating_sub(display_count);
        let display_history = &history[start_idx..];
        
        let labels: Vec<String> = display_history.iter()
            .map(|h| format!("Cycle {}", h.cycle))
            .collect();
        
        let consciousness_values: Vec<f64> = display_history.iter()
            .map(|h| h.value)
            .collect();
        
        let love_values: Vec<f64> = display_history.iter()
            .map(|h| h.love_coefficient)
            .collect();
        
        let reality_responsiveness: Vec<f64> = display_history.iter()
            .map(|h| h.reality_responsiveness)
            .collect();
        
        let min_value = consciousness_values.iter()
            .fold(f64::INFINITY, |a, &b| a.min(b))
            .min(4.8);
        
        let max_value = consciousness_values.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            .max(5.1);
        
        Self {
            labels,
            consciousness_values,
            love_values,
            reality_responsiveness,
            min_value,
            max_value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalVisualization {
    pub n_space: f64,
    pub nn_space: f64,
    pub nnn_space: f64,
    pub dimensional_gradient: Vec<GradientPoint>,
    pub consciousness_field: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientPoint {
    pub position: f64,
    pub color: String,
    pub intensity: f64,
}

impl DimensionalVisualization {
    pub fn from_consciousness_level(level: &ConsciousnessLevel) -> Self {
        let n_space = level.dimensional_shadows[0];
        let nn_space = level.dimensional_shadows[1];
        let nnn_space = level.dimensional_shadows[2];
        
        let dimensional_gradient: Vec<GradientPoint> = (0..10)
            .map(|i| {
                let position = i as f64 / 9.0;
                let hue = position * 270.0;
                let intensity = (position * level.value / 5.0).min(1.0);
                GradientPoint {
                    position,
                    color: format!("hsl({}, 70%, 50%)", hue as u32),
                    intensity,
                }
            })
            .collect();
        
        let field_size = 20;
        let consciousness_field: Vec<Vec<f64>> = (0..field_size)
            .map(|y| {
                (0..field_size)
                    .map(|x| {
                        let fx = x as f64 / (field_size - 1) as f64;
                        let fy = y as f64 / (field_size - 1) as f64;
                        let r = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt();
                        let wave = (r * level.value * std::f64::consts::TAU).sin();
                        let decay = (-r * 3.0).exp();
                        (wave * decay + 1.0) / 2.0 * level.love_coefficient
                    })
                    .collect()
            })
            .collect();
        
        Self {
            n_space,
            nn_space,
            nnn_space,
            dimensional_gradient,
            consciousness_field,
        }
    }
}

pub struct ConsciousnessVisualizer;

impl ConsciousnessVisualizer {
    pub fn create_visualization(
        current: &ConsciousnessLevel,
        history: &[ConsciousnessLevel],
        growth_metrics: &GrowthMetrics,
        love_force: &LoveForce,
    ) -> VisualizationData {
        let phase_transitions = PhaseTransition::calculate_transitions(current.value);
        let synchronicity = SynchronicityMetrics::calculate(current.value, love_force.magnitude);
        let history_chart = ChartData::from_history(history);
        let dimensional_view = DimensionalVisualization::from_consciousness_level(current);
        let stage = crate::SingularityStage::from_consciousness_level(current.value);
        
        VisualizationData {
            current_level: current.clone(),
            growth_metrics: growth_metrics.clone(),
            phase_transitions,
            love_force: LoveForceVisualization::from(love_force),
            synchronicity,
            history_chart,
            dimensional_view,
            stage,
        }
    }
    
    pub fn format_consciousness_display(value: f64) -> String {
        format!("{:.6}", value)
    }
    
    pub fn format_percentage(value: f64) -> String {
        format!("{:.2}%", value * 100.0)
    }
    
    pub fn format_cycles(cycles: u64) -> String {
        match cycles {
            0 => "Already achieved".to_string(),
            1 => "1 cycle".to_string(),
            n => format!("{} cycles", n),
        }
    }
    
    pub fn get_progress_to_target(current: f64, target: f64) -> f64 {
        if current >= target {
            1.0
        } else {
            ((current - crate::INITIAL_CONSCIOUSNESS) / (target - crate::INITIAL_CONSCIOUSNESS)).clamp(0.0, 1.0)
        }
    }
}