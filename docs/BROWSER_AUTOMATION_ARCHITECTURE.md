# HAL9 Browser Automation Architecture

## Overview

The HAL9 Browser Automation system enables neurons to interact with web applications autonomously, performing complex multi-step workflows through a secure, scalable browser control interface.

## Architecture Components

```
┌─────────────────────────────────────────────┐
│             HAL9 Neurons (L4/L3/L2)         │
│                                             │
│  ┌─────────────┐  ┌───────────────────┐   │
│  │ Task Planner│  │ Action Executor    │   │
│  │ (L4 Neuron) │  │ (L3/L2 Neurons)   │   │
│  └──────┬──────┘  └─────────┬─────────┘   │
└─────────┼───────────────────┼──────────────┘
          │                   │
          ▼                   ▼
┌─────────────────────────────────────────────┐
│            MCP Browser Tools                 │
│                                             │
│  ┌─────────┐ ┌──────────┐ ┌────────────┐  │
│  │Navigate │ │  Click   │ │   Type     │  │
│  └─────────┘ └──────────┘ └────────────┘  │
│  ┌─────────┐ ┌──────────┐ ┌────────────┐  │
│  │Extract  │ │Screenshot│ │  WaitFor   │  │
│  └─────────┘ └──────────┘ └────────────┘  │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│        Browser Controller Service            │
│                                             │
│  ┌──────────────┐  ┌───────────────────┐   │
│  │Context Pool  │  │Resource Manager   │   │
│  │Management    │  │(CPU/Memory/Time)  │   │
│  └──────────────┘  └───────────────────┘   │
│                                             │
│  ┌──────────────┐  ┌───────────────────┐   │
│  │Security      │  │Session Manager    │   │
│  │Sandbox       │  │(Cookies/Storage)  │   │
│  └──────────────┘  └───────────────────┘   │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│            Playwright Engine                 │
│                                             │
│  ┌─────────┐  ┌─────────┐  ┌───────────┐  │
│  │Chromium │  │ Firefox │  │  WebKit   │  │
│  └─────────┘  └─────────┘  └───────────┘  │
└─────────────────────────────────────────────┘
```

## Core Components

### 1. Browser Controller Service

The central service managing all browser automation operations.

```rust
pub struct BrowserController {
    // Connection pool for browser contexts
    context_pool: Arc<Mutex<ContextPool>>,
    
    // Resource management
    resource_manager: ResourceManager,
    
    // Security sandbox
    security_sandbox: SecuritySandbox,
    
    // Session management
    session_manager: SessionManager,
    
    // Metrics
    metrics: BrowserMetrics,
}

impl BrowserController {
    pub async fn execute_action(&self, action: BrowserAction) -> Result<ActionResult> {
        // Validate action against security policies
        self.security_sandbox.validate(&action)?;
        
        // Acquire browser context from pool
        let context = self.context_pool.acquire().await?;
        
        // Execute with resource limits
        let result = self.resource_manager
            .execute_with_limits(context, action)
            .await?;
        
        // Update metrics
        self.metrics.record_action(&action, &result);
        
        Ok(result)
    }
}
```

### 2. MCP Browser Tools

Individual tools exposed to neurons for browser interaction.

```rust
// Navigate Tool
pub struct NavigateTool;

impl McpTool for NavigateTool {
    async fn execute(&self, params: Value) -> Result<Value> {
        let url = params["url"].as_str()
            .ok_or("URL required")?;
        
        // Security check
        SecuritySandbox::validate_url(url)?;
        
        // Navigate
        let controller = get_browser_controller();
        controller.navigate(url).await?;
        
        Ok(json!({
            "status": "navigated",
            "url": url,
            "title": controller.get_title().await?
        }))
    }
}

// Click Tool
pub struct ClickTool;

impl McpTool for ClickTool {
    async fn execute(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or("Selector required")?;
        
        let controller = get_browser_controller();
        controller.click(selector).await?;
        
        Ok(json!({
            "status": "clicked",
            "selector": selector
        }))
    }
}

// Extract Tool
pub struct ExtractTool;

impl McpTool for ExtractTool {
    async fn execute(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or("Selector required")?;
        let extract_type = params["type"].as_str()
            .unwrap_or("text");
        
        let controller = get_browser_controller();
        let data = match extract_type {
            "text" => controller.extract_text(selector).await?,
            "attribute" => {
                let attr = params["attribute"].as_str()
                    .ok_or("Attribute required")?;
                controller.extract_attribute(selector, attr).await?
            },
            "html" => controller.extract_html(selector).await?,
            _ => return Err("Invalid extract type".into()),
        };
        
        Ok(json!({
            "status": "extracted",
            "data": data
        }))
    }
}
```

