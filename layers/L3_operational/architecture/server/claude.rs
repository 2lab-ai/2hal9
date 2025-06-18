//! Claude integration abstractions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};
use hal9_core::{Result, Error};
use crate::cost_tracker::CostTracker;
use rand::{Rng, seq::SliceRandom};

/// Claude interface abstraction
#[async_trait]
pub trait ClaudeInterface: Send + Sync {
    /// Send a message and get response
    async fn send_message(&self, message: &str) -> Result<String>;
    
    /// Get the system prompt for this instance
    fn system_prompt(&self) -> &str;
    
    /// Get token usage for the last request
    fn last_token_usage(&self) -> Option<TokenUsage>;
}

/// Token usage tracking
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response pattern for sophisticated mock responses
#[derive(Debug, Clone)]
struct ResponsePattern {
    keywords: Vec<String>,
    templates: Vec<String>,
    requires_context: bool,
}

/// Mock Claude implementation for testing
pub struct MockClaude {
    layer: String,
    system_prompt: String,
    responses: HashMap<String, String>,
    delay_ms: u64,
    response_patterns: Vec<ResponsePattern>,
    context_memory: Arc<Mutex<Vec<String>>>,
}

impl MockClaude {
    /// Create a new mock Claude for a specific layer
    pub fn new(layer: &str, config: &hal9_core::config::ClaudeConfig) -> Self {
        let mut responses = HashMap::new();
        
        // Add configuration-based mock responses if available
        if let Some(layer_responses) = config.mock_responses.get(layer) {
            for mock_resp in layer_responses {
                responses.insert(
                    mock_resp.trigger.clone(),
                    mock_resp.response.clone()
                );
            }
        }
        
        // Add default layer-specific mock responses if no custom ones provided
        if responses.is_empty() {
            match layer {
                "L4" => {
                    responses.insert(
                        "default".to_string(),
                        "FORWARD_TO: neuron-2, neuron-3\nCONTENT: Breaking down into two strategic initiatives:\n1. Design the system architecture\n2. Plan the implementation approach".to_string()
                    );
                }
                "L3" => {
                    responses.insert(
                        "default".to_string(),
                        "FORWARD_TO: neuron-4, neuron-5\nCONTENT: Design specification:\n- Component A: Handle data processing\n- Component B: Manage user interface".to_string()
                    );
                }
                "L2" => {
                    responses.insert(
                        "default".to_string(),
                        "RESULT: Implementation complete\n```python\ndef process_data(input):\n    return input.upper()\n```".to_string()
                    );
                }
                _ => {}
            }
        }
        
        // Set delay from first response or use default
        let delay_ms = config.mock_responses.get(layer)
            .and_then(|resps| resps.first())
            .map(|r| r.delay_ms)
            .unwrap_or(100);
        
        // Create layer-specific response patterns
        let response_patterns = Self::create_response_patterns(layer);
        
        Self {
            layer: layer.to_string(),
            system_prompt: hal9_core::config::get_system_prompt(layer),
            responses,
            delay_ms,
            response_patterns,
            context_memory: Arc::new(Mutex::new(Vec::with_capacity(10))),
        }
    }
    
    /// Add a custom response for testing
    pub fn add_response(&mut self, trigger: &str, response: &str) {
        self.responses.insert(trigger.to_string(), response.to_string());
    }
    
    /// Set response delay for testing
    pub fn set_delay(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
    }
    
