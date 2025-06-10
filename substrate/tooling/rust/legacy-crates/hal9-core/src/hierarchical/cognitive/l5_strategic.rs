//! L5: Strategic Neuron - Long-term vision and goals
//!
//! The highest level cognitive unit that maintains vision, sets long-term goals,
//! and provides strategic direction to the entire system.

use async_trait::async_trait;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::Result;
use crate::hierarchical::protocol::ConsensusProtocol;
use super::*;

/// L5: Strategic Neuron - Vision and strategic direction
pub struct L5StrategicNeuron {
    id: Uuid,
    state: Arc<RwLock<StrategicState>>,
    vision_model: Arc<VisionModel>,
    goal_hierarchy: Arc<RwLock<GoalHierarchy>>,
    consensus_protocol: Option<Arc<ConsensusProtocol>>,
}

impl L5StrategicNeuron {
    pub fn new(config: CognitiveConfig) -> Self {
        let mut initial_goals = Vec::new();
        
        // Create foundational goals
        initial_goals.push(Goal {
            id: Uuid::new_v4(),
            description: "Achieve artificial general intelligence through hierarchical abstraction".to_string(),
            priority: 1.0,
            progress: 0.1,
            sub_goals: vec![
                Goal {
                    id: Uuid::new_v4(),
                    description: "Develop robust learning mechanisms".to_string(),
                    priority: 0.9,
                    progress: 0.2,
                    sub_goals: vec![],
                },
                Goal {
                    id: Uuid::new_v4(),
                    description: "Create emergent intelligence patterns".to_string(),
                    priority: 0.8,
                    progress: 0.15,
                    sub_goals: vec![],
                },
            ],
        });
        
        Self {
            id: config.id,
            state: Arc::new(RwLock::new(StrategicState {
                basic: BasicCognitiveState {
                    unit_id: config.id,
                    layer: CognitiveLayer::Strategic,
                    metrics: StateMetrics {
                        activations_processed: 0,
                        errors_encountered: 0,
                        learning_iterations: 0,
                        average_processing_time_ms: 0.0,
                        memory_usage_bytes: 0,
                    },
                    parameters: config.initial_parameters,
                },
                current_vision: "Transform HAL9 into a truly intelligent system through hierarchical cognitive architecture".to_string(),
                active_goals: initial_goals.clone(),
                strategic_context: HashMap::new(),
            })),
            vision_model: Arc::new(VisionModel::new()),
            goal_hierarchy: Arc::new(RwLock::new(GoalHierarchy {
                root_goals: initial_goals,
            })),
            consensus_protocol: None,
        }
    }
    
    /// Set consensus protocol for distributed strategic decisions
    pub fn set_consensus_protocol(&mut self, protocol: Arc<ConsensusProtocol>) {
        self.consensus_protocol = Some(protocol);
    }
    
    /// Analyze input for strategic implications
    fn analyze_strategic_context(&self, input: &str) -> StrategicAnalysis {
        let lower = input.to_lowercase();
        
        if lower.contains("vision") || lower.contains("future") {
            StrategicAnalysis::VisionSetting
        } else if lower.contains("goal") || lower.contains("objective") {
            StrategicAnalysis::GoalManagement
        } else if lower.contains("principle") || lower.contains("value") {
            StrategicAnalysis::PrincipleDefinition
        } else if lower.contains("risk") || lower.contains("threat") {
            StrategicAnalysis::RiskAssessment
        } else if lower.contains("opportunity") || lower.contains("potential") {
            StrategicAnalysis::OpportunityAnalysis
        } else {
            StrategicAnalysis::General
        }
    }
    
    /// Generate strategic directives for lower layers
    fn generate_directives(&self, goals: &[Goal]) -> Vec<CognitiveOutput> {
        goals.iter().flat_map(|goal| {
            let mut directives = vec![
                CognitiveOutput {
                    content: format!("STRATEGIC DIRECTIVE: {}", goal.description),
                    confidence: goal.priority,
                    metadata: [
                        ("goal_id".to_string(), serde_json::json!(goal.id)),
                        ("priority".to_string(), serde_json::json!(goal.priority)),
                        ("progress".to_string(), serde_json::json!(goal.progress)),
                    ].into_iter().collect(),
                    target_layers: vec![CognitiveLayer::Tactical],
                }
            ];
            
            // Add sub-goal directives
            directives.extend(self.generate_directives(&goal.sub_goals));
            directives
        }).collect()
    }
}