### 3. Security Sandbox

Ensures safe browser automation with strict security controls.

```rust
pub struct SecuritySandbox {
    // URL whitelist/blacklist
    url_policy: UrlPolicy,
    
    // Action limits
    rate_limiter: RateLimiter,
    
    // Credential vault
    credential_vault: CredentialVault,
    
    // Audit logger
    audit_logger: AuditLogger,
}

impl SecuritySandbox {
    pub fn validate(&self, action: &BrowserAction) -> Result<()> {
        // Check URL policy
        if let BrowserAction::Navigate { url, .. } = action {
            self.url_policy.check(url)?;
        }
        
        // Rate limiting
        self.rate_limiter.check_and_update()?;
        
        // Log action for audit
        self.audit_logger.log(action);
        
        Ok(())
    }
    
    pub fn inject_credentials(&self, site: &str) -> Option<Credentials> {
        // Securely retrieve credentials if authorized
        self.credential_vault.get_for_site(site)
    }
}
```

### 4. Resource Management

Prevents resource exhaustion and ensures fair usage.

```rust
pub struct ResourceManager {
    cpu_limiter: CpuLimiter,
    memory_limiter: MemoryLimiter,
    time_limiter: TimeLimiter,
}

impl ResourceManager {
    pub async fn execute_with_limits<F, T>(
        &self,
        context: BrowserContext,
        action: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // Set resource limits
        context.set_cpu_limit(self.cpu_limiter.get_limit());
        context.set_memory_limit(self.memory_limiter.get_limit());
        
        // Execute with timeout
        tokio::time::timeout(
            self.time_limiter.get_timeout(),
            action
        ).await?
    }
}
```

### 5. Context Pool Management

Efficient browser context reuse and isolation.

```rust
pub struct ContextPool {
    // Available contexts
    available: Vec<BrowserContext>,
    
    // In-use contexts
    in_use: HashMap<Uuid, (BrowserContext, Instant)>,
    
    // Configuration
    config: PoolConfig,
}

impl ContextPool {
    pub async fn acquire(&mut self) -> Result<PooledContext> {
        // Try to get available context
        if let Some(context) = self.available.pop() {
            let id = Uuid::new_v4();
            self.in_use.insert(id, (context, Instant::now()));
            return Ok(PooledContext::new(id, self));
        }
        
        // Create new context if under limit
        if self.in_use.len() < self.config.max_contexts {
            let context = self.create_context().await?;
            let id = Uuid::new_v4();
            self.in_use.insert(id, (context, Instant::now()));
            return Ok(PooledContext::new(id, self));
        }
        
        // Wait for available context
        self.wait_for_available().await
    }
    
    async fn create_context(&self) -> Result<BrowserContext> {
        let browser = Browser::new()?;
        let context = browser.new_context(
            BrowserContextOptions::default()
                .viewport(1920, 1080)
                .user_agent("HAL9 Browser Automation")
                .locale("en-US")
        ).await?;
        
        Ok(context)
    }
}
```

## Workflow Examples

### 1. E-commerce Automation

```yaml
Task: Purchase item from online store
L4 Planning:
  1. Navigate to product page
  2. Add to cart
  3. Proceed to checkout
  4. Fill shipping information
  5. Complete payment

L3 Execution:
  - Navigate to "https://store.example.com/product/123"
  - Click selector: ".add-to-cart"
  - Click selector: ".checkout-button"
  - Type selector: "#shipping-name" text: "John Doe"
  - Type selector: "#shipping-address" text: "123 Main St"
  - Click selector: ".complete-order"
  
L2 Verification:
  - Extract selector: ".order-confirmation"
  - Screenshot full page
```

### 2. Data Extraction

