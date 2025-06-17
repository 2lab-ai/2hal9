//! Embeddings generation for semantic memory search

use crate::Result;

/// Simple embedding generator (placeholder for real implementation)
pub struct EmbeddingGenerator {
    dimension: usize,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Generate embedding for text
    /// In a real implementation, this would use a model like BERT or Sentence Transformers
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // For now, generate a simple hash-based embedding
        let mut embedding = vec![0.0; self.dimension];

        // Simple character-based hashing
        for (i, ch) in text.chars().enumerate() {
            let idx = i % self.dimension;
            embedding[idx] += (ch as u32) as f32 / 1000.0;
        }

        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }

        Ok(embedding)
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
        let mut similarities: Vec<(String, f32)> = candidates
            .iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query, embedding);
                (id.clone(), similarity)
            })
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(top_k);

        similarities
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
