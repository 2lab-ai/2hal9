//! Browser context pool for efficient resource management

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use crate::playwright_stub::{Browser, BrowserContext, Page, BrowserContextOptions};
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::{BrowserConfig, BrowserError, Result};

/// Pool of browser contexts for reuse
pub struct ContextPool {
    /// Maximum number of contexts
    max_contexts: usize,
    
    /// Browser instance
    browser: Browser,
    
    /// Available contexts ready for use
    available: Vec<PooledContextInner>,
    
    /// Contexts currently in use
    in_use: HashMap<Uuid, (PooledContextInner, Instant)>,
    
    /// Semaphore to limit concurrent contexts
    semaphore: Arc<Semaphore>,
    
    /// Configuration
    config: BrowserConfig,
}

impl ContextPool {
    /// Create a new context pool
    pub fn new(max_contexts: usize, browser: Browser, config: BrowserConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_contexts));
        
        Self {
            max_contexts,
            browser,
            available: Vec::new(),
            in_use: HashMap::new(),
            semaphore,
            config,
        }
    }
    
    /// Acquire a context from the pool
    pub async fn acquire(&mut self) -> Result<PooledContext> {
        debug!("Acquiring browser context from pool");
        
        // Acquire semaphore permit
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| BrowserError::PoolExhausted)?;
        
        // Try to reuse available context
        if let Some(context) = self.available.pop() {
            let id = Uuid::new_v4();
            self.in_use.insert(id, (context, Instant::now()));
            
            debug!("Reused existing context: {}", id);
            return Ok(PooledContext {
                id,
                _permit: permit,
            });
        }
        
        // Create new context if under limit
        if self.in_use.len() < self.max_contexts {
            let context = self.create_context().await?;
            let id = Uuid::new_v4();
            self.in_use.insert(id, (context, Instant::now()));
            
            debug!("Created new context: {}", id);
            return Ok(PooledContext {
                id,
                _permit: permit,
            });
        }
        
        // Pool exhausted
        warn!("Context pool exhausted");
        Err(BrowserError::PoolExhausted)
    }
    
    /// Create a new browser context
    async fn create_context(&self) -> Result<PooledContextInner> {
        let options = BrowserContextOptions::default()
            .viewport_width(self.config.viewport_width)
            .viewport_height(self.config.viewport_height)
            .user_agent("HAL9 Browser Automation/1.0")
            .locale("en-US")
            .timezone_id("UTC")
            .ignore_https_errors(false);
        
        let context = self.browser.new_context(options)
            .await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        // Set default timeout
        context.set_default_timeout(self.config.default_timeout as f64);
        
        // Create default page
        let page = context.new_page()
            .await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        Ok(PooledContextInner {
            context,
            page,
            created_at: Instant::now(),
            last_used: Instant::now(),
        })
    }
    
    /// Return a context to the pool
    pub fn release(&mut self, id: Uuid) {
        if let Some((mut context, _)) = self.in_use.remove(&id) {
            debug!("Releasing context: {}", id);
            
            // Update last used time
            context.last_used = Instant::now();
            
            // Clear cookies and storage for privacy
            // Note: This is async but we're in a sync context
            // In production, this should be handled differently
            
            // Return to available pool if still fresh
            if context.created_at.elapsed() < Duration::from_secs(300) {
                self.available.push(context);
            } else {
                // Context too old, let it be dropped
                debug!("Context {} too old, dropping", id);
            }
        }
    }
    
    /// Clear all contexts
    pub async fn clear(&mut self) {
        info!("Clearing context pool");
        
        // Close all in-use contexts
        for (id, (context, _)) in self.in_use.drain() {
            if let Err(e) = context.context.close().await {
                warn!("Error closing context {}: {}", id, e);
            }
        }
        
        // Close all available contexts
        for context in self.available.drain(..) {
            if let Err(e) = context.context.close().await {
                warn!("Error closing available context: {}", e);
            }
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_capacity: self.max_contexts,
            in_use: self.in_use.len(),
            available: self.available.len(),
        }
    }
}

/// Inner context wrapper
struct PooledContextInner {
    context: BrowserContext,
    page: Page,
    created_at: Instant,
    last_used: Instant,
}

/// Handle to a pooled context
pub struct PooledContext {
    id: Uuid,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PooledContext {
    /// Get the page for this context
    pub fn page(&self) -> &Page {
        // This is a simplified version - in production we'd need
        // a way to access the actual page from the pool
        unimplemented!("Need pool reference to get page")
    }
    
    /// Get context ID
    pub fn id(&self) -> Uuid {
        self.id
    }
}

/// Pool statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    pub total_capacity: usize,
    pub in_use: usize,
    pub available: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            total_capacity: 10,
            in_use: 3,
            available: 2,
        };
        
        assert_eq!(stats.total_capacity, 10);
        assert_eq!(stats.in_use, 3);
        assert_eq!(stats.available, 2);
    }
}