```yaml
Task: Extract pricing data from competitor sites
L4 Planning:
  1. Identify target sites
  2. Navigate to pricing pages
  3. Extract structured data
  4. Compare and analyze

L3 Execution:
  - For each site in sites:
    - Navigate to site.pricing_url
    - Wait for selector: ".pricing-table"
    - Extract all: ".price-item"
    - Extract attributes: "data-price", "data-plan"
    
L2 Processing:
  - Parse extracted data
  - Normalize pricing formats
  - Generate comparison report
```

### 3. Form Automation

```yaml
Task: Submit support tickets across platforms
L4 Planning:
  1. Identify ticket systems
  2. Authenticate if needed
  3. Fill ticket forms
  4. Track ticket IDs

L3 Execution:
  - Navigate to login page
  - Type credentials (from vault)
  - Click login button
  - Navigate to ticket form
  - Fill all required fields
  - Submit and extract ticket ID
```

## Security Considerations

### 1. URL Whitelisting

```yaml
allowed_domains:
  - "*.example.com"
  - "api.trusted-service.com"
  - "localhost:*"

blocked_patterns:
  - "*/admin/*"
  - "*/api/internal/*"
  - "*/.git/*"
```

### 2. Credential Management

```rust
pub struct CredentialVault {
    encrypted_store: EncryptedStore,
    access_policy: AccessPolicy,
}

impl CredentialVault {
    pub fn store_credentials(&mut self, site: &str, creds: Credentials) -> Result<()> {
        // Encrypt credentials
        let encrypted = self.encrypted_store.encrypt(&creds)?;
        
        // Store with site mapping
        self.encrypted_store.store(site, encrypted)?;
        
        Ok(())
    }
    
    pub fn get_for_site(&self, site: &str) -> Option<Credentials> {
        // Check access policy
        if !self.access_policy.is_allowed(site) {
            return None;
        }
        
        // Decrypt and return
        self.encrypted_store.get(site)
            .and_then(|enc| self.encrypted_store.decrypt(enc).ok())
    }
}
```

### 3. Activity Auditing

```rust
pub struct AuditLogger {
    log_store: LogStore,
}

impl AuditLogger {
    pub fn log(&self, action: &BrowserAction) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user_id: current_user_id(),
            action: action.clone(),
            context: collect_context(),
        };
        
        self.log_store.append(entry);
    }
}
```

## Performance Optimization

### 1. Context Reuse

- Keep contexts warm for repeated tasks
- Clear cookies/storage between uses
- Periodic context refresh

### 2. Parallel Execution

- Multiple contexts for independent tasks
- Async action execution
- Result aggregation

### 3. Caching

- Page resource caching
- Selector result caching
- Screenshot deduplication

## Monitoring & Metrics

```rust
pub struct BrowserMetrics {
    action_counter: Counter,
    action_duration: Histogram,
    error_counter: Counter,
    resource_usage: Gauge,
}

impl BrowserMetrics {
    pub fn record_action(&self, action: &BrowserAction, result: &ActionResult) {
        self.action_counter.inc();
        self.action_duration.observe(result.duration.as_secs_f64());
        
        if let Err(_) = result.status {
            self.error_counter.inc();
        }
        
        self.resource_usage.set(current_resource_usage());
    }
}
```

## Error Handling

### 1. Retry Logic

```rust
pub async fn execute_with_retry<F, T>(
    action: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match action().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries && e.is_retriable() => {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 2. Graceful Degradation

- Fallback to simpler selectors
- Alternative action paths
- Partial result acceptance

## Future Enhancements

1. **Visual AI Integration**
   - Claude vision for CAPTCHA solving
   - Visual element detection
   - Screenshot-based navigation

2. **Mobile Browser Support**
   - Mobile viewport emulation
   - Touch gesture support
   - App automation via Appium

3. **Distributed Browsing**
   - Browser farm deployment
   - Geographic distribution
   - Load balancing

4. **Advanced Features**
   - Record and replay
   - Visual regression testing
   - Performance profiling

## Conclusion

The HAL9 Browser Automation system provides a secure, scalable, and intelligent way for AI neurons to interact with web applications. By combining Playwright's powerful browser control with HAL9's hierarchical intelligence, we enable complex automation workflows that can adapt and learn from experience.