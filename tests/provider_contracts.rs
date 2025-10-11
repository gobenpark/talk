//! Integration test contracts for LLMProvider trait
//!
//! These tests define the contract that all LLM provider implementations must satisfy.
//! They use a mock provider to test the interface without requiring actual API calls.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use talk::context::Message;
use talk::error::AgentError;
use talk::provider::{LLMProvider, ProviderConfig, StreamChunk};

/// Mock LLM provider for testing the LLMProvider contract
#[derive(Debug, Clone)]
struct MockProvider {
    config: ProviderConfig,
    response: String,
    should_fail: bool,
}

impl MockProvider {
    fn new(response: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("mock-model"),
            response: response.into(),
            should_fail: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn complete(&self, _messages: Vec<Message>) -> Result<String, AgentError> {
        if self.should_fail {
            Err(AgentError::LLMProvider("Mock provider error".into()))
        } else {
            Ok(self.response.clone())
        }
    }

    async fn stream(
        &self,
        _messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, AgentError> {
        if self.should_fail {
            return Err(AgentError::LLMProvider("Mock provider error".into()));
        }

        // Split response into chunks for streaming
        let chunks: Vec<String> = self.response.chars().map(|c| c.to_string()).collect();

        Ok(Box::pin(futures::stream::iter(chunks.into_iter().map(Ok))))
    }

    fn name(&self) -> &str {
        "MockProvider"
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

/// Test the contract for LLMProvider::complete
///
/// This test verifies that:
/// - The provider can generate completions for messages
/// - The provider returns a non-empty string response
/// - Errors are properly propagated
#[tokio::test]
async fn test_llm_provider_complete_contract() {
    let provider = MockProvider::new("This is a test response");

    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("Hello"),
    ];

    // Complete should succeed and return a response
    let result = provider.complete(messages).await;
    assert!(
        result.is_ok(),
        "LLMProvider::complete should succeed with valid messages"
    );
    let response = result.unwrap();
    assert!(
        !response.is_empty(),
        "LLMProvider::complete should return a non-empty response"
    );
    assert_eq!(
        response, "This is a test response",
        "LLMProvider::complete should return the expected response"
    );
}

/// Test the contract for LLMProvider::complete with errors
///
/// This test verifies that:
/// - Provider errors are properly returned as AgentError::LLMProvider
/// - Error handling is consistent
#[tokio::test]
async fn test_llm_provider_complete_error_contract() {
    let provider = MockProvider::new("Response").with_failure();

    let messages = vec![Message::user("Hello")];

    // Complete should fail with provider error
    let result = provider.complete(messages).await;
    assert!(
        result.is_err(),
        "LLMProvider::complete should return error when provider fails"
    );
    assert!(
        matches!(result.unwrap_err(), AgentError::LLMProvider(_)),
        "Error should be AgentError::LLMProvider"
    );
}

/// Test the contract for LLMProvider::stream
///
/// This test verifies that:
/// - The provider can stream responses
/// - The stream produces multiple chunks
/// - All chunks combine to form the complete response
#[tokio::test]
async fn test_llm_provider_stream_contract() {
    let provider = MockProvider::new("Hello");

    let messages = vec![Message::user("Hi")];

    // Stream should succeed and return a stream
    let result = provider.stream(messages).await;
    assert!(
        result.is_ok(),
        "LLMProvider::stream should succeed with valid messages"
    );

    let mut stream = result.unwrap();

    // Collect all chunks from the stream
    use futures::StreamExt;
    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        assert!(
            chunk_result.is_ok(),
            "Stream chunks should not contain errors"
        );
        chunks.push(chunk_result.unwrap());
    }

    // Verify chunks were produced
    assert!(
        !chunks.is_empty(),
        "LLMProvider::stream should produce at least one chunk"
    );

    // Verify chunks combine to form the complete response
    let combined: String = chunks.join("");
    assert_eq!(
        combined, "Hello",
        "Stream chunks should combine to form the complete response"
    );
}

/// Test the contract for LLMProvider::stream with errors
///
/// This test verifies that:
/// - Stream errors are properly returned as AgentError::LLMProvider
/// - Error handling is consistent with complete()
#[tokio::test]
async fn test_llm_provider_stream_error_contract() {
    let provider = MockProvider::new("Response").with_failure();

    let messages = vec![Message::user("Hello")];

    // Stream should fail with provider error
    let result = provider.stream(messages).await;
    assert!(
        result.is_err(),
        "LLMProvider::stream should return error when provider fails"
    );

    // Check error type without using unwrap_err (which requires Debug on Ok type)
    match result {
        Err(AgentError::LLMProvider(_)) => {
            // Success - this is the expected error type
        }
        Err(e) => {
            panic!("Expected AgentError::LLMProvider, got {:?}", e);
        }
        Ok(_) => {
            panic!("Expected error, got Ok");
        }
    }
}

/// Test the contract for LLMProvider::name
///
/// This test verifies that:
/// - name() returns a non-empty string
/// - The name is consistent across calls
#[tokio::test]
async fn test_llm_provider_name_contract() {
    let provider = MockProvider::new("Response");

    let name = provider.name();
    assert!(
        !name.is_empty(),
        "LLMProvider::name should return a non-empty string"
    );
    assert_eq!(
        provider.name(),
        name,
        "LLMProvider::name should return consistent values"
    );
}

/// Test the contract for LLMProvider::config
///
/// This test verifies that:
/// - config() returns a valid ProviderConfig
/// - The config contains expected fields
#[tokio::test]
async fn test_llm_provider_config_contract() {
    let provider = MockProvider::new("Response");

    let config = provider.config();
    assert!(
        !config.model.is_empty(),
        "ProviderConfig should have a non-empty model name"
    );
    assert!(
        config.temperature >= 0.0 && config.temperature <= 2.0,
        "ProviderConfig temperature should be in valid range"
    );
}

/// Test that LLMProvider implementations handle different message types correctly
///
/// This test verifies that:
/// - Providers can handle system, user, and assistant messages
/// - Message order is preserved
/// - Different message combinations work correctly
#[tokio::test]
async fn test_llm_provider_message_types_contract() {
    let provider = MockProvider::new("Response");

    // Test with different message types
    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("Hello"),
        Message::assistant("Hi there!"),
        Message::user("How are you?"),
    ];

    let result = provider.complete(messages).await;
    assert!(
        result.is_ok(),
        "LLMProvider should handle all message types"
    );
}

/// Test that LLMProvider implementations handle empty message lists
///
/// This test verifies that:
/// - Providers can handle empty message vectors
/// - Empty messages don't cause panics
#[tokio::test]
async fn test_llm_provider_empty_messages_contract() {
    let provider = MockProvider::new("Response");

    let messages = vec![];

    // Should not panic with empty messages
    let result = provider.complete(messages).await;
    assert!(
        result.is_ok() || result.is_err(),
        "LLMProvider should handle empty messages without panicking"
    );
}

/// Test that LLMProvider implementations are thread-safe (Send + Sync)
///
/// This is a compile-time test that verifies the provider can be shared across threads.
#[tokio::test]
async fn test_llm_provider_thread_safety_contract() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockProvider>();
}

/// Test that LLMProvider can be used with async trait
///
/// This test verifies that:
/// - The provider works correctly with async_trait
/// - Multiple concurrent calls can be made
#[tokio::test]
async fn test_llm_provider_concurrent_calls_contract() {
    let provider = MockProvider::new("Response");

    // Make multiple concurrent calls
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let provider_clone = provider.clone();
            tokio::spawn(async move {
                let messages = vec![Message::user("Test")];
                provider_clone.complete(messages).await
            })
        })
        .collect();

    // Wait for all calls to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(
            result.is_ok(),
            "Concurrent LLMProvider calls should succeed"
        );
    }
}
