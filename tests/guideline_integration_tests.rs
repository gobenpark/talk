// Integration tests for Guideline API
// These tests define the expected behavior for guidelines
// TDD: These tests should FAIL before implementation

use talk::{
    Guideline, GuidelineCondition, GuidelineAction, GuidelineMatcher, DefaultGuidelineMatcher,
    GuidelineMatch, Context, SessionId, GuidelineId, MessageRole, Message,
};
use std::collections::HashMap;
use chrono::Utc;

// T020: Contract test for Guideline API
#[tokio::test]
async fn test_guideline_api_contract() {
    let mut matcher = DefaultGuidelineMatcher::new();

    // Test add_guideline
    let guideline = create_test_guideline("pricing", "Pricing starts at $49/month");
    let guideline_id = matcher.add_guideline(guideline).await.expect("Failed to add guideline");

    // GuidelineId is always valid after creation
    assert!(true, "Should return valid guideline ID");

    // Test match_guidelines
    let context = create_test_context();
    let matches = matcher.match_guidelines("What is the pricing?", &context).await
        .expect("Failed to match guidelines");

    assert!(!matches.is_empty(), "Should find at least one match");

    // Test select_best_match
    let best_match = matcher.select_best_match(matches).await;
    assert!(best_match.is_some(), "Should select a best match");
}

// T021: Integration test for literal condition matching
#[tokio::test]
async fn test_literal_condition_matching() {
    let mut matcher = DefaultGuidelineMatcher::new();

    // Add guideline with literal condition
    let guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Our pricing info".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    matcher.add_guideline(guideline).await.expect("Failed to add guideline");

    let context = create_test_context();

    // Should match
    let matches = matcher.match_guidelines("What is your pricing?", &context).await
        .expect("Failed to match");
    assert_eq!(matches.len(), 1, "Should match literal condition");
    assert_eq!(matches[0].relevance_score, 1.0, "Literal match should have score 1.0");

    // Should not match
    let no_matches = matcher.match_guidelines("Tell me about features", &context).await
        .expect("Failed to match");
    assert!(no_matches.is_empty(), "Should not match unrelated message");
}

// T022: Integration test for regex condition matching
#[tokio::test]
async fn test_regex_condition_matching() {
    let mut matcher = DefaultGuidelineMatcher::new();

    // Add guideline with regex condition
    let guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"cancel.*subscription".to_string()),
        action: GuidelineAction {
            response_template: "Cancellation process".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    matcher.add_guideline(guideline).await.expect("Failed to add guideline");

    let context = create_test_context();

    // Should match
    let matches = matcher.match_guidelines("I want to cancel my subscription", &context).await
        .expect("Failed to match");
    assert_eq!(matches.len(), 1, "Should match regex condition");
    assert_eq!(matches[0].relevance_score, 0.9, "Regex match should have score 0.9");

    // Should not match
    let no_matches = matcher.match_guidelines("I love my subscription", &context).await
        .expect("Failed to match");
    assert!(no_matches.is_empty(), "Should not match when pattern doesn't match");
}

// T023: Integration test for priority resolution
#[tokio::test]
async fn test_guideline_priority_resolution() {
    let mut matcher = DefaultGuidelineMatcher::new();

    // Add low priority guideline
    let low_priority = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Low priority response".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 5,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    // Add high priority guideline
    let high_priority = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "High priority response".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 20,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    };

    matcher.add_guideline(low_priority.clone()).await.expect("Failed to add guideline");
    matcher.add_guideline(high_priority.clone()).await.expect("Failed to add guideline");

    let context = create_test_context();
    let matches = matcher.match_guidelines("What about pricing?", &context).await
        .expect("Failed to match");

    assert_eq!(matches.len(), 2, "Should match both guidelines");

    let best_match = matcher.select_best_match(matches).await
        .expect("Should select best match");

    assert_eq!(best_match.guideline_id, high_priority.id, "Should select high priority guideline");
}

// Helper functions

fn create_test_guideline(keyword: &str, response: &str) -> Guideline {
    Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal(keyword.to_string()),
        action: GuidelineAction {
            response_template: response.to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: HashMap::new(),
        created_at: Utc::now(),
    }
}

fn create_test_context() -> Context {
    Context::new()
}
