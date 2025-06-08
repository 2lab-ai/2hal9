# HAL9-Operator Technical Implementation Guide

## Overview

This document provides detailed technical implementation guidance for the HAL9-Operator Telegram service, building on the existing HAL9 infrastructure and MCP protocol.

## Project Structure

```
hal9-operator/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── telegram/
│   │   ├── mod.rs           # Telegram module
│   │   ├── bot.rs           # Bot implementation
│   │   ├── webhook.rs       # Webhook handler
│   │   ├── commands.rs      # Command processing
│   │   └── stars.rs         # Telegram Stars integration
│   ├── user/
│   │   ├── mod.rs           # User module
│   │   ├── manager.rs       # User management
│   │   ├── storage.rs       # User data storage
│   │   └── subscription.rs  # Subscription handling
│   ├── neuron/
│   │   ├── mod.rs           # Neuron module
│   │   ├── pool.rs          # Neuron pool management
│   │   ├── allocator.rs     # Neuron allocation
│   │   └── injector.rs      # Context injection
│   ├── mcp/
│   │   ├── mod.rs           # MCP extensions
│   │   ├── telegram_tools.rs # Telegram-specific tools
│   │   ├── user_tools.rs    # User data tools
│   │   └── bridge.rs        # Telegram-MCP bridge
│   ├── scheduler/
│   │   ├── mod.rs           # Scheduler module
│   │   ├── notifications.rs # Notification scheduler
│   │   └── jobs.rs          # Background jobs
│   └── api/
│       ├── mod.rs           # API module
│       ├── health.rs        # Health checks
│       └── admin.rs         # Admin endpoints
```

## Core Components Implementation

### 1. Telegram Bot Module

```rust
// src/telegram/bot.rs
use teloxide::prelude::*;
use teloxide::types::{Update, Message, User};
use crate::user::UserManager;
use crate::neuron::NeuronPool;

pub struct HAL9Bot {
    bot: Bot,
    user_manager: Arc<UserManager>,
    neuron_pool: Arc<NeuronPool>,
}

impl HAL9Bot {
    pub async fn new(token: String) -> Result<Self> {
        let bot = Bot::new(token);
        let user_manager = Arc::new(UserManager::new().await?);
        let neuron_pool = Arc::new(NeuronPool::new().await?);
        
        Ok(Self {
            bot,
            user_manager,
            neuron_pool,
        })
    }
    
    pub async fn handle_update(&self, update: Update) -> Result<()> {
        match update {
            Update::Message(msg) => self.handle_message(msg).await?,
            Update::CallbackQuery(query) => self.handle_callback(query).await?,
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_message(&self, msg: Message) -> Result<()> {
        let user = msg.from().ok_or("No user in message")?;
        let chat_id = msg.chat.id;
        
        // Check if message is a command
        if let Some(text) = msg.text() {
            if text.starts_with('/') {
                return self.handle_command(user, chat_id, text).await;
            }
        }
        
        // Regular message - route to user's neuron
        self.route_to_neuron(user.id, chat_id, msg).await
    }
}
```

### 2. Command Handler

```rust
// src/telegram/commands.rs
use teloxide::types::{ChatId, UserId};

pub enum Command {
    Start,
    Register,
    Subscribe,
    Connect,
    Status,
    Help,
}

impl Command {
    pub fn parse(text: &str) -> Option<Self> {
        match text.split_whitespace().next()? {
            "/start" => Some(Command::Start),
            "/register" => Some(Command::Register),
            "/subscribe" => Some(Command::Subscribe),
            "/connect" => Some(Command::Connect),
            "/status" => Some(Command::Status),
            "/help" => Some(Command::Help),
            _ => None,
        }
    }
}

pub async fn handle_command(
    bot: &Bot,
    user_manager: &UserManager,
    user: &User,
    chat_id: ChatId,
    command: Command,
) -> Result<()> {
    match command {
        Command::Register => {
            handle_register(bot, user_manager, user, chat_id).await
        }
        Command::Subscribe => {
            handle_subscribe(bot, user_manager, user, chat_id).await
        }
        Command::Connect => {
            handle_connect(bot, user_manager, user, chat_id).await
        }
        // ... other commands
    }
}

async fn handle_register(
    bot: &Bot,
    user_manager: &UserManager,
    user: &User,
    chat_id: ChatId,
) -> Result<()> {
    // Check whitelist
    if !user_manager.is_whitelisted(user.id).await? {
        bot.send_message(
            chat_id,
            "Sorry, registration is currently limited to invited users."
        ).await?;
        return Ok(());
    }
    
    // Create user profile
    let profile = user_manager.create_user(user).await?;
    
    bot.send_message(
        chat_id,
        format!(
            "Welcome to HAL9! 🤖\n\n\
            You have been registered successfully.\n\
            You have {} free conversations remaining.\n\n\
            Use /subscribe to unlock unlimited conversations.",
            profile.remaining_trial_conversations
        )
    ).await?;
    
    Ok(())
}
```

