//! Semantic agent example demonstrating vector similarity matching
//!
//! This example shows how to use semantic matching with embeddings to match
//! user messages based on meaning rather than exact keywords.
//!
//! **Note**: Requires the `semantic-matching` feature to be enabled.
//!
//! Run with: cargo run --example semantic_agent --features semantic-matching

#[cfg(not(feature = "semantic-matching"))]
fn main() {
    println!("This example requires the 'semantic-matching' feature.");
    println!("Run with: cargo run --example semantic_agent --features semantic-matching");
}

#[cfg(feature = "semantic-matching")]
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "semantic-matching")]
use talk::{
    Agent, AgentConfig, DefaultGuidelineMatcher, Guideline, GuidelineAction, GuidelineCondition,
    GuidelineId, LLMProvider, LogLevel, Message, OpenAIProvider, ProviderConfig,
    SentenceEmbedder,
};

#[cfg(feature = "semantic-matching")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: You can replace this with a valid API key for real LLM responses
    let provider = OpenAIProvider::new(
        "",
    );

    println!("🤖 Semantic Matching Agent Example\n");
    println!("Initializing sentence embedder (this may take a moment)...");

    // Create sentence embedder for semantic matching
    let embedder = Arc::new(SentenceEmbedder::new()?);
    println!("✓ Embedder initialized\n");

    // Create matcher with semantic matching enabled
    let matcher = DefaultGuidelineMatcher::with_embedder(embedder.clone());

    // Build agent manually to use custom matcher
    let mut agent = Agent::builder()
        .name("Semantic Support Agent")
        .description("A customer support assistant with semantic understanding")
        .provider(Box::new(provider))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: LogLevel::Info,
        })
        .build()?;

    println!("✓ Agent created: Semantic Support Agent\n");

    // Add semantic pricing guideline
    // This will match queries about cost, price, fees, etc.
    let pricing_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Semantic {
            description: "pricing, cost, price, fee, payment, subscription, plan".to_string(),
            threshold: 0.7, // Match if similarity >= 70%
        },
        action: GuidelineAction {
            response_template: "Our pricing starts at $49/month for the Basic plan, $99/month for Pro, and $199/month for Enterprise. All plans include a 14-day free trial!".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(pricing_guideline).await?;
    println!("✓ Added semantic guideline: pricing (similarity threshold: 0.7)");

    // Add semantic support guideline
    let support_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Semantic {
            description: "help, support, assistance, question, issue, problem".to_string(),
            threshold: 0.7,
        },
        action: GuidelineAction {
            response_template: "I'm here to help! I can assist you with: pricing information, product features, technical support, and account management. What would you like to know?".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 5,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(support_guideline).await?;
    println!("✓ Added semantic guideline: support (similarity threshold: 0.7)");

    // Add literal greeting for comparison
    let greeting_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("hello".to_string()),
        action: GuidelineAction {
            response_template: "Hello! 👋 Welcome to our support chat. How can I help you today?"
                .to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 3,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(greeting_guideline).await?;
    println!("✓ Added literal guideline: greeting\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("✓ Session created: {}\n", session_id);

    // Test messages - semantic matching should work even with different wording
    let test_messages = vec![
        ("Hello!", "Literal match"),
        ("What's the cost?", "Semantic match - 'cost' → 'pricing'"),
        ("How much does it cost?", "Semantic match - different phrasing"),
        ("I need assistance", "Semantic match - 'assistance' → 'support'"),
        ("Can you help me?", "Semantic match - 'help' → 'support'"),
        ("What are your fees?", "Semantic match - 'fees' → 'pricing'"),
        ("Tell me about the weather", "No match - unrelated topic"),
    ];

    println!("=== Testing Semantic Matching ===\n");

    for (i, (message, expected)) in test_messages.iter().enumerate() {
        println!("{}. User: {}", i + 1, message);
        println!("   Expected: {}", expected);

        let response = agent
            .process_message(session_id, message.to_string())
            .await?;

        println!("   Agent: {}", response.message);

        if let Some(matched) = &response.matched_guideline {
            println!(
                "   ✓ Matched (relevance: {:.2}, semantic: {:.2})",
                matched.relevance_score, matched.semantic_score
            );
        }

        if let Some(explanation) = &response.explanation {
            println!(
                "   📊 Confidence: {:.2}, {} guidelines considered",
                explanation.confidence,
                explanation.guideline_matches.len()
            );
        }

        println!();
    }

    // Demonstrate similarity scores
    println!("=== Similarity Scores Demo ===\n");

    let test_pairs = vec![
        ("pricing", "cost"),
        ("pricing", "price"),
        ("pricing", "fees"),
        ("pricing", "weather"),
        ("help", "support"),
        ("help", "assistance"),
    ];

    for (text1, text2) in test_pairs {
        let similarity = embedder.similarity(text1, text2)?;
        println!(
            "'{}' vs '{}': {:.3} similarity",
            text1, text2, similarity
        );
    }

    // Show session summary
    let session = agent.get_session(&session_id).await?;
    if let Some(s) = session {
        println!("\n=== Session Summary ===");
        println!("Messages exchanged: {}", s.context.messages.len());
        println!("Session status: {:?}", s.status);
        println!("Last updated: {}", s.updated_at);
    }

    // End session
    agent.end_session(&session_id).await?;
    println!("\n✓ Session ended successfully");

    Ok(())
}
