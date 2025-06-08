# HAL9 Browser Automation MCP Tools - Technical Specifications

## Overview

This document provides detailed technical specifications for implementing browser automation as MCP tools within the HAL9-Operator ecosystem, including TON blockchain integration for payment processing.

## Browser Automation MCP Tools

### Core Browser Tools

#### 1. BrowserSessionTool
```rust
pub struct BrowserSessionTool {
    browser_pool: Arc<BrowserPool>,
    session_store: Arc<SessionStore>,
}

impl Tool for BrowserSessionTool {
    fn name(&self) -> &str {
        "browser_session"
    }
    
    fn description(&self) -> &str {
        "Create or retrieve a browser session for web automation"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "get", "destroy"],
                    "description": "Session action to perform"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID for get/destroy actions"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "headless": {
                            "type": "boolean",
                            "default": true
                        },
                        "viewport": {
                            "type": "object",
                            "properties": {
                                "width": {"type": "integer", "default": 1920},
                                "height": {"type": "integer", "default": 1080}
                            }
                        },
                        "user_agent": {
                            "type": "string",
                            "description": "Custom user agent string"
                        }
                    }
                }
            },
            "required": ["action"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let action = params["action"].as_str().unwrap();
        
        match action {
            "create" => {
                let options = BrowserOptions::from_json(&params["options"])?;
                let session = self.browser_pool.create_session(options).await?;
                
                Ok(ToolResult::success(json!({
                    "session_id": session.id,
                    "status": "created"
                })))
            },
            "get" => {
                let session_id = params["session_id"].as_str()
                    .ok_or("Missing session_id")?;
                
                let session = self.session_store.get(session_id).await?;
                
                Ok(ToolResult::success(json!({
                    "session_id": session.id,
                    "status": session.status,
                    "current_url": session.current_url
                })))
            },
            "destroy" => {
                let session_id = params["session_id"].as_str()
                    .ok_or("Missing session_id")?;
                
                self.browser_pool.destroy_session(session_id).await?;
                
                Ok(ToolResult::success(json!({
                    "status": "destroyed"
                })))
            },
            _ => Err("Invalid action".into())
        }
    }
}
```

#### 2. BrowserNavigateTool
```rust
pub struct BrowserNavigateTool {
    browser_pool: Arc<BrowserPool>,
}

impl Tool for BrowserNavigateTool {
    fn name(&self) -> &str {
        "browser_navigate"
    }
    
    fn description(&self) -> &str {
        "Navigate to a URL in the browser session"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Browser session ID"
                },
                "url": {
                    "type": "string",
                    "description": "URL to navigate to"
                },
                "wait_until": {
                    "type": "string",
                    "enum": ["load", "domcontentloaded", "networkidle"],
                    "default": "networkidle",
                    "description": "When to consider navigation complete"
                },
                "timeout": {
                    "type": "integer",
                    "default": 30000,
                    "description": "Navigation timeout in milliseconds"
                }
            },
            "required": ["session_id", "url"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let session_id = params["session_id"].as_str().unwrap();
        let url = params["url"].as_str().unwrap();
        let wait_until = params.get("wait_until")
            .and_then(|v| v.as_str())
            .unwrap_or("networkidle");
        
        let browser = self.browser_pool.get_browser(session_id).await?;
        let page = browser.get_current_page().await?;
        
        // Navigate with retry logic
        let navigation_result = retry_with_backoff(3, || async {
            page.goto(url)
                .wait_until(wait_until.parse()?)
                .send()
                .await
        }).await?;
        
        // Capture page info
        let title = page.title().await?;
        let current_url = page.url().await?;
        
        Ok(ToolResult::success(json!({
            "url": current_url,
            "title": title,
            "status": "navigated"
        })))
    }
}
```