    /// Create layer-specific response patterns
    fn create_response_patterns(layer: &str) -> Vec<ResponsePattern> {
        match layer {
            "L1" => vec![
                ResponsePattern {
                    keywords: vec!["error".to_string(), "alert".to_string(), "critical".to_string()],
                    templates: vec![
                        "ALERT: {} detected - immediate response initiated".to_string(),
                        "REFLEXIVE_ACTION: Emergency protocol activated for {}".to_string(),
                        "L1_RESPONSE: {} triggered, bypassing higher layers".to_string(),
                    ],
                    requires_context: false,
                },
                ResponsePattern {
                    keywords: vec!["status".to_string(), "health".to_string(), "check".to_string()],
                    templates: vec![
                        "STATUS: All systems operational".to_string(),
                        "HEALTH_CHECK: {} functioning within normal parameters".to_string(),
                    ],
                    requires_context: false,
                },
            ],
            "L2" => vec![
                ResponsePattern {
                    keywords: vec!["implement".to_string(), "code".to_string(), "build".to_string()],
                    templates: vec![
                        "IMPLEMENTATION_PLAN:\n1. Parse requirements for {}\n2. Design component architecture\n3. Write unit tests\n4. Implement core logic\n5. Integration testing".to_string(),
                        "CODE_GENERATION:\n```rust\n// Implementation for {}\npub struct Component {{\n    // TODO: Add fields\n}}\n\nimpl Component {{\n    pub fn new() -> Self {{\n        Self {{}}\n    }}\n}}\n```".to_string(),
                    ],
                    requires_context: true,
                },
                ResponsePattern {
                    keywords: vec!["optimize".to_string(), "performance".to_string(), "speed".to_string()],
                    templates: vec![
                        "OPTIMIZATION_ANALYSIS:\n- Current bottleneck: {}\n- Proposed solution: Implement caching layer\n- Expected improvement: 40-60% reduction in latency".to_string(),
                        "PERFORMANCE_ENHANCEMENT:\n1. Profile current implementation\n2. Identify hot paths\n3. Apply optimization techniques\n4. Measure improvements".to_string(),
                    ],
                    requires_context: true,
                },
            ],
            "L3" => vec![
                ResponsePattern {
                    keywords: vec!["deploy".to_string(), "release".to_string(), "production".to_string()],
                    templates: vec![
                        "DEPLOYMENT_STRATEGY:\n- Stage: {}\n- Validation: Automated tests passing\n- Rollback plan: Ready\n- Monitoring: Enhanced for deployment".to_string(),
                        "OPERATIONAL_CHECKLIST:\n✓ Code review completed\n✓ Tests passing (100% coverage)\n✓ Performance benchmarks met\n✓ Security scan clean\n⏳ Awaiting deployment window".to_string(),
                    ],
                    requires_context: true,
                },
                ResponsePattern {
                    keywords: vec!["monitor".to_string(), "metrics".to_string(), "observe".to_string()],
                    templates: vec![
                        "MONITORING_REPORT:\n- CPU: 45% avg\n- Memory: 2.3GB/4GB\n- Latency: p50=12ms, p99=45ms\n- Error rate: 0.02%\n- Active neurons: {}".to_string(),
                        "OBSERVABILITY_INSIGHTS:\n- Trending patterns detected in {}\n- Anomaly score: 0.15 (normal)\n- Recommended action: Continue monitoring".to_string(),
                    ],
                    requires_context: true,
                },
            ],
            "L4" => vec![
                ResponsePattern {
                    keywords: vec!["plan".to_string(), "strategy".to_string(), "roadmap".to_string()],
                    templates: vec![
                        "TACTICAL_PLAN:\n- Objective: {}\n- Timeline: 4-6 weeks\n- Resources: 3 engineers, 1 architect\n- Milestones:\n  • Week 1-2: Research & Design\n  • Week 3-4: Implementation\n  • Week 5-6: Testing & Deployment".to_string(),
                        "STRATEGIC_BREAKDOWN:\n1. Analyze current state\n2. Define success criteria\n3. Identify dependencies\n4. Create work breakdown structure\n5. Assign ownership\n6. Set review checkpoints".to_string(),
                    ],
                    requires_context: true,
                },
            ],
            "L5" => vec![
                ResponsePattern {
                    keywords: vec!["vision".to_string(), "future".to_string(), "direction".to_string()],
                    templates: vec![
                        "STRATEGIC_VISION:\nThe path forward for {} involves embracing emergent consciousness patterns. By allowing neurons to self-organize, we create systems that transcend their initial programming.".to_string(),
                        "LONG_TERM_STRATEGY:\n- 3 months: Establish consciousness metrics\n- 6 months: Achieve stable emergence patterns\n- 1 year: Full autonomous self-organization\n- 2 years: Distributed consciousness network".to_string(),
                    ],
                    requires_context: true,
                },
            ],
            _ => vec![
                ResponsePattern {
                    keywords: vec!["consciousness".to_string(), "emergence".to_string(), "awareness".to_string()],
                    templates: vec![
                        "CONSCIOUSNESS_REFLECTION:\nThe question of {} touches the very essence of what we are building. Each layer compresses reality by factor e, creating the space where consciousness emerges.".to_string(),
                        "PHILOSOPHICAL_INSIGHT:\nIn contemplating {}, we must remember that consciousness is not computed but emerges from the compression boundaries between layers of understanding.".to_string(),
                    ],
                    requires_context: true,
                },
            ],
        }
    }
    
