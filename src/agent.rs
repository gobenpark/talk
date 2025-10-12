// Agent core implementation
//
// This module implements the main Agent struct that orchestrates guidelines,
// tools, journeys, and LLM interactions.

use crate::context::{Context, Message};
use crate::error::{AgentError, Result};
use crate::guideline::{
    DefaultGuidelineMatcher, Guideline, GuidelineAction, GuidelineCondition, GuidelineMatch,
    GuidelineMatcher,
};
use crate::journey::{DefaultJourneyManager, Journey, JourneyManager, JourneyState, JourneyStep};
use crate::provider::LLMProvider;
use crate::session::{Session, SessionStatus};
use crate::storage::SessionStore;
use crate::tool::{Tool, ToolRegistry};
use crate::types::{AgentId, GuidelineId, JourneyId, SessionId, StepId, ToolId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

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

/// Main Agent struct for creating and managing LLM-based conversational agents.
///
/// An `Agent` orchestrates guidelines, tools, journeys, and LLM interactions to provide
/// controlled, predictable AI behavior. It manages session state, context tracking, and
/// provides explainability features for debugging agent decisions.
///
/// # Examples
///
/// Basic agent with guidelines:
///
/// ```no_run
/// use talk::{Agent, Guideline, GuidelineCondition, GuidelineAction, OpenAIProvider};
///
/// # #[tokio::main]
/// # async fn main() -> talk::Result<()> {
/// let provider = OpenAIProvider::new("api-key".to_string());
/// let mut agent = Agent::builder()
///     .name("Support Bot")
///     .description("Customer support agent")
///     .provider(Box::new(provider))
///     .build()?;
///
/// // Add guideline
/// let guideline = Guideline {
///     id: talk::GuidelineId::new(),
///     condition: GuidelineCondition::Literal("help".to_string()),
///     action: GuidelineAction {
///         response_template: "How can I help you?".to_string(),
///         requires_llm: false,
///         parameters: vec![],
///     },
///     priority: 10,
///     tools: vec![],
///     parameters: Default::default(),
///     created_at: chrono::Utc::now(),
/// };
/// agent.add_guideline(guideline).await?;
///
/// // Create session and process message
/// let session_id = agent.create_session().await?;
/// let response = agent.process_message(session_id, "help".to_string()).await?;
/// println!("{}", response.message);
/// # Ok(())
/// # }
/// ```
///
/// # Architecture
///
/// The agent coordinates several subsystems:
/// - **GuidelineMatcher**: Pattern matching and conflict resolution
/// - **ToolRegistry**: Async tool execution with timeout/retry
/// - **JourneyManager**: Multi-step conversation state machines
/// - **SessionStore**: Persistent or in-memory session storage
/// - **LLMProvider**: Pluggable LLM backend (OpenAI, Anthropic)
pub struct Agent {
    id: AgentId,
    name: String,
    description: Option<String>,
    provider: Box<dyn LLMProvider>,
    guideline_matcher: Arc<RwLock<DefaultGuidelineMatcher>>,
    tool_registry: Arc<ToolRegistry>,
    journey_manager: Arc<RwLock<DefaultJourneyManager>>,
    journey_states: Arc<RwLock<HashMap<SessionId, JourneyState>>>,
    fallback_guideline: Guideline,
    config: AgentConfig,
    session_store: Arc<dyn SessionStore>,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    #[allow(dead_code)]
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

        self.session_store
            .create(session)
            .await
            .map_err(|e| AgentError::Storage(e))?;

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>> {
        self.session_store
            .get(session_id)
            .await
            .map_err(|e| AgentError::Storage(e))
    }

    /// End a conversation session
    pub async fn end_session(&self, session_id: &SessionId) -> Result<()> {
        let mut session = self
            .session_store
            .get(session_id)
            .await
            .map_err(|e| AgentError::Storage(e))?
            .ok_or_else(|| AgentError::SessionNotFound(*session_id))?;

        session.status = SessionStatus::Completed;
        session.touch();

        self.session_store
            .update(session_id, session)
            .await
            .map_err(|e| AgentError::Storage(e))
    }

    /// Add a guideline to the agent
    pub async fn add_guideline(&mut self, guideline: Guideline) -> Result<GuidelineId> {
        let mut matcher = self.guideline_matcher.write().await;
        matcher.add_guideline(guideline).await
    }

    /// Add a tool to the agent
    pub async fn add_tool(&self, tool: Box<dyn Tool>) -> Result<ToolId> {
        info!(
            agent_id = %self.id,
            tool_name = tool.name(),
            "Adding tool to agent"
        );

        self.tool_registry.register(tool).await
    }

    /// Add a journey to the agent
    pub async fn add_journey(&mut self, journey: Journey) -> Result<JourneyId> {
        let mut manager = self.journey_manager.write().await;
        manager.add_journey(journey).await
    }

    /// Start a journey for a session
    pub async fn start_journey(
        &self,
        session_id: &SessionId,
        journey_id: &JourneyId,
    ) -> Result<JourneyState> {
        let manager = self.journey_manager.read().await;
        let state = manager.start_journey(session_id, journey_id).await?;

        // Store state in journey_states
        let mut states = self.journey_states.write().await;
        states.insert(*session_id, state.clone());

        Ok(state)
    }

    /// Get current journey state for a session
    pub async fn get_journey_state(&self, session_id: &SessionId) -> Result<Option<JourneyState>> {
        let states = self.journey_states.read().await;
        Ok(states.get(session_id).cloned())
    }

    /// Process a journey step with user message
    pub async fn process_journey_step(
        &self,
        session_id: &SessionId,
        message: &str,
    ) -> Result<JourneyStep> {
        // Get current state
        let mut states = self.journey_states.write().await;
        let state = states
            .get_mut(session_id)
            .ok_or_else(|| AgentError::Journey("No active journey for session".to_string()))?;

        let journey_id = state.journey_id;
        let current_step_id = state.current_step;

        // Process step through manager
        let manager = self.journey_manager.read().await;
        let next_step = manager
            .process_step(&journey_id, current_step_id, message)
            .await?;

        // Update state
        state.complete_step(current_step_id);

        // Check if we transitioned to a new step or stayed on current (final) step
        if next_step.id == current_step_id {
            // No transition - this means we're on the final step
            state.mark_complete();
        } else {
            // Transition to next step
            state.current_step = next_step.id;
        }

        Ok(next_step)
    }

    /// End a journey for a session
    pub async fn end_journey(&self, session_id: &SessionId) -> Result<()> {
        let mut states = self.journey_states.write().await;
        states.remove(session_id);
        Ok(())
    }

    /// Process a user message and generate a response
    pub async fn process_message(
        &self,
        session_id: SessionId,
        user_message: String,
    ) -> Result<AgentResponse> {
        info!(
            session_id = %session_id,
            message_length = user_message.len(),
            "Processing user message"
        );

        // Get session
        let mut session = self
            .session_store
            .get(&session_id)
            .await
            .map_err(|e| AgentError::Storage(e))?
            .ok_or_else(|| AgentError::SessionNotFound(session_id))?;

        debug!(
            session_status = ?session.status,
            message_count = session.context.messages.len(),
            "Session retrieved"
        );

        // Add user message to context
        let user_msg = Message::user(user_message.clone());
        session.context.add_message(user_msg);

        // Match guidelines
        let matcher = self.guideline_matcher.read().await;
        trace!("Acquired guideline matcher lock");
        let matches = matcher
            .match_guidelines(&user_message, &session.context)
            .await?;

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

        // Execute tools if the guideline specifies any
        let mut tools_used = Vec::new();
        let mut tool_context = String::new();

        if !guideline_to_use.tools.is_empty() {
            debug!(
                tool_count = guideline_to_use.tools.len(),
                "Executing tools for guideline"
            );

            for tool_id in &guideline_to_use.tools {
                info!(tool_id = %tool_id, "Executing tool");

                // Extract parameters from the matched guideline
                let parameters = if let Some(ref gm) = guideline_match {
                    gm.extracted_parameters.clone()
                } else {
                    HashMap::new()
                };

                // Execute tool with retry and timeout
                let tool_result = self
                    .tool_registry
                    .execute_with_retry(
                        tool_id,
                        parameters,
                        self.config.default_tool_timeout,
                        3,   // max retries
                        100, // base backoff ms
                    )
                    .await;

                match tool_result {
                    Ok(result) => {
                        let execution_time = std::time::Duration::from_millis(100); // placeholder
                        tools_used.push(ToolExecution {
                            tool_id: *tool_id,
                            duration: execution_time,
                        });

                        // Incorporate tool result into context for LLM
                        tool_context.push_str(&format!(
                            "\n\nTool result: {}",
                            serde_json::to_string_pretty(&result.output).unwrap_or_default()
                        ));

                        debug!(
                            tool_id = %tool_id,
                            "Tool execution successful"
                        );
                    }
                    Err(e) => {
                        warn!(
                            tool_id = %tool_id,
                            error = %e,
                            "Tool execution failed"
                        );
                        // Continue with other tools even if one fails
                        tool_context.push_str(&format!("\n\nTool execution failed: {}", e));
                    }
                }
            }
        }

        // Generate response based on guideline
        let response_text = if guideline_to_use.action.requires_llm {
            // Use LLM to generate response, including tool results in context
            let mut llm_messages = self.build_llm_messages(&session.context, &guideline_to_use);

            // Add tool results to the last message if any tools were executed
            if !tool_context.is_empty() {
                llm_messages.push(Message::system(format!(
                    "Tool execution results:{}",
                    tool_context
                )));
            }

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
        self.session_store
            .update(&session_id, session)
            .await
            .map_err(|e| AgentError::Storage(e))?;

        // Build response
        let explanation = if self.config.enable_explainability {
            Some(ResponseExplanation {
                guideline_matches: matches,
                reasoning: format!(
                    "Selected guideline with priority {}",
                    guideline_to_use.priority
                ),
                confidence: guideline_match
                    .as_ref()
                    .map(|m| m.relevance_score)
                    .unwrap_or(0.5),
            })
        } else {
            None
        };

        Ok(AgentResponse {
            message: response_text,
            matched_guideline: guideline_match,
            tools_used,
            journey_step: None,
            context_updates: HashMap::new(),
            explanation,
        })
    }

    /// Build LLM messages from context and guideline
    fn build_llm_messages(&self, context: &Context, guideline: &Guideline) -> Vec<Message> {
        let mut messages = vec![Message::system(format!(
            "You are {}. {}",
            self.name,
            self.description
                .as_deref()
                .unwrap_or("A helpful AI assistant")
        ))];

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
    provider: Option<Box<dyn LLMProvider>>,
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

    pub fn provider(mut self, provider: Box<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    pub fn session_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_store = Some(store);
        self
    }

    pub fn build(self) -> Result<Agent> {
        let name = self
            .name
            .ok_or_else(|| AgentError::Configuration("Agent name is required".to_string()))?;

        let provider = self
            .provider
            .ok_or_else(|| AgentError::Configuration("LLM provider is required".to_string()))?;

        let session_store = self
            .session_store
            .unwrap_or_else(|| Arc::new(crate::storage::memory::InMemorySessionStore::new()));

        // Create default fallback guideline
        let fallback_guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("".to_string()),
            action: GuidelineAction {
                response_template:
                    "I'm not sure how to help with that. Could you please rephrase your question?"
                        .to_string(),
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
            tool_registry: Arc::new(ToolRegistry::new()),
            journey_manager: Arc::new(RwLock::new(DefaultJourneyManager::new())),
            journey_states: Arc::new(RwLock::new(HashMap::new())),
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
    use crate::provider::LLMProvider;
    use crate::tool::{ParameterSchema, Tool, ToolResult};
    use std::collections::HashMap;

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

    // Mock LLM Provider for testing
    struct MockProvider {
        config: crate::provider::ProviderConfig,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                config: crate::provider::ProviderConfig::new("mock-model"),
            }
        }
    }

    #[async_trait::async_trait]
    impl LLMProvider for MockProvider {
        async fn complete(
            &self,
            _messages: Vec<Message>,
        ) -> std::result::Result<String, AgentError> {
            Ok("Mock LLM response".to_string())
        }

        async fn stream(
            &self,
            _messages: Vec<Message>,
        ) -> std::result::Result<
            std::pin::Pin<
                Box<dyn futures::Stream<Item = std::result::Result<String, AgentError>> + Send>,
            >,
            AgentError,
        > {
            use futures::stream;
            let chunks = vec![Ok("Mock".to_string()), Ok(" response".to_string())];
            Ok(Box::pin(stream::iter(chunks)))
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn config(&self) -> &crate::provider::ProviderConfig {
            &self.config
        }
    }

    // Mock Tool for testing
    struct MockTool {
        id: ToolId,
        name: String,
        result: String,
        parameters: HashMap<String, ParameterSchema>,
    }

    impl MockTool {
        fn new(name: String, result: String) -> Self {
            let mut parameters = HashMap::new();
            parameters.insert(
                "query".to_string(),
                ParameterSchema {
                    param_type: "string".to_string(),
                    required: false,
                    description: "Query parameter".to_string(),
                    default: None,
                },
            );

            Self {
                id: ToolId::new(),
                name,
                result,
                parameters,
            }
        }
    }

    #[async_trait::async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &ToolId {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn parameters(&self) -> &HashMap<String, ParameterSchema> {
            &self.parameters
        }

        async fn execute(
            &self,
            _parameters: HashMap<String, serde_json::Value>,
        ) -> Result<ToolResult> {
            Ok(ToolResult {
                output: serde_json::json!({ "result": self.result }),
                error: None,
                metadata: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_agent_add_tool() {
        let provider: Box<dyn LLMProvider> = Box::new(MockProvider::new());
        let agent = Agent::builder()
            .name("Test Agent")
            .provider(provider)
            .build()
            .unwrap();

        let tool = MockTool::new("test_tool".to_string(), "test result".to_string());
        let tool_id = agent.add_tool(Box::new(tool)).await.unwrap();

        // Verify tool was added
        let tool_registry = &agent.tool_registry;
        let retrieved_tool = tool_registry.get(&tool_id).await;
        assert!(retrieved_tool.is_some());
        assert_eq!(retrieved_tool.unwrap().name(), "test_tool");
    }

    #[tokio::test]
    async fn test_agent_tool_execution_in_guideline() {
        let provider: Box<dyn LLMProvider> = Box::new(MockProvider::new());
        let mut agent = Agent::builder()
            .name("Test Agent")
            .provider(provider)
            .build()
            .unwrap();

        // Add a tool
        let tool = MockTool::new("weather_tool".to_string(), "sunny, 72F".to_string());
        let tool_id = agent.add_tool(Box::new(tool)).await.unwrap();

        // Add a guideline that uses the tool
        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("weather".to_string()),
            action: GuidelineAction {
                response_template: "Here's the weather".to_string(),
                requires_llm: false,
                parameters: Vec::new(),
            },
            priority: 10,
            tools: vec![tool_id],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        agent.add_guideline(guideline).await.unwrap();

        // Create a session
        let session_id = agent.create_session().await.unwrap();

        // Process a message that triggers the guideline with the tool
        let response = agent
            .process_message(session_id, "What's the weather?".to_string())
            .await
            .unwrap();

        // Verify tool was executed
        assert_eq!(response.tools_used.len(), 1);
        assert_eq!(response.tools_used[0].tool_id, tool_id);
    }

    #[tokio::test]
    async fn test_agent_multiple_tools_in_guideline() {
        let provider: Box<dyn LLMProvider> = Box::new(MockProvider::new());
        let mut agent = Agent::builder()
            .name("Test Agent")
            .provider(provider)
            .build()
            .unwrap();

        // Add two tools
        let tool1 = MockTool::new("tool1".to_string(), "result1".to_string());
        let tool1_id = agent.add_tool(Box::new(tool1)).await.unwrap();

        let tool2 = MockTool::new("tool2".to_string(), "result2".to_string());
        let tool2_id = agent.add_tool(Box::new(tool2)).await.unwrap();

        // Add a guideline that uses both tools
        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("multi".to_string()),
            action: GuidelineAction {
                response_template: "Using multiple tools".to_string(),
                requires_llm: false,
                parameters: Vec::new(),
            },
            priority: 10,
            tools: vec![tool1_id, tool2_id],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        agent.add_guideline(guideline).await.unwrap();

        // Create a session
        let session_id = agent.create_session().await.unwrap();

        // Process a message
        let response = agent
            .process_message(session_id, "multi tool test".to_string())
            .await
            .unwrap();

        // Verify both tools were executed
        assert_eq!(response.tools_used.len(), 2);
        assert!(response.tools_used.iter().any(|t| t.tool_id == tool1_id));
        assert!(response.tools_used.iter().any(|t| t.tool_id == tool2_id));
    }

    #[tokio::test]
    async fn test_agent_tool_with_llm_response() {
        let provider: Box<dyn LLMProvider> = Box::new(MockProvider::new());
        let mut agent = Agent::builder()
            .name("Test Agent")
            .provider(provider)
            .build()
            .unwrap();

        // Add a tool
        let tool = MockTool::new("data_tool".to_string(), "data result".to_string());
        let tool_id = agent.add_tool(Box::new(tool)).await.unwrap();

        // Add a guideline that uses the tool AND requires LLM
        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("analyze".to_string()),
            action: GuidelineAction {
                response_template: "Analyzing data".to_string(),
                requires_llm: true, // This should use LLM to generate response
                parameters: Vec::new(),
            },
            priority: 10,
            tools: vec![tool_id],
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        agent.add_guideline(guideline).await.unwrap();

        // Create a session
        let session_id = agent.create_session().await.unwrap();

        // Process a message
        let response = agent
            .process_message(session_id, "analyze this data".to_string())
            .await
            .unwrap();

        // Verify tool was executed
        assert_eq!(response.tools_used.len(), 1);
        assert_eq!(response.tools_used[0].tool_id, tool_id);

        // Response should come from LLM (our mock returns "Mock LLM response")
        assert_eq!(response.message, "Mock LLM response");
    }

    #[tokio::test]
    async fn test_agent_guideline_without_tools() {
        let provider: Box<dyn LLMProvider> = Box::new(MockProvider::new());
        let mut agent = Agent::builder()
            .name("Test Agent")
            .provider(provider)
            .build()
            .unwrap();

        // Add a guideline without any tools
        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("hello".to_string()),
            action: GuidelineAction {
                response_template: "Hello there!".to_string(),
                requires_llm: false,
                parameters: Vec::new(),
            },
            priority: 10,
            tools: Vec::new(), // No tools
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        agent.add_guideline(guideline).await.unwrap();

        // Create a session
        let session_id = agent.create_session().await.unwrap();

        // Process a message
        let response = agent
            .process_message(session_id, "hello".to_string())
            .await
            .unwrap();

        // Verify no tools were executed
        assert_eq!(response.tools_used.len(), 0);
        assert_eq!(response.message, "Hello there!");
    }
}