#### 3. BrowserInteractTool
```rust
pub struct BrowserInteractTool {
    browser_pool: Arc<BrowserPool>,
    interaction_validator: Arc<InteractionValidator>,
}

impl Tool for BrowserInteractTool {
    fn name(&self) -> &str {
        "browser_interact"
    }
    
    fn description(&self) -> &str {
        "Interact with elements on the page (click, type, select)"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Browser session ID"
                },
                "action": {
                    "type": "string",
                    "enum": ["click", "type", "select", "hover", "scroll"],
                    "description": "Interaction type"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector or XPath for the element"
                },
                "value": {
                    "type": "string",
                    "description": "Value for type/select actions"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "delay": {
                            "type": "integer",
                            "description": "Typing delay in ms"
                        },
                        "clear_before": {
                            "type": "boolean",
                            "description": "Clear field before typing"
                        }
                    }
                }
            },
            "required": ["session_id", "action", "selector"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        // Validate interaction for security
        self.interaction_validator.validate(&params).await?;
        
        let session_id = params["session_id"].as_str().unwrap();
        let action = params["action"].as_str().unwrap();
        let selector = params["selector"].as_str().unwrap();
        
        let browser = self.browser_pool.get_browser(session_id).await?;
        let page = browser.get_current_page().await?;
        
        // Wait for element to be ready
        page.wait_for_selector(selector)
            .visible(true)
            .send()
            .await?;
        
        match action {
            "click" => {
                page.click(selector).await?;
                Ok(ToolResult::success(json!({
                    "action": "clicked",
                    "selector": selector
                })))
            },
            "type" => {
                let text = params["value"].as_str()
                    .ok_or("Missing value for type action")?;
                
                let clear_before = params.get("options")
                    .and_then(|o| o.get("clear_before"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                if clear_before {
                    page.click(selector).await?;
                    page.keyboard().press_key("Control+a").await?;
                    page.keyboard().press_key("Delete").await?;
                }
                
                page.type_str(selector, text).await?;
                
                Ok(ToolResult::success(json!({
                    "action": "typed",
                    "selector": selector,
                    "length": text.len()
                })))
            },
            _ => Err(format!("Unsupported action: {}", action).into())
        }
    }
}
```

#### 4. BrowserExtractTool
```rust
pub struct BrowserExtractTool {
    browser_pool: Arc<BrowserPool>,
    content_sanitizer: Arc<ContentSanitizer>,
}

impl Tool for BrowserExtractTool {
    fn name(&self) -> &str {
        "browser_extract"
    }
    
    fn description(&self) -> &str {
        "Extract data from the current page"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Browser session ID"
                },
                "extraction_type": {
                    "type": "string",
                    "enum": ["text", "attribute", "screenshot", "table", "list"],
                    "description": "Type of data to extract"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector for element(s)"
                },
                "attribute": {
                    "type": "string",
                    "description": "Attribute name for attribute extraction"
                },
                "format": {
                    "type": "string",
                    "enum": ["json", "csv", "text"],
                    "default": "json",
                    "description": "Output format"
                }
            },
            "required": ["session_id", "extraction_type"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let session_id = params["session_id"].as_str().unwrap();
        let extraction_type = params["extraction_type"].as_str().unwrap();
        
        let browser = self.browser_pool.get_browser(session_id).await?;
        let page = browser.get_current_page().await?;
        
        match extraction_type {
            "text" => {
                let selector = params["selector"].as_str()
                    .unwrap_or("body");
                
                let text = page.text_content(selector).await?;
                let sanitized = self.content_sanitizer.sanitize(&text)?;
                
                Ok(ToolResult::success(json!({
                    "type": "text",
                    "content": sanitized,
                    "length": sanitized.len()
                })))
            },
            "screenshot" => {
                let selector = params.get("selector").and_then(|s| s.as_str());
                
                let screenshot = if let Some(sel) = selector {
                    page.screenshot()
                        .element(page.query_selector(sel).await?)
                        .await?
                } else {
                    page.screenshot().await?
                };
                
                Ok(ToolResult::success(json!({
                    "type": "screenshot",
                    "data": base64::encode(screenshot),
                    "format": "png"
                })))
            },
            "table" => {
                let selector = params["selector"].as_str()
                    .unwrap_or("table");
                
                let table_data = page.eval(&format!(r#"
                    Array.from(document.querySelectorAll('{}'))
                        .map(table => {{
                            const headers = Array.from(table.querySelectorAll('th'))
                                .map(th => th.textContent.trim());
                            const rows = Array.from(table.querySelectorAll('tr'))
                                .slice(1)
                                .map(tr => Array.from(tr.querySelectorAll('td'))
                                    .map(td => td.textContent.trim()));
                            return {{ headers, rows }};
                        }})
                "#, selector)).await?;
                
                Ok(ToolResult::success(json!({
                    "type": "table",
                    "data": table_data
                })))
            },
            _ => Err("Unsupported extraction type".into())
        }
    }
}
```

