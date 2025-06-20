//! Embeddings generation for semantic memory search

use crate::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Advanced embedding generator with caching and improved algorithms
pub struct EmbeddingGenerator {
    dimension: usize,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new(dimension: usize) -> Self {
        Self { 
            dimension,
            cache: Arc::new(RwLock::new(HashMap::with_capacity(10000))),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Generate embedding for text with enhanced algorithm and caching
    /// TODO: Integrate with real embedding model (BERT/Sentence Transformers)
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(text) {
                *self.cache_hits.write().await += 1;
                return Ok(cached.clone());
            }
        }
        
        *self.cache_misses.write().await += 1;
        
        // Enhanced embedding generation with n-gram features
        let mut embedding = vec![0.0; self.dimension];
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        // Word-level features
        for (word_idx, word) in words.iter().enumerate() {
            // Unigram features
            let hash = Self::hash_string(word) as usize;
            embedding[hash % self.dimension] += 1.0;
            
            // Bigram features
            if word_idx > 0 {
                let bigram = format!("{} {}", words[word_idx - 1], word);
                let bigram_hash = Self::hash_string(&bigram) as usize;
                embedding[(bigram_hash % self.dimension + self.dimension / 4) % self.dimension] += 0.7;
            }
            
            // Character n-grams (3-grams)
            if word.len() >= 3 {
                for i in 0..word.len() - 2 {
                    let trigram = &word[i..i + 3];
                    let trigram_hash = Self::hash_string(trigram) as usize;
                    embedding[(trigram_hash % self.dimension + self.dimension / 2) % self.dimension] += 0.5;
                }
            }
        }
        
        // Apply TF-IDF-like weighting
        let doc_length = words.len() as f32;
        if doc_length > 0.0 {
            for val in &mut embedding {
                *val = (*val / doc_length.sqrt()).tanh(); // Smooth normalization
            }
        }
        
        // L2 Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        // Cache the result
        {
            let mut cache = self.cache.write().await;
            // Implement LRU eviction if cache is too large
            if cache.len() >= 10000 {
                // Simple eviction: remove random entry
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }
            cache.insert(text.to_string(), embedding.clone());
        }
        
        Ok(embedding)
    }
    
    /// Fast string hashing using FNV-1a algorithm
    fn hash_string(s: &str) -> u64 {
        const FNV_PRIME: u64 = 0x100000001b3;
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        
        s.bytes().fold(FNV_OFFSET, |hash, byte| {
            (hash ^ byte as u64).wrapping_mul(FNV_PRIME)
        })
    }
    
    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }
    
    /// Find most similar embeddings
    pub fn find_similar(
        query: &[f32],
        candidates: &[(String, Vec<f32>)],
        top_k: usize,
    ) -> Vec<(String, f32)> {
        use std::collections::BinaryHeap;
        use std::cmp::Ordering;
        
        #[derive(Clone)]
        struct SimilarityScore {
            index: usize,
            score: f32,
        }
        
        impl PartialEq for SimilarityScore {
            fn eq(&self, other: &Self) -> bool {
                self.score == other.score
            }
        }
        
        impl Eq for SimilarityScore {}
        
        impl PartialOrd for SimilarityScore {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        impl Ord for SimilarityScore {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse order for min-heap behavior
                other.score.partial_cmp(&self.score).unwrap_or(Ordering::Equal)
            }
        }
        
        let mut heap = BinaryHeap::with_capacity(top_k + 1);
        
        for (idx, (_, embedding)) in candidates.iter().enumerate() {
            let similarity = Self::cosine_similarity(query, embedding);
            heap.push(SimilarityScore { index: idx, score: similarity });
            
            if heap.len() > top_k {
                heap.pop(); // Remove the lowest score
            }
        }
        
        // Extract results, maintaining original string references
        let mut results: Vec<(String, f32)> = heap
            .into_iter()
            .map(|item| (candidates[item.index].0.clone(), item.score))
            .collect();
            
        // Sort by descending similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        
        results
    }
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> (u64, u64, f64) {
        let hits = *self.cache_hits.read().await;
        let misses = *self.cache_misses.read().await;
        let total = hits + misses;
        let hit_rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };
        (hits, misses, hit_rate)
    }
    
    /// Clear the embedding cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        *self.cache_hits.write().await = 0;
        *self.cache_misses.write().await = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedding_generation() {
        let generator = EmbeddingGenerator::new(128);
        
        let embedding1 = generator.generate("Hello world").await.unwrap();
        let embedding2 = generator.generate("Hello world").await.unwrap();
        let embedding3 = generator.generate("Goodbye world").await.unwrap();
        
        assert_eq!(embedding1.len(), 128);
        assert_eq!(embedding2.len(), 128);
        assert_eq!(embedding3.len(), 128);
        
        // Same text should produce same embedding
        assert_eq!(embedding1, embedding2);
        
        // Different text should produce different embedding
        assert_ne!(embedding1, embedding3);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert_eq!(EmbeddingGenerator::cosine_similarity(&a, &b), 1.0);
        assert_eq!(EmbeddingGenerator::cosine_similarity(&a, &c), 0.0);
    }
}