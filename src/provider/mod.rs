//! LLM provider abstraction and implementations
//!
//! This module provides a trait-based abstraction for LLM providers,
//! allowing the agent to work with different LLM backends (OpenAI, Anthropic, etc.).

use crate::context::{Message, MessageRole};
use crate::error::AgentError;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Configuration for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Model name to use
    pub model: String,
    /// Temperature for response generation (0.0-2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Top-p sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
}

fn default_temperature() -> f32 {
    0.7
}

impl ProviderConfig {
    /// Create a new provider configuration with default values
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: default_temperature(),
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the top-p sampling parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    /// Set the frequency penalty
    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty.clamp(-2.0, 2.0));
        self
    }

    /// Set the presence penalty
    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty.clamp(-2.0, 2.0));
        self
    }
}

/// Type alias for streaming response chunks
pub type StreamChunk = std::result::Result<String, AgentError>;

/// Trait for LLM provider implementations
///
/// This trait defines the interface that all LLM providers must implement
/// to be compatible with the Talk agent framework.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a complete response for the given messages
    ///
    /// # Arguments
    ///
    /// * `messages` - Vector of messages representing the conversation history
    ///
    /// # Returns
    ///
    /// The generated response text or an error
    async fn complete(&self, messages: Vec<Message>) -> std::result::Result<String, AgentError>;

    /// Generate a streaming response for the given messages
    ///
    /// # Arguments
    ///
    /// * `messages` - Vector of messages representing the conversation history
    ///
    /// # Returns
    ///
    /// A stream of response chunks or an error
    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, AgentError>;

    /// Get the name of the provider
    ///
    /// # Returns
    ///
    /// A string slice containing the provider name
    fn name(&self) -> &str;

    /// Get the provider configuration
    ///
    /// # Returns
    ///
    /// A reference to the provider configuration
    fn config(&self) -> &ProviderConfig;
}

/// Convert Talk messages to the format expected by the provider
///
/// This is a helper function that can be used by provider implementations
/// to convert Talk's message format to their specific format.
pub fn messages_to_provider_format(messages: &[Message]) -> Vec<(MessageRole, String)> {
    messages
        .iter()
        .map(|m| (m.role, m.content.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new("gpt-5");
        assert_eq!(config.model, "gpt-5");
        assert_eq!(config.temperature, 0.7);
        assert!(config.max_tokens.is_none());
    }

    #[test]
    fn test_provider_config_with_temperature() {
        let config = ProviderConfig::new("gpt-5").with_temperature(0.5);
        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn test_provider_config_temperature_clamping() {
        let config1 = ProviderConfig::new("gpt-5").with_temperature(-0.5);
        assert_eq!(config1.temperature, 0.0);

        let config2 = ProviderConfig::new("gpt-5").with_temperature(3.0);
        assert_eq!(config2.temperature, 2.0);
    }

    #[test]
    fn test_provider_config_with_max_tokens() {
        let config = ProviderConfig::new("gpt-5").with_max_tokens(1000);
        assert_eq!(config.max_tokens, Some(1000));
    }

    #[test]
    fn test_provider_config_with_top_p() {
        let config = ProviderConfig::new("gpt-5").with_top_p(0.9);
        assert_eq!(config.top_p, Some(0.9));
    }

    #[test]
    fn test_provider_config_top_p_clamping() {
        let config1 = ProviderConfig::new("gpt-5").with_top_p(-0.1);
        assert_eq!(config1.top_p, Some(0.0));

        let config2 = ProviderConfig::new("gpt-5").with_top_p(1.5);
        assert_eq!(config2.top_p, Some(1.0));
    }

    #[test]
    fn test_provider_config_with_penalties() {
        let config = ProviderConfig::new("gpt-5")
            .with_frequency_penalty(0.5)
            .with_presence_penalty(0.3);

        assert_eq!(config.frequency_penalty, Some(0.5));
        assert_eq!(config.presence_penalty, Some(0.3));
    }

    #[test]
    fn test_provider_config_penalty_clamping() {
        let config1 = ProviderConfig::new("gpt-5")
            .with_frequency_penalty(-3.0)
            .with_presence_penalty(3.0);

        assert_eq!(config1.frequency_penalty, Some(-2.0));
        assert_eq!(config1.presence_penalty, Some(2.0));
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig::new("gpt-5")
            .with_temperature(0.8)
            .with_max_tokens(500);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.temperature, deserialized.temperature);
        assert_eq!(config.max_tokens, deserialized.max_tokens);
    }

    #[test]
    fn test_messages_to_provider_format() {
        let messages = vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let provider_messages = messages_to_provider_format(&messages);

        assert_eq!(provider_messages.len(), 3);
        assert_eq!(provider_messages[0].0, MessageRole::System);
        assert_eq!(provider_messages[0].1, "You are a helpful assistant");
        assert_eq!(provider_messages[1].0, MessageRole::User);
        assert_eq!(provider_messages[1].1, "Hello");
        assert_eq!(provider_messages[2].0, MessageRole::Assistant);
        assert_eq!(provider_messages[2].1, "Hi there!");
    }
}