### E-commerce Specific Tools

#### 5. AmazonSearchTool
```rust
pub struct AmazonSearchTool {
    browser_automation: Arc<BrowserAutomationService>,
    product_analyzer: Arc<ProductAnalyzer>,
}

impl Tool for AmazonSearchTool {
    fn name(&self) -> &str {
        "amazon_search"
    }
    
    fn description(&self) -> &str {
        "Search for products on Amazon with filters"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "filters": {
                    "type": "object",
                    "properties": {
                        "min_price": {"type": "number"},
                        "max_price": {"type": "number"},
                        "prime_only": {"type": "boolean"},
                        "min_rating": {"type": "number"},
                        "category": {"type": "string"}
                    }
                },
                "max_results": {
                    "type": "integer",
                    "default": 10,
                    "maximum": 50
                }
            },
            "required": ["query"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let query = params["query"].as_str().unwrap();
        let filters = params.get("filters");
        let max_results = params.get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;
        
        // Get browser session from context
        let session_id = self.get_context("browser_session_id")?;
        
        // Navigate to Amazon
        self.browser_automation
            .navigate(session_id, "https://www.amazon.com")
            .await?;
        
        // Perform search
        self.browser_automation
            .search_amazon(session_id, query)
            .await?;
        
        // Apply filters
        if let Some(filters) = filters {
            self.apply_amazon_filters(session_id, filters).await?;
        }
        
        // Extract product data
        let products = self.extract_amazon_products(session_id, max_results).await?;
        
        // Analyze products
        let analyzed = self.product_analyzer
            .analyze_products(&products)
            .await?;
        
        Ok(ToolResult::success(json!({
            "query": query,
            "results_count": analyzed.len(),
            "products": analyzed
        })))
    }
}
```

#### 6. AmazonCheckoutTool
```rust
pub struct AmazonCheckoutTool {
    browser_automation: Arc<BrowserAutomationService>,
    payment_processor: Arc<PaymentProcessor>,
    order_manager: Arc<OrderManager>,
}

impl Tool for AmazonCheckoutTool {
    fn name(&self) -> &str {
        "amazon_checkout"
    }
    
    fn description(&self) -> &str {
        "Complete Amazon checkout process"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "product_id": {
                    "type": "string",
                    "description": "Amazon product ASIN"
                },
                "quantity": {
                    "type": "integer",
                    "default": 1
                },
                "shipping_address": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "address1": {"type": "string"},
                        "address2": {"type": "string"},
                        "city": {"type": "string"},
                        "state": {"type": "string"},
                        "zip": {"type": "string"},
                        "country": {"type": "string", "default": "US"}
                    },
                    "required": ["name", "address1", "city", "state", "zip"]
                },
                "payment_method": {
                    "type": "string",
                    "enum": ["saved_card", "usdt_balance"],
                    "description": "Payment method to use"
                }
            },
            "required": ["product_id", "shipping_address", "payment_method"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let product_id = params["product_id"].as_str().unwrap();
        let quantity = params.get("quantity")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        
        // Verify user has sufficient balance
        let user_id = self.get_context("user_id")?;
        let estimated_total = self.estimate_total(&params).await?;
        
        if params["payment_method"] == "usdt_balance" {
            self.payment_processor
                .verify_balance(user_id, estimated_total)
                .await?;
        }
        
        // Create order record
        let order = self.order_manager
            .create_pending_order(user_id, &params)
            .await?;
        
        // Execute checkout flow
        let session_id = self.get_context("browser_session_id")?;
        
        // Add to cart
        self.add_to_cart(session_id, product_id, quantity).await?;
        
        // Proceed to checkout
        self.proceed_to_checkout(session_id).await?;
        
        // Fill shipping info
        self.fill_shipping_info(session_id, &params["shipping_address"]).await?;
        
        // Complete payment
        let payment_result = self.complete_payment(
            session_id,
            &params["payment_method"],
            estimated_total
        ).await?;
        
        // Update order status
        self.order_manager
            .update_order_status(&order.id, OrderStatus::Completed)
            .await?;
        
        Ok(ToolResult::success(json!({
            "order_id": order.id,
            "status": "completed",
            "total": payment_result.total,
            "estimated_delivery": payment_result.estimated_delivery,
            "tracking_number": payment_result.tracking_number
        })))
    }
}
```

### TON Blockchain Integration Tools

