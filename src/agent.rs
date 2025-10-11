// Agent core implementation
//
// This module implements the main Agent struct that orchestrates guidelines,
// tools, journeys, and LLM interactions.

use crate::context::{Context, Message, MessageRole};
use crate::error::{AgentError, Result};
use crate::guideline::{Guideline, GuidelineMatch, GuidelineMatcher, DefaultGuidelineMatcher, GuidelineCondition, GuidelineAction};
use crate::provider::LlmProvider;
use crate::session::{Session, SessionStatus};
use crate::storage::SessionStore;
use crate::types::{AgentId, GuidelineId, SessionId, ToolId, StepId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Log level for agent operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Agent configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_max_context_messages")]
    pub max_context_messages: usize,

    #[serde(
        default = "default_tool_timeout",
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub default_tool_timeout: Duration,

    #[serde(default = "default_enable_explainability")]
    pub enable_explainability: bool,

    #[serde(default)]
    pub log_level: LogLevel,
}

fn default_max_context_messages() -> usize {
    100
}

fn default_tool_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_enable_explainability() -> bool {
    true
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

fn deserialize_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_context_messages: default_max_context_messages(),
            default_tool_timeout: default_tool_timeout(),
            enable_explainability: default_enable_explainability(),
            log_level: LogLevel::default(),
        }
    }
}

/// Response from agent processing a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub message: String,
    pub matched_guideline: Option<GuidelineMatch>,
    pub tools_used: Vec<ToolExecution>,
    pub journey_step: Option<StepId>,
    pub context_updates: HashMap<String, serde_json::Value>,
    pub explanation: Option<ResponseExplanation>,
}

/// Tool execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub tool_id: ToolId,
    pub duration: Duration,
}

/// Explanation of agent's decision-making process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseExplanation {
    pub guideline_matches: Vec<GuidelineMatch>,
    pub reasoning: String,
    pub confidence: f32,
}

