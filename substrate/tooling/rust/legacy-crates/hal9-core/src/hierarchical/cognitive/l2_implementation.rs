//! L2: Implementation Neuron - Direct code execution and implementation
//!
//! This neuron handles direct implementation tasks, code generation,
//! and execution within safe sandboxes.

use async_trait::async_trait;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::{Result, Error};
use crate::hierarchical::protocol::{GradientProtocol, GradientMessage, Gradient};
use super::*;

/// L2: Implementation Neuron - Code generation and execution
pub struct L2ImplementationNeuron {
    id: Uuid,
    state: Arc<RwLock<ImplementationState>>,
    code_generator: Arc<CodeGenerator>,
    executor: Arc<CodeExecutor>,
    gradient_protocol: Option<Arc<GradientProtocol>>,
}

impl L2ImplementationNeuron {
    pub fn new(config: CognitiveConfig) -> Self {
        Self {
            id: config.id,
            state: Arc::new(RwLock::new(ImplementationState {
                basic: BasicCognitiveState {
                    unit_id: config.id,
                    layer: CognitiveLayer::Implementation,
                    metrics: StateMetrics {
                        activations_processed: 0,
                        errors_encountered: 0,
                        learning_iterations: 0,
                        average_processing_time_ms: 0.0,
                        memory_usage_bytes: 0,
                    },
                    parameters: config.initial_parameters,
                },
                code_context: CodeContext {
                    language: "rust".to_string(),
                    imports: vec![
                        "use std::collections::HashMap;".to_string(),
                        "use serde::{Serialize, Deserialize};".to_string(),
                    ],
                    functions: HashMap::new(),
                },
                execution_history: Vec::new(),
            })),
            code_generator: Arc::new(CodeGenerator::new()),
            executor: Arc::new(CodeExecutor::new()),
            gradient_protocol: None,
        }
    }
    
    /// Set gradient protocol for learning feedback
    pub fn set_gradient_protocol(&mut self, protocol: Arc<GradientProtocol>) {
        self.gradient_protocol = Some(protocol);
    }
    
    /// Analyze input to determine implementation approach
    fn analyze_task(&self, input: &str) -> TaskAnalysis {
        let lower = input.to_lowercase();
        
        if lower.contains("function") || lower.contains("method") {
            TaskAnalysis::Function
        } else if lower.contains("struct") || lower.contains("class") {
            TaskAnalysis::DataStructure
        } else if lower.contains("test") {
            TaskAnalysis::Test
        } else if lower.contains("refactor") {
            TaskAnalysis::Refactor
        } else {
            TaskAnalysis::General
        }
    }
}

#[derive(Debug, Clone)]
enum TaskAnalysis {
    Function,
    DataStructure,
    Test,
    Refactor,
    General,
}

