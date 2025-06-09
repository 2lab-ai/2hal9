//! Distributed session management for 1000+ users

use anyhow::Result;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::net::IpAddr;
use redis::{AsyncCommands, RedisResult};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use rand::{RngCore, SeedableRng};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::cache::RedisPool;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub region: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub data: HashMap<String, serde_json::Value>,
    pub ip_address: IpAddr,
    pub user_agent: String,
}

/// Client information for session creation
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub preferred_region: Option<String>,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub ttl: ChronoDuration,
    pub refresh_threshold: ChronoDuration,
    pub max_sessions_per_user: usize,
    pub encryption_key: Vec<u8>,
    pub enable_geo_affinity: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            ttl: ChronoDuration::hours(24),
            refresh_threshold: ChronoDuration::hours(1),
            max_sessions_per_user: 5,
            encryption_key: vec![0u8; 32], // In production, use proper key
            enable_geo_affinity: true,
        }
    }
}

/// Distributed session manager
pub struct DistributedSessionManager {
    redis_pool: RedisPool,
    config: SessionConfig,
    cipher: Aes256Gcm,
    local_region: String,
}

impl DistributedSessionManager {
    /// Create new session manager
    pub fn new(redis_pool: RedisPool, config: SessionConfig, local_region: String) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from_slice(&config.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self {
            redis_pool,
            config,
            cipher,
            local_region,
        })
    }
    
    /// Create new session
    pub async fn create_session(
        &self,
        user_id: Uuid,
        organization_id: Option<Uuid>,
        client_info: ClientInfo,
    ) -> Result<Session> {
        // Check session limit
        self.enforce_session_limit(user_id).await?;
        
        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            region: self.determine_region(&client_info),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at: Utc::now() + self.config.ttl,
            data: HashMap::new(),
            ip_address: client_info.ip_address,
            user_agent: client_info.user_agent,
        };
        
        // Store session
        self.store_session(&session).await?;
        
        // Add to user's session list
        self.add_user_session(user_id, session.id).await?;
        
        Ok(session)
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>> {
        // Try local region first
        let key = self.get_session_key(&self.local_region, session_id);
        
        if let Some(session) = self.get_session_from_key(&key).await? {
            // Update last accessed time
            self.touch_session(&session).await?;
            return Ok(Some(session));
        }
        
        // Try other regions if not found locally
        if self.config.enable_geo_affinity {
            for region in self.get_all_regions() {
                if region != self.local_region {
                    let key = self.get_session_key(&region, session_id);
                    if let Some(mut session) = self.get_session_from_key(&key).await? {
                        // Migrate session to local region
                        session = self.migrate_session(session, &self.local_region).await?;
                        return Ok(Some(session));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Update session data
    pub async fn update_session(&self, session_id: Uuid, data: HashMap<String, serde_json::Value>) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.data = data;
            session.last_accessed = Utc::now();
            self.store_session(&session).await?;
        }
        Ok(())
    }
    
    /// Invalidate session
    pub async fn invalidate_session(&self, session_id: Uuid) -> Result<()> {
        // Find and delete from all regions
        for region in self.get_all_regions() {
            let key = self.get_session_key(&region, session_id);
            let mut conn = self.redis_pool.get().await?;
            let _: RedisResult<()> = conn.del(&key).await;
        }
        
        // Remove from user's session list
        if let Some(session) = self.get_session(session_id).await? {
            self.remove_user_session(session.user_id, session_id).await?;
        }
        
        Ok(())
    }
    
    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("user_sessions:{}", user_id);
        
        let session_ids: Vec<String> = conn.smembers(&key).await?;
        let mut sessions = Vec::new();
        
        for session_id_str in session_ids {
            if let Ok(session_id) = Uuid::parse_str(&session_id_str) {
                if let Some(session) = self.get_session(session_id).await? {
                    sessions.push(session);
                }
            }
        }
        
        sessions.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        Ok(sessions)
    }
    
    /// Clean expired sessions
    pub async fn clean_expired_sessions(&self) -> Result<usize> {
        let mut cleaned = 0;
        let mut conn = self.redis_pool.get().await?;
        
        // Scan for session keys
        let pattern = format!("session:*");
        let keys: Vec<String> = redis::cmd("SCAN")
            .arg(0)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(1000)
            .query_async(&mut conn)
            .await?;
        
        for key in keys {
            if let Ok(encrypted_data) = conn.get::<_, Vec<u8>>(&key).await {
                if let Ok(session) = self.decrypt_session(&encrypted_data) {
                    if session.expires_at < Utc::now() {
                        let _: RedisResult<()> = conn.del(&key).await;
                        self.remove_user_session(session.user_id, session.id).await?;
                        cleaned += 1;
                    }
                }
            }
        }
        
        Ok(cleaned)
    }
    
    /// Store session in Redis
    async fn store_session(&self, session: &Session) -> Result<()> {
        let key = self.get_session_key(&session.region, session.id);
        let encrypted_data = self.encrypt_session(session)?;
        
        let mut conn = self.redis_pool.get().await?;
        let ttl = (session.expires_at - Utc::now()).num_seconds().max(1) as usize;
        
        conn.setex(&key, encrypted_data, ttl).await?;
        
        Ok(())
    }
    
    /// Get session from specific key
    async fn get_session_from_key(&self, key: &str) -> Result<Option<Session>> {
        let mut conn = self.redis_pool.get().await?;
        
        if let Ok(encrypted_data) = conn.get::<_, Vec<u8>>(key).await {
            let session = self.decrypt_session(&encrypted_data)?;
            if session.expires_at > Utc::now() {
                return Ok(Some(session));
            }
        }
        
        Ok(None)
    }
    
    /// Touch session to update last accessed time
    async fn touch_session(&self, session: &Session) -> Result<()> {
        let mut updated_session = session.clone();
        updated_session.last_accessed = Utc::now();
        
        // Extend expiration if within refresh threshold
        let time_until_expiry = session.expires_at - Utc::now();
        if time_until_expiry < self.config.refresh_threshold {
            updated_session.expires_at = Utc::now() + self.config.ttl;
        }
        
        self.store_session(&updated_session).await
    }
    
    /// Migrate session to different region
    async fn migrate_session(&self, mut session: Session, new_region: &str) -> Result<Session> {
        // Delete from old region
        let old_key = self.get_session_key(&session.region, session.id);
        let mut conn = self.redis_pool.get().await?;
        let _: RedisResult<()> = conn.del(&old_key).await;
        
        // Update region and store in new location
        session.region = new_region.to_string();
        self.store_session(&session).await?;
        
        tracing::info!(
            "Migrated session {} from {} to {}",
            session.id,
            old_key,
            new_region
        );
        
        Ok(session)
    }
    
    /// Encrypt session data
    fn encrypt_session(&self, session: &Session) -> Result<Vec<u8>> {
        let json_data = serde_json::to_vec(session)?;
        
        let mut nonce_bytes = [0u8; 12];
        let mut rng = rand::rngs::StdRng::from_entropy();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted = self.cipher
            .encrypt(nonce, json_data.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Prepend nonce to encrypted data
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&encrypted);
        
        Ok(result)
    }
    
    /// Decrypt session data
    fn decrypt_session(&self, encrypted_data: &[u8]) -> Result<Session> {
        if encrypted_data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        let session: Session = serde_json::from_slice(&decrypted)?;
        Ok(session)
    }
    
    /// Determine region for session
    fn determine_region(&self, client_info: &ClientInfo) -> String {
        if let Some(preferred) = &client_info.preferred_region {
            return preferred.clone();
        }
        
        if self.config.enable_geo_affinity {
            // In production, use GeoIP lookup
            // For now, use local region
            self.local_region.clone()
        } else {
            self.local_region.clone()
        }
    }
    
    /// Get session key
    fn get_session_key(&self, region: &str, session_id: Uuid) -> String {
        format!("session:{}:{}", region, session_id)
    }
    
    /// Get all regions
    fn get_all_regions(&self) -> Vec<String> {
        vec![
            "us-west".to_string(),
            "eu-central".to_string(),
            "ap-south".to_string(),
        ]
    }
    
    /// Add session to user's session list
    async fn add_user_session(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("user_sessions:{}", user_id);
        
        conn.sadd(&key, session_id.to_string()).await?;
        conn.expire(&key, 86400 * 30).await?; // 30 days
        
        Ok(())
    }
    
    /// Remove session from user's session list
    async fn remove_user_session(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("user_sessions:{}", user_id);
        
        conn.srem(&key, session_id.to_string()).await?;
        
        Ok(())
    }
    
    /// Enforce session limit per user
    async fn enforce_session_limit(&self, user_id: Uuid) -> Result<()> {
        let sessions = self.get_user_sessions(user_id).await?;
        
        if sessions.len() >= self.config.max_sessions_per_user {
            // Remove oldest sessions
            let to_remove = sessions.len() - self.config.max_sessions_per_user + 1;
            for session in sessions.iter().rev().take(to_remove) {
                self.invalidate_session(session.id).await?;
            }
        }
        
        Ok(())
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub sessions_by_region: HashMap<String, usize>,
    pub average_session_duration: ChronoDuration,
    pub active_users: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_test_client_info() -> ClientInfo {
        ClientInfo {
            ip_address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            user_agent: "Mozilla/5.0".to_string(),
            preferred_region: Some("us-west".to_string()),
        }
    }

    #[test]
    fn test_session_encryption() {
        let config = SessionConfig::default();
        let key = Key::<Aes256Gcm>::from_slice(&config.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        let session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            organization_id: None,
            region: "us-west".to_string(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at: Utc::now() + ChronoDuration::hours(1),
            data: HashMap::new(),
            ip_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            user_agent: "test".to_string(),
        };
        
        // Encrypt
        let json_data = serde_json::to_vec(&session).unwrap();
        let mut nonce_bytes = [0u8; 12];
        let nonce = Nonce::from_slice(&nonce_bytes);
        let encrypted = cipher.encrypt(nonce, json_data.as_ref()).unwrap();
        
        // Decrypt
        let decrypted = cipher.decrypt(nonce, encrypted.as_ref()).unwrap();
        let recovered: Session = serde_json::from_slice(&decrypted).unwrap();
        
        assert_eq!(session.id, recovered.id);
        assert_eq!(session.user_id, recovered.user_id);
    }
}