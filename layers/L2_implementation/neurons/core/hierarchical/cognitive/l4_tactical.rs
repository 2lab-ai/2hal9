//! L4: Tactical Neuron - Planning and strategy execution
//!
//! This neuron handles medium-term planning, strategy development,
//! and coordination of operational activities to achieve goals.

use async_trait::async_trait;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::Result;
use super::*;

/// L4: Tactical Neuron - Strategic planning and execution
pub struct L4TacticalNeuron {
    id: Uuid,
    state: Arc<RwLock<TacticalState>>,
    planner: Arc<TaskPlanner>,
    strategy_executor: Arc<StrategyExecutor>,
}

impl L4TacticalNeuron {
    pub fn new(config: CognitiveConfig) -> Self {
        Self {
            id: config.id,
            state: Arc::new(RwLock::new(TacticalState {
                basic: BasicCognitiveState {
                    unit_id: config.id,
                    layer: CognitiveLayer::Tactical,
                    metrics: StateMetrics {
                        activations_processed: 0,
                        errors_encountered: 0,
                        learning_iterations: 0,
                        average_processing_time_ms: 0.0,
                        memory_usage_bytes: 0,
                    },
                    parameters: config.initial_parameters,
                },
                current_plan: None,
                active_strategies: Vec::new(),
            })),
            planner: Arc::new(TaskPlanner::new()),
            strategy_executor: Arc::new(StrategyExecutor::new()),
        }
    }
    
    /// Analyze input to determine tactical approach
    fn analyze_objective(&self, input: &str) -> TacticalAnalysis {
        let lower = input.to_lowercase();
        
        if lower.contains("plan") || lower.contains("roadmap") {
            TacticalAnalysis::Planning
        } else if lower.contains("strategy") || lower.contains("approach") {
            TacticalAnalysis::StrategyDevelopment
        } else if lower.contains("execute") || lower.contains("implement") {
            TacticalAnalysis::Execution
        } else if lower.contains("adapt") || lower.contains("adjust") {
            TacticalAnalysis::Adaptation
        } else if lower.contains("evaluate") || lower.contains("assess") {
            TacticalAnalysis::Evaluation
        } else {
            TacticalAnalysis::General
        }
    }
    
    /// Decompose plan into operational tasks
    #[allow(dead_code)]
    fn decompose_plan(&self, plan: &Plan) -> Vec<CognitiveOutput> {
        plan.steps.iter().map(|step| {
            CognitiveOutput {
                content: step.description.clone(),
                confidence: 0.85,
                metadata: [
                    ("plan_id".to_string(), serde_json::json!(plan.id)),
                    ("step_type".to_string(), serde_json::json!("plan_step")),
                ].into_iter().collect(),
                target_layers: vec![
                    step.assigned_to.unwrap_or(CognitiveLayer::Operational)
                ],
            }
        }).collect()
    }
}

#[derive(Debug, Clone)]
enum TacticalAnalysis {
    Planning,
    StrategyDevelopment,
    Execution,
    Adaptation,
    Evaluation,
    General,
}

