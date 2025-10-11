// Integration tests for Agent with Guidelines
// TDD: These tests should FAIL before implementation

use talk::{
    Agent, AgentBuilder, Guideline, GuidelineCondition, GuidelineAction,
    SessionId, MessageRole, AgentConfig,
};
use std::time::Duration;
use std::collections::HashMap;
use chrono::Utc;

// T024: Integration test for fallback guideline when no match
#[tokio::test]
async fn test_fallback_guideline_when_no_match() {
    let mut agent = create_test_agent().await;

    // Add specific guideline
    let guideline = Guideline {
        id: talk::GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Pricing info".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };
    agent.add_guideline(guideline).await.expect("Failed to add guideline");

    // Create session
    let session_id = agent.create_session().await.expect("Failed to create session");

    // Process message that doesn't match any guideline - should use fallback
    let response = agent.process_message(session_id, "Tell me about your company".to_string()).await
        .expect("Failed to process message");

    // Fallback should provide some response
    assert!(!response.message.is_empty(), "Fallback should provide a response");
    assert!(response.matched_guideline.is_some(), "Should match fallback guideline");
}

// T025: End-to-end agent test with multiple guidelines
#[tokio::test]
async fn test_agent_with_multiple_guidelines() {
    let mut agent = create_test_agent().await;

    // Add multiple guidelines
    let pricing_guideline = Guideline {
        id: talk::GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Our pricing starts at $49/month for the basic plan.".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    let support_guideline = Guideline {
        id: talk::GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"help|support".to_string()),
        action: GuidelineAction {
            response_template: "How can I help you today?".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    agent.add_guideline(pricing_guideline.clone()).await.expect("Failed to add guideline");
    agent.add_guideline(support_guideline.clone()).await.expect("Failed to add guideline");

    // Create session
    let session_id = agent.create_session().await.expect("Failed to create session");

    // Test pricing guideline
    let response1 = agent.process_message(session_id, "What is your pricing?".to_string()).await
        .expect("Failed to process message");
    assert!(response1.message.contains("$49/month"), "Should respond with pricing info");
    assert_eq!(response1.matched_guideline.as_ref().unwrap().guideline_id, pricing_guideline.id);

    // Test support guideline
    let response2 = agent.process_message(session_id, "I need help with something".to_string()).await
        .expect("Failed to process message");
    assert!(response2.message.contains("How can I help"), "Should respond with support message");
    assert_eq!(response2.matched_guideline.as_ref().unwrap().guideline_id, support_guideline.id);

    // Verify session maintains context
    let session = agent.get_session(&session_id).await
        .expect("Failed to get session")
        .expect("Session should exist");
    assert_eq!(session.context.messages.len(), 4, "Should have 2 user + 2 agent messages");
}

// Helper function to create test agent
async fn create_test_agent() -> Agent {
    // Create a mock provider for testing
    let provider = create_mock_provider();

    Agent::builder()
        .name("Test Agent")
        .provider(Box::new(provider))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: talk::LogLevel::Debug,
        })
        .build()
        .expect("Failed to build agent")
}

// Mock provider for testing (doesn't call real LLM)
struct MockProvider;

#[async_trait::async_trait]
impl talk::LlmProvider for MockProvider {
    async fn complete(&self, _messages: Vec<talk::Message>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("Mock LLM response".to_string())
    }

    async fn stream(
        &self,
        _messages: Vec<talk::Message>
    ) -> Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
        Box<dyn std::error::Error + Send + Sync>
    > {
        unimplemented!("Stream not needed for tests")
    }

    fn name(&self) -> &str {
        "MockProvider"
    }

    fn config(&self) -> &talk::ProviderConfig {
        &talk::ProviderConfig {
            model: "mock".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout: Duration::from_secs(30),
        }
    }
}

fn create_mock_provider() -> MockProvider {
    MockProvider
}
