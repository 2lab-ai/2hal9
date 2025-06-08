//! Security sandbox for browser automation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use governor::{Quota, RateLimiter, clock::{QuantaInstant, QuantaClock}};
use governor::state::{InMemoryState, NotKeyed};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use ring::rand::{SecureRandom, SystemRandom};
use tracing::{info, warn, debug};

use crate::{SecurityConfig, BrowserError, Result};
use crate::controller::BrowserAction;

/// Security sandbox for browser operations
pub struct SecuritySandbox {
    /// URL policy enforcement
    url_policy: UrlPolicy,
    
    /// Rate limiter
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
    
    /// Credential vault
    credential_vault: Arc<Mutex<CredentialVault>>,
    
    /// Audit logger
    audit_logger: Arc<Mutex<AuditLogger>>,
    
    /// Configuration
    config: SecurityConfig,
}

impl SecuritySandbox {
    /// Create new security sandbox
    pub fn new(config: SecurityConfig) -> Self {
        // Create rate limiter
        let quota = Quota::per_minute(std::num::NonZeroU32::new(config.rate_limit_per_minute).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));
        
        // Create URL policy
        let url_policy = UrlPolicy::new(
            config.url_whitelist.clone(),
            config.url_blacklist.clone(),
        );
        
        // Create credential vault
        let credential_vault = Arc::new(Mutex::new(CredentialVault::new()));
        
        // Create audit logger
        let audit_logger = Arc::new(Mutex::new(AuditLogger::new()));
        
        Self {
            url_policy,
            rate_limiter,
            credential_vault,
            audit_logger,
            config,
        }
    }
    
    /// Validate a browser action
    pub fn validate_action(&self, action: &BrowserAction) -> Result<()> {
        // Check rate limit
        if self.rate_limiter.check().is_err() {
            return Err(BrowserError::RateLimitExceeded);
        }
        
        // Validate based on action type
        match action {
            BrowserAction::Navigate { url } => {
                self.validate_url(url)?;
            }
            BrowserAction::Type { text, .. } => {
                // Check for potential credential leakage
                if self.looks_like_credential(text) {
                    warn!("Potential credential in type action");
                }
            }
            _ => {
                // Other actions are generally safe
            }
        }
        
        // Log action if auditing enabled
        if self.config.enable_audit_log {
            let audit_logger = self.audit_logger.clone();
            let action_clone = action.clone();
            tokio::spawn(async move {
                let mut logger = audit_logger.lock().await;
                logger.log_action(action_clone).await;
            });
        }
        
        Ok(())
    }
    
    /// Validate URL against policy
    pub fn validate_url(&self, url: &str) -> Result<()> {
        self.url_policy.check(url)
    }
    
    /// Check if text looks like a credential
    fn looks_like_credential(&self, text: &str) -> bool {
        // Simple heuristics - in production use more sophisticated detection
        text.len() > 8 && (
            text.contains("password") ||
            text.contains("secret") ||
            text.contains("api_key") ||
            text.contains("token") ||
            // High entropy check
            self.calculate_entropy(text) > 4.0
        )
    }
    
    /// Calculate Shannon entropy of a string
    fn calculate_entropy(&self, s: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let len = s.len() as f64;
        
        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }
        
        char_counts.values()
            .map(|&count| {
                let p = count as f64 / len;
                -p * p.log2()
            })
            .sum()
    }
}

/// URL policy enforcement
pub struct UrlPolicy {
    whitelist_patterns: Vec<glob::Pattern>,
    blacklist_patterns: Vec<glob::Pattern>,
}

impl UrlPolicy {
    /// Create new URL policy
    pub fn new(whitelist: Vec<String>, blacklist: Vec<String>) -> Self {
        let whitelist_patterns = whitelist.iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();
        
        let blacklist_patterns = blacklist.iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();
        
        Self {
            whitelist_patterns,
            blacklist_patterns,
        }
    }
    
