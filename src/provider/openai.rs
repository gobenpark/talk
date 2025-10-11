//! OpenAI LLM provider implementation
//!
//! This module implements the LLMProvider trait for OpenAI's GPT models.

use crate::context::{Message, MessageRole};
use crate::error::AgentError;
use crate::provider::{LLMProvider, ProviderConfig, StreamChunk};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use tracing::{debug, info, trace, warn};

/// OpenAI LLM provider
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    config: ProviderConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with the given API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - OpenAI API key
    ///
    /// # Returns
    ///
    /// A new OpenAI provider instance with default configuration (gpt-4)
    pub fn new(api_key: impl Into<String>) -> Self {
        let openai_config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(openai_config);

        Self {
            client,
            config: ProviderConfig::new("gpt-4"),
        }
    }

    /// Create a new OpenAI provider from environment variable OPENAI_API_KEY
    ///
    /// # Returns
    ///
    /// A new OpenAI provider instance or an error if the environment variable is not set
    pub fn from_env() -> Result<Self, AgentError> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            AgentError::Configuration("OPENAI_API_KEY environment variable not set".to_string())
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

    /// Convert Talk messages to OpenAI format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<ChatCompletionRequestMessage> {
        messages
            .into_iter()
            .map(|m| match m.role {
                MessageRole::System => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content:
                            async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                                m.content,
                            ),
                        name: None,
                    })
                }
                MessageRole::User => {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                            m.content,
                        ),
                        name: None,
                    })
                }
                MessageRole::Assistant => {
                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(
                            async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                                m.content,
                            ),
                        ),
                        name: None,
                        tool_calls: None,
                        refusal: None,
                        #[allow(deprecated)]
                        function_call: None,
                    })
                }
                MessageRole::Tool => {
                    // Convert tool messages to system messages for now
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content:
                            async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                                format!("Tool result: {}", m.content),
                            ),
                        name: None,
                    })
                }
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, messages: Vec<Message>) -> std::result::Result<String, AgentError> {
        info!(
            model = %self.config.model,
            message_count = messages.len(),
            "Requesting OpenAI completion"
        );

        let openai_messages = self.convert_messages(messages);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.config.model)
            .messages(openai_messages)
            .temperature(self.config.temperature);

        if let Some(max_tokens) = self.config.max_tokens {
            request_builder.max_tokens(max_tokens);
        }

        if let Some(top_p) = self.config.top_p {
            request_builder.top_p(top_p);
        }

        if let Some(frequency_penalty) = self.config.frequency_penalty {
            request_builder.frequency_penalty(frequency_penalty);
        }

        if let Some(presence_penalty) = self.config.presence_penalty {
            request_builder.presence_penalty(presence_penalty);
        }

        let request = request_builder
            .build()
            .map_err(|e| AgentError::ProviderError(format!("Failed to build request: {}", e)))?;

        trace!("Sending request to OpenAI");

        let response = self.client.chat().create(request).await.map_err(|e| {
            warn!(error = %e, "OpenAI API error");
            AgentError::ProviderError(format!("OpenAI API error: {}", e))
        })?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| {
                warn!("No content in OpenAI response");
                AgentError::ProviderError("No content in OpenAI response".to_string())
            })?;

        debug!(
            response_length = message.len(),
            "OpenAI completion successful"
        );

        Ok(message)
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> std::result::Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, AgentError> {
        info!(
            model = %self.config.model,
            message_count = messages.len(),
            "Requesting OpenAI streaming completion"
        );

        let openai_messages = self.convert_messages(messages);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.config.model)
            .messages(openai_messages)
            .temperature(self.config.temperature);

        if let Some(max_tokens) = self.config.max_tokens {
            request_builder.max_tokens(max_tokens);
        }

        if let Some(top_p) = self.config.top_p {
            request_builder.top_p(top_p);
        }

        let request = request_builder
            .build()
            .map_err(|e| AgentError::ProviderError(format!("Failed to build request: {}", e)))?;

        trace!("Sending streaming request to OpenAI");

        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| {
                warn!(error = %e, "OpenAI streaming error");
                AgentError::ProviderError(format!("OpenAI streaming error: {}", e))
            })?;

        let mapped_stream = stream.map(|result| {
            result
                .map_err(|e| AgentError::ProviderError(format!("Stream error: {}", e)))
                .and_then(|response| {
                    response
                        .choices
                        .first()
                        .and_then(|choice| choice.delta.content.clone())
                        .ok_or_else(|| {
                            AgentError::ProviderError("No content in stream chunk".to_string())
                        })
                })
        });

        Ok(Box::pin(mapped_stream))
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test-api-key");
        assert_eq!(provider.name(), "OpenAI");
        assert_eq!(provider.config().model, "gpt-4");
        assert_eq!(provider.config().temperature, 0.7);
    }

    #[test]
    fn test_openai_provider_with_model() {
        let provider = OpenAIProvider::new("test-api-key").with_model("gpt-3.5-turbo");
        assert_eq!(provider.config().model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_openai_provider_with_temperature() {
        let provider = OpenAIProvider::new("test-api-key").with_temperature(0.5);
        assert_eq!(provider.config().temperature, 0.5);
    }

    #[test]
    fn test_openai_provider_with_max_tokens() {
        let provider = OpenAIProvider::new("test-api-key").with_max_tokens(1000);
        assert_eq!(provider.config().max_tokens, Some(1000));
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenAIProvider::new("test-api-key");
        let messages = vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let converted = provider.convert_messages(messages);
        assert_eq!(converted.len(), 3);
    }
}
