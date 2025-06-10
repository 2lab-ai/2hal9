//! L3: Operational Neuron - Design and task coordination
//!
//! This neuron handles system design, architecture decisions, and
//! task coordination across multiple implementation units.

use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::Result;
use super::*;

/// L3: Operational Neuron - System design and coordination
pub struct L3OperationalNeuron {
    id: Uuid,
    state: Arc<RwLock<OperationalState>>,
    designer: Arc<SystemDesigner>,
    coordinator: Arc<TaskCoordinator>,
}

impl L3OperationalNeuron {
    pub fn new(config: CognitiveConfig) -> Self {
        Self {
            id: config.id,
            state: Arc::new(RwLock::new(OperationalState {
                basic: BasicCognitiveState {
                    unit_id: config.id,
                    layer: CognitiveLayer::Operational,
                    metrics: StateMetrics {
                        activations_processed: 0,
                        errors_encountered: 0,
                        learning_iterations: 0,
                        average_processing_time_ms: 0.0,
                        memory_usage_bytes: 0,
                    },
                    parameters: config.initial_parameters,
                },
                current_design: None,
                task_queue: Vec::new(),
            })),
            designer: Arc::new(SystemDesigner::new()),
            coordinator: Arc::new(TaskCoordinator::new(config.connections)),
        }
    }
    
    /// Analyze input to determine operational approach
    fn analyze_request(&self, input: &str) -> OperationalAnalysis {
        let lower = input.to_lowercase();
        
        if lower.contains("design") || lower.contains("architect") {
            OperationalAnalysis::SystemDesign
        } else if lower.contains("coordinate") || lower.contains("distribute") {
            OperationalAnalysis::TaskCoordination
        } else if lower.contains("integrate") || lower.contains("connect") {
            OperationalAnalysis::Integration
        } else if lower.contains("optimize") || lower.contains("improve") {
            OperationalAnalysis::Optimization
        } else {
            OperationalAnalysis::General
        }
    }
    
    /// Create tasks from a design
    fn decompose_design(&self, design: &SystemDesign) -> Vec<Task> {
        let mut tasks = Vec::new();
        
        // Create implementation tasks for each component
        for component in &design.components {
            tasks.push(Task {
                id: Uuid::new_v4(),
                description: format!("Implement {} - {}", component.name, component.responsibility),
                priority: 0.8,
                assigned_to: Some(CognitiveLayer::Implementation),
            });
        }
        
        // Create integration tasks for interactions
        for interaction in &design.interactions {
            tasks.push(Task {
                id: Uuid::new_v4(),
                description: format!("Integrate {} with {} using {}", 
                                   interaction.from, interaction.to, interaction.protocol),
                priority: 0.7,
                assigned_to: Some(CognitiveLayer::Implementation),
            });
        }
        
        // Create testing task
        tasks.push(Task {
            id: Uuid::new_v4(),
            description: "Create comprehensive tests for the system".to_string(),
            priority: 0.9,
            assigned_to: Some(CognitiveLayer::Implementation),
        });
        
        tasks
    }
}

#[derive(Debug, Clone)]
enum OperationalAnalysis {
    SystemDesign,
    TaskCoordination,
    Integration,
    Optimization,
    General,
}

