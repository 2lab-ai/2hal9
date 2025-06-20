//! Compact neuron IDs for memory efficiency
//!
//! Replaces UUID (16 bytes) with u32 (4 bytes) for 4x memory savings

use std::sync::atomic::{AtomicU32, Ordering};

/// Compact 32-bit neuron ID (replaces UUID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NeuronId(u32);

impl NeuronId {
    /// Create a new neuron ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub fn value(&self) -> u32 {
        self.0
    }
    
    /// Invalid/null ID
    pub const INVALID: Self = Self(0);
    
    /// Check if ID is valid
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl std::fmt::Display for NeuronId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "N{:08x}", self.0)
    }
}

/// Thread-safe ID generator
pub struct NeuronIdGenerator {
    next_id: AtomicU32,
}

impl NeuronIdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        Self {
            // Start at 1, 0 is reserved for INVALID
            next_id: AtomicU32::new(1),
        }
    }
    
    /// Generate a new unique ID
    pub fn next(&self) -> NeuronId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        NeuronId(id)
    }
    
    /// Get the number of IDs generated
    pub fn count(&self) -> u32 {
        self.next_id.load(Ordering::Relaxed) - 1
    }
    
    /// Reset the generator (useful for tests)
    pub fn reset(&self) {
        self.next_id.store(1, Ordering::Relaxed);
    }
}

impl Default for NeuronIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// ID mapping for legacy UUID compatibility
pub struct IdMapper {
    uuid_to_compact: std::collections::HashMap<uuid::Uuid, NeuronId>,
    compact_to_uuid: std::collections::HashMap<NeuronId, uuid::Uuid>,
    generator: NeuronIdGenerator,
}

impl Default for IdMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl IdMapper {
    /// Create a new ID mapper
    pub fn new() -> Self {
        Self {
            uuid_to_compact: std::collections::HashMap::new(),
            compact_to_uuid: std::collections::HashMap::new(),
            generator: NeuronIdGenerator::new(),
        }
    }
    
    /// Map a UUID to a compact ID
    pub fn map_uuid(&mut self, uuid: uuid::Uuid) -> NeuronId {
        if let Some(&id) = self.uuid_to_compact.get(&uuid) {
            id
        } else {
            let id = self.generator.next();
            self.uuid_to_compact.insert(uuid, id);
            self.compact_to_uuid.insert(id, uuid);
            id
        }
    }
    
    /// Get UUID from compact ID
    pub fn get_uuid(&self, id: NeuronId) -> Option<uuid::Uuid> {
        self.compact_to_uuid.get(&id).copied()
    }
    
    /// Get compact ID from UUID
    pub fn get_compact(&self, uuid: &uuid::Uuid) -> Option<NeuronId> {
        self.uuid_to_compact.get(uuid).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neuron_id() {
        let id1 = NeuronId::new(42);
        let id2 = NeuronId::new(42);
        let id3 = NeuronId::new(43);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1.is_valid());
        assert!(!NeuronId::INVALID.is_valid());
    }
    
    #[test]
    fn test_id_generator() {
        let gen = NeuronIdGenerator::new();
        
        let id1 = gen.next();
        let id2 = gen.next();
        let id3 = gen.next();
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_eq!(gen.count(), 3);
        
        // IDs should be sequential
        assert_eq!(id1.value() + 1, id2.value());
        assert_eq!(id2.value() + 1, id3.value());
    }
    
    #[test]
    fn test_id_mapper() {
        let mut mapper = IdMapper::new();
        
        let uuid1 = uuid::Uuid::new_v4();
        let uuid2 = uuid::Uuid::new_v4();
        
        let id1 = mapper.map_uuid(uuid1);
        let id2 = mapper.map_uuid(uuid2);
        let id1_again = mapper.map_uuid(uuid1);
        
        assert_eq!(id1, id1_again);
        assert_ne!(id1, id2);
        
        assert_eq!(mapper.get_uuid(id1), Some(uuid1));
        assert_eq!(mapper.get_compact(&uuid1), Some(id1));
    }
}