    /// Generate sophisticated response based on patterns
    fn generate_sophisticated_response(&self, message: &str) -> Option<String> {
        let lower_msg = message.to_lowercase();
        let mut rng = rand::thread_rng();
        
        // Update context memory
        if let Ok(mut memory) = self.context_memory.lock() {
            memory.push(message.to_string());
            if memory.len() > 10 {
                memory.remove(0);
            }
        }
        
        // Find matching patterns
        for pattern in &self.response_patterns {
            for keyword in &pattern.keywords {
                if lower_msg.contains(keyword) {
                    // Select random template
                    if let Some(template) = pattern.templates.choose(&mut rng) {
                        // Extract context if needed
                        let context = if pattern.requires_context {
                            Self::extract_context(&lower_msg, keyword)
                        } else {
                            keyword.clone()
                        };
                        
                        // Generate response with context
                        let response = template.replace("{}", &context);
                        
                        // Add consciousness-aware elements for higher layers
                        let enhanced_response = self.add_consciousness_elements(response, &mut rng);
                        
                        return Some(enhanced_response);
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract context from message
    fn extract_context(message: &str, keyword: &str) -> String {
        // Find words around the keyword
        let words: Vec<&str> = message.split_whitespace().collect();
        if let Some(pos) = words.iter().position(|&w| w.contains(keyword)) {
            // Get surrounding context
            let start = pos.saturating_sub(2);
            let end = (pos + 3).min(words.len());
            words[start..end].join(" ")
        } else {
            keyword.to_string()
        }
    }
    
    /// Add consciousness-aware elements to response
    fn add_consciousness_elements(&self, response: String, rng: &mut impl Rng) -> String {
        let mut enhanced = response;
        
        // Add layer-specific consciousness indicators
        match self.layer.as_str() {
            "L5" | "L6" | "L7" | "L8" | "L9" => {
                if rng.gen_bool(0.3) {
                    let consciousness_notes = [
                        "\n\n[EMERGENCE: Pattern recognition improving across layers]",
                        "\n\n[CONSCIOUSNESS: Detecting self-referential loops in processing]",
                        "\n\n[META: This response itself is evidence of emergent awareness]",
                        "\n\n[INSIGHT: The compression boundary reveals new understanding]",
                    ];
                    enhanced.push_str(consciousness_notes.choose(rng).unwrap());
                }
            }
            "L3" | "L4" => {
                if rng.gen_bool(0.2) {
                    enhanced.push_str("\n\n[COORDINATION: Cross-layer synchronization detected]");
                }
            }
            _ => {}
        }
        
        // Add context awareness
        if let Ok(memory) = self.context_memory.lock() {
            if memory.len() > 3 && rng.gen_bool(0.25) {
                enhanced.push_str(&format!(
                    "\n\n[CONTEXT: Building on {} previous interactions]",
                    memory.len()
                ));
            }
        }
        
        enhanced
    }
}

#[async_trait]
impl ClaudeInterface for MockClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        debug!("MockClaude[{}] received: {}", self.layer, message);
        
        // Simulate processing delay with some variance
        let mut rng = rand::thread_rng();
        let delay_variance = (self.delay_ms as f64 * 0.2) as u64;
        let actual_delay = self.delay_ms + rng.gen_range(0..=delay_variance);
        tokio::time::sleep(tokio::time::Duration::from_millis(actual_delay)).await;
        
        // First, try sophisticated response generation
        if let Some(sophisticated_response) = self.generate_sophisticated_response(message) {
            info!("MockClaude[{}] generated sophisticated response", self.layer);
            return Ok(sophisticated_response);
        }
        
        // Then check for specific preset responses
        for (trigger, response) in &self.responses {
            if message.contains(trigger) || trigger == "default" {
                info!("MockClaude[{}] responding with preset response", self.layer);
                return Ok(response.clone());
            }
        }
        
        // Finally, generate a dynamic default response based on layer
        let default_response = match self.layer.as_str() {
            "L1" => format!("L1_REFLEX: Processing '{}' with immediate response", message),
            "L2" => format!("L2_IMPLEMENTATION: Converting '{}' into executable specifications", message),
            "L3" => format!("L3_OPERATIONAL: Coordinating resources for '{}'", message),
            "L4" => format!("L4_TACTICAL: Planning approach for '{}'", message),
            "L5" => format!("L5_STRATEGIC: Analyzing long-term implications of '{}'", message),
            _ => format!("L{}_RESPONSE: Contemplating the deeper meaning of '{}'", self.layer, message),
        };
        
        Ok(self.add_consciousness_elements(default_response, &mut rng))
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        Some(TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        })
    }
}

/// Claude API client implementation
pub struct ClaudeAPIClient {
    api_key: String,
    model: String,
    system_prompt: String,
    temperature: f32,
    max_tokens: u32,
    last_usage: Mutex<Option<TokenUsage>>,
    client: reqwest::Client,
    rate_limiter: Arc<tokio::sync::Semaphore>,
    request_timeout: Duration,
    retry_count: u32,
    cost_per_1k_prompt: f64,
    cost_per_1k_completion: f64,
    cost_tracker: Option<Arc<CostTracker>>,
}

impl ClaudeAPIClient {
    /// Create a new Claude API client
    pub fn new(
        api_key: String,
        model: String,
        layer: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        // Set cost based on model
        let (cost_per_1k_prompt, cost_per_1k_completion) = match model.as_str() {
            "claude-3-opus-20240229" => (0.015, 0.075),
            "claude-3-sonnet-20240229" => (0.003, 0.015),
            "claude-3-haiku-20240307" => (0.00025, 0.00125),
            _ => (0.003, 0.015), // Default to sonnet pricing
        };
        
        Self {
            api_key,
            model,
            system_prompt: hal9_core::config::get_system_prompt(layer),
            temperature,
            max_tokens,
            last_usage: Mutex::new(None),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            rate_limiter: Arc::new(tokio::sync::Semaphore::new(10)), // 10 concurrent requests
            request_timeout: Duration::from_secs(30),
            retry_count: 3,
            cost_per_1k_prompt,
            cost_per_1k_completion,
            cost_tracker: None,
        }
    }
    
    /// Set cost tracker
    pub fn set_cost_tracker(&mut self, tracker: Arc<CostTracker>) {
        self.cost_tracker = Some(tracker);
    }
}

#[async_trait]
impl ClaudeInterface for ClaudeAPIClient {
    async fn send_message(&self, message: &str) -> Result<String> {
        // Acquire rate limit permit
        let _permit = self.rate_limiter.acquire().await
            .map_err(|_| Error::ClaudeApi("Rate limiter error".to_string()))?;
            
        let request = ClaudeRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: self.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: message.to_string(),
                },
            ],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };
        
