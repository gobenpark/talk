//! Simple agent example demonstrating basic guideline usage
//!
//! This example creates an agent with a few behavioral guidelines and processes
//! user messages to demonstrate the guideline matching system.
//!
//! Run with: cargo run --example simple_agent

use std::collections::HashMap;
use std::time::Duration;
use talk::{
    Agent, AgentConfig, Guideline, GuidelineAction, GuidelineCondition, GuidelineId, LLMProvider, LogLevel, Message, OpenAIProvider, ProviderConfig
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let provider = OpenAIProvider::new("");

    // Create agent with mock provider
    let mut agent = Agent::builder()
        .name("Customer Support Agent")
        .description("A helpful customer support assistant")
        .provider(Box::new(provider))
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

    let sample_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("sample".to_string()),
        action: GuidelineAction {
            response_template: "This is a sample guideline response.".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 1,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };
    agent.add_guideline(sample_guideline).await?;

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