### 3. Telegram Stars Integration

```rust
// src/telegram/stars.rs
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct StarsPayment {
    amount: u32,
    description: String,
}

impl StarsPayment {
    pub fn subscription() -> Self {
        Self {
            amount: 300,
            description: "HAL9 Premium Subscription".to_string(),
        }
    }
    
    pub fn create_payment_button(&self) -> InlineKeyboardMarkup {
        let button = InlineKeyboardButton::pay(
            "Subscribe with Stars ⭐",
        );
        
        InlineKeyboardMarkup::new(vec![vec![button]])
    }
}

pub async fn handle_subscription_payment(
    bot: &Bot,
    user_manager: &UserManager,
    user_id: UserId,
    transaction_id: String,
) -> Result<()> {
    // Verify payment with Telegram
    let payment_verified = verify_stars_payment(&transaction_id).await?;
    
    if payment_verified {
        // Activate subscription
        user_manager.activate_subscription(
            user_id,
            SubscriptionType::Premium,
            Duration::days(30),
        ).await?;
        
        // Notify user
        bot.send_message(
            ChatId(user_id.0 as i64),
            "🎉 Your subscription has been activated!\n\n\
            You now have unlimited access to your personal AI assistant.\n\
            Use /connect to start chatting!"
        ).await?;
    }
    
    Ok(())
}
```

### 4. Neuron Pool Management

```rust
// src/neuron/pool.rs
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct NeuronPool {
    neurons: RwLock<HashMap<String, Neuron>>,
    user_assignments: RwLock<HashMap<UserId, String>>,
    config: NeuronPoolConfig,
}

impl NeuronPool {
    pub async fn allocate_neuron(&self, user_id: UserId) -> Result<String> {
        // Check if user already has a neuron
        let assignments = self.user_assignments.read().await;
        if let Some(neuron_id) = assignments.get(&user_id) {
            return Ok(neuron_id.clone());
        }
        drop(assignments);
        
        // Find available neuron or create new one
        let neuron_id = self.find_or_create_neuron().await?;
        
        // Assign to user
        let mut assignments = self.user_assignments.write().await;
        assignments.insert(user_id, neuron_id.clone());
        
        // Configure neuron for user
        self.configure_neuron_for_user(&neuron_id, user_id).await?;
        
        Ok(neuron_id)
    }
    
    async fn find_or_create_neuron(&self) -> Result<String> {
        let mut neurons = self.neurons.write().await;
        
        // Try to find idle neuron
        for (id, neuron) in neurons.iter() {
            if neuron.is_idle() {
                return Ok(id.clone());
            }
        }
        
        // Create new neuron if under limit
        if neurons.len() < self.config.max_neurons {
            let neuron_id = Uuid::new_v4().to_string();
            let neuron = self.spawn_neuron(&neuron_id).await?;
            neurons.insert(neuron_id.clone(), neuron);
            Ok(neuron_id)
        } else {
            Err("Neuron pool at capacity".into())
        }
    }
    
    async fn spawn_neuron(&self, neuron_id: &str) -> Result<Neuron> {
        // Create new neuron with MCP server
        let mut neuron = Neuron::new(neuron_id, "telegram_operator")?;
        
        // Start MCP server
        neuron.start_mcp_server(self.config.mcp_port_range.next()).await?;
        
        Ok(neuron)
    }
}
```

### 5. MCP Tools for Telegram