#[async_trait]
impl CognitiveUnit for L4TacticalNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = TacticalState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Tactical
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        
        // Analyze the objective
        let analysis = self.analyze_objective(&input.content);
        
        let (content, confidence, metadata) = match analysis {
            TacticalAnalysis::Planning => {
                // Create a tactical plan
                let plan = self.planner.create_plan(&input.content)?;
                
                // Store the plan
                {
                    let mut state = self.state.write();
                    state.current_plan = Some(plan.clone());
                }
                
                // Format plan
                let content = format!(
                    "Tactical Plan: {}\n\nSteps:\n{}",
                    plan.objective,
                    plan.steps.iter()
                        .enumerate()
                        .map(|(i, step)| format!("{}. {} [{}]", 
                                               i + 1, 
                                               step.description,
                                               if step.completed { "✓" } else { "○" }))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                
                let metadata: HashMap<String, serde_json::Value> = [
                    ("plan_id".to_string(), serde_json::json!(plan.id)),
                    ("total_steps".to_string(), serde_json::json!(plan.steps.len())),
                    ("estimated_duration".to_string(), serde_json::json!("2-4 weeks")),
                ].into_iter().collect();
                
                (content, 0.8, metadata)
            }
            
            TacticalAnalysis::StrategyDevelopment => {
                // Develop strategy
                let strategy = self.strategy_executor.develop_strategy(&input.content)?;
                
                // Store strategy
                {
                    let mut state = self.state.write();
                    state.active_strategies.push(strategy.clone());
                }
                
                let content = format!(
                    "Strategy: {}\n\nTactics:\n{}",
                    strategy.name,
                    strategy.tactics.iter()
                        .enumerate()
                        .map(|(i, tactic)| format!("{}. {}", i + 1, tactic))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                
                let metadata = [
                    ("strategy_name".to_string(), serde_json::json!(strategy.name)),
                    ("tactics_count".to_string(), serde_json::json!(strategy.tactics.len())),
                ].into_iter().collect();
                
                (content, 0.85, metadata)
            }
            
            TacticalAnalysis::Execution => {
                // Execute current plan/strategy
                let execution_report = if let Some(plan) = &self.state.read().current_plan {
                    self.planner.execute_next_step(plan)?
                } else {
                    "No active plan to execute. Please create a plan first.".to_string()
                };
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("execution")),
                ].into_iter().collect();
                
                (execution_report, 0.75, metadata)
            }
            
            TacticalAnalysis::Adaptation => {
                // Adapt current strategies
                let mut state = self.state.write();
                let adaptation_report = if !state.active_strategies.is_empty() {
                    // Adapt strategies based on feedback
                    for strategy in &mut state.active_strategies {
                        self.strategy_executor.adapt_strategy(strategy, &input.content)?;
                    }
                    format!("Adapted {} strategies based on: {}", 
                           state.active_strategies.len(), 
                           input.content)
                } else {
                    "No active strategies to adapt.".to_string()
                };
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("adaptation")),
                ].into_iter().collect();
                
                (adaptation_report, 0.8, metadata)
            }
            
            TacticalAnalysis::Evaluation => {
                // Evaluate progress
                let evaluation = self.planner.evaluate_progress(&self.state.read())?;
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("evaluation")),
                ].into_iter().collect();
                
                (evaluation, 0.9, metadata)
            }
            
            TacticalAnalysis::General => {
                // General tactical response
                let response = format!(
                    "Tactical analysis: {}. Recommend strategic planning approach.",
                    input.content
                );
                
                let metadata = [
                    ("action".to_string(), serde_json::json!("analysis")),
                ].into_iter().collect();
                
                (response, 0.7, metadata)
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
        
        // Add common metadata
        let mut final_metadata = metadata;
        final_metadata.insert("processing_time_ms".to_string(), 
                            serde_json::json!(start.elapsed().as_millis()));
        final_metadata.insert("analysis_type".to_string(), 
                            serde_json::json!(format!("{:?}", analysis)));
        
        Ok(CognitiveOutput {
            content,
            confidence,
            metadata: final_metadata,
            target_layers: vec![CognitiveLayer::Operational],
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        let mut state = self.state.write();
        state.basic.metrics.learning_iterations += 1;
        
        // Learn from plan execution feedback
        if let Some(ref mut plan) = state.current_plan {
            if gradient.error_signal.magnitude < 0.2 {
                // Good progress - increase plan confidence
                plan.progress = (plan.progress + 0.1).min(1.0);
            } else {
                // Poor progress - may need to revise plan
                plan.progress = (plan.progress - 0.05).max(0.0);
            }
        }
        
        // Adjust strategic parameters
        for adjustment in &gradient.adjustments {
            if let Some(param) = state.basic.parameters.get_mut(&adjustment.parameter) {
                *param += adjustment.suggested_delta * 0.01; // Very conservative learning
            }
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        self.state.read().clone()
    }
    
    async fn reset(&mut self) -> Result<()> {
        let mut state = self.state.write();
        state.current_plan = None;
        state.active_strategies.clear();
        Ok(())
    }
}

/// Task planner for creating and managing plans
pub struct TaskPlanner {
    planning_templates: RwLock<Vec<PlanTemplate>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct PlanTemplate {
    name: String,
    typical_steps: Vec<String>,
    applicable_to: Vec<String>,
}

impl Default for TaskPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskPlanner {
    pub fn new() -> Self {
        let templates = vec![
            PlanTemplate {
                name: "Software Development".to_string(),
                typical_steps: vec![
                    "Requirements Analysis".to_string(),
                    "System Design".to_string(),
                    "Implementation".to_string(),
                    "Testing".to_string(),
                    "Deployment".to_string(),
                    "Monitoring".to_string(),
                ],
                applicable_to: vec!["software".to_string(), "application".to_string(), "system".to_string()],
            },
            PlanTemplate {
                name: "Problem Solving".to_string(),
                typical_steps: vec![
                    "Problem Definition".to_string(),
                    "Root Cause Analysis".to_string(),
                    "Solution Generation".to_string(),
                    "Solution Evaluation".to_string(),
                    "Implementation".to_string(),
                    "Verification".to_string(),
                ],
                applicable_to: vec!["problem".to_string(), "issue".to_string(), "bug".to_string()],
            },
            PlanTemplate {
                name: "Learning".to_string(),
                typical_steps: vec![
                    "Assess Current Knowledge".to_string(),
                    "Identify Learning Goals".to_string(),
                    "Gather Resources".to_string(),
                    "Study and Practice".to_string(),
                    "Apply Knowledge".to_string(),
                    "Evaluate Progress".to_string(),
                ],
                applicable_to: vec!["learn".to_string(), "study".to_string(), "understand".to_string()],
            },
        ];
        
        Self {
            planning_templates: RwLock::new(templates),
        }
    }
    
    pub fn create_plan(&self, objective: &str) -> Result<Plan> {
        // Select appropriate template
        let templates = self.planning_templates.read();
        let template = templates.iter()
            .find(|t| t.applicable_to.iter().any(|keyword| objective.to_lowercase().contains(keyword)))
            .cloned()
            .unwrap_or_else(|| PlanTemplate {
                name: "Generic".to_string(),
                typical_steps: vec![
                    "Analyze".to_string(),
                    "Design".to_string(),
                    "Execute".to_string(),
                    "Verify".to_string(),
                ],
                applicable_to: vec![],
            });
        
        // Create plan from template
        let steps = template.typical_steps.iter()
            .map(|step_desc| PlanStep {
                description: format!("{} for: {}", step_desc, objective),
                assigned_to: Some(match step_desc.to_lowercase().as_str() {
                    s if s.contains("implement") || s.contains("execute") => CognitiveLayer::Implementation,
                    s if s.contains("design") || s.contains("analyz") => CognitiveLayer::Operational,
                    _ => CognitiveLayer::Operational,
                }),
                completed: false,
            })
            .collect();
        
        Ok(Plan {
            id: Uuid::new_v4(),
            objective: objective.to_string(),
            steps,
            progress: 0.0,
        })
    }
    
    pub fn execute_next_step(&self, plan: &Plan) -> Result<String> {
        // Find next uncompleted step
        let next_step = plan.steps.iter()
            .find(|step| !step.completed);
        
        match next_step {
            Some(step) => Ok(format!(
                "Executing: {}\nAssigned to: {:?}\nPlease coordinate with the {} layer.",
                step.description,
                step.assigned_to,
                step.assigned_to.as_ref().map(|l| l.name()).unwrap_or("appropriate")
            )),
            None => Ok("All steps in the plan have been completed!".to_string()),
        }
    }
    
    pub fn evaluate_progress(&self, state: &TacticalState) -> Result<String> {
        let mut report = String::from("Progress Evaluation:\n\n");
        
        if let Some(plan) = &state.current_plan {
            let completed = plan.steps.iter().filter(|s| s.completed).count();
            let total = plan.steps.len();
            
            report.push_str(&format!(
                "Current Plan: {}\n\
                 Progress: {}/{} steps ({:.0}%)\n\
                 Confidence: {:.0}%\n",
                plan.objective,
                completed,
                total,
                (completed as f32 / total as f32) * 100.0,
                plan.progress * 100.0
            ));
        } else {
            report.push_str("No active plan.\n");
        }
        
        report.push_str(&format!(
            "\nActive Strategies: {}\n\
             Total Activations: {}\n\
             Learning Iterations: {}",
            state.active_strategies.len(),
            state.basic.metrics.activations_processed,
            state.basic.metrics.learning_iterations
        ));
        
        Ok(report)
    }
}

/// Strategy executor for tactical operations
pub struct StrategyExecutor {
    strategy_library: RwLock<Vec<StrategyTemplate>>,
}

#[derive(Clone)]
struct StrategyTemplate {
    name: String,
    tactics: Vec<String>,
}

impl Default for StrategyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyExecutor {
    pub fn new() -> Self {
        let strategies = vec![
            StrategyTemplate {
                name: "Incremental Development".to_string(),
                tactics: vec![
                    "Start with minimal viable product".to_string(),
                    "Add features iteratively".to_string(),
                    "Gather feedback at each iteration".to_string(),
                    "Refactor and optimize regularly".to_string(),
                ],
            },
            StrategyTemplate {
                name: "Divide and Conquer".to_string(),
                tactics: vec![
                    "Break problem into smaller subproblems".to_string(),
                    "Solve subproblems independently".to_string(),
                    "Integrate solutions".to_string(),
                    "Optimize the overall solution".to_string(),
                ],
            },
            StrategyTemplate {
                name: "Test-Driven Approach".to_string(),
                tactics: vec![
                    "Write tests first".to_string(),
                    "Implement minimal code to pass tests".to_string(),
                    "Refactor while keeping tests green".to_string(),
                    "Add more tests for edge cases".to_string(),
                ],
            },
        ];
        
        Self {
            strategy_library: RwLock::new(strategies),
        }
    }
    
    pub fn develop_strategy(&self, context: &str) -> Result<Strategy> {
        let library = self.strategy_library.read();
        
        // Select appropriate strategy based on context
        let template = library.iter()
            .find(|s| context.to_lowercase().contains(&s.name.to_lowercase()))
            .or_else(|| library.first())
            .cloned()
            .unwrap();
        
        Ok(Strategy {
            name: format!("{} Strategy for: {}", template.name, context),
            tactics: template.tactics,
        })
    }
    
    pub fn adapt_strategy(&self, strategy: &mut Strategy, feedback: &str) -> Result<()> {
        // Simple adaptation based on feedback
        if feedback.contains("slow") || feedback.contains("inefficient") {
            strategy.tactics.push("Optimize performance bottlenecks".to_string());
        }
        
        if feedback.contains("complex") || feedback.contains("difficult") {
            strategy.tactics.insert(0, "Simplify the approach".to_string());
        }
        
        if feedback.contains("error") || feedback.contains("fail") {
            strategy.tactics.push("Add error handling and recovery".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tactical_neuron() {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Tactical,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        };
        
        let mut neuron = L4TacticalNeuron::new(config);
        
        // Test planning
        let input = CognitiveInput {
            content: "Create a plan to develop a new software feature".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Strategic),
        };
        
        let output = neuron.process(input).await.unwrap();
        assert!(output.content.contains("Tactical Plan"));
        assert!(output.metadata.contains_key("plan_id"));
        
        // Test strategy development
        let input2 = CognitiveInput {
            content: "Develop an incremental development strategy".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Strategic),
        };
        
        let output2 = neuron.process(input2).await.unwrap();
        assert!(output2.content.contains("Strategy"));
        assert!(output2.content.contains("Tactics"));
        
        // Verify state
        let state = neuron.introspect().await;
        assert!(state.current_plan.is_some());
        assert!(!state.active_strategies.is_empty());
    }
}