#[async_trait]
impl CognitiveUnit for L3OperationalNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = OperationalState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Operational
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        
        // Analyze the request
        let analysis = self.analyze_request(&input.content);
        
        let (content, target_layers) = match analysis {
            OperationalAnalysis::SystemDesign => {
                // Create a system design
                let design = self.designer.create_design(&input.content)?;
                
                // Decompose into tasks
                let tasks = self.decompose_design(&design);
                
                // Store design and tasks
                {
                    let mut state = self.state.write();
                    state.current_design = Some(design.clone());
                    state.task_queue.extend(tasks.clone());
                }
                
                // Format output
                let output = format!(
                    "System Design:\n{}\n\nDecomposed into {} tasks",
                    serde_json::to_string_pretty(&design).unwrap_or_default(),
                    tasks.len()
                );
                
                (output, vec![CognitiveLayer::Implementation, CognitiveLayer::Tactical])
            }
            
            OperationalAnalysis::TaskCoordination => {
                // Coordinate tasks
                let coordination_plan = self.coordinator.create_coordination_plan(&input.content)?;
                
                (coordination_plan, vec![CognitiveLayer::Implementation])
            }
            
            OperationalAnalysis::Integration => {
                // Plan integration
                let integration_plan = self.designer.plan_integration(&input.content)?;
                
                (integration_plan, vec![CognitiveLayer::Implementation])
            }
            
            OperationalAnalysis::Optimization => {
                // Suggest optimizations
                let optimizations = self.designer.suggest_optimizations(&input.content)?;
                
                (optimizations, vec![CognitiveLayer::Implementation, CognitiveLayer::Tactical])
            }
            
            OperationalAnalysis::General => {
                // General operational response
                let response = format!("Operational analysis: {}", input.content);
                
                (response, vec![CognitiveLayer::Implementation])
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
        
        Ok(CognitiveOutput {
            content,
            confidence: 0.8,
            metadata: [
                ("analysis_type".to_string(), serde_json::json!(format!("{:?}", analysis))),
                ("processing_time_ms".to_string(), serde_json::json!(start.elapsed().as_millis())),
                ("tasks_queued".to_string(), serde_json::json!(self.state.read().task_queue.len())),
            ].into_iter().collect(),
            target_layers,
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        let mut state = self.state.write();
        state.basic.metrics.learning_iterations += 1;
        
        // Learn from design feedback
        if gradient.error_signal.magnitude > 0.3 {
            // Design could be improved
            if let Some(ref mut design) = state.current_design {
                // Simplify design if too complex
                if design.components.len() > 10 {
                    self.designer.simplify_design(design);
                }
            }
        }
        
        // Adjust parameters
        for adjustment in &gradient.adjustments {
            if let Some(param) = state.basic.parameters.get_mut(&adjustment.parameter) {
                *param += adjustment.suggested_delta * 0.02; // Slower learning rate
            }
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        self.state.read().clone()
    }
    
    async fn reset(&mut self) -> Result<()> {
        let mut state = self.state.write();
        state.current_design = None;
        state.task_queue.clear();
        Ok(())
    }
}

/// System designer for creating architectures
pub struct SystemDesigner {
    patterns: RwLock<Vec<DesignPattern>>,
}

#[derive(Clone)]
struct DesignPattern {
    name: String,
    components: Vec<String>,
    applicable_to: Vec<String>,
}

impl Default for SystemDesigner {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDesigner {
    pub fn new() -> Self {
        let patterns = vec![
            DesignPattern {
                name: "MVC".to_string(),
                components: vec!["Model".to_string(), "View".to_string(), "Controller".to_string()],
                applicable_to: vec!["web".to_string(), "gui".to_string()],
            },
            DesignPattern {
                name: "Microservices".to_string(),
                components: vec!["Service".to_string(), "API Gateway".to_string(), "Message Queue".to_string()],
                applicable_to: vec!["distributed".to_string(), "scalable".to_string()],
            },
            DesignPattern {
                name: "Event Sourcing".to_string(),
                components: vec!["Event Store".to_string(), "Projections".to_string(), "Command Handler".to_string()],
                applicable_to: vec!["audit".to_string(), "history".to_string()],
            },
        ];
        
        Self {
            patterns: RwLock::new(patterns),
        }
    }
    
    pub fn create_design(&self, description: &str) -> Result<SystemDesign> {
        // Select appropriate pattern
        let patterns = self.patterns.read();
        let pattern = patterns.iter()
            .find(|p| p.applicable_to.iter().any(|keyword| description.contains(keyword)))
            .cloned()
            .unwrap_or_else(|| DesignPattern {
                name: "Custom".to_string(),
                components: vec!["Component1".to_string(), "Component2".to_string()],
                applicable_to: vec![],
            });
        
        // Create design based on pattern
        let components: Vec<Component> = pattern.components.iter()
            .map(|name| Component {
                name: name.clone(),
                responsibility: format!("Handle {} concerns", name.to_lowercase()),
            })
            .collect();
        
        let interactions = if components.len() > 1 {
            vec![Interaction {
                from: components[0].name.clone(),
                to: components[1].name.clone(),
                protocol: "REST API".to_string(),
            }]
        } else {
            vec![]
        };
        
        Ok(SystemDesign {
            components,
            interactions,
        })
    }
    
    pub fn plan_integration(&self, description: &str) -> Result<String> {
        Ok(format!(
            "Integration Plan:\n\
             1. Define interfaces between components\n\
             2. Implement message passing protocols\n\
             3. Set up error handling and recovery\n\
             4. Add monitoring and logging\n\
             5. Test integration points\n\
             Context: {}",
            description
        ))
    }
    
    pub fn suggest_optimizations(&self, description: &str) -> Result<String> {
        let mut suggestions = vec![
            "Consider caching frequently accessed data",
            "Implement connection pooling for database access",
            "Use async/await for I/O operations",
            "Add indices to frequently queried fields",
            "Consider horizontal scaling for high load",
        ];
        
        if description.contains("slow") {
            suggestions.push("Profile the application to identify bottlenecks");
        }
        
        if description.contains("memory") {
            suggestions.push("Analyze memory usage patterns and fix leaks");
        }
        
        Ok(suggestions.join("\n"))
    }
    
    pub fn simplify_design(&self, design: &mut SystemDesign) {
        // Remove redundant components
        if design.components.len() > 5 {
            design.components.truncate(5);
        }
        
        // Simplify interactions
        if design.interactions.len() > design.components.len() * 2 {
            design.interactions.truncate(design.components.len() * 2);
        }
    }
}

/// Task coordinator for distributed work
pub struct TaskCoordinator {
    connections: ConnectionConfig,
}

impl TaskCoordinator {
    pub fn new(connections: ConnectionConfig) -> Self {
        Self { connections }
    }
    
    pub fn create_coordination_plan(&self, description: &str) -> Result<String> {
        let plan = format!(
            "Task Coordination Plan:\n\
             1. Analyze task dependencies\n\
             2. Assign tasks to appropriate layers:\n\
             - Implementation: {} connections\n\
             - Lateral: {} connections\n\
             3. Set up communication channels\n\
             4. Monitor task progress\n\
             5. Handle task failures and retries\n\
             Context: {}",
            self.connections.downward_connections.len(),
            self.connections.lateral_connections.len(),
            description
        );
        
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_operational_neuron() {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Operational,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![Uuid::new_v4(), Uuid::new_v4()],
            },
        };
        
        let mut neuron = L3OperationalNeuron::new(config);
        
        // Test system design
        let input = CognitiveInput {
            content: "Design a web application for user management".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Tactical),
        };
        
        let output = neuron.process(input).await.unwrap();
        assert!(output.content.contains("System Design"));
        assert!(output.target_layers.contains(&CognitiveLayer::Implementation));
        
        // Check that tasks were created
        let state = neuron.introspect().await;
        assert!(!state.task_queue.is_empty());
        assert!(state.current_design.is_some());
    }
}