```rust
// src/mcp/telegram_tools.rs
use twohal9_core::mcp::{Tool, ToolResult};
use serde_json::Value;

pub struct TelegramSendMessageTool {
    bot: Bot,
}

impl Tool for TelegramSendMessageTool {
    fn name(&self) -> &str {
        "telegram_send_message"
    }
    
    fn description(&self) -> &str {
        "Send a message to the connected Telegram user"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Message text to send"
                },
                "parse_mode": {
                    "type": "string",
                    "enum": ["Markdown", "HTML", "Plain"],
                    "default": "Markdown"
                },
                "reply_markup": {
                    "type": "object",
                    "description": "Optional keyboard markup"
                }
            },
            "required": ["text"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let text = params["text"].as_str()
            .ok_or("Missing text parameter")?;
        
        let parse_mode = params.get("parse_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("Markdown");
        
        // Get chat_id from neuron context
        let chat_id = self.get_context("chat_id")?;
        
        // Send message
        self.bot.send_message(ChatId(chat_id), text)
            .parse_mode(parse_mode.parse()?)
            .await?;
        
        Ok(ToolResult::success(json!({
            "status": "sent",
            "chat_id": chat_id
        })))
    }
}

pub struct TelegramEditMessageTool {
    bot: Bot,
}

impl Tool for TelegramEditMessageTool {
    fn name(&self) -> &str {
        "telegram_edit_message"
    }
    
    fn description(&self) -> &str {
        "Edit a previously sent message"
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let message_id = params["message_id"].as_i64()
            .ok_or("Missing message_id")?;
        let new_text = params["text"].as_str()
            .ok_or("Missing text")?;
        
        let chat_id = self.get_context("chat_id")?;
        
        self.bot.edit_message_text(
            ChatId(chat_id),
            MessageId(message_id as i32),
            new_text
        ).await?;
        
        Ok(ToolResult::success(json!({
            "status": "edited",
            "message_id": message_id
        })))
    }
}
```

### 6. User File System Tools

```rust
// src/mcp/user_tools.rs
use std::path::PathBuf;
use tokio::fs;

pub struct UserFileSystemTool {
    user_id: UserId,
    base_path: PathBuf,
}

impl Tool for UserFileSystemTool {
    fn name(&self) -> &str {
        "user_file_system"
    }
    
    fn description(&self) -> &str {
        "Manage user's personal file system"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read", "write", "list", "delete", "mkdir"]
                },
                "path": {
                    "type": "string",
                    "description": "Relative path within user's directory"
                },
                "content": {
                    "type": "string",
                    "description": "File content for write operations"
                }
            },
            "required": ["operation", "path"]
        })
    }
    
    async fn call(&self, params: Value) -> Result<ToolResult> {
        let operation = params["operation"].as_str()
            .ok_or("Missing operation")?;
        let relative_path = params["path"].as_str()
            .ok_or("Missing path")?;
        
        // Ensure path is within user directory
        let full_path = self.sanitize_path(relative_path)?;
        
        match operation {
            "read" => self.read_file(full_path).await,
            "write" => {
                let content = params["content"].as_str()
                    .ok_or("Missing content for write")?;
                self.write_file(full_path, content).await
            }
            "list" => self.list_directory(full_path).await,
            "delete" => self.delete_file(full_path).await,
            "mkdir" => self.create_directory(full_path).await,
            _ => Err("Unknown operation".into())
        }
    }
}

impl UserFileSystemTool {
    fn sanitize_path(&self, relative_path: &str) -> Result<PathBuf> {
        // Prevent directory traversal
        if relative_path.contains("..") {
            return Err("Invalid path: contains ..".into());
        }
        
        let path = self.base_path.join(relative_path);
        
        // Ensure path is within user directory
        if !path.starts_with(&self.base_path) {
            return Err("Invalid path: outside user directory".into());
        }
        
        Ok(path)
    }
    
    async fn write_file(&self, path: PathBuf, content: &str) -> Result<ToolResult> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&path, content).await?;
        
        Ok(ToolResult::success(json!({
            "status": "written",
            "path": path.to_string_lossy(),
            "size": content.len()
        })))
    }
}
```

### 7. Notification Scheduler

