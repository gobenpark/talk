//! Anthropic LLM provider implementation
//!
//! This module implements the LLMProvider trait for Anthropic's Claude models.

use crate::context::{Message, MessageRole};
use crate::error::AgentError;
use crate::provider::{LLMProvider, ProviderConfig, StreamChunk};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use tracing::info;

/// Anthropic LLM provider
pub struct AnthropicProvider {
    api_key: String,
    config: ProviderConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            config: ProviderConfig::new("claude-3-5-sonnet-20241022"),
        }
    }

    /// Create a new Anthropic provider from environment variable ANTHROPIC_API_KEY
    pub fn from_env() -> Result<Self, AgentError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AgentError::Configuration("ANTHROPIC_API_KEY environment variable not set".to_string())
        })?;

        Ok(Self::new(api_key))
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config = self.config.with_temperature(temperature);
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.config = self.config.with_max_tokens(max_tokens);
        self
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, messages: Vec<Message>) -> std::result::Result<String, AgentError> {
        use serde_json::json;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        info!(
            model = %self.config.model,
            message_count = messages.len(),
            "Requesting Anthropic completion"
        );

        // Convert Talk messages to Anthropic JSON format
        let mut anthropic_messages = Vec::new();
        let mut system_prompt = String::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    system_prompt = msg.content;
                }
                MessageRole::User => {
                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": msg.content
                    }));
                }
                MessageRole::Assistant => {
                    anthropic_messages.push(json!({
                        "role": "assistant",
                        "content": msg.content
                    }));
                }
                MessageRole::Tool => {
                    // Anthropic uses "user" role for tool results
                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": msg.content
                    }));
                }
            }
        }

        let messages_value = json!(anthropic_messages);

        // Build request - create new client each time since Client doesn't implement Clone
        let mut client_builder = anthropic_sdk::Client::new()
            .auth(&self.api_key)
            .model(&self.config.model)
            .messages(&messages_value)
            .max_tokens(self.config.max_tokens.unwrap_or(4096) as i32)
            .temperature(self.config.temperature);

        if !system_prompt.is_empty() {
            client_builder = client_builder.system(&system_prompt);
        }

        let request = client_builder
            .build()
            .map_err(|e| AgentError::ProviderError(format!("Failed to build request: {}", e)))?;

        // Collect response text
        let response_text = Arc::new(Mutex::new(String::new()));
        let response_text_clone = Arc::clone(&response_text);

        request
            .execute(|chunk| {
                let response_text = Arc::clone(&response_text_clone);
                async move {
                    let mut text = response_text.lock().await;
                    text.push_str(&chunk);
                }
            })
            .await
            .map_err(|e| AgentError::ProviderError(format!("Anthropic API error: {}", e)))?;

        let final_text = response_text.lock().await.clone();
        Ok(final_text)
    }

    async fn stream(
        &self,
        _messages: Vec<Message>,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, AgentError> {
        // TODO: Implement actual Anthropic streaming API call
        Err(AgentError::ProviderError(
            "Anthropic streaming not yet implemented. Use complete() method for now.".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        let provider = AnthropicProvider::new("test-api-key");
        assert_eq!(provider.name(), "Anthropic");
        assert_eq!(provider.config().model, "claude-3-5-sonnet-20241022");
        assert_eq!(provider.config().temperature, 0.7);
    }

    #[test]
    fn test_anthropic_provider_with_model() {
        let provider = AnthropicProvider::new("test-api-key").with_model("claude-3-opus-20240229");
        assert_eq!(provider.config().model, "claude-3-opus-20240229");
    }

    #[test]
    fn test_anthropic_provider_with_temperature() {
        let provider = AnthropicProvider::new("test-api-key").with_temperature(0.5);
        assert_eq!(provider.config().temperature, 0.5);
    }

    #[test]
    fn test_anthropic_provider_with_max_tokens() {
        let provider = AnthropicProvider::new("test-api-key").with_max_tokens(1000);
        assert_eq!(provider.config().max_tokens, Some(1000));
    }
}