#### 7. TONWalletTool
```rust
pub struct TONWalletTool {
    ton_service: Arc<TONWalletService>,
    wallet_manager: Arc<WalletManager>,
}

impl Tool for TONWalletTool {
    fn name(&self) -> &str {
        "ton_wallet"
    }
    
    fn description(&self) -> &str {
        "Manage TON wallet operations"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create_deposit_address", "check_balance", "get_transactions"],
                    "description": "Wallet action to perform"
                },
                "currency": {
                    "type": "string",
                    "enum": ["USDT", "TON"],
                    "default": "USDT"
                },
                "limit": {
                    "type": "integer",
                    "default": 10,
                    "description": "Transaction history limit"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let action = params["action"].as_str().unwrap();
        let user_id = self.get_context("user_id")?;
        
        match action {
            "create_deposit_address" => {
                let address = self.ton_service
                    .create_deposit_address(user_id)
                    .await?;
                
                // Generate QR code
                let qr_code = self.generate_deposit_qr(&address).await?;
                
                Ok(ToolResult::success(json!({
                    "address": address.to_string(),
                    "network": "TON",
                    "currency": params.get("currency").unwrap_or(&json!("USDT")),
                    "qr_code": qr_code,
                    "memo": address.memo
                })))
            },
            "check_balance" => {
                let currency = params.get("currency")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USDT");
                
                let balance = self.wallet_manager
                    .get_balance(user_id, currency)
                    .await?;
                
                Ok(ToolResult::success(json!({
                    "currency": currency,
                    "balance": balance.available,
                    "pending": balance.pending,
                    "total": balance.total
                })))
            },
            "get_transactions" => {
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                
                let transactions = self.wallet_manager
                    .get_transaction_history(user_id, limit)
                    .await?;
                
                Ok(ToolResult::success(json!({
                    "transactions": transactions,
                    "count": transactions.len()
                })))
            },
            _ => Err("Invalid action".into())
        }
    }
}
```

#### 8. USDTDepositMonitorTool
```rust
pub struct USDTDepositMonitorTool {
    blockchain_monitor: Arc<BlockchainMonitor>,
    notification_service: Arc<NotificationService>,
}

impl Tool for USDTDepositMonitorTool {
    fn name(&self) -> &str {
        "usdt_deposit_monitor"
    }
    
    fn description(&self) -> &str {
        "Monitor USDT deposits on TON blockchain"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start_monitoring", "stop_monitoring", "check_status"],
                    "description": "Monitoring action"
                },
                "address": {
                    "type": "string",
                    "description": "TON address to monitor"
                },
                "callback_url": {
                    "type": "string",
                    "description": "Webhook URL for deposit notifications"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let action = params["action"].as_str().unwrap();
        
        match action {
            "start_monitoring" => {
                let address = params["address"].as_str()
                    .ok_or("Missing address")?;
                let callback_url = params.get("callback_url")
                    .and_then(|v| v.as_str());
                
                let monitor_id = self.blockchain_monitor
                    .start_monitoring(address, callback_url)
                    .await?;
                
                Ok(ToolResult::success(json!({
                    "monitor_id": monitor_id,
                    "status": "monitoring",
                    "address": address
                })))
            },
            _ => Err("Not implemented".into())
        }
    }
}
```

## Security Implementation

### 1. Credential Encryption
```rust
pub struct SecureCredentialStore {
    kms_client: KMSClient,
    vault: Arc<RwLock<HashMap<String, EncryptedCredential>>>,
}

impl SecureCredentialStore {
    pub async fn store_credential(
        &self,
        user_id: UserId,
        platform: Platform,
        credential: Credential,
    ) -> Result<()> {
        // Generate unique encryption key per user
        let key = self.kms_client
            .generate_data_key(format!("user/{}/platform/{}", user_id, platform))
            .await?;
        
        // Encrypt credential
        let encrypted = encrypt_aes_256_gcm(&credential, &key)?;
        
        // Store encrypted credential
        let mut vault = self.vault.write().await;
        vault.insert(
            format!("{}/{}", user_id, platform),
            EncryptedCredential {
                ciphertext: encrypted,
                key_id: key.id,
                created_at: Utc::now(),
            }
        );
        
        Ok(())
    }
}
```