```rust
// src/scheduler/notifications.rs
use tokio_cron_scheduler::{Job, JobScheduler};
use chrono::{DateTime, Utc};

pub struct NotificationScheduler {
    scheduler: JobScheduler,
    neuron_pool: Arc<NeuronPool>,
}

impl NotificationScheduler {
    pub async fn new(neuron_pool: Arc<NeuronPool>) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(Self {
            scheduler,
            neuron_pool,
        })
    }
    
    pub async fn schedule_reminder(
        &self,
        user_id: UserId,
        schedule: ReminderSchedule,
    ) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        let pool = self.neuron_pool.clone();
        
        let job = match schedule {
            ReminderSchedule::Daily { hour, minute } => {
                Job::new_async(
                    format!("{} {} * * *", minute, hour).as_str(),
                    move |_uuid, _lock| {
                        let pool = pool.clone();
                        let user_id = user_id.clone();
                        Box::pin(async move {
                            Self::trigger_notification(pool, user_id, "daily").await;
                        })
                    }
                )?
            }
            ReminderSchedule::Weekly { day, hour, minute } => {
                Job::new_async(
                    format!("{} {} * * {}", minute, hour, day).as_str(),
                    move |_uuid, _lock| {
                        let pool = pool.clone();
                        let user_id = user_id.clone();
                        Box::pin(async move {
                            Self::trigger_notification(pool, user_id, "weekly").await;
                        })
                    }
                )?
            }
            ReminderSchedule::Once { datetime } => {
                // Schedule one-time job
                self.schedule_once(user_id, datetime).await?
            }
        };
        
        self.scheduler.add(job).await?;
        Ok(job_id)
    }
    
    async fn trigger_notification(
        pool: Arc<NeuronPool>,
        user_id: UserId,
        notification_type: &str,
    ) {
        // Get user's neuron
        let neuron_id = match pool.get_user_neuron(user_id).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get neuron for user {}: {}", user_id.0, e);
                return;
            }
        };
        
        // Send initiate signal to neuron
        let request = json!({
            "type": "notification",
            "notification_type": notification_type,
            "user_id": user_id.0,
            "timestamp": Utc::now().to_rfc3339()
        });
        
        if let Err(e) = pool.send_to_neuron(&neuron_id, request).await {
            error!("Failed to send notification: {}", e);
        }
    }
}
```

### 8. Telegram-MCP Bridge

```rust
// src/mcp/bridge.rs
use teloxide::types::{Message, Update};
use twohal9_core::mcp::{MCPMessage, ProcessTaskRequest};

pub struct TelegramMCPBridge {
    client: MCPClient,
}

impl TelegramMCPBridge {
    pub async fn process_message(
        &self,
        neuron_id: &str,
        message: Message,
        user_context: UserContext,
    ) -> Result<String> {
        // Convert Telegram message to MCP request
        let mcp_request = self.telegram_to_mcp(message, user_context)?;
        
        // Send to neuron
        let response = self.client.call_tool(
            neuron_id,
            "process_message",
            mcp_request
        ).await?;
        
        // Convert MCP response to Telegram format
        let telegram_response = self.mcp_to_telegram(response)?;
        
        Ok(telegram_response)
    }
    
    fn telegram_to_mcp(
        &self,
        message: Message,
        context: UserContext,
    ) -> Result<ProcessTaskRequest> {
        let content = message.text()
            .or_else(|| message.caption())
            .unwrap_or("");
        
        let mut metadata = HashMap::new();
        metadata.insert("telegram_message_id".to_string(), message.id.to_string());
        metadata.insert("telegram_chat_id".to_string(), message.chat.id.to_string());
        metadata.insert("user_id".to_string(), context.user_id.to_string());
        
        // Handle media
        if let Some(photo) = message.photo() {
            metadata.insert("media_type".to_string(), "photo".to_string());
            // Store photo file_id for later retrieval
        }
        
        Ok(ProcessTaskRequest {
            task_id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            context: TaskContext {
                layer_from: "telegram".to_string(),
                layer_to: "operator".to_string(),
                batch_id: None,
                metadata,
            },
        })
    }
    
    fn mcp_to_telegram(&self, response: MCPMessage) -> Result<String> {
        // Extract text content from MCP response
        match response {
            MCPMessage::Response { result, .. } => {
                if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                    Ok(content.to_string())
                } else {
                    Ok("I processed your request.".to_string())
                }
            }
            _ => Err("Unexpected MCP response type".into())
        }
    }
}
```

### 9. Database Schema Implementation

```rust
// src/user/storage.rs
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct UserStorage {
    pool: PgPool,
}

impl UserStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect(database_url)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool })
    }
    
    pub async fn create_user(&self, user: &TelegramUser) -> Result<UserProfile> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            INSERT INTO users (telegram_id, username, registered_at, subscription_status)
            VALUES ($1, $2, NOW(), 'trial')
            RETURNING *
            "#,
            user.id.0 as i64,
            user.username.as_deref()
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(profile)
    }
    
    pub async fn record_conversation(
        &self,
        user_id: UserId,
        neuron_id: &str,
    ) -> Result<Conversation> {
        let conversation = sqlx::query_as!(
            Conversation,
            r#"
            INSERT INTO conversations (id, user_id, neuron_id, started_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING *
            "#,
            Uuid::new_v4(),
            user_id.0 as i64,
            neuron_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Update user conversation count
        sqlx::query!(
            r#"
            UPDATE users 
            SET conversation_count = conversation_count + 1,
                last_active = NOW()
            WHERE telegram_id = $1
            "#,
            user_id.0 as i64
        )
        .execute(&self.pool)
        .await?;
        
        Ok(conversation)
    }
}
```