/// Main Agent struct
pub struct Agent {
    id: AgentId,
    name: String,
    description: Option<String>,
    provider: Box<dyn LlmProvider>,
    guideline_matcher: Arc<RwLock<DefaultGuidelineMatcher>>,
    fallback_guideline: Guideline,
    config: AgentConfig,
    session_store: Arc<dyn SessionStore>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Agent {
    /// Create a new agent builder
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create a new conversation session
    pub async fn create_session(&self) -> Result<SessionId> {
        let session = Session::new(self.id);
        let session_id = session.id;

        self.session_store.create(session).await
            .map_err(|e| AgentError::Storage(e))?;

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>> {
        self.session_store.get(session_id).await
            .map_err(|e| AgentError::Storage(e))
    }

    /// End a conversation session
    pub async fn end_session(&self, session_id: &SessionId) -> Result<()> {
        let mut session = self.session_store.get(session_id).await
            .map_err(|e| AgentError::Storage(e))?
            .ok_or_else(|| AgentError::SessionNotFound(*session_id))?;

        session.status = SessionStatus::Completed;
        session.touch();

        self.session_store.update(session_id, session).await
            .map_err(|e| AgentError::Storage(e))
    }

    /// Add a guideline to the agent
    pub async fn add_guideline(&mut self, guideline: Guideline) -> Result<GuidelineId> {
        let mut matcher = self.guideline_matcher.write().await;
        matcher.add_guideline(guideline).await
    }

    /// Process a user message and generate a response
    pub async fn process_message(
        &self,
        session_id: SessionId,
        user_message: String,
    ) -> Result<AgentResponse> {
        // Get session
        let mut session = self.session_store.get(&session_id).await
            .map_err(|e| AgentError::Storage(e))?
            .ok_or_else(|| AgentError::SessionNotFound(session_id))?;

        // Add user message to context
        let user_msg = Message::user(user_message.clone());
        session.context.add_message(user_msg);

        // Match guidelines
        let matcher = self.guideline_matcher.read().await;
        let matches = matcher.match_guidelines(&user_message, &session.context).await?;

        // Select best match or use fallback
        let best_match = if !matches.is_empty() {
            matcher.select_best_match(matches.clone()).await
        } else {
            None
        };

        let guideline_match = best_match.or_else(|| {
            // Use fallback guideline
            Some(GuidelineMatch {
                guideline_id: self.fallback_guideline.id,
                relevance_score: 0.5,
                matched_condition: "fallback".to_string(),
                extracted_parameters: HashMap::new(),
                explanation: Some("No matching guideline found, using fallback".to_string()),
            })
        });

        // Get the guideline to use
        let guideline_to_use = if let Some(ref gm) = guideline_match {
            matcher
                .get_guidelines()
                .iter()
                .find(|g| g.id == gm.guideline_id)
                .cloned()
                .unwrap_or_else(|| self.fallback_guideline.clone())
        } else {
            self.fallback_guideline.clone()
        };

        // Generate response based on guideline
        let response_text = if guideline_to_use.action.requires_llm {
            // Use LLM to generate response
            let llm_messages = self.build_llm_messages(&session.context, &guideline_to_use);
            self.provider.complete(llm_messages).await?
        } else {
            // Use template response
            guideline_to_use.action.response_template.clone()
        };

        // Add agent response to context
        let agent_msg = Message::assistant(response_text.clone());
        session.context.add_message(agent_msg);

        // Update session
        session.touch();
        self.session_store.update(&session_id, session).await
            .map_err(|e| AgentError::Storage(e))?;

        // Build response
        let explanation = if self.config.enable_explainability {
            Some(ResponseExplanation {
                guideline_matches: matches,
                reasoning: format!("Selected guideline with priority {}", guideline_to_use.priority),
                confidence: guideline_match.as_ref().map(|m| m.relevance_score).unwrap_or(0.5),
            })
        } else {
            None
        };

        Ok(AgentResponse {
            message: response_text,
            matched_guideline: guideline_match,
            tools_used: vec![],
            journey_step: None,
            context_updates: HashMap::new(),
            explanation,
        })
    }

    /// Build LLM messages from context and guideline
    fn build_llm_messages(&self, context: &Context, guideline: &Guideline) -> Vec<Message> {
        let mut messages = vec![
            Message::system(format!(
                "You are {}. {}",
                self.name,
                self.description.as_deref().unwrap_or("A helpful AI assistant")
            ))
        ];

        // Add guideline context
        messages.push(Message::system(format!(
            "Guideline: {}",
            guideline.action.response_template
        )));

        // Add conversation history
        messages.extend(context.messages.clone());

        messages
    }
}

/// Builder for Agent
pub struct AgentBuilder {
    name: Option<String>,
    description: Option<String>,
    provider: Option<Box<dyn LlmProvider>>,
    config: AgentConfig,
    session_store: Option<Arc<dyn SessionStore>>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            provider: None,
            config: AgentConfig::default(),
            session_store: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn provider(mut self, provider: Box<dyn LlmProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    pub fn session_store(
        mut self,
        store: Arc<dyn SessionStore>,
    ) -> Self {
        self.session_store = Some(store);
        self
    }

    pub fn build(self) -> Result<Agent> {
        let name = self.name.ok_or_else(|| {
            AgentError::Configuration("Agent name is required".to_string())
        })?;

        let provider = self.provider.ok_or_else(|| {
            AgentError::Configuration("LLM provider is required".to_string())
        })?;

        let session_store = self.session_store.unwrap_or_else(|| {
            Arc::new(crate::storage::memory::InMemorySessionStore::new())
        });

        // Create default fallback guideline
        let fallback_guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("".to_string()),
            action: GuidelineAction {
                response_template: "I'm not sure how to help with that. Could you please rephrase your question?".to_string(),
                requires_llm: true,
                parameters: vec![],
            },
            priority: -1,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: Utc::now(),
        };

        Ok(Agent {
            id: AgentId::new(),
            name,
            description: self.description,
            provider,
            guideline_matcher: Arc::new(RwLock::new(DefaultGuidelineMatcher::new())),
            fallback_guideline,
            config: self.config,
            session_store,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_context_messages, 100);
        assert_eq!(config.default_tool_timeout, Duration::from_secs(30));
        assert_eq!(config.enable_explainability, true);
        assert_eq!(config.log_level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_serialization() {
        let level = LogLevel::Debug;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"debug\"");

        let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, LogLevel::Debug);
    }
}
