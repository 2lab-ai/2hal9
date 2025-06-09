//! State migration engine for transferring neuron states from flat to hierarchical

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::{Result, Error};

/// Progress tracking for state migration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationProgress {
    pub total_neurons: usize,
    pub migrated_neurons: usize,
    pub failed_neurons: usize,
    pub percentage_complete: f32,
    pub estimated_time_remaining: std::time::Duration,
    pub current_batch: usize,
    pub total_batches: usize,
}

/// State migration engine
pub struct StateMigrationEngine {
    batch_size: usize,
    parallel_workers: usize,
    progress: Arc<RwLock<MigrationProgress>>,
    checkpoints: Arc<RwLock<Vec<MigrationCheckpoint>>>,
    validators: Arc<Vec<Arc<dyn StateValidator>>>,
}

impl StateMigrationEngine {
    pub fn new(batch_size: usize, parallel_workers: usize) -> Self {
        Self {
            batch_size,
            parallel_workers,
            progress: Arc::new(RwLock::new(MigrationProgress::default())),
            checkpoints: Arc::new(RwLock::new(Vec::new())),
            validators: Arc::new(vec![
                Arc::new(SchemaValidator) as Arc<dyn StateValidator>,
                Arc::new(IntegrityValidator) as Arc<dyn StateValidator>,
                Arc::new(ConsistencyValidator) as Arc<dyn StateValidator>,
            ]),
        }
    }
    
