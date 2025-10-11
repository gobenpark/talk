//! Simple agent example demonstrating basic guideline usage
//!
//! This example creates an agent with a few behavioral guidelines and processes
//! user messages to demonstrate the guideline matching system.
//!
//! Run with: cargo run --example simple_agent

use talk::{
    Agent, AgentConfig, Guideline, GuidelineAction, GuidelineCondition, GuidelineId, LogLevel,
    LlmProvider, Message, ProviderConfig,
};
use std::collections::HashMap;
use std::time::Duration;

/// Mock LLM provider for demonstration purposes
struct MockProvider {
    config: ProviderConfig,
}

impl MockProvider {
    fn new() -> Self {
        Self {
            config: ProviderConfig {
                model: "mock-model".to_string(),
                temperature: 0.7,
                max_tokens: Some(1000),
                top_p: None,
                frequency_penalty: None,
                presence_penalty: None,
            },
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
    ) -> Result<String, talk::AgentError> {
        // In a real implementation, this would call an actual LLM API
        // For this example, we'll generate a simple response
        let last_message = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("Hello");

        Ok(format!(
            "I'm a helpful assistant. You said: '{}'. How can I help you today?",
            last_message
        ))
    }

    async fn stream(
        &self,
        _messages: Vec<Message>,
    ) -> Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Result<String, talk::AgentError>> + Send>,
        >,
        talk::AgentError,
    > {
        unimplemented!("Streaming not implemented in mock provider")
    }

    fn name(&self) -> &str {
        "MockProvider"
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simple Agent Example\n");
    println!("This example demonstrates basic guideline matching with a customer support agent.\n");

    // Create agent with mock provider
    let mut agent = Agent::builder()
        .name("Customer Support Agent")
        .description("A helpful customer support assistant")
        .provider(Box::new(MockProvider::new()))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: LogLevel::Info,
        })
        .build()?;

    println!("✓ Agent created: Customer Support Agent\n");

    // Add pricing guideline (high priority, literal match)
    let pricing_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
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
    println!("✓ Added guideline: pricing (literal match, priority 10)");

    // Add support guideline (medium priority, regex match)
    let support_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"help|support|assist".to_string()),
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
    println!("✓ Added guideline: help/support (regex match, priority 5)");

    // Add greeting guideline (low priority, regex match)
    let greeting_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"^(hi|hello|hey|greetings)".to_string()),
        action: GuidelineAction {
            response_template: "Hello! 👋 Welcome to our support chat. How can I help you today?".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 3,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(greeting_guideline).await?;
    println!("✓ Added guideline: greeting (regex match, priority 3)\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("✓ Session created: {}\n", session_id);

    // Test different messages
    let test_messages = vec![
        "Hello!",
        "What is your pricing?",
        "I need help with my account",
        "Tell me about your company",
    ];

    println!("=== Testing Guideline Matching ===\n");

    for (i, message) in test_messages.iter().enumerate() {
        println!("{}. User: {}", i + 1, message);

        let response = agent
            .process_message(session_id, message.to_string())
            .await?;

        println!("   Agent: {}", response.message);

        if let Some(matched) = &response.matched_guideline {
            println!(
                "   [Matched guideline with score: {:.2}]",
                matched.relevance_score
            );
        }

        if let Some(explanation) = &response.explanation {
            println!(
                "   [Confidence: {:.2}, {} guidelines considered]",
                explanation.confidence,
                explanation.guideline_matches.len()
            );
        }

        println!();
    }

    // Show session summary
    let session = agent.get_session(&session_id).await?;
    if let Some(s) = session {
        println!("=== Session Summary ===");
        println!("Messages exchanged: {}", s.context.messages.len());
        println!("Session status: {:?}", s.status);
        println!("Last updated: {}", s.updated_at);
    }

    // End session
    agent.end_session(&session_id).await?;
    println!("\n✓ Session ended successfully");

    Ok(())
}
