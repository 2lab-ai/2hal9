use anyhow::{Context, Result};
use ethers::core::types::H256;
use futures::stream::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============ IPFS Storage ============

pub struct IPFSStorage {
    client: IpfsClient,
    config: IPFSConfig,
    pinned_content: Arc<RwLock<HashMap<String, PinnedContent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPFSConfig {
    pub api_url: String,
    pub gateway_url: String,
    pub pin_content: bool,
    pub max_file_size: u64,
    pub replication_factor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinnedContent {
    cid: String,
    content_type: ContentType,
    size: u64,
    pinned_at: chrono::DateTime<chrono::Utc>,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContentType {
    ComputationResult,
    NeuronModel,
    TrainingData,
    ProofData,
    Metadata,
}

impl Default for IPFSConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:5001".to_string(),
            gateway_url: "http://localhost:8080".to_string(),
            pin_content: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            replication_factor: 3,
        }
    }
}

impl IPFSStorage {
    pub fn new(config: IPFSConfig) -> Result<Self> {
        let client =
            IpfsClient::from_str(&config.api_url).context("Failed to create IPFS client")?;

        Ok(Self {
            client,
            config,
            pinned_content: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store computation result
    pub async fn store_computation_result(
        &self,
        result_id: Uuid,
        data: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        if data.len() as u64 > self.config.max_file_size {
            return Err(anyhow::anyhow!("Data exceeds maximum file size"));
        }

        // Add to IPFS
        let cursor = Cursor::new(data);
        let response = self
            .client
            .add(cursor)
            .await
            .context("Failed to add to IPFS")?;

        let cid = response.hash;

        // Pin if configured
        if self.config.pin_content {
            self.client
                .pin_add(&cid, false)
                .await
                .context("Failed to pin content")?;

            // Track pinned content
            let pinned = PinnedContent {
                cid: cid.clone(),
                content_type: ContentType::ComputationResult,
                size: data.len() as u64,
                pinned_at: chrono::Utc::now(),
                metadata,
            };

            self.pinned_content
                .write()
                .await
                .insert(result_id.to_string(), pinned);
        }

        Ok(cid)
    }

    /// Retrieve computation result
    pub async fn retrieve_computation_result(&self, cid: &str) -> Result<Vec<u8>> {
        let stream = self
            .client
            .cat(cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .context("Failed to retrieve from IPFS")?;

        Ok(stream)
    }

    /// Store neuron model
    pub async fn store_neuron_model(
        &self,
        neuron_id: Uuid,
        model_data: &[u8],
        version: u32,
    ) -> Result<String> {
        let mut metadata = HashMap::new();
        metadata.insert("neuron_id".to_string(), neuron_id.to_string());
        metadata.insert("version".to_string(), version.to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        // Create wrapper with metadata
        let model_package = ModelPackage {
            neuron_id,
            version,
            data: model_data.to_vec(),
            metadata: metadata.clone(),
        };

        let serialized = serde_json::to_vec(&model_package)?;

        // Add to IPFS
        let cursor = Cursor::new(serialized);
        let response = self.client.add(cursor).await?;
        let cid = response.hash;

        // Pin the model
        if self.config.pin_content {
            self.client.pin_add(&cid, false).await?;

            let pinned = PinnedContent {
                cid: cid.clone(),
                content_type: ContentType::NeuronModel,
                size: serialized.len() as u64,
                pinned_at: chrono::Utc::now(),
                metadata,
            };

            self.pinned_content
                .write()
                .await
                .insert(format!("{}_v{}", neuron_id, version), pinned);
        }

        Ok(cid)
    }

    /// List pinned content
    pub async fn list_pinned(&self) -> Result<Vec<PinnedContent>> {
        let pinned = self.pinned_content.read().await;
        Ok(pinned.values().cloned().collect())
    }

    /// Garbage collect unpinned content
    pub async fn garbage_collect(&self) -> Result<()> {
        self.client
            .repo_gc()
            .await
            .context("Failed to run garbage collection")?;
        Ok(())
    }

    /// Get content statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let repo_stat = self.client.stats_repo().await?;
        let pinned = self.pinned_content.read().await;

        Ok(StorageStats {
            repo_size: repo_stat.repo_size,
            num_objects: repo_stat.num_objects,
            pinned_count: pinned.len() as u64,
            total_pinned_size: pinned.values().map(|p| p.size).sum(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelPackage {
    neuron_id: Uuid,
    version: u32,
    data: Vec<u8>,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub repo_size: u64,
    pub num_objects: u64,
    pub pinned_count: u64,
    pub total_pinned_size: u64,
}

// ============ Decentralized Storage Manager ============

pub struct DecentralizedStorage {
    ipfs: Option<IPFSStorage>,
    arweave: Option<ArweaveStorage>,
    filecoin: Option<FilecoinStorage>,
    redundancy_level: u32,
    storage_proofs: Arc<RwLock<HashMap<String, StorageProof>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    pub content_hash: H256,
    pub storage_locations: Vec<StorageLocation>,
    pub proof_type: ProofType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub expiry: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    pub provider: StorageProvider,
    pub identifier: String,
    pub replicas: u32,
    pub cost: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    IPFS,
    Arweave,
    Filecoin,
    Sia,
    Storj,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    ProofOfReplication,
    ProofOfSpacetime,
    SimpleHash,
}

impl DecentralizedStorage {
    pub fn new(redundancy_level: u32) -> Self {
        Self {
            ipfs: None,
            arweave: None,
            filecoin: None,
            redundancy_level,
            storage_proofs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add IPFS backend
    pub fn with_ipfs(mut self, config: IPFSConfig) -> Result<Self> {
        self.ipfs = Some(IPFSStorage::new(config)?);
        Ok(self)
    }

    /// Store data with redundancy across multiple providers
    pub async fn store_with_redundancy(
        &self,
        data: &[u8],
        content_type: &str,
    ) -> Result<StorageProof> {
        let mut storage_locations = Vec::new();

        // Calculate content hash
        let content_hash = self.calculate_hash(data);

        // Store on IPFS
        if let Some(ref ipfs) = self.ipfs {
            let cid = ipfs
                .store_computation_result(Uuid::new_v4(), data, HashMap::new())
                .await?;

            storage_locations.push(StorageLocation {
                provider: StorageProvider::IPFS,
                identifier: cid,
                replicas: ipfs.config.replication_factor,
                cost: None,
            });
        }

        // Store on Arweave (if available)
        if let Some(ref arweave) = self.arweave {
            let tx_id = arweave.store(data).await?;
            storage_locations.push(StorageLocation {
                provider: StorageProvider::Arweave,
                identifier: tx_id,
                replicas: 1, // Arweave has permanent storage
                cost: Some(arweave.calculate_cost(data.len())),
            });
        }

        // Store on Filecoin (if available)
        if let Some(ref filecoin) = self.filecoin {
            let deal_id = filecoin.store_with_deal(data).await?;
            storage_locations.push(StorageLocation {
                provider: StorageProvider::Filecoin,
                identifier: deal_id,
                replicas: self.redundancy_level,
                cost: Some(filecoin.calculate_cost(data.len())),
            });
        }

        // Create storage proof
        let proof = StorageProof {
            content_hash,
            storage_locations,
            proof_type: ProofType::SimpleHash,
            timestamp: chrono::Utc::now(),
            expiry: None,
        };

        // Store proof
        self.storage_proofs
            .write()
            .await
            .insert(content_hash.to_string(), proof.clone());

        Ok(proof)
    }

    /// Retrieve data from any available provider
    pub async fn retrieve(&self, content_hash: &H256) -> Result<Vec<u8>> {
        let proofs = self.storage_proofs.read().await;
        let proof = proofs
            .get(&content_hash.to_string())
            .ok_or_else(|| anyhow::anyhow!("Storage proof not found"))?;

        // Try each storage location
        for location in &proof.storage_locations {
            match location.provider {
                StorageProvider::IPFS => {
                    if let Some(ref ipfs) = self.ipfs {
                        match ipfs.retrieve_computation_result(&location.identifier).await {
                            Ok(data) => {
                                // Verify hash
                                if self.calculate_hash(&data) == *content_hash {
                                    return Ok(data);
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to retrieve from IPFS: {}", e);
                            }
                        }
                    }
                }
                _ => {
                    // Other providers not implemented yet
                    tracing::warn!("Provider {:?} not implemented", location.provider);
                }
            }
        }

        Err(anyhow::anyhow!("Failed to retrieve from any provider"))
    }

    /// Calculate content hash
    fn calculate_hash(&self, data: &[u8]) -> H256 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        H256::from_slice(&hasher.finalize())
    }

    /// Verify storage proof
    pub async fn verify_storage(&self, content_hash: &H256) -> Result<bool> {
        let proofs = self.storage_proofs.read().await;
        let proof = proofs
            .get(&content_hash.to_string())
            .ok_or_else(|| anyhow::anyhow!("Storage proof not found"))?;

        // Check if at least one location is accessible
        for location in &proof.storage_locations {
            if self.check_availability(&location).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if storage location is available
    async fn check_availability(&self, location: &StorageLocation) -> Result<bool> {
        match location.provider {
            StorageProvider::IPFS => {
                if let Some(ref ipfs) = self.ipfs {
                    // Try to stat the object
                    match ipfs.client.object_stat(&location.identifier).await {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false), // Not implemented
        }
    }
}

// ============ Arweave Storage (Placeholder) ============

struct ArweaveStorage {
    gateway_url: String,
}

impl ArweaveStorage {
    async fn store(&self, _data: &[u8]) -> Result<String> {
        // Placeholder implementation
        Ok("ar://mock-transaction-id".to_string())
    }

    fn calculate_cost(&self, size: usize) -> u64 {
        // Mock cost calculation
        (size as u64 / 1024) * 10 // 10 units per KB
    }
}

// ============ Filecoin Storage (Placeholder) ============

struct FilecoinStorage {
    lotus_api_url: String,
}

impl FilecoinStorage {
    async fn store_with_deal(&self, _data: &[u8]) -> Result<String> {
        // Placeholder implementation
        Ok("fil://mock-deal-id".to_string())
    }

    fn calculate_cost(&self, size: usize) -> u64 {
        // Mock cost calculation
        (size as u64 / 1024 / 1024) * 100 // 100 units per MB
    }
}

// ============ Storage Index ============

pub struct StorageIndex {
    content_mapping: Arc<RwLock<HashMap<Uuid, Vec<H256>>>>,
    metadata_index: Arc<RwLock<HashMap<String, Vec<H256>>>>,
}

impl StorageIndex {
    pub fn new() -> Self {
        Self {
            content_mapping: Arc::new(RwLock::new(HashMap::new())),
            metadata_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Index content for a neuron
    pub async fn index_neuron_content(
        &self,
        neuron_id: Uuid,
        content_hash: H256,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        // Add to content mapping
        self.content_mapping
            .write()
            .await
            .entry(neuron_id)
            .or_insert_with(Vec::new)
            .push(content_hash);

        // Index metadata
        let mut index = self.metadata_index.write().await;
        for (key, value) in metadata {
            let index_key = format!("{}:{}", key, value);
            index
                .entry(index_key)
                .or_insert_with(Vec::new)
                .push(content_hash);
        }

        Ok(())
    }

    /// Query content by metadata
    pub async fn query_by_metadata(&self, key: &str, value: &str) -> Result<Vec<H256>> {
        let index_key = format!("{}:{}", key, value);
        let index = self.metadata_index.read().await;

        Ok(index.get(&index_key).cloned().unwrap_or_default())
    }

    /// Get all content for a neuron
    pub async fn get_neuron_content(&self, neuron_id: Uuid) -> Result<Vec<H256>> {
        let mapping = self.content_mapping.read().await;
        Ok(mapping.get(&neuron_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_proof_creation() {
        let storage = DecentralizedStorage::new(3);

        let data = b"test computation result";
        let content_hash = storage.calculate_hash(data);

        assert_eq!(content_hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_ipfs_config() {
        let config = IPFSConfig::default();
        assert_eq!(config.api_url, "http://localhost:5001");
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
    }
}