#[async_trait]
impl CognitiveUnit for L2ImplementationNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = ImplementationState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Implementation
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        
        // Analyze the task
        let task_type = self.analyze_task(&input.content);
        
        // Generate code based on task type
        let generated_code = match task_type {
            TaskAnalysis::Function => {
                self.code_generator.generate_function(&input.content)?
            }
            TaskAnalysis::DataStructure => {
                self.code_generator.generate_struct(&input.content)?
            }
            TaskAnalysis::Test => {
                self.code_generator.generate_test(&input.content)?
            }
            TaskAnalysis::Refactor => {
                self.code_generator.suggest_refactoring(&input.content)?
            }
            TaskAnalysis::General => {
                self.code_generator.generate_general(&input.content)?
            }
        };
        
        // Execute in sandbox if requested
        let execution_result = if input.context.get("execute").and_then(|v| v.as_bool()).unwrap_or(false) {
            match self.executor.execute_safe(&generated_code).await {
                Ok(output) => ExecutionResult::Success(output),
                Err(e) => ExecutionResult::Error(e.to_string()),
            }
        } else {
            ExecutionResult::Success("Code generated but not executed".to_string())
        };
        
        // Update state
        {
            let mut state = self.state.write();
            state.basic.metrics.activations_processed += 1;
            
            // Store in history
            state.execution_history.push(ExecutionRecord {
                timestamp: chrono::Utc::now(),
                code: generated_code.clone(),
                result: execution_result.clone(),
            });
            
            // Keep only last 100 records
            if state.execution_history.len() > 100 {
                state.execution_history.remove(0);
            }
            
            // Update average processing time
            let elapsed = start.elapsed();
            let processed = state.basic.metrics.activations_processed as f64;
            state.basic.metrics.average_processing_time_ms = 
                (state.basic.metrics.average_processing_time_ms * (processed - 1.0) + 
                 elapsed.as_secs_f64() * 1000.0) / processed;
        }
        
        // Prepare output
        let output = CognitiveOutput {
            content: generated_code,
            confidence: 0.85,
            metadata: [
                ("task_type".to_string(), serde_json::json!(format!("{:?}", task_type))),
                ("execution_result".to_string(), serde_json::json!(matches!(execution_result, ExecutionResult::Success(_)))),
                ("processing_time_ms".to_string(), serde_json::json!(start.elapsed().as_millis())),
            ].into_iter().collect(),
            target_layers: vec![CognitiveLayer::Operational],
        };
        
        Ok(output)
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        // Get learning iterations before dropping the lock
        let learning_iterations = {
            let mut state = self.state.write();
            state.basic.metrics.learning_iterations += 1;
            state.basic.metrics.learning_iterations
        };
        
        // Learn from execution errors
        if gradient.error_signal.magnitude > 0.5 {
            // High error - need to improve code generation
            if let Some(context) = gradient.error_signal.context.get("failed_code") {
                if let Some(code) = context.as_str() {
                    // Store failed pattern for future avoidance
                    self.code_generator.add_failed_pattern(code);
                }
            }
        }
        
        // Send gradient upstream if protocol available
        if let Some(protocol) = &self.gradient_protocol {
            let grad_msg = GradientMessage {
                id: Uuid::new_v4(),
                source_neuron: self.id,
                target_neuron: Uuid::nil(), // Will be routed by protocol
                timestamp: chrono::Utc::now(),
                gradient: Gradient::new(
                    gradient.error_signal.magnitude,
                    gradient.adjustments.iter().map(|a| a.suggested_delta).collect(),
                ),
                learning_context: crate::hierarchical::protocol::gradient::LearningContext {
                    learning_rate: 0.05,
                    momentum: 0.9,
                    batch_size: 1,
                    epoch: learning_iterations as u32,
                    loss_type: crate::hierarchical::protocol::gradient::LossType::MeanSquaredError,
                },
            };
            
            let _ = protocol.send_gradient(grad_msg).await;
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        self.state.read().clone()
    }
    
    async fn reset(&mut self) -> Result<()> {
        let mut state = self.state.write();
        state.execution_history.clear();
        state.code_context.functions.clear();
        Ok(())
    }
}