### 2. Transaction Authorization
```rust
pub struct TransactionAuthorizer {
    risk_analyzer: Arc<RiskAnalyzer>,
    mfa_service: Arc<MFAService>,
}

impl TransactionAuthorizer {
    pub async fn authorize_transaction(
        &self,
        user_id: UserId,
        transaction: &Transaction,
    ) -> Result<AuthorizationResult> {
        // Analyze risk
        let risk_score = self.risk_analyzer
            .analyze_transaction(user_id, transaction)
            .await?;
        
        // Determine authorization requirements
        let auth_required = match risk_score {
            score if score < 0.3 => AuthLevel::None,
            score if score < 0.7 => AuthLevel::Confirmation,
            _ => AuthLevel::MFA,
        };
        
        // Request authorization
        match auth_required {
            AuthLevel::None => Ok(AuthorizationResult::Approved),
            AuthLevel::Confirmation => {
                self.request_user_confirmation(user_id, transaction).await
            },
            AuthLevel::MFA => {
                self.mfa_service.verify_transaction(user_id, transaction).await
            }
        }
    }
}
```

## Performance Optimization

### 1. Browser Pool Management
```rust
pub struct BrowserPool {
    pool: Arc<RwLock<Vec<BrowserInstance>>>,
    config: BrowserPoolConfig,
}

impl BrowserPool {
    pub async fn optimize_pool(&self) {
        let mut pool = self.pool.write().await;
        
        // Remove idle browsers
        pool.retain(|browser| {
            browser.last_used.elapsed() < self.config.idle_timeout
        });
        
        // Pre-warm browsers
        while pool.len() < self.config.min_browsers {
            let browser = BrowserInstance::new().await.unwrap();
            pool.push(browser);
        }
        
        // Scale based on demand
        if pool.iter().all(|b| b.is_busy()) && pool.len() < self.config.max_browsers {
            for _ in 0..self.config.scale_increment {
                let browser = BrowserInstance::new().await.unwrap();
                pool.push(browser);
            }
        }
    }
}
```

### 2. Caching Strategy
```rust
pub struct ProductCache {
    redis: RedisClient,
    ttl: Duration,
}

impl ProductCache {
    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: &str,
        fetch_fn: F,
    ) -> Result<Product>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Product>>,
    {
        // Try cache first
        if let Some(cached) = self.redis.get(key).await? {
            return Ok(cached);
        }
        
        // Fetch and cache
        let product = fetch_fn().await?;
        self.redis.set_ex(key, &product, self.ttl).await?;
        
        Ok(product)
    }
}
```

## Monitoring & Analytics

### 1. Transaction Metrics
```rust
pub struct TransactionMetrics {
    prometheus: PrometheusClient,
}

impl TransactionMetrics {
    pub fn record_transaction(&self, transaction: &Transaction) {
        self.prometheus.increment_counter(
            "hal9_transactions_total",
            &[
                ("platform", &transaction.platform.to_string()),
                ("status", &transaction.status.to_string()),
            ]
        );
        
        self.prometheus.observe_histogram(
            "hal9_transaction_amount_usd",
            transaction.amount_usd,
            &[("platform", &transaction.platform.to_string())]
        );
        
        self.prometheus.observe_histogram(
            "hal9_transaction_duration_seconds",
            transaction.duration.as_secs_f64(),
            &[("platform", &transaction.platform.to_string())]
        );
    }
}
```

## Testing Framework

### 1. Browser Automation Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_amazon_search_and_extract() {
        let test_app = TestApp::new().await;
        
        // Create browser session
        let session = test_app.create_browser_session().await;
        
        // Search for product
        let search_result = test_app.call_tool(
            "amazon_search",
            json!({
                "query": "laptop",
                "filters": {
                    "max_price": 1000,
                    "min_rating": 4.0
                }
            })
        ).await.unwrap();
        
        assert!(search_result["products"].as_array().unwrap().len() > 0);
    }
    
    #[tokio::test]
    async fn test_ton_wallet_integration() {
        let test_app = TestApp::new().await;
        let user_id = UserId(12345);
        
        // Create deposit address
        let address_result = test_app.call_tool(
            "ton_wallet",
            json!({
                "action": "create_deposit_address",
                "currency": "USDT"
            })
        ).await.unwrap();
        
        assert!(address_result["address"].as_str().unwrap().starts_with("EQ"));
    }
}
```

This technical specification provides the foundation for implementing browser automation and blockchain integration within the HAL9-Operator ecosystem using MCP tools.