## Configuration

```yaml
# config/hal9-operator.yaml
telegram:
  bot_token: "${TELEGRAM_BOT_TOKEN}"
  webhook_url: "https://hal9.example.com/webhook"
  webhook_port: 8443
  
subscription:
  trial_conversations: 10
  premium_price_stars: 300
  premium_duration_days: 30
  
neuron_pool:
  max_neurons: 1000
  min_idle_neurons: 10
  neuron_timeout_minutes: 30
  mcp_port_range:
    start: 20000
    end: 30000
    
storage:
  database_url: "${DATABASE_URL}"
  redis_url: "${REDIS_URL}"
  user_data_path: "/data/users"
  
scheduler:
  max_jobs_per_user: 10
  notification_retry_attempts: 3
  
monitoring:
  prometheus_port: 9090
  health_check_interval: 30
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_neuron_allocation() {
        let pool = NeuronPool::new_test().await.unwrap();
        let user_id = UserId(12345);
        
        // First allocation should create new neuron
        let neuron_id1 = pool.allocate_neuron(user_id).await.unwrap();
        assert!(!neuron_id1.is_empty());
        
        // Second allocation should return same neuron
        let neuron_id2 = pool.allocate_neuron(user_id).await.unwrap();
        assert_eq!(neuron_id1, neuron_id2);
    }
    
    #[tokio::test]
    async fn test_telegram_mcp_bridge() {
        let bridge = TelegramMCPBridge::new_test();
        let message = create_test_message("Hello, HAL9!");
        
        let mcp_request = bridge.telegram_to_mcp(
            message,
            UserContext::test()
        ).unwrap();
        
        assert_eq!(mcp_request.content, "Hello, HAL9!");
        assert_eq!(
            mcp_request.context.metadata.get("telegram_message_id"),
            Some(&"123".to_string())
        );
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_end_to_end_conversation() {
    let app = TestApp::spawn().await;
    
    // Register user
    let user = create_test_user();
    app.register_user(&user).await;
    
    // Send message
    let response = app.send_message(&user, "Hello!").await;
    assert!(response.contains("Hello"));
    
    // Check neuron allocation
    let neuron_id = app.get_user_neuron(&user).await;
    assert!(neuron_id.is_some());
}
```

## Deployment

### Docker Configuration

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hal9-operator /usr/local/bin/

EXPOSE 8443 9090

CMD ["hal9-operator"]
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-operator
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hal9-operator
  template:
    metadata:
      labels:
        app: hal9-operator
    spec:
      containers:
      - name: hal9-operator
        image: hal9/operator:latest
        ports:
        - containerPort: 8443  # Webhook
        - containerPort: 9090  # Metrics
        env:
        - name: TELEGRAM_BOT_TOKEN
          valueFrom:
            secretKeyRef:
              name: hal9-secrets
              key: telegram-bot-token
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: hal9-secrets
              key: database-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
```

## Monitoring & Observability

### Prometheus Metrics

```rust
// src/metrics.rs
use prometheus::{IntCounter, Histogram, register_int_counter, register_histogram};

lazy_static! {
    pub static ref MESSAGES_RECEIVED: IntCounter = 
        register_int_counter!("hal9_messages_received_total", "Total messages received").unwrap();
    
    pub static ref CONVERSATIONS_STARTED: IntCounter = 
        register_int_counter!("hal9_conversations_started_total", "Total conversations started").unwrap();
    
    pub static ref RESPONSE_TIME: Histogram = 
        register_histogram!("hal9_response_time_seconds", "Response time in seconds").unwrap();
    
    pub static ref NEURON_UTILIZATION: IntGauge = 
        register_int_gauge!("hal9_neuron_utilization", "Current neuron utilization").unwrap();
}
```

## Security Considerations

1. **Input Validation**: Sanitize all user inputs
2. **Rate Limiting**: Implement per-user rate limits
3. **Authentication**: Verify Telegram user identity
4. **Data Encryption**: Encrypt user data at rest
5. **Access Control**: Isolate user data and neurons
6. **Audit Logging**: Log all sensitive operations

## Performance Optimization

1. **Connection Pooling**: Reuse database connections
2. **Caching**: Cache user profiles and settings
3. **Async Processing**: Use Tokio for concurrency
4. **Batch Operations**: Batch database writes
5. **Resource Limits**: Set memory/CPU limits per neuron

This implementation guide provides a comprehensive foundation for building the HAL9-Operator Telegram service, leveraging the existing HAL9 infrastructure while adding Telegram-specific functionality.