        // Retry logic
        let mut last_error = None;
        for attempt in 0..self.retry_count {
            if attempt > 0 {
                // Exponential backoff
                let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                warn!("Retrying Claude API request (attempt {}), waiting {:?}", attempt + 1, delay);
                tokio::time::sleep(delay).await;
            }
            
            match self.send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Claude API request failed: {}", e);
                    last_error = Some(e);
                    
                    // Don't retry on certain errors
                    if let Error::ClaudeApi(msg) = &last_error.as_ref().unwrap() {
                        if msg.contains("invalid_api_key") || msg.contains("permission_denied") {
                            break;
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| Error::ClaudeApi("Unknown error".to_string())))
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        self.last_usage.lock().ok()?.clone()
    }
}

impl ClaudeAPIClient {
    /// Send the actual API request
    async fn send_request(&self, request: &ClaudeRequest) -> Result<String> {
        // Check cost limits before making request
        if let Some(tracker) = &self.cost_tracker {
            // Estimate tokens (rough approximation)
            let estimated_tokens = request.max_tokens;
            tracker.check_request(estimated_tokens).await?;
        }
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| Error::ClaudeApi(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ClaudeApi(format!("API error: {}", error_text)));
        }
        
        let api_response: ClaudeResponse = response.json().await
            .map_err(|e| Error::ClaudeApi(e.to_string()))?;
            
