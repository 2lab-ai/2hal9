//! Protocol versioning support for backward compatibility

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{Result, Error};

/// Version migration for protocol messages
pub trait VersionMigration: Send + Sync {
    /// Source version
    fn from_version(&self) -> &ProtocolVersion;
    
    /// Target version
    fn to_version(&self) -> &ProtocolVersion;
    
    /// Migrate message from source to target version
    fn migrate(&self, message: &[u8]) -> Result<Vec<u8>>;
}

use super::ProtocolVersion;

/// Version registry for managing protocol versions
pub struct VersionRegistry {
    migrations: HashMap<(ProtocolVersion, ProtocolVersion), Box<dyn VersionMigration>>,
    current_version: ProtocolVersion,
}

impl VersionRegistry {
    pub fn new(current: ProtocolVersion) -> Self {
        Self {
            migrations: HashMap::new(),
            current_version: current,
        }
    }
    
    /// Register a migration between versions
    pub fn register_migration(&mut self, migration: Box<dyn VersionMigration>) {
        let key = (migration.from_version().clone(), migration.to_version().clone());
        self.migrations.insert(key, migration);
    }
    
    /// Find migration path between versions
    pub fn find_migration_path(&self, from: &ProtocolVersion, to: &ProtocolVersion) -> Result<Vec<&dyn VersionMigration>> {
        if from == to {
            return Ok(vec![]);
        }
        
        // Simple direct migration check (could be extended with path finding)
        if let Some(migration) = self.migrations.get(&(from.clone(), to.clone())) {
            Ok(vec![migration.as_ref()])
        } else {
            Err(Error::Protocol(format!("No migration path from {:?} to {:?}", from, to)))
        }
    }
    
    /// Migrate a message to current version
    pub fn migrate_to_current(&self, from_version: &ProtocolVersion, message: &[u8]) -> Result<Vec<u8>> {
        let migrations = self.find_migration_path(from_version, &self.current_version)?;
        
        let mut current_message = message.to_vec();
        for migration in migrations {
            current_message = migration.migrate(&current_message)?;
        }
        
        Ok(current_message)
    }
}

/// Version header for versioned messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedMessage {
    pub version: ProtocolVersion,
    pub protocol_id: String,
    pub payload: Vec<u8>,
}

impl VersionedMessage {
    pub fn new(protocol_id: &str, version: ProtocolVersion, payload: Vec<u8>) -> Self {
        Self {
            version,
            protocol_id: protocol_id.to_string(),
            payload,
        }
    }
}

/// Example migration from v1.0.0 to v1.1.0
pub struct V1_0_to_V1_1_Migration;

impl VersionMigration for V1_0_to_V1_1_Migration {
    fn from_version(&self) -> &ProtocolVersion {
        &ProtocolVersion::new(1, 0, 0)
    }
    
    fn to_version(&self) -> &ProtocolVersion {
        &ProtocolVersion::new(1, 1, 0)
    }
    
    fn migrate(&self, message: &[u8]) -> Result<Vec<u8>> {
        // Example: Add a new field with default value
        let mut old_msg: serde_json::Value = serde_json::from_slice(message)
            .map_err(|e| Error::Protocol(format!("Failed to parse message: {}", e)))?;
            
        if let serde_json::Value::Object(ref mut map) = old_msg {
            map.insert("new_field".to_string(), serde_json::Value::String("default".to_string()));
        }
        
        serde_json::to_vec(&old_msg)
            .map_err(|e| Error::Protocol(format!("Failed to serialize message: {}", e)))
    }
}