    /// Check if URL is allowed
    pub fn check(&self, url: &str) -> Result<()> {
        // Parse URL
        let parsed = Url::parse(url)
            .map_err(|_| BrowserError::InvalidUrl(url.to_string()))?;
        
        let url_str = parsed.as_str();
        
        // Check blacklist first
        for pattern in &self.blacklist_patterns {
            if pattern.matches(url_str) {
                return Err(BrowserError::SecurityViolation(
                    format!("URL matches blacklist: {}", url)
                ));
            }
        }
        
        // Check whitelist if not empty
        if !self.whitelist_patterns.is_empty() {
            let whitelisted = self.whitelist_patterns.iter()
                .any(|p| p.matches(url_str));
            
            if !whitelisted {
                return Err(BrowserError::SecurityViolation(
                    format!("URL not in whitelist: {}", url)
                ));
            }
        }
        
        Ok(())
    }
}

/// Encrypted credential storage
pub struct CredentialVault {
    /// Encrypted credentials by site
    credentials: HashMap<String, EncryptedCredential>,
    
    /// Encryption key
    key: Key<Aes256Gcm>,
    
    /// Random number generator
    rng: SystemRandom,
}

impl CredentialVault {
    /// Create new credential vault
    pub fn new() -> Self {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes).unwrap();
        let key = Key::from_slice(&key_bytes);
        
        Self {
            credentials: HashMap::new(),
            key: key.clone(),
            rng,
        }
    }
    
    /// Store credentials for a site
    pub fn store(&mut self, site: &str, username: String, password: String) -> Result<()> {
        let credential = Credential { username, password };
        let encrypted = self.encrypt_credential(&credential)?;
        
        self.credentials.insert(site.to_string(), encrypted);
        info!("Stored credentials for site: {}", site);
        
        Ok(())
    }
    
    /// Retrieve credentials for a site
    pub fn get(&self, site: &str) -> Result<Option<Credential>> {
        match self.credentials.get(site) {
            Some(encrypted) => {
                let credential = self.decrypt_credential(encrypted)?;
                Ok(Some(credential))
            }
            None => Ok(None),
        }
    }
    
    /// Encrypt credential
    fn encrypt_credential(&self, credential: &Credential) -> Result<EncryptedCredential> {
        let cipher = Aes256Gcm::new(&self.key);
        
        // Generate nonce
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to generate nonce: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Serialize credential
        let plaintext = serde_json::to_vec(credential)?;
        
        // Encrypt
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        Ok(EncryptedCredential {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }
    
    /// Decrypt credential
    fn decrypt_credential(&self, encrypted: &EncryptedCredential) -> Result<Credential> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        // Decrypt
        let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        // Deserialize
        let credential: Credential = serde_json::from_slice(&plaintext)?;
        Ok(credential)
    }
}

/// Credential data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

/// Encrypted credential storage
#[derive(Debug, Clone)]
struct EncryptedCredential {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

/// Audit logger for security events
pub struct AuditLogger {
    /// Log entries
    entries: Vec<AuditEntry>,
    
    /// Maximum entries to keep
    max_entries: usize,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 10000,
        }
    }
    
    /// Log a browser action
    pub async fn log_action(&mut self, action: BrowserAction) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            action,
            user_id: "system".to_string(), // TODO: Get actual user
            session_id: uuid::Uuid::new_v4(),
        };
        
        self.entries.push(entry);
        
        // Trim old entries
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
    
    /// Get recent entries
    pub fn get_recent(&self, count: usize) -> Vec<&AuditEntry> {
        self.entries.iter()
            .rev()
            .take(count)
            .collect()
    }
}

/// Audit log entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: BrowserAction,
    pub user_id: String,
    pub session_id: uuid::Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_policy() {
        let policy = UrlPolicy::new(
            vec!["https://*.example.com/*".to_string()],
            vec!["*/admin/*".to_string()],
        );
        
        assert!(policy.check("https://app.example.com/home").is_ok());
        assert!(policy.check("https://app.example.com/admin/settings").is_err());
        assert!(policy.check("https://evil.com/hack").is_err());
    }
    
    #[test]
    fn test_credential_vault() {
        let mut vault = CredentialVault::new();
        
        vault.store("example.com", "user".to_string(), "pass".to_string()).unwrap();
        
        let cred = vault.get("example.com").unwrap().unwrap();
        assert_eq!(cred.username, "user");
        assert_eq!(cred.password, "pass");
        
        assert!(vault.get("unknown.com").unwrap().is_none());
    }
}