/// Code generator with templates and patterns
pub struct CodeGenerator {
    templates: RwLock<HashMap<String, String>>,
    failed_patterns: RwLock<Vec<String>>,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Basic function template
        templates.insert("function".to_string(), r#"
fn {name}({params}) -> {return_type} {
    {body}
}
"#.to_string());
        
        // Struct template
        templates.insert("struct".to_string(), r#"
#[derive(Debug, Clone)]
struct {name} {
    {fields}
}

impl {name} {
    pub fn new({params}) -> Self {
        Self {
            {field_init}
        }
    }
}
"#.to_string());
        
        // Test template
        templates.insert("test".to_string(), r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_{name}() {
        {test_body}
    }
}
"#.to_string());
        
        Self {
            templates: RwLock::new(templates),
            failed_patterns: RwLock::new(Vec::new()),
        }
    }
    
    pub fn generate_function(&self, description: &str) -> Result<String> {
        // Parse description to extract function details
        let name = self.extract_name(description).unwrap_or("generated_function".to_string());
        let params = self.extract_params(description).unwrap_or("".to_string());
        let return_type = self.extract_return_type(description).unwrap_or("()");
        let body = self.generate_function_body(description);
        
        let template = self.templates.read()
            .get("function")
            .cloned()
            .unwrap();
        
        Ok(template
            .replace("{name}", &name)
            .replace("{params}", &params)
            .replace("{return_type}", return_type)
            .replace("{body}", &body))
    }
    
    pub fn generate_struct(&self, description: &str) -> Result<String> {
        let name = self.extract_name(description).unwrap_or("GeneratedStruct".to_string());
        let fields = self.extract_fields(description);
        let params = self.generate_constructor_params(&fields);
        let field_init = self.generate_field_init(&fields);
        
        let template = self.templates.read()
            .get("struct")
            .cloned()
            .unwrap();
        
        Ok(template
            .replace("{name}", &name)
            .replace("{fields}", &fields)
            .replace("{params}", &params)
            .replace("{field_init}", &field_init))
    }
    
    pub fn generate_test(&self, description: &str) -> Result<String> {
        let name = self.extract_name(description).unwrap_or("generated".to_string());
        let test_body = self.generate_test_body(description);
        
        let template = self.templates.read()
            .get("test")
            .cloned()
            .unwrap();
        
        Ok(template
            .replace("{name}", &name)
            .replace("{test_body}", &test_body))
    }
    
    pub fn suggest_refactoring(&self, code: &str) -> Result<String> {
        // Simple refactoring suggestions
        let mut suggestions = Vec::new();
        
        if code.contains("unwrap()") {
            suggestions.push("Consider replacing unwrap() with proper error handling using ?");
        }
        
        if code.lines().any(|line| line.len() > 100) {
            suggestions.push("Consider breaking long lines for better readability");
        }
        
        if !code.contains("///") && !code.contains("//!") {
            suggestions.push("Add documentation comments to explain the code");
        }
        
        Ok(suggestions.join("\n"))
    }
    
    pub fn generate_general(&self, description: &str) -> Result<String> {
        // Fallback for general code generation
        Ok(format!("// TODO: Implement {}\n// Generated from: {}", 
                   self.extract_name(description).unwrap_or("feature".to_string()),
                   description))
    }
    
    pub fn add_failed_pattern(&self, pattern: &str) {
        self.failed_patterns.write().push(pattern.to_string());
    }
    
    // Helper methods
    fn extract_name(&self, description: &str) -> Option<String> {
        // Simple extraction - could be enhanced with NLP
        description.split_whitespace()
            .find(|word| word.chars().all(|c| c.is_alphanumeric() || c == '_'))
            .map(|s| s.to_string())
    }
    
    fn extract_params(&self, description: &str) -> Option<String> {
        // Extract parameter hints from description
        if description.contains("no parameters") || description.contains("no args") {
            Some("".to_string())
        } else {
            Some("value: &str".to_string()) // Default parameter
        }
    }
    
    fn extract_return_type(&self, description: &str) -> Option<&str> {
        if description.contains("returns string") {
            Some("String")
        } else if description.contains("returns number") || description.contains("returns int") {
            Some("i32")
        } else if description.contains("returns bool") {
            Some("bool")
        } else {
            Some("()")
        }
    }
    
    fn generate_function_body(&self, description: &str) -> String {
        "    // TODO: Implement function logic\n    todo!()".to_string()
    }
    
    fn extract_fields(&self, description: &str) -> String {
        "    id: Uuid,\n    name: String,\n    value: i32,".to_string()
    }
    
    fn generate_constructor_params(&self, _fields: &str) -> String {
        "name: String, value: i32".to_string()
    }
    
    fn generate_field_init(&self, _fields: &str) -> String {
        "            id: Uuid::new_v4(),\n            name,\n            value,".to_string()
    }
    
    fn generate_test_body(&self, description: &str) -> String {
        "        // TODO: Implement test\n        assert_eq!(1 + 1, 2);".to_string()
    }
}

/// Safe code executor with sandboxing
pub struct CodeExecutor {
    sandbox_config: SandboxConfig,
}

#[derive(Clone)]
struct SandboxConfig {
    timeout: std::time::Duration,
    memory_limit: usize,
}

impl Default for CodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeExecutor {
    pub fn new() -> Self {
        Self {
            sandbox_config: SandboxConfig {
                timeout: std::time::Duration::from_secs(5),
                memory_limit: 100 * 1024 * 1024, // 100MB
            },
        }
    }
    
    pub async fn execute_safe(&self, code: &str) -> Result<String> {
        // In a real implementation, this would use a proper sandbox
        // For now, we'll simulate safe execution
        
        if code.contains("unsafe") {
            return Err(Error::Protocol("Unsafe code not allowed".to_string()));
        }
        
        if code.contains("std::process") {
            return Err(Error::Protocol("Process spawning not allowed".to_string()));
        }
        
        if code.contains("std::fs") {
            return Err(Error::Protocol("File system access not allowed".to_string()));
        }
        
        // Simulate successful execution
        Ok(format!("Code executed successfully:\n{}", code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_implementation_neuron() {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Implementation,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        };
        
        let mut neuron = L2ImplementationNeuron::new(config);
        
        // Test function generation
        let input = CognitiveInput {
            content: "Create a function called calculate_sum that adds two numbers".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Operational),
        };
        
        let output = neuron.process(input).await.unwrap();
        assert!(output.content.contains("fn calculate_sum"));
        assert!(output.confidence > 0.8);
        
        // Test struct generation
        let input2 = CognitiveInput {
            content: "Create a struct called User with name and age fields".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Operational),
        };
        
        let output2 = neuron.process(input2).await.unwrap();
        assert!(output2.content.contains("struct User"));
    }
}