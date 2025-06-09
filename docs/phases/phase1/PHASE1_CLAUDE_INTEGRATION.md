# Phase 1: Claude Integration Strategy

## Overview

For Phase 1 MVP, we need to integrate with Claude Opus 4 to power our neurons. This document outlines the integration approach and technical considerations.

## Integration Options

### Option 1: Claude API (Recommended for MVP)
**Pros**:
- Direct API access
- Programmatic control
- No CLI dependencies
- Easier testing/mocking

**Cons**:
- Requires API key management
- Usage-based pricing
- Rate limits

**Implementation**:
```rust
// Use anthropic-sdk-rust or similar
pub struct ClaudeAPIClient {
    api_key: String,
    model: String, // "claude-3-opus-20240229"
}

impl ClaudeNeuron {
    pub async fn process_via_api(&self, prompt: String) -> Result<String> {
        // Direct API call
    }
}
```

### Option 2: Claude CLI Wrapper (Original Design)
**Pros**:
- Matches original architecture
- Uses existing Claude installation
- No API key in code

**Cons**:
- Requires Claude desktop app
- Complex process management
- Harder to scale

**Implementation**:
```rust
// Original approach from draft
pub struct ClaudeNeuron {
    process: Child, // claude CLI process
}
```

## MVP Integration Plan

### Phase 1a: Mock Implementation (Week 1)
Create a mock Claude that simulates responses for testing:

```rust
pub trait ClaudeInterface: Send + Sync {
    async fn send_message(&self, message: &str) -> Result<String>;
}

pub struct MockClaude {
    layer: String,
    responses: HashMap<String, String>,
}

impl ClaudeInterface for MockClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        // Return canned responses based on layer
        match self.layer.as_str() {
            "L4" => Ok("Strategic analysis: Break down into 3 sub-tasks...".into()),
            "L3" => Ok("Design approach: Implement using pattern X...".into()),
            "L2" => Ok("Implementation: Here's the code...".into()),
            _ => Ok("Acknowledged".into()),
        }
    }
}
```

### Phase 1b: API Integration (Week 2-3)
Implement real Claude API client:

```rust
pub struct ClaudeAPIClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeInterface for ClaudeAPIClient {
    async fn send_message(&self, message: &str) -> Result<String> {
        let request = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": message
            }],
            "max_tokens": 4096,
            "temperature": 0.7,
            "system": self.get_system_prompt()
        });
        
        // API call implementation
    }
}
```

### System Prompts by Layer

#### L4 - Strategic Layer
```
You are a strategic planning AI neuron in a hierarchical neural network.
Your role is to receive high-level objectives and break them down into strategic initiatives.
Output format: List of 2-3 strategic directives for L3 neurons.
Focus on WHAT needs to be achieved, not HOW.
```

#### L3 - Design Layer  
```
You are a system design AI neuron in a hierarchical neural network.
Your role is to receive strategic directives and create architectural designs.
Output format: Technical design specifications for L2 neurons.
Focus on system architecture and component interaction.
```

#### L2 - Implementation Layer
```
You are an implementation AI neuron in a hierarchical neural network.
Your role is to receive design specifications and implement solutions.
Output format: Actual code, configurations, or detailed procedures.
Focus on concrete implementation details.
```

## Configuration

### Environment Variables
```bash
# For API integration
ANTHROPIC_API_KEY=sk-ant-...
CLAUDE_MODEL=claude-3-opus-20240229

# For mock mode
USE_MOCK_CLAUDE=true
```

### Neuron Configuration Enhancement
```yaml
neurons:
  - id: "neuron-1"
    layer: "L4"
    claude_config:
      mode: "api"  # or "cli" or "mock"
      model: "claude-3-opus-20240229"
      temperature: 0.7
      max_tokens: 4096
      system_prompt_file: "prompts/L4_strategic.txt"
```

## Testing Strategy

### 1. Unit Tests with Mocks
```rust
#[tokio::test]
async fn test_neuron_processing() {
    let mock = MockClaude::new("L4");
    let neuron = ClaudeNeuron::new("test-1", "L4", Box::new(mock));
    
    let signal = create_test_signal();
    let response = neuron.process_signal(&signal).await.unwrap();
    
    assert!(response.contains("Strategic analysis"));
}
```

### 2. Integration Tests
```rust
#[tokio::test]
#[ignore] // Run with --ignored flag when API key available
async fn test_real_claude_integration() {
    let client = ClaudeAPIClient::from_env().unwrap();
    let response = client.send_message("Hello").await.unwrap();
    assert!(!response.is_empty());
}
```

### 3. Cost Management
- Implement token counting before API calls
- Add spending limits per neuron
- Log all API usage for monitoring
- Use caching for repeated queries

## Migration Path

### Week 1: Mock Development
- Build with `MockClaude`
- Focus on neuron orchestration
- No external dependencies

### Week 2-3: API Integration
- Add `ClaudeAPIClient`
- Keep mock for testing
- Add configuration switching

### Future: CLI Integration (Phase 2)
- Research Claude CLI subscription options
- Implement `ClaudeCLIClient`
- Support hybrid deployment

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Invalid response format")]
    InvalidResponse,
    
    #[error("Connection timeout")]
    Timeout,
}
```

## Monitoring & Observability

### Metrics to Track
- API call latency per layer
- Token usage per neuron
- Error rates by type
- Cost per signal processed

### Logging
```rust
tracing::info!(
    neuron_id = %self.id,
    layer = %self.layer,
    tokens_used = %response.usage.total_tokens,
    latency_ms = %elapsed.as_millis(),
    "Processed signal via Claude API"
);
```

## Security Considerations

1. **API Key Management**
   - Never commit API keys
   - Use environment variables
   - Rotate keys regularly

2. **Prompt Injection**
   - Sanitize user inputs
   - Validate signal content
   - Monitor for anomalies

3. **Rate Limiting**
   - Implement local rate limits
   - Queue management
   - Graceful degradation

## Cost Estimation

### MVP Usage (Single Server, 7 Neurons)
- Average tokens per signal: ~1000
- Signals per minute: ~10
- Daily token usage: ~14.4M tokens
- Estimated daily cost: ~$200-300
- Monthly cost: ~$6,000-9,000

### Optimization Strategies
1. Cache frequent queries
2. Compress prompts
3. Use smaller models for L1/L2
4. Batch similar requests
5. Implement local filtering

## Next Steps

1. **Immediate** (Week 1):
   - Implement `MockClaude`
   - Build core neuron system
   - Test orchestration

2. **Short-term** (Week 2-3):
   - Integrate Anthropic API
   - Add cost monitoring
   - Performance testing

3. **Medium-term** (Phase 2):
   - Research CLI options
   - Implement hybrid mode
   - Optimize costs