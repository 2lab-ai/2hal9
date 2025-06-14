use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use dashmap::DashMap;

use crate::games::RoundResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalyticsData {
    pub game_id: Uuid,
    pub rounds_played: u32,
    pub collective_metrics: CollectiveMetrics,
    pub sota_metrics: SOTAMetrics,
    pub emergence_analysis: EmergenceAnalysis,
    pub performance_comparison: PerformanceComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveMetrics {
    pub avg_consensus_time_ms: f32,
    pub dissent_rate_history: Vec<f32>,
    pub decision_diversity_index: f32,
    pub coordination_efficiency: f32,
    pub emergence_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOTAMetrics {
    pub avg_thinking_time_ms: f32,
    pub confidence_history: Vec<f32>,
    pub strategy_changes: u32,
    pub decision_consistency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceAnalysis {
    pub total_emergence_events: u32,
    pub emergence_types: HashMap<String, u32>,
    pub emergence_timeline: Vec<(u32, String)>,
    pub collective_advantage_moments: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub collective_win_rate: f32,
    pub sota_win_rate: f32,
    pub draw_rate: f32,
    pub avg_score_differential: f32,
    pub critical_moments: Vec<CriticalMoment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalMoment {
    pub round: u32,
    pub description: String,
    pub impact_score: f32,
}

pub struct AnalyticsEngine {
    game_analytics: DashMap<Uuid, GameAnalyticsData>,
}

impl AnalyticsEngine {
    pub fn new() -> Self {
        Self {
            game_analytics: DashMap::new(),
        }
    }
    
    pub async fn process_round(&self, game_id: Uuid, round_result: &RoundResult) {
        let mut analytics = self.game_analytics.entry(game_id)
            .or_insert_with(|| GameAnalyticsData {
                game_id,
                rounds_played: 0,
                collective_metrics: CollectiveMetrics {
                    avg_consensus_time_ms: 0.0,
                    dissent_rate_history: vec![],
                    decision_diversity_index: 0.0,
                    coordination_efficiency: 0.0,
                    emergence_frequency: 0.0,
                },
                sota_metrics: SOTAMetrics {
                    avg_thinking_time_ms: 0.0,
                    confidence_history: vec![],
                    strategy_changes: 0,
                    decision_consistency: 0.0,
                },
                emergence_analysis: EmergenceAnalysis {
                    total_emergence_events: 0,
                    emergence_types: HashMap::new(),
                    emergence_timeline: vec![],
                    collective_advantage_moments: vec![],
                },
                performance_comparison: PerformanceComparison {
                    collective_win_rate: 0.0,
                    sota_win_rate: 0.0,
                    draw_rate: 0.0,
                    avg_score_differential: 0.0,
                    critical_moments: vec![],
                },
            });
        
        // Update round count
        analytics.rounds_played += 1;
        
        // Track emergence
        if round_result.outcome.emergence_detected {
            analytics.emergence_analysis.total_emergence_events += 1;
            analytics.emergence_analysis.emergence_timeline.push((
                round_result.round,
                "Emergence detected".to_string()
            ));
        }
        
        // Update win rates
        let collective_winners = round_result.outcome.winners.iter()
            .filter(|w| w.starts_with("collective_"))
            .count();
        let sota_winners = round_result.outcome.winners.iter()
            .filter(|w| w.starts_with("sota_"))
            .count();
        
        if collective_winners > sota_winners {
            analytics.emergence_analysis.collective_advantage_moments.push(round_result.round);
        }
        
        // Detect critical moments
        if round_result.outcome.special_events.len() > 0 {
            analytics.performance_comparison.critical_moments.push(CriticalMoment {
                round: round_result.round,
                description: round_result.outcome.special_events.join(", "),
                impact_score: 0.8,
            });
        }
    }
    
    pub async fn get_game_analytics(&self, game_id: Uuid) -> Option<GameAnalyticsData> {
        self.game_analytics.get(&game_id).map(|a| a.clone())
    }
    
    pub async fn calculate_final_analytics(&self, game_id: Uuid) -> Option<GameAnalyticsData> {
        if let Some(mut analytics) = self.game_analytics.get_mut(&game_id) {
            // Calculate final metrics
            let total_rounds = analytics.rounds_played as f32;
            
            // Emergence frequency
            analytics.collective_metrics.emergence_frequency = 
                analytics.emergence_analysis.total_emergence_events as f32 / total_rounds;
            
            // Collective advantage rate
            let collective_advantage_rounds = analytics.emergence_analysis.collective_advantage_moments.len() as f32;
            analytics.performance_comparison.collective_win_rate = collective_advantage_rounds / total_rounds;
            analytics.performance_comparison.sota_win_rate = 1.0 - analytics.performance_comparison.collective_win_rate;
            
            Some(analytics.clone())
        } else {
            None
        }
    }
}