    /// Start migrating states from flat to hierarchical
    pub async fn migrate_states(
        &self,
        source: Arc<dyn StateSource>,
        target: Arc<dyn StateTarget>,
    ) -> Result<()> {
        // Get total count
        let total_neurons = source.count_neurons().await?;
        {
            let mut progress = self.progress.write();
            progress.total_neurons = total_neurons;
            progress.total_batches = (total_neurons + self.batch_size - 1) / self.batch_size;
        }
        
        // Process in batches
        let mut offset = 0;
        let mut batch_num = 0;
        
        while offset < total_neurons {
            batch_num += 1;
            
            // Create checkpoint before batch
            self.create_checkpoint(batch_num, offset).await?;
            
            // Process batch
            match self.process_batch(source.clone(), target.clone(), offset, self.batch_size).await {
                Ok(migrated) => {
                    offset += migrated;
                    self.update_progress(batch_num, offset, total_neurons);
                }
                Err(e) => {
                    tracing::error!("Batch {} failed: {}", batch_num, e);
                    
                    // Try to recover
                    if !self.recover_from_checkpoint(batch_num - 1).await? {
                        return Err(e);
                    }
                }
            }
            
            // Throttle if needed
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        
        // Final validation
        self.validate_migration(source, target).await?;
        
        Ok(())
    }
    
    async fn process_batch(
        &self,
        source: Arc<dyn StateSource>,
        target: Arc<dyn StateTarget>,
        offset: usize,
        limit: usize,
    ) -> Result<usize> {
        // Fetch batch from source
        let flat_states = source.fetch_batch(offset, limit).await?;
        let batch_size = flat_states.len();
        
        if batch_size == 0 {
            return Ok(0);
        }
        
        // Convert states in parallel
        let mut handles = Vec::new();
        let chunk_size = (batch_size + self.parallel_workers - 1) / self.parallel_workers;
        
        for chunk in flat_states.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let validators = Arc::clone(&self.validators);
            
            let handle = tokio::spawn(async move {
                let mut converted = Vec::new();
                
                for flat_state in chunk {
                    let neuron_id = flat_state.neuron_id;
                    match Self::convert_state(flat_state, &validators).await {
                        Ok(hier_state) => converted.push(hier_state),
                        Err(e) => {
                            tracing::error!("Failed to convert neuron {}: {}", neuron_id, e);
                            return Err(e);
                        }
                    }
                }
                
                Ok(converted)
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut all_converted = Vec::new();
        for handle in handles {
            let converted = handle.await
                .map_err(|e| Error::Migration(format!("Worker panic: {}", e)))??;
            all_converted.extend(converted);
        }
        
        // Write to target
        target.store_batch(all_converted).await?;
        
        Ok(batch_size)
    }
    
    async fn convert_state(
        flat_state: FlatNeuronState,
        validators: &[Arc<dyn StateValidator>],
    ) -> Result<HierarchicalNeuronState> {
        // Determine appropriate layer based on neuron characteristics
        let layer = Self::determine_layer(&flat_state);
        
        // Convert state format
        let hier_state = HierarchicalNeuronState {
            unit_id: flat_state.neuron_id,
            layer,
            cognitive_state: Self::convert_cognitive_state(&flat_state),
            learning_state: Self::convert_learning_state(&flat_state),
            connections: Self::convert_connections(&flat_state),
            metadata: Self::convert_metadata(&flat_state),
        };
        
        // Validate converted state
        for validator in validators {
            validator.validate(&flat_state, &hier_state)?;
        }
        
        Ok(hier_state)
    }
    
    fn determine_layer(flat_state: &FlatNeuronState) -> CognitiveLayer {
        // Analyze neuron behavior and capabilities to assign layer
        match flat_state.neuron_type.as_str() {
            "reflex" | "reactive" => CognitiveLayer::L1Reflexive,
            "executor" | "implementer" => CognitiveLayer::L2Implementation,
            "coordinator" | "manager" => CognitiveLayer::L3Operational,
            "planner" | "strategist" => CognitiveLayer::L4Tactical,
            "visionary" | "architect" => CognitiveLayer::L5Strategic,
            _ => {
                // Use heuristics based on connections and processing time
                if flat_state.avg_response_time < 10.0 {
                    CognitiveLayer::L1Reflexive
                } else if flat_state.avg_response_time < 200.0 {
                    CognitiveLayer::L2Implementation
                } else if flat_state.connection_count > 10 {
                    CognitiveLayer::L4Tactical
                } else {
                    CognitiveLayer::L3Operational
                }
            }
        }
    }
    
    fn convert_cognitive_state(flat: &FlatNeuronState) -> CognitiveState {
        CognitiveState {
            activation_level: flat.activation,
            attention_focus: flat.context.get("focus").cloned(),
            working_memory: flat.memory.clone(),
            processing_mode: if flat.is_async { "async" } else { "sync" }.to_string(),
        }
    }
    
    fn convert_learning_state(flat: &FlatNeuronState) -> LearningState {
        LearningState {
            weights: flat.weights.clone(),
            biases: flat.biases.clone(),
            learning_rate: flat.learning_rate,
            momentum: flat.momentum.unwrap_or(0.9),
            gradient_history: Vec::new(), // Not preserved from flat
        }
    }
    
    fn convert_connections(flat: &FlatNeuronState) -> Vec<Connection> {
        flat.connections.iter().map(|conn| {
            Connection {
                target_id: conn.target,
                connection_type: Self::map_connection_type(&conn.conn_type),
                weight: conn.weight,
                metadata: HashMap::new(),
            }
        }).collect()
    }
    
    fn map_connection_type(flat_type: &str) -> ConnectionType {
        match flat_type {
            "forward" => ConnectionType::Forward,
            "backward" => ConnectionType::Backward,
            "lateral" => ConnectionType::Lateral,
            _ => ConnectionType::Forward,
        }
    }
    
    fn convert_metadata(flat: &FlatNeuronState) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        // Preserve important metadata
        metadata.insert("original_id".to_string(), serde_json::json!(flat.neuron_id));
        metadata.insert("created_at".to_string(), serde_json::json!(flat.created_at));
        metadata.insert("migrated_at".to_string(), serde_json::json!(chrono::Utc::now()));
        metadata.insert("original_type".to_string(), serde_json::json!(flat.neuron_type));
        
        metadata
    }
    
    async fn create_checkpoint(&self, batch_num: usize, offset: usize) -> Result<()> {
        let checkpoint = MigrationCheckpoint {
            id: Uuid::new_v4(),
            batch_number: batch_num,
            offset,
            timestamp: chrono::Utc::now(),
            progress: self.progress.read().clone(),
        };
        
        self.checkpoints.write().push(checkpoint);
        
        // TODO: Persist checkpoint to durable storage
        
        Ok(())
    }
    
    async fn recover_from_checkpoint(&self, batch_num: usize) -> Result<bool> {
        let checkpoints = self.checkpoints.read();
        
        if let Some(checkpoint) = checkpoints.iter().find(|cp| cp.batch_number == batch_num) {
            *self.progress.write() = checkpoint.progress.clone();
            
            // TODO: Restore state from checkpoint
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn update_progress(&self, batch_num: usize, migrated: usize, total: usize) {
        let mut progress = self.progress.write();
        progress.migrated_neurons = migrated;
        progress.current_batch = batch_num;
        progress.percentage_complete = (migrated as f32 / total as f32) * 100.0;
        
        // Estimate time remaining based on current rate
        // TODO: Implement proper time estimation
        progress.estimated_time_remaining = std::time::Duration::from_secs(
            ((total - migrated) * 10 / self.batch_size) as u64
        );
    }
    
    async fn validate_migration(
        &self,
        source: Arc<dyn StateSource>,
        target: Arc<dyn StateTarget>,
    ) -> Result<()> {
        tracing::info!("Validating migration integrity...");
        
        // Sample validation - check counts match
        let source_count = source.count_neurons().await?;
        let target_count = target.count_neurons().await?;
        
        if source_count != target_count {
            return Err(Error::Migration(format!(
                "Count mismatch: source={}, target={}",
                source_count, target_count
            )));
        }
        
        // TODO: More comprehensive validation
        
        Ok(())
    }
    
    /// Get current migration progress
    pub async fn get_progress(&self) -> MigrationProgress {
        self.progress.read().clone()
    }
    
    /// Verify integrity of migrated states
    pub async fn verify_integrity(&self, check_all: bool) -> Result<bool> {
        // TODO: Implement integrity verification
        Ok(true)
    }
    
    /// Check for data loss
    pub async fn check_data_loss(&self) -> Result<bool> {
        // TODO: Implement data loss detection
        Ok(false)
    }
}

/// Source of flat neuron states
#[async_trait::async_trait]
pub trait StateSource: Send + Sync {
    async fn count_neurons(&self) -> Result<usize>;
    async fn fetch_batch(&self, offset: usize, limit: usize) -> Result<Vec<FlatNeuronState>>;
}

/// Target for hierarchical neuron states
#[async_trait::async_trait]
pub trait StateTarget: Send + Sync {
    async fn store_batch(&self, states: Vec<HierarchicalNeuronState>) -> Result<()>;
    async fn count_neurons(&self) -> Result<usize>;
}

/// Validator for state conversion
pub trait StateValidator: Send + Sync {
    fn validate(&self, flat: &FlatNeuronState, hierarchical: &HierarchicalNeuronState) -> Result<()>;
}

/// Schema validator
#[derive(Clone)]
struct SchemaValidator;

impl StateValidator for SchemaValidator {
    fn validate(&self, _flat: &FlatNeuronState, hier: &HierarchicalNeuronState) -> Result<()> {
        // Validate required fields are present
        if hier.unit_id == Uuid::nil() {
            return Err(Error::Migration("Invalid unit ID".to_string()));
        }
        Ok(())
    }
}

/// Integrity validator
#[derive(Clone)]
struct IntegrityValidator;

impl StateValidator for IntegrityValidator {
    fn validate(&self, flat: &FlatNeuronState, hier: &HierarchicalNeuronState) -> Result<()> {
        // Ensure critical data is preserved
        if flat.weights.len() != hier.learning_state.weights.len() {
            return Err(Error::Migration("Weight dimension mismatch".to_string()));
        }
        Ok(())
    }
}

/// Consistency validator
#[derive(Clone)]
struct ConsistencyValidator;

impl StateValidator for ConsistencyValidator {
    fn validate(&self, flat: &FlatNeuronState, hier: &HierarchicalNeuronState) -> Result<()> {
        // Ensure connections are preserved
        if flat.connections.len() != hier.connections.len() {
            return Err(Error::Migration("Connection count mismatch".to_string()));
        }
        Ok(())
    }
}

/// Checkpoint for migration recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MigrationCheckpoint {
    id: Uuid,
    batch_number: usize,
    offset: usize,
    timestamp: chrono::DateTime<chrono::Utc>,
    progress: MigrationProgress,
}

/// Flat neuron state (legacy format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatNeuronState {
    pub neuron_id: Uuid,
    pub neuron_type: String,
    pub activation: f32,
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
    pub learning_rate: f32,
    pub momentum: Option<f32>,
    pub connections: Vec<FlatConnection>,
    pub memory: HashMap<String, serde_json::Value>,
    pub context: HashMap<String, serde_json::Value>,
    pub avg_response_time: f32,
    pub connection_count: usize,
    pub is_async: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatConnection {
    pub target: Uuid,
    pub weight: f32,
    pub conn_type: String,
}

/// Hierarchical neuron state (new format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalNeuronState {
    pub unit_id: Uuid,
    pub layer: CognitiveLayer,
    pub cognitive_state: CognitiveState,
    pub learning_state: LearningState,
    pub connections: Vec<Connection>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitiveLayer {
    L1Reflexive,
    L2Implementation,
    L3Operational,
    L4Tactical,
    L5Strategic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub activation_level: f32,
    pub attention_focus: Option<serde_json::Value>,
    pub working_memory: HashMap<String, serde_json::Value>,
    pub processing_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningState {
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
    pub learning_rate: f32,
    pub momentum: f32,
    pub gradient_history: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub target_id: Uuid,
    pub connection_type: ConnectionType,
    pub weight: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Forward,
    Backward,
    Lateral,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layer_determination() {
        let mut flat_state = FlatNeuronState {
            neuron_id: Uuid::new_v4(),
            neuron_type: "reflex".to_string(),
            activation: 0.5,
            weights: vec![],
            biases: vec![],
            learning_rate: 0.1,
            momentum: None,
            connections: vec![],
            memory: HashMap::new(),
            context: HashMap::new(),
            avg_response_time: 5.0,
            connection_count: 2,
            is_async: false,
            created_at: chrono::Utc::now(),
        };
        
        assert!(matches!(
            StateMigrationEngine::determine_layer(&flat_state),
            CognitiveLayer::L1Reflexive
        ));
        
        flat_state.neuron_type = "planner".to_string();
        assert!(matches!(
            StateMigrationEngine::determine_layer(&flat_state),
            CognitiveLayer::L4Tactical
        ));
    }
}