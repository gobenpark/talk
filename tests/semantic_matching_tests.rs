//! Integration tests for semantic matching functionality
//!
//! These tests verify that semantic matching using embeddings works correctly
//! and integrates properly with the agent system.

#[cfg(feature = "semantic-matching")]
mod semantic_tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use talk::{
        Context, DefaultGuidelineMatcher, Guideline, GuidelineAction, GuidelineCondition,
        GuidelineId, SentenceEmbedder, GuidelineMatcher, cosine_similarity,
    };

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001, "Identical vectors should have similarity ~1.0");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001, "Orthogonal vectors should have similarity ~0.0");
    }

    #[test]
    fn test_embedder_creation() {
        let result = SentenceEmbedder::new();
        assert!(result.is_ok(), "Should be able to create embedder");
    }

    #[test]
    fn test_embedding_generation() {
        let embedder = SentenceEmbedder::new().unwrap();
        let embedding = embedder.embed("Hello world");
        assert!(embedding.is_ok(), "Should generate embedding");

        let emb = embedding.unwrap();
        assert_eq!(emb.len(), 384, "all-MiniLM-L6-v2 produces 384-dim vectors");
    }

    #[test]
    fn test_similarity_calculation() {
        let embedder = SentenceEmbedder::new().unwrap();

        // Similar words should have high similarity
        let sim1 = embedder.similarity("price", "cost").unwrap();
        assert!(sim1 > 0.5, "Similar words should have >0.5 similarity, got {}", sim1);

        // Unrelated words should have low similarity
        let sim2 = embedder.similarity("price", "weather").unwrap();
        assert!(sim2 < 0.5, "Unrelated words should have <0.5 similarity, got {}", sim2);
    }

    #[test]
    fn test_embedding_cache() {
        let embedder = SentenceEmbedder::new().unwrap();

        // First call
        let _ = embedder.embed("test text").unwrap();
        assert_eq!(embedder.cache_size(), 1);

        // Second call - should hit cache
        let _ = embedder.embed("test text").unwrap();
        assert_eq!(embedder.cache_size(), 1);

        // Different text
        let _ = embedder.embed("different text").unwrap();
        assert_eq!(embedder.cache_size(), 2);

        // Clear cache
        embedder.clear_cache();
        assert_eq!(embedder.cache_size(), 0);
    }

    #[tokio::test]
    async fn test_semantic_guideline_matching() {
        let embedder = Arc::new(SentenceEmbedder::new().unwrap());
        let mut matcher = DefaultGuidelineMatcher::with_embedder(embedder);

        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Semantic {
                description: "pricing, cost, price, fee".to_string(),
                threshold: 0.7,
            },
            action: GuidelineAction {
                response_template: "Pricing info".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 10,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        matcher.add_guideline(guideline).await.unwrap();

        let context = Context::new();

        // Should match similar words
        let matches = matcher
            .match_guidelines("What's the cost?", &context)
            .await
            .unwrap();

        assert_eq!(matches.len(), 1, "Should match semantic guideline");
        assert!(matches[0].semantic_score > 0.7, "Should have high semantic score");
    }

    #[tokio::test]
    async fn test_semantic_below_threshold() {
        let embedder = Arc::new(SentenceEmbedder::new().unwrap());
        let mut matcher = DefaultGuidelineMatcher::with_embedder(embedder);

        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Semantic {
                description: "pricing, cost, price".to_string(),
                threshold: 0.8, // High threshold
            },
            action: GuidelineAction {
                response_template: "Pricing info".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 10,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        matcher.add_guideline(guideline).await.unwrap();

        let context = Context::new();

        // Unrelated message should not match
        let matches = matcher
            .match_guidelines("Tell me about the weather", &context)
            .await
            .unwrap();

        assert_eq!(matches.len(), 0, "Should not match unrelated message");
    }

    #[tokio::test]
    async fn test_hybrid_matching_priority() {
        let embedder = Arc::new(SentenceEmbedder::new().unwrap());
        let mut matcher = DefaultGuidelineMatcher::with_embedder(embedder);

        // Add literal guideline (high priority)
        let literal_guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("pricing".to_string()),
            action: GuidelineAction {
                response_template: "Literal pricing".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 20,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        let literal_id = literal_guideline.id;
        matcher.add_guideline(literal_guideline).await.unwrap();

        // Add semantic guideline (lower priority)
        let semantic_guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Semantic {
                description: "cost, price, fee".to_string(),
                threshold: 0.7,
            },
            action: GuidelineAction {
                response_template: "Semantic pricing".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 10,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        matcher.add_guideline(semantic_guideline).await.unwrap();

        let context = Context::new();

        // Exact word "pricing" should match both
        let matches = matcher
            .match_guidelines("What is your pricing?", &context)
            .await
            .unwrap();

        assert!(matches.len() >= 1, "Should match at least literal guideline");

        // Select best should prefer higher priority (literal)
        let best = matcher.select_best_match(matches).await.unwrap();
        assert_eq!(best.guideline_id, literal_id, "Should prefer higher priority");
    }
}

#[cfg(not(feature = "semantic-matching"))]
mod no_semantic {
    #[test]
    fn test_feature_disabled() {
        // Just verify compilation works without semantic-matching feature
        assert!(true);
    }
}
