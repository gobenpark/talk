// Semantic embedding module for text similarity
//
// This module provides sentence embedding functionality using rust-bert's
// sentence-transformers models for semantic similarity matching.

#[cfg(feature = "semantic-matching")]
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Sentence embedder for semantic similarity
#[cfg(feature = "semantic-matching")]
pub struct SentenceEmbedder {
    model: Arc<SentenceEmbeddingsModel>,
    cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
}

#[cfg(feature = "semantic-matching")]
impl SentenceEmbedder {
    /// Create a new sentence embedder with the default model
    ///
    /// Uses `all-MiniLM-L6-v2` model which provides:
    /// - 384-dimensional embeddings
    /// - Fast inference (~50ms per sentence)
    /// - Good balance between speed and quality
    ///
    /// # Note
    /// First initialization will download the model (~90MB) and may take 1-2 seconds
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing sentence embedder with all-MiniLM-L6-v2 model");

        let model = SentenceEmbeddingsBuilder::remote(
            SentenceEmbeddingsModelType::AllMiniLmL6V2,
        )
        .create_model()?;

        Ok(Self {
            model: Arc::new(model),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Generate embedding for a single text
    ///
    /// Results are cached to avoid redundant computation for repeated texts
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(embedding) = cache.get(text) {
                debug!(text_len = text.len(), "Embedding retrieved from cache");
                return Ok(embedding.clone());
            }
        }

        // Generate new embedding
        debug!(text_len = text.len(), "Generating new embedding");
        let embeddings = self.model.encode(&[text])?;

        if embeddings.is_empty() {
            return Err("Failed to generate embedding".into());
        }

        let embedding = embeddings[0].clone();

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    /// Calculate cosine similarity between two texts
    ///
    /// Returns a value between 0.0 (completely different) and 1.0 (identical)
    pub fn similarity(&self, text1: &str, text2: &str) -> Result<f32, Box<dyn std::error::Error>> {
        let emb1 = self.embed(text1)?;
        let emb2 = self.embed(text2)?;

        Ok(cosine_similarity(&emb1, &emb2))
    }

    /// Clear the embedding cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        info!("Embedding cache cleared");
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

/// Calculate cosine similarity between two vectors
///
/// # Formula
/// cosine_similarity(A, B) = (A · B) / (||A|| * ||B||)
///
/// # Returns
/// Value between -1.0 and 1.0, where:
/// - 1.0 means identical direction
/// - 0.0 means orthogonal (no similarity)
/// - -1.0 means opposite direction
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Stub implementation when semantic-matching feature is disabled
#[cfg(not(feature = "semantic-matching"))]
pub struct SentenceEmbedder;

#[cfg(not(feature = "semantic-matching"))]
impl SentenceEmbedder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Err("semantic-matching feature is not enabled".into())
    }

    pub fn embed(&self, _text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        Err("semantic-matching feature is not enabled".into())
    }

    pub fn similarity(&self, _text1: &str, _text2: &str) -> Result<f32, Box<dyn std::error::Error>> {
        Err("semantic-matching feature is not enabled".into())
    }

    pub fn clear_cache(&self) {}

    pub fn cache_size(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[cfg(feature = "semantic-matching")]
    #[test]
    fn test_embedder_creation() {
        let embedder = SentenceEmbedder::new();
        assert!(embedder.is_ok());
    }

    #[cfg(feature = "semantic-matching")]
    #[test]
    fn test_embedding_generation() {
        let embedder = SentenceEmbedder::new().unwrap();
        let embedding = embedder.embed("Hello world");
        assert!(embedding.is_ok());
        let emb = embedding.unwrap();
        assert_eq!(emb.len(), 384); // all-MiniLM-L6-v2 produces 384-dim vectors
    }

    #[cfg(feature = "semantic-matching")]
    #[test]
    fn test_similarity_calculation() {
        let embedder = SentenceEmbedder::new().unwrap();
        let sim = embedder.similarity("pricing information", "cost details");
        assert!(sim.is_ok());
        let similarity = sim.unwrap();
        assert!(similarity > 0.5); // Should be reasonably similar
        assert!(similarity <= 1.0);
    }

    #[cfg(feature = "semantic-matching")]
    #[test]
    fn test_embedding_cache() {
        let embedder = SentenceEmbedder::new().unwrap();

        // First call - cache miss
        let _ = embedder.embed("test text").unwrap();
        assert_eq!(embedder.cache_size(), 1);

        // Second call - cache hit
        let _ = embedder.embed("test text").unwrap();
        assert_eq!(embedder.cache_size(), 1);

        // Different text - cache miss
        let _ = embedder.embed("different text").unwrap();
        assert_eq!(embedder.cache_size(), 2);

        // Clear cache
        embedder.clear_cache();
        assert_eq!(embedder.cache_size(), 0);
    }
}
