//! Neuron management and registry

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, instrument};
use crate::{log_performance, log_structured, logging::neuron_span};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use hal9_core::{
    Error, Result, NeuronSignal, Layer, NeuronInterface,
    NeuronConfig, PropagationType, Gradient,
    neuron::{NeuronState, NeuronHealth},
    mcp::{ToolRegistry, FilesystemReadTool, FilesystemWriteTool, 
          ShellTool, WebFetchTool},
    memory::{MemoryStore, MemoryBuilder, MemoryType},
    learning::{ErrorGradient, GradientCalculator, PromptAdjuster, 
               PatternMatcher},
};
use md5;

use crate::{
    claude::ClaudeInterface,
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    performance::{ResponseCache, PerformanceMonitor},
};

/// A managed neuron that wraps a Claude instance
pub struct ManagedNeuron {
    pub id: String,
    pub layer: Layer,
    pub config: NeuronConfig,
    claude: Box<dyn ClaudeInterface>,
    state: RwLock<NeuronState>,
    stats: RwLock<NeuronStats>,
    metrics: Option<Arc<crate::metrics::Metrics>>,
    circuit_breaker: CircuitBreaker,
    response_cache: Option<ResponseCache>,
    performance_monitor: PerformanceMonitor,
    tool_registry: ToolRegistry,
    memory_store: Option<Arc<dyn MemoryStore>>,
    prompt_adjuster: Option<RwLock<PromptAdjuster>>,
    pattern_matcher: Option<RwLock<PatternMatcher>>,
    gradient_calculator: Option<GradientCalculator>,
}

#[derive(Default)]
struct NeuronStats {
    signals_processed: u64,
    errors_count: u64,
    last_signal: Option<chrono::DateTime<chrono::Utc>>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ManagedNeuron {
    /// Create a new managed neuron
    pub fn new(
        config: NeuronConfig,
        claude: Box<dyn ClaudeInterface>,
    ) -> Result<Self> {
        let layer = Layer::from_str(&config.layer)
            .ok_or_else(|| Error::Config(format!("Invalid layer: {}", config.layer)))?;
            
        let circuit_breaker = CircuitBreaker::new(
            format!("neuron-{}", config.id),
            CircuitBreakerConfig::default()
        );
        
        // Enable caching for all layers with different TTLs
        // L2 gets longest TTL, L3/L4 get shorter TTLs
        let response_cache = match layer {
            Layer::L2 => Some(ResponseCache::new(
                std::time::Duration::from_secs(600), // 10 minute TTL for implementation
                2000 // max entries
            )),
            Layer::L3 => Some(ResponseCache::new(
                std::time::Duration::from_secs(300), // 5 minute TTL for design
                1000 // max entries
            )),
            Layer::L4 => Some(ResponseCache::new(
                std::time::Duration::from_secs(120), // 2 minute TTL for strategy
                500 // max entries
            )),
            _ => None,
        };
        
        // Initialize tool registry based on layer
        let mut tool_registry = ToolRegistry::new();
        
        // Different layers have different tool permissions
        match layer {
            Layer::L4 => {
                // Strategic layer: can read docs and fetch web resources
                tool_registry.register(Box::new(FilesystemReadTool::new(vec![
                    "./docs".to_string(),
                    "./README.md".to_string(),
                    "./PRD.md".to_string(),
                ])));
                tool_registry.register(Box::new(WebFetchTool::new(None))); // No domain restrictions
            }
            Layer::L3 => {
                // Design layer: can read source code and examples
                tool_registry.register(Box::new(FilesystemReadTool::new(vec![
                    "./src".to_string(),
                    "./examples".to_string(),
                    "./Cargo.toml".to_string(),
                ])));
                tool_registry.register(Box::new(ShellTool::new(vec![
                    "cargo".to_string(),
                    "rustfmt".to_string(),
                    "clippy".to_string(),
                ])));
            }
            Layer::L2 => {
                // Implementation layer: full file access and safe shell commands
                tool_registry.register(Box::new(FilesystemReadTool::new(vec![
                    ".".to_string(), // Full project access
                ])));
                tool_registry.register(Box::new(FilesystemWriteTool::new(vec![
                    "./src".to_string(),
                    "./tests".to_string(),
                    "./examples".to_string(),
                    "/tmp".to_string(),
                ])));
                tool_registry.register(Box::new(ShellTool::new(vec![
                    "cargo".to_string(),
                    "ls".to_string(),
                    "echo".to_string(),
                    "date".to_string(),
                    "pwd".to_string(),
                ])));
            }
            Layer::L1 => {
                // Base layer: minimal tools
                tool_registry.register(Box::new(ShellTool::new(vec![
                    "echo".to_string(),
                    "date".to_string(),
                ])));
            }
        }
        
        Ok(Self {
            id: config.id.clone(),
            layer,
            config,
            claude,
            state: RwLock::new(NeuronState::Starting),
            stats: RwLock::new(NeuronStats {
                started_at: Some(Utc::now()),
                ..Default::default()
            }),
            metrics: None,
            circuit_breaker,
            response_cache,
            performance_monitor: PerformanceMonitor::new(),
            tool_registry,
            memory_store: None,
            prompt_adjuster: None,
            pattern_matcher: None,
            gradient_calculator: None,
        })
    }
    
    /// Set metrics collector
    pub fn set_metrics(&mut self, metrics: Arc<crate::metrics::Metrics>) {
        self.metrics = Some(metrics);
    }
    
    /// Set memory store
    pub fn set_memory_store(&mut self, memory_store: Arc<dyn MemoryStore>) {
        self.memory_store = Some(memory_store);
    }
    
    /// Enable backward propagation
    pub fn enable_backward_propagation(&mut self, config: hal9_core::config::BackwardPropagationConfig, base_prompt: String) {
        if config.enabled {
            self.prompt_adjuster = Some(RwLock::new(
                PromptAdjuster::new(self.id.clone(), base_prompt, config.learning_rate)
            ));
            self.pattern_matcher = Some(RwLock::new(
                PatternMatcher::new(config.pattern_threshold)
            ));
            self.gradient_calculator = Some(GradientCalculator::new(
                hal9_core::learning::BackwardPropagationConfig::from(config)
            ));
        }
    }
    
    /// Start the neuron
    pub async fn start(&self) -> Result<()> {
        info!("Starting neuron {} on layer {}", self.id, self.layer.as_str());
        *self.state.write().await = NeuronState::Running;
        Ok(())
    }
    
    /// Format a signal into a prompt for Claude
    async fn format_prompt(&self, signal: &NeuronSignal) -> String {
        let tool_definitions = self.tool_registry.definitions();
        let tool_info = if !tool_definitions.is_empty() {
            let tool_list = tool_definitions.iter()
                .map(|t| format!("- {}: {}", t.name, t.description))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\nAVAILABLE TOOLS:\n{}\n\nTo use a tool, respond with:\nTOOL: <tool_name> <json_params>\n", tool_list)
        } else {
            String::new()
        };
        
        // Build memory context if available
        let memory_context = if let Some(memory_store) = &self.memory_store {
            match memory_store.build_context(&self.id, &signal.payload.activation.content).await {
                Ok(context) => {
                    let mut context_str = String::new();
                    
                    if !context.recent_tasks.is_empty() {
                        context_str.push_str("\n\nRECENT TASKS:\n");
                        for entry in context.recent_tasks.iter().take(3) {
                            context_str.push_str(&format!("- {}\n", entry.content));
                        }
                    }
                    
                    if !context.relevant_learnings.is_empty() {
                        context_str.push_str("\nRELEVANT LEARNINGS:\n");
                        for entry in context.relevant_learnings.iter() {
                            context_str.push_str(&format!("- {}\n", entry.content));
                        }
                    }
                    
                    if !context.error_patterns.is_empty() {
                        context_str.push_str("\nKNOWN ERROR PATTERNS:\n");
                        for entry in context.error_patterns.iter() {
                            context_str.push_str(&format!("- {}\n", entry.content));
                        }
                    }
                    
                    context_str
                }
                Err(e) => {
                    warn!("Failed to build memory context: {}", e);
                    String::new()
                }
            }
        } else {
            String::new()
        };
        
        match signal.propagation_type {
            PropagationType::Forward => {
                format!(
                    "FORWARD_SIGNAL\nFrom: {}\nLayer: {}\nStrength: {}\nContent: {}\nFeatures: {:?}\n{}{}\nYour response should include FORWARD_TO: <target_neurons> and CONTENT: <your_analysis>",
                    signal.from_neuron,
                    signal.layer_from,
                    signal.payload.activation.strength,
                    signal.payload.activation.content,
                    signal.payload.activation.features,
                    memory_context,
                    tool_info
                )
            }
            PropagationType::Backward => {
                let gradient = signal.payload.gradient.as_ref().unwrap();
                format!(
                    "BACKWARD_SIGNAL\nFrom: {}\nError: {}\nMagnitude: {}\nLoss: {}\nAdjustments: {:?}\n{}{}\nYour response should include BACKWARD_TO: <target_neurons> and ERROR_TYPE: <error_description>",
                    signal.from_neuron,
                    gradient.error_type,
                    gradient.magnitude,
                    gradient.loss,
                    gradient.adjustments,
                    memory_context,
                    tool_info
                )
            }
        }
    }
    
    /// Process backward propagation signal
    async fn process_backward_signal(&self, signal: &NeuronSignal) -> Result<()> {
        if let Some(gradient_data) = &signal.payload.gradient {
            if let (Some(pattern_matcher), Some(prompt_adjuster)) = 
                (self.pattern_matcher.as_ref(), self.prompt_adjuster.as_ref()) 
            {
                // Create error gradient from signal
                let error_gradient = ErrorGradient {
                    id: Uuid::new_v4(),
                    error_type: hal9_core::learning::ErrorType::TaskFailed { 
                        reason: gradient_data.error_type.clone() 
                    },
                    magnitude: gradient_data.magnitude,
                    source_neuron: signal.from_neuron.clone(),
                    target_neuron: self.id.clone(),
                    timestamp: signal.timestamp,
                    context: hal9_core::learning::ErrorContext {
                        original_task: String::new(),
                        attempted_solution: String::new(),
                        failure_point: gradient_data.error_type.clone(),
                        environmental_factors: HashMap::new(),
                    },
                    suggested_adjustments: gradient_data.adjustments.iter()
                        .map(|adj| hal9_core::learning::Adjustment {
                            parameter: "guideline".to_string(),
                            current_value: serde_json::Value::Null,
                            suggested_value: serde_json::Value::String(adj.clone()),
                            confidence: 0.7,
                            rationale: "Learned from error".to_string(),
                        })
                        .collect(),
                    propagation_depth: 0,
                };
                
                // Process with pattern matcher
                let mut matcher = pattern_matcher.write().await;
                if let Some(pattern) = matcher.process_error(&error_gradient) {
                    info!("Error pattern identified: {}", pattern.error_signature);
                    
                    // Store learning in memory
                    if let Some(memory_store) = &self.memory_store {
                        let learning_memory = MemoryBuilder::new(
                            self.id.clone(), 
                            self.layer.as_str().to_string()
                        )
                        .with_type(MemoryType::Learning)
                        .with_content(format!(
                            "Pattern identified: {}. Prevention: {:?}",
                            pattern.error_signature,
                            pattern.prevention_strategy
                        ))
                        .with_importance(0.8)
                        .build();
                        
                        let _ = memory_store.store(learning_memory).await;
                    }
                }
                
                // Apply adjustments
                if !error_gradient.suggested_adjustments.is_empty() {
                    let mut adjuster = prompt_adjuster.write().await;
                    for adjustment in &error_gradient.suggested_adjustments {
                        let _ = adjuster.apply_adjustment(&error_gradient.error_type, adjustment);
                    }
                    info!("Applied {} adjustments to neuron {}", 
                          error_gradient.suggested_adjustments.len(), self.id);
                }
            }
        }
        Ok(())
    }

    /// Parse response and determine next signals
    pub fn parse_response(&self, response: &str, _original_signal: &NeuronSignal) -> Vec<NeuronSignal> {
        let mut signals = Vec::new();
        
        // Parse FORWARD_TO directive
        if let Some(forward_line) = response.lines().find(|l| l.starts_with("FORWARD_TO:")) {
            let targets = forward_line
                .strip_prefix("FORWARD_TO:")
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
                
            // Extract content after CONTENT: line
            let content = response.lines()
                .skip_while(|l| !l.starts_with("CONTENT:"))
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");
                
            for target in targets {
                if self.config.forward_connections.contains(&target.to_string()) {
                    signals.push(NeuronSignal::forward(
                        &self.id,
                        target,
                        self.layer.as_str(),
                        &self.get_target_layer(target),
                        content.clone(),
                    ));
                }
            }
        }
        
        // Parse BACKWARD_TO directive for error propagation
        if let Some(backward_line) = response.lines().find(|l| l.starts_with("BACKWARD_TO:")) {
            let targets = backward_line
                .strip_prefix("BACKWARD_TO:")
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
                
            let error_type = response.lines()
                .find(|l| l.starts_with("ERROR_TYPE:"))
                .and_then(|l| l.strip_prefix("ERROR_TYPE:"))
                .unwrap_or("Unknown")
                .trim()
                .to_string();
                
            for target in targets {
                if self.config.backward_connections.contains(&target.to_string()) {
                    signals.push(NeuronSignal::backward(
                        &self.id,
                        target,
                        self.layer.as_str(),
                        &self.get_target_layer(target),
                        Gradient::new(error_type.clone(), 0.5),
                    ));
                }
            }
        }
        
        signals
    }
    
    /// Infer target layer from neuron ID (simplified)
    fn get_target_layer(&self, target_id: &str) -> String {
        // In a real implementation, this would look up the target's actual layer
        match self.layer {
            Layer::L4 => "L3",
            Layer::L3 => "L2", 
            Layer::L2 => "L1",
            Layer::L1 => "L1",
        }.to_string()
    }
}

#[async_trait]
impl NeuronInterface for ManagedNeuron {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn layer(&self) -> Layer {
        self.layer
    }
    
    #[instrument(skip(self, signal), fields(neuron_id = %self.id, layer = %self.layer, signal_id = %signal.id))]
    async fn process_signal(&self, signal: &NeuronSignal) -> Result<String> {
        let perf_timer = std::time::Instant::now();
        let span = neuron_span(&self.id, self.layer.as_str(), "process_signal");
        let _enter = span.enter();
        
        info!(
            target: "neuron.signal",
            signal_type = ?signal.propagation_type,
            from_neuron = %signal.from_neuron,
            strength = signal.payload.activation.strength,
            "Processing signal"
        );
        
        // Handle backward propagation signals
        if signal.propagation_type == PropagationType::Backward {
            self.process_backward_signal(signal).await?;
            let duration = perf_timer.elapsed();
            log_performance!(
                "neuron_backward_propagation",
                duration,
                true,
                "neuron_id" => &self.id,
                "layer" => self.layer.as_str()
            );
            return Ok("Backward propagation processed".to_string());
        }
        
        // Check circuit breaker first
        if !self.circuit_breaker.allow_request().await {
            warn!(
                target: "neuron.circuit_breaker",
                neuron_id = %self.id,
                "Circuit breaker open - rejecting signal"
            );
            if let Some(metrics) = &self.metrics {
                metrics.record_error("circuit_breaker_open");
            }
            let duration = perf_timer.elapsed();
            log_performance!(
                "neuron_signal_processing",
                duration,
                false,
                "neuron_id" => &self.id,
                "layer" => self.layer.as_str(),
                "error" => "circuit_breaker_open"
            );
            return Err(Error::CircuitBreakerOpen { 
                service: format!("neuron-{}", self.id) 
            });
        }
        
        let start_time = std::time::Instant::now();
        
        // Update state and metrics
        *self.state.write().await = NeuronState::Processing;
        if let Some(metrics) = &self.metrics {
            metrics.record_neuron_processing_start();
        }
        
        // Get base prompt (potentially adjusted by learning)
        let base_prompt = if let Some(adjuster) = &self.prompt_adjuster {
            adjuster.read().await.get_current_prompt().to_string()
        } else {
            self.format_prompt(signal).await
        };
        
        // Format prompt with signal context
        let prompt = format!("{}\n\n{}", base_prompt, self.format_prompt(signal).await);
        let cache_key = format!("{}-{}", self.layer.as_str(), prompt.len());
        
        // Check cache first (for all layers, not just L2)
        // Cache key includes signal content hash for better hit rate
        let cache_key = format!("{}-{}-{:x}", 
            self.layer.as_str(), 
            signal.from_neuron,
            md5::compute(&prompt).0[..8].iter().map(|b| format!("{:02x}", b)).collect::<String>()
        );
        
        if let Some(cache) = &self.response_cache {
            if let Some(cached_response) = cache.get(&cache_key) {
                debug!(
                    target: "neuron.cache",
                    neuron_id = %self.id,
                    cache_key = %cache_key,
                    "Using cached response"
                );
                
                // Fast path for cached responses - minimal overhead
                let duration = start_time.elapsed();
                if let Some(metrics) = &self.metrics {
                    metrics.record_processing_time(&self.id, duration);
                    metrics.record_signal_processed();
                }
                
                *self.state.write().await = NeuronState::Running;
                self.performance_monitor.record("cache_hit", duration);
                log_performance!(
                    "neuron_signal_processing",
                    duration,
                    true,
                    "neuron_id" => &self.id,
                    "layer" => self.layer.as_str(),
                    "cache_hit" => true
                );
                return Ok(cached_response);
            }
        }
        
        debug!(
            target: "neuron.processing",
            neuron_id = %self.id,
            signal_id = %signal.signal_id,
            "Processing signal without cache"
        );
        
        // Process the signal, potentially using tools
        let mut full_response = String::new();
        let mut current_prompt = prompt.clone();
        let mut iterations = 0;
        
        loop {
            iterations += 1;
            if iterations > 5 {
                warn!(
                    target: "neuron.tool_loop",
                    neuron_id = %self.id,
                    iterations = iterations,
                    "Exceeded max tool iterations"
                );
                break;
            }
            
            // Send to Claude with timeout to prevent hanging
            let timeout_duration = std::time::Duration::from_secs(30);
            let response = match tokio::time::timeout(
                timeout_duration,
                self.claude.send_message(&current_prompt)
            ).await {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => {
                    // Update error stats
                    let mut stats = self.stats.write().await;
                    stats.errors_count += 1;
                    drop(stats);
                    
                    // Record error metrics
                    if let Some(metrics) = &self.metrics {
                        metrics.record_error(&e.to_string());
                        metrics.record_signal_failed();
                    }
                    
                    // Record failure with circuit breaker
                    self.circuit_breaker.record_failure().await;
                    
                    error!("Neuron {} failed to process signal: {}", self.id, e);
                    return Err(e);
                }
                Err(_) => {
                    // Timeout error
                    let timeout_err = Error::Generic("Claude API timeout".to_string());
                    
                    // Update error stats
                    let mut stats = self.stats.write().await;
                    stats.errors_count += 1;
                    drop(stats);
                    
                    // Record error metrics
                    if let Some(metrics) = &self.metrics {
                        metrics.record_error("timeout");
                        metrics.record_signal_failed();
                    }
                    
                    // Record failure with circuit breaker
                    self.circuit_breaker.record_failure().await;
                    
                    error!(
                        target: "neuron.timeout",
                        neuron_id = %self.id,
                        timeout_ms = timeout_duration.as_millis(),
                        "Request timed out"
                    );
                    let duration = perf_timer.elapsed();
                    log_performance!(
                        "neuron_signal_processing",
                        duration,
                        false,
                        "neuron_id" => &self.id,
                        "layer" => self.layer.as_str(),
                        "error" => "timeout"
                    );
                    return Err(timeout_err);
                }
            };
            
            // Early success recording for circuit breaker
            self.circuit_breaker.record_success().await;
            
            // Check if response contains tool requests
            if let Some(tool_line) = response.lines().find(|l| l.starts_with("TOOL:")) {
                // Parse tool request
                let parts: Vec<&str> = tool_line.strip_prefix("TOOL:").unwrap_or("").splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let tool_name = parts[0].trim();
                    let params_str = parts[1].trim();
                    
                    debug!("Neuron {} executing tool: {}", self.id, tool_name);
                    
                    // Parse JSON params
                    match serde_json::from_str::<Value>(params_str) {
                        Ok(params) => {
                            // Execute tool
                            match self.tool_registry.execute(tool_name, params).await {
                                Ok(result) => {
                                    // Add tool result to response
                                    full_response.push_str(&response);
                                    full_response.push_str("\n\nTOOL_RESULT:\n");
                                    full_response.push_str(&serde_json::to_string_pretty(&result).unwrap_or_default());
                                    full_response.push_str("\n\n");
                                    
                                    // Continue with tool result in context
                                    current_prompt = format!(
                                        "{}\n\nTOOL_RESULT:\n{}\n\nContinue processing the signal with this information.",
                                        current_prompt,
                                        serde_json::to_string_pretty(&result).unwrap_or_default()
                                    );
                                    continue;
                                }
                                Err(e) => {
                                    warn!("Tool execution failed: {}", e);
                                    full_response.push_str(&response);
                                    full_response.push_str(&format!("\n\nTOOL_ERROR: {}\n", e));
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse tool params: {}", e);
                            full_response.push_str(&response);
                            break;
                        }
                    }
                } else {
                    full_response.push_str(&response);
                    break;
                }
            } else {
                // No tool request, we're done
                full_response.push_str(&response);
                break;
            }
        }
        
        // Update stats after all iterations
        let mut stats = self.stats.write().await;
        stats.signals_processed += 1;
        stats.last_signal = Some(Utc::now());
        drop(stats);
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            let duration = start_time.elapsed();
            metrics.record_processing_time(&self.id, duration);
            metrics.record_latency(self.layer.as_str(), duration);
            
            // Record token usage if available
            if let Some(usage) = self.claude.last_token_usage() {
                metrics.record_token_usage(usage.prompt_tokens, usage.completion_tokens);
            }
        }
        
        // Record success with circuit breaker
        self.circuit_breaker.record_success().await;
        
        // Cache the final response for L2 neurons
        if let Some(cache) = &self.response_cache {
            cache.put(cache_key, full_response.clone());
        }
        
        // Store memory of this interaction
        if let Some(memory_store) = &self.memory_store {
            // Store the task
            let task_memory = MemoryBuilder::new(self.id.clone(), self.layer.as_str().to_string())
                .with_type(MemoryType::Task)
                .with_content(signal.payload.activation.content.clone())
                .with_metadata(serde_json::json!({
                    "signal_id": signal.signal_id.to_string(),
                    "from_neuron": signal.from_neuron,
                    "iterations": iterations,
                }))
                .with_importance(0.7)
                .build();
                
            if let Err(e) = memory_store.store(task_memory).await {
                warn!("Failed to store task memory: {}", e);
            }
            
            // Store the result
            let result_memory = MemoryBuilder::new(self.id.clone(), self.layer.as_str().to_string())
                .with_type(MemoryType::Result)
                .with_content(full_response.clone())
                .with_metadata(serde_json::json!({
                    "signal_id": signal.signal_id.to_string(),
                    "tools_used": iterations > 1,
                }))
                .with_importance(0.6)
                .build();
                
            if let Err(e) = memory_store.store(result_memory).await {
                warn!("Failed to store result memory: {}", e);
            }
        }
        
        // Return to running state
        *self.state.write().await = NeuronState::Running;
        if let Some(metrics) = &self.metrics {
            metrics.record_neuron_processing_end();
            metrics.record_signal_processed();
        }
        
        // Record performance
        let final_duration = perf_timer.elapsed();
        self.performance_monitor.record("process_signal", final_duration);
        self.performance_monitor.record(&format!("layer_{}", self.layer.as_str()), final_duration);
        
        info!(
            target: "neuron.signal",
            neuron_id = %self.id,
            duration_ms = final_duration.as_millis(),
            response_length = full_response.len(),
            "Signal processed successfully"
        );
        
        log_performance!(
            "neuron_signal_processing",
            final_duration,
            true,
            "neuron_id" => &self.id,
            "layer" => self.layer.as_str(),
            "response_length" => full_response.len(),
            "tool_iterations" => iterations
        );
        
        Ok(full_response)
    }
    
    async fn health(&self) -> Result<NeuronHealth> {
        let state = *self.state.read().await;
        let stats = self.stats.read().await;
        
        let uptime_seconds = stats.started_at
            .map(|start| (Utc::now() - start).num_seconds() as u64)
            .unwrap_or(0);
            
        Ok(NeuronHealth {
            state,
            last_signal: stats.last_signal,
            signals_processed: stats.signals_processed,
            errors_count: stats.errors_count,
            uptime_seconds,
        })
    }
    
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down neuron {}", self.id);
        *self.state.write().await = NeuronState::Stopped;
        Ok(())
    }
}

/// Registry for managing multiple neurons
pub struct NeuronRegistry {
    neurons: Arc<DashMap<String, Arc<ManagedNeuron>>>,
    metrics: Option<Arc<crate::metrics::Metrics>>,
    parallel_executor: crate::performance::ParallelExecutor,
}

impl NeuronRegistry {
    /// Create a new neuron registry
    pub fn new() -> Self {
        Self {
            neurons: Arc::new(DashMap::new()),
            metrics: None,
            parallel_executor: crate::performance::ParallelExecutor::new(10), // 10 concurrent operations
        }
    }
    
    /// Set metrics collector
    pub fn set_metrics(&mut self, metrics: Arc<crate::metrics::Metrics>) {
        self.metrics = Some(metrics);
    }
    