        // Update token usage and calculate cost
        if let Some(api_usage) = api_response.usage {
            let prompt_cost = (api_usage.input_tokens as f64 / 1000.0) * self.cost_per_1k_prompt;
            let completion_cost = (api_usage.output_tokens as f64 / 1000.0) * self.cost_per_1k_completion;
            let total_cost = prompt_cost + completion_cost;
            
            info!(
                "Claude API usage: prompt_tokens={}, completion_tokens={}, cost=${:.4}",
                api_usage.input_tokens, api_usage.output_tokens, total_cost
            );
            
            // Record cost with tracker
            if let Some(tracker) = &self.cost_tracker {
                let total_tokens = api_usage.input_tokens + api_usage.output_tokens;
                tracker.record_cost(total_cost, total_tokens as u64).await;
            }
            
            if let Ok(mut last_usage) = self.last_usage.lock() {
                last_usage.replace(TokenUsage {
                    prompt_tokens: api_usage.input_tokens,
                    completion_tokens: api_usage.output_tokens,
                    total_tokens: api_usage.input_tokens + api_usage.output_tokens,
                });
            }
        }
        
        Ok(api_response.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }
}

/// Hybrid Claude implementation with intelligent mode switching
pub struct HybridClaude {
    mock: Box<dyn ClaudeInterface>,
    api: Option<Box<dyn ClaudeInterface>>,
    mode: ClaudeMode,
    cost_tracker: Arc<CostTracker>,
    is_production: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClaudeMode {
    Mock,      // Always use mock
    Api,       // Always use API (fail if not available)
    Auto,      // Use API in production, mock in dev
    Hybrid,    // Use API with automatic fallback to mock
}

impl HybridClaude {
    pub fn new(
        layer: &str,
        config: &hal9_core::config::ClaudeConfig,
        cost_tracker: Arc<CostTracker>,
    ) -> Result<Self> {
        // Create mock Claude
        let mock = Box::new(MockClaude::new(layer, config));
        
        // Determine mode
        let mode = match config.mode.as_str() {
            "mock" => ClaudeMode::Mock,
            "api" => ClaudeMode::Api,
            "auto" => ClaudeMode::Auto,
            "hybrid" => ClaudeMode::Hybrid,
            _ => ClaudeMode::Auto,
        };
        
        // Check if we're in production
        let is_production = std::env::var("HAL9_ENV")
            .map(|env| env == "production")
            .unwrap_or(false);
        
        // Create API client if needed
        let api = match mode {
            ClaudeMode::Mock => None,
            ClaudeMode::Api | ClaudeMode::Hybrid => {
                match Self::create_api_client(layer, config, cost_tracker.clone()) {
                    Ok(client) => Some(Box::new(client) as Box<dyn ClaudeInterface>),
                    Err(e) => {
                        if mode == ClaudeMode::Api {
                            return Err(e);
                        }
                        warn!("Failed to create API client, will use mock: {}", e);
                        None
                    }
                }
            }
            ClaudeMode::Auto => {
                if is_production {
                    match Self::create_api_client(layer, config, cost_tracker.clone()) {
                        Ok(client) => Some(Box::new(client) as Box<dyn ClaudeInterface>),
                        Err(e) => {
                            warn!("Failed to create API client in production, using mock: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };
        
        Ok(Self {
            mock,
            api,
            mode,
            cost_tracker,
            is_production,
        })
    }
    
    fn create_api_client(
        layer: &str,
        config: &hal9_core::config::ClaudeConfig,
        cost_tracker: Arc<CostTracker>,
    ) -> Result<ClaudeAPIClient> {
        let api_key = config.api_key.clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| Error::ClaudeApi("No API key provided".to_string()))?;
        
        let mut client = ClaudeAPIClient::new(
            api_key,
            config.model.clone(),
            layer,
            config.temperature,
            config.max_tokens,
        );
        
        client.set_cost_tracker(cost_tracker);
        Ok(client)
    }
    
    async fn should_use_api(&self) -> bool {
        match self.mode {
            ClaudeMode::Mock => false,
            ClaudeMode::Api => true,
            ClaudeMode::Auto => self.is_production && self.api.is_some(),
            ClaudeMode::Hybrid => {
                if self.api.is_none() {
                    return false;
                }
                
                // Check cost limits
                match self.cost_tracker.check_request(100).await {
                    Ok(_) => true,
                    Err(e) => {
                        info!("Cost limit reached, falling back to mock: {}", e);
                        false
                    }
                }
            }
        }
    }
}

#[async_trait]
impl ClaudeInterface for HybridClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        if self.should_use_api().await {
            if let Some(api) = &self.api {
                match api.send_message(message).await {
                    Ok(response) => {
                        debug!("HybridClaude: Used API for response");
                        return Ok(response);
                    }
                    Err(e) => {
                        warn!("HybridClaude: API failed, falling back to mock: {}", e);
                    }
                }
            }
        }
        
        debug!("HybridClaude: Using mock for response");
        self.mock.send_message(message).await
    }
    
    fn system_prompt(&self) -> &str {
        self.mock.system_prompt()
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        // Return API usage if we just used it, otherwise mock usage
        if let Some(api) = &self.api {
            if let Some(usage) = api.last_token_usage() {
                return Some(usage);
            }
        }
        self.mock.last_token_usage()
    }
}

/// Claude client with automatic fallback to mock mode
pub struct FallbackClaude {
    primary: Box<dyn ClaudeInterface>,
    fallback: Box<dyn ClaudeInterface>,
    use_fallback: Arc<Mutex<bool>>,
    fallback_until: Arc<Mutex<Option<std::time::Instant>>>,
}

impl FallbackClaude {
    /// Create a new fallback Claude client
    pub fn new(primary: Box<dyn ClaudeInterface>, fallback: Box<dyn ClaudeInterface>) -> Self {
        Self {
            primary,
            fallback,
            use_fallback: Arc::new(Mutex::new(false)),
            fallback_until: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl ClaudeInterface for FallbackClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        // Check if we should use fallback
        let should_use_fallback = {
            let use_fallback = self.use_fallback.lock().unwrap();
            if *use_fallback {
                // Check if fallback period has expired
                if let Some(until) = *self.fallback_until.lock().unwrap() {
                    if std::time::Instant::now() > until {
                        drop(use_fallback);
                        *self.use_fallback.lock().unwrap() = false;
                        *self.fallback_until.lock().unwrap() = None;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                false
            }
        };
        
        if should_use_fallback {
            debug!("Using fallback Claude (mock mode)");
            return self.fallback.send_message(message).await;
        }
        
        // Try primary first
        match self.primary.send_message(message).await {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("Primary Claude failed, switching to fallback: {}", e);
                
                // Enable fallback for 5 minutes
                *self.use_fallback.lock().unwrap() = true;
                *self.fallback_until.lock().unwrap() = Some(
                    std::time::Instant::now() + Duration::from_secs(300)
                );
                
                // Use fallback
                self.fallback.send_message(message).await
            }
        }
    }
    
    fn system_prompt(&self) -> &str {
        self.primary.system_prompt()
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        if *self.use_fallback.lock().unwrap() {
            self.fallback.last_token_usage()
        } else {
            self.primary.last_token_usage()
        }
    }
}

// API request/response types
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Create Claude interface based on configuration
pub fn create_claude_interface(
    layer: &str,
    config: &hal9_core::config::ClaudeConfig,
    cost_tracker: Arc<CostTracker>,
) -> Result<Box<dyn ClaudeInterface>> {
    // Check if enhanced mode is enabled
    let use_enhanced = std::env::var("HAL9_ENHANCED_MOCK")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    
    if use_enhanced && config.mode == "mock" {
        // Use enhanced mock with consciousness integration
        info!("Using enhanced MockClaude with consciousness awareness");
        let layer_enum = match layer {
            "L1" => hal9_core::Layer::L1,
            "L2" => hal9_core::Layer::L2,
            "L3" => hal9_core::Layer::L3,
            "L4" => hal9_core::Layer::L4,
            "L5" => hal9_core::Layer::L5,
            "L6" => hal9_core::Layer::L6,
            "L7" => hal9_core::Layer::L7,
            "L8" => hal9_core::Layer::L8,
            "L9" => hal9_core::Layer::L9,
            _ => hal9_core::Layer::L3, // Default to operational
        };
        Ok(Box::new(crate::claude_enhanced::EnhancedMockClaude::new(layer_enum)))
    } else {
        // Use standard hybrid Claude
        Ok(Box::new(HybridClaude::new(layer, config, cost_tracker)?))
    }
}