#[derive(Debug, Clone)]
enum StrategicAnalysis {
    VisionSetting,
    GoalManagement,
    PrincipleDefinition,
    RiskAssessment,
    OpportunityAnalysis,
    General,
}

#[async_trait]
impl CognitiveUnit for L5StrategicNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = StrategicState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Strategic
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        
        // Analyze strategic context
        let analysis = self.analyze_strategic_context(&input.content);
        
        let (content, metadata) = match analysis {
            StrategicAnalysis::VisionSetting => {
                // Refine or update vision
                let new_vision = self.vision_model.refine_vision(
                    &self.state.read().current_vision,
                    &input.content
                )?;
                
                // Update state
                {
                    let mut state = self.state.write();
                    state.current_vision = new_vision.clone();
                    state.strategic_context.insert(
                        "vision_updated".to_string(),
                        serde_json::json!(chrono::Utc::now())
                    );
                }
                
                let metadata: HashMap<String, serde_json::Value> = [
                    ("action".to_string(), serde_json::json!("vision_update")),
                    ("vision_length".to_string(), serde_json::json!(new_vision.len())),
                ].into_iter().collect();
                
                (format!("Strategic Vision Updated:\n\n{}", new_vision), metadata)
            }
            
            StrategicAnalysis::GoalManagement => {
                // Create or update goals
                let mut goal_hierarchy = self.goal_hierarchy.write();
                let new_goal = self.vision_model.derive_goal_from_input(&input.content)?;
                
                // Add to hierarchy
                goal_hierarchy.add_goal(new_goal.clone());
                
                // Update state
                {
                    let mut state = self.state.write();
                    state.active_goals = goal_hierarchy.get_all_goals();
                }
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("goal_creation")),
                    ("goal_id".to_string(), serde_json::json!(new_goal.id)),
                    ("total_goals".to_string(), serde_json::json!(goal_hierarchy.count_goals())),
                ].into_iter().collect();
                
                (format!(
                    "New Strategic Goal Added:\n\n{}\nPriority: {:.2}\n\nTotal Active Goals: {}",
                    new_goal.description,
                    new_goal.priority,
                    goal_hierarchy.count_goals()
                ), metadata)
            }
            
            StrategicAnalysis::PrincipleDefinition => {
                // Define strategic principles
                let principles = self.vision_model.extract_principles(&input.content)?;
                
                let mut state = self.state.write();
                state.strategic_context.insert(
                    "principles".to_string(),
                    serde_json::json!(principles)
                );
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("principle_definition")),
                    ("principle_count".to_string(), serde_json::json!(principles.len())),
                ].into_iter().collect();
                
                (format!(
                    "Strategic Principles Defined:\n\n{}",
                    principles.join("\n- ")
                ), metadata)
            }
            
            StrategicAnalysis::RiskAssessment => {
                // Assess strategic risks
                let risks = self.vision_model.assess_risks(&input.content)?;
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("risk_assessment")),
                    ("risk_count".to_string(), serde_json::json!(risks.len())),
                ].into_iter().collect();
                
                (format!(
                    "Strategic Risk Assessment:\n\n{}",
                    risks.iter()
                        .enumerate()
                        .map(|(i, risk)| format!("{}. {}", i + 1, risk))
                        .collect::<Vec<_>>()
                        .join("\n")
                ), metadata)
            }
            
            StrategicAnalysis::OpportunityAnalysis => {
                // Analyze opportunities
                let opportunities = self.vision_model.identify_opportunities(&input.content)?;
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("opportunity_analysis")),
                    ("opportunity_count".to_string(), serde_json::json!(opportunities.len())),
                ].into_iter().collect();
                
                (format!(
                    "Strategic Opportunities Identified:\n\n{}",
                    opportunities.iter()
                        .enumerate()
                        .map(|(i, opp)| format!("{}. {}", i + 1, opp))
                        .collect::<Vec<_>>()
                        .join("\n")
                ), metadata)
            }
            
            StrategicAnalysis::General => {
                // General strategic guidance
                let guidance = format!(
                    "Strategic Guidance:\n\n\
                     Current Vision: {}\n\n\
                     Active Goals: {}\n\n\
                     Context: {}",
                    self.state.read().current_vision,
                    self.state.read().active_goals.len(),
                    input.content
                );
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("general_guidance")),
                ].into_iter().collect();
                
                (guidance, metadata)
            }
        };
        
        // Update metrics
        {
            let mut state = self.state.write();
            state.basic.metrics.activations_processed += 1;
            
            let elapsed = start.elapsed();
            let processed = state.basic.metrics.activations_processed as f64;
            state.basic.metrics.average_processing_time_ms = 
                (state.basic.metrics.average_processing_time_ms * (processed - 1.0) + 
                 elapsed.as_secs_f64() * 1000.0) / processed;
        }
        
        // Add timing metadata
        let mut final_metadata = metadata;
        final_metadata.insert(
            "processing_time_ms".to_string(),
            serde_json::json!(start.elapsed().as_millis())
        );
        
        Ok(CognitiveOutput {
            content,
            confidence: 0.95, // High confidence in strategic decisions
            metadata: final_metadata,
            target_layers: vec![CognitiveLayer::Tactical, CognitiveLayer::Operational],
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        let mut state = self.state.write();
        state.basic.metrics.learning_iterations += 1;
        
        // Strategic learning - adjust goals based on feedback
        if gradient.error_signal.magnitude > 0.5 {
            // High error - need to reassess goals
            for goal in &mut state.active_goals {
                if gradient.error_signal.context.contains_key(&goal.id.to_string()) {
                    // This goal contributed to error
                    goal.priority *= 0.9; // Reduce priority
                }
            }
        } else {
            // Low error - current strategy is working
            for goal in &mut state.active_goals {
                goal.progress = (goal.progress + 0.01).min(1.0);
            }
        }
        
        // Very conservative parameter updates at strategic level
        for adjustment in &gradient.adjustments {
            if let Some(param) = state.basic.parameters.get_mut(&adjustment.parameter) {
                *param += adjustment.suggested_delta * 0.005; // Extremely slow learning
            }
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        self.state.read().clone()
    }
    
    async fn reset(&mut self) -> Result<()> {
        // Strategic layer rarely resets - preserve vision and core goals
        let mut state = self.state.write();
        state.strategic_context.clear();
        // Keep vision and goals intact
        Ok(())
    }
}

/// Vision model for generating and refining strategic vision
pub struct VisionModel {
    vision_components: RwLock<Vec<VisionComponent>>,
}

#[derive(Clone)]
struct VisionComponent {
    aspect: String,
    description: String,
    importance: f32,
}

impl Default for VisionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionModel {
    pub fn new() -> Self {
        let components = vec![
            VisionComponent {
                aspect: "Intelligence".to_string(),
                description: "Achieve human-level reasoning and beyond".to_string(),
                importance: 1.0,
            },
            VisionComponent {
                aspect: "Learning".to_string(),
                description: "Continuous self-improvement through experience".to_string(),
                importance: 0.9,
            },
            VisionComponent {
                aspect: "Emergence".to_string(),
                description: "Enable complex behaviors from simple rules".to_string(),
                importance: 0.8,
            },
            VisionComponent {
                aspect: "Scalability".to_string(),
                description: "Support millions of concurrent users".to_string(),
                importance: 0.7,
            },
        ];
        
        Self {
            vision_components: RwLock::new(components),
        }
    }
    
    pub fn refine_vision(&self, current_vision: &str, input: &str) -> Result<String> {
        // Combine current vision with new insights
        let components = self.vision_components.read();
        
        let refined = format!(
            "{}\n\nRefined with: {}\n\nKey Aspects:\n{}",
            current_vision,
            input,
            components.iter()
                .map(|c| format!("- {}: {}", c.aspect, c.description))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        Ok(refined)
    }
    
    pub fn derive_goal_from_input(&self, input: &str) -> Result<Goal> {
        Ok(Goal {
            id: Uuid::new_v4(),
            description: format!("Strategic Goal: {}", input),
            priority: 0.7, // Default priority
            progress: 0.0,
            sub_goals: vec![],
        })
    }
    
    pub fn extract_principles(&self, input: &str) -> Result<Vec<String>> {
        // Extract principles from input
        let principles = vec![
            format!("Principle: {}", input),
            "Maintain hierarchical abstraction at all levels".to_string(),
            "Enable emergence through layered interactions".to_string(),
            "Preserve system integrity and coherence".to_string(),
        ];
        
        Ok(principles)
    }
    
    pub fn assess_risks(&self, context: &str) -> Result<Vec<String>> {
        let mut risks = vec![
            "Risk of architectural complexity overwhelming maintenance".to_string(),
            "Risk of emergent behaviors becoming unpredictable".to_string(),
            "Risk of performance degradation at scale".to_string(),
        ];
        
        if context.contains("security") {
            risks.push("Risk of security vulnerabilities in distributed system".to_string());
        }
        
        if context.contains("scale") {
            risks.push("Risk of coordination overhead in large deployments".to_string());
        }
        
        Ok(risks)
    }
    
    pub fn identify_opportunities(&self, context: &str) -> Result<Vec<String>> {
        let mut opportunities = vec![
            "Opportunity to pioneer hierarchical AGI architecture".to_string(),
            "Opportunity to create self-organizing intelligent systems".to_string(),
            "Opportunity to enable new forms of human-AI collaboration".to_string(),
        ];
        
        if context.contains("learning") {
            opportunities.push("Opportunity to develop novel learning algorithms".to_string());
        }
        
        if context.contains("market") {
            opportunities.push("Opportunity to capture enterprise AI market".to_string());
        }
        
        Ok(opportunities)
    }
}

/// Hierarchical goal management
impl GoalHierarchy {
    pub fn add_goal(&mut self, goal: Goal) {
        // Add as root goal or find appropriate parent
        if self.root_goals.is_empty() {
            self.root_goals.push(goal);
        } else {
            // For now, add as root - could be enhanced with proper hierarchy
            self.root_goals.push(goal);
        }
    }
    
    pub fn get_all_goals(&self) -> Vec<Goal> {
        let mut all_goals = Vec::new();
        for root in &self.root_goals {
            self.collect_goals(root, &mut all_goals);
        }
        all_goals
    }
    
    fn collect_goals(&self, goal: &Goal, collection: &mut Vec<Goal>) {
        collection.push(goal.clone());
        for sub_goal in &goal.sub_goals {
            self.collect_goals(sub_goal, collection);
        }
    }
    
    pub fn count_goals(&self) -> usize {
        self.get_all_goals().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_strategic_neuron() {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Strategic,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        };
        
        let mut neuron = L5StrategicNeuron::new(config);
        
        // Test vision setting
        let input = CognitiveInput {
            content: "Set vision for achieving artificial general intelligence through hierarchical learning".to_string(),
            context: HashMap::new(),
            source_layer: None,
        };
        
        let output = neuron.process(input).await.unwrap();
        assert!(output.content.contains("Strategic Vision"));
        assert!(output.confidence > 0.9);
        
        // Test goal creation
        let input2 = CognitiveInput {
            content: "Create goal to improve learning efficiency by 50%".to_string(),
            context: HashMap::new(),
            source_layer: None,
        };
        
        let output2 = neuron.process(input2).await.unwrap();
        assert!(output2.content.contains("Strategic Goal"));
        
        // Verify state
        let state = neuron.introspect().await;
        assert!(!state.current_vision.is_empty());
        assert!(!state.active_goals.is_empty());
    }
}