    /// Register a neuron
    pub async fn register(&self, mut neuron: ManagedNeuron) -> Result<()> {
        let id = neuron.id.clone();
        
        // Set metrics if available
        if let Some(metrics) = &self.metrics {
            neuron.set_metrics(metrics.clone());
        }
        
        neuron.start().await?;
        self.neurons.insert(id.clone(), Arc::new(neuron));
        info!("Registered neuron: {}", id);
        Ok(())
    }
    
    /// Get a neuron by ID
    pub fn get(&self, id: &str) -> Option<Arc<ManagedNeuron>> {
        self.neurons.get(id).map(|entry| entry.clone())
    }
    
    /// Get all neurons
    pub fn all(&self) -> Vec<Arc<ManagedNeuron>> {
        self.neurons.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get neurons by layer
    pub fn by_layer(&self, layer: Layer) -> Vec<Arc<ManagedNeuron>> {
        self.neurons.iter()
            .filter(|entry| entry.value().layer == layer)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Remove a neuron
    pub async fn remove(&self, id: &str) -> Result<()> {
        if let Some((_, neuron)) = self.neurons.remove(id) {
            neuron.shutdown().await?;
            info!("Removed neuron: {}", id);
        }
        Ok(())
    }
    
    /// Shutdown all neurons
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all neurons");
        let neurons: Vec<_> = self.all();
        
        for neuron in neurons {
            if let Err(e) = neuron.shutdown().await {
                warn!("Error shutting down neuron {}: {}", neuron.id(), e);
            }
        }
        
        self.neurons.clear();
        Ok(())
    }
    
    /// Health check all neurons
    pub async fn health_check(&self) -> HashMap<String, NeuronHealth> {
        let mut health_map = HashMap::new();
        
        for neuron in self.all() {
            match neuron.health().await {
                Ok(health) => {
                    health_map.insert(neuron.id().to_string(), health);
                }
                Err(e) => {
                    warn!("Failed to get health for neuron {}: {}", neuron.id(), e);
                }
            }
        }
        
        health_map
    }
    
    /// List all neurons with their info
    pub async fn list_all(&self) -> Vec<crate::server::NeuronInfo> {
        let mut infos = Vec::new();
        
        for neuron in self.all() {
            let state = *neuron.state.read().await;
            let health = neuron.health().await.ok();
            
            infos.push(crate::server::NeuronInfo {
                id: neuron.id.clone(),
                layer: neuron.layer.as_str().to_string(),
                state: format!("{:?}", state),
                is_healthy: health.map(|h| h.errors_count == 0).unwrap_or(false),
            });
        }
        
        infos
    }
    
    /// Get specific neuron info
    pub async fn get_info(&self, id: &str) -> Option<crate::server::NeuronInfo> {
        if let Some(neuron) = self.get(id) {
            let state = *neuron.state.read().await;
            let health = neuron.health().await.ok();
            
            Some(crate::server::NeuronInfo {
                id: neuron.id.clone(),
                layer: neuron.layer.as_str().to_string(),
                state: format!("{:?}", state),
                is_healthy: health.map(|h| h.errors_count == 0).unwrap_or(false),
            })
        } else {
            None
        }
    }
}