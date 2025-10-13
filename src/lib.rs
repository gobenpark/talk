//! # Talk - Controlled LLM Agents for Rust
//!
//! Talk is a Rust library for creating production-ready AI agents with predictable behavior.
//! Build conversational agents with behavioral guidelines, tool integration, and multi-step
//! journeys in under 50 lines of code.
//!
//! ## Features
//!
//! - 🎯 **Behavioral Guidelines**: Pattern matching (literal, regex) with priority-based execution
//! - 🔧 **Tool Integration**: Async functions with timeout, retry, and error handling
//! - 🗺️ **Conversation Journeys**: Multi-step state machines for guided user flows
//! - 🔌 **LLM Providers**: OpenAI and Anthropic support with trait-based extensibility
//! - 💾 **Session Storage**: In-memory default, optional Redis and PostgreSQL backends
//! - ⚡ **Performance**: <2s response times, 1000+ concurrent sessions
//! - 🦀 **Type-Safe**: Full Rust type safety with compile-time guarantees
//!
//! ## Quick Start
//!
//! ### Simple Agent with Guidelines
//!
//! ```no_run
//! use talk::{Agent, Guideline, GuidelineCondition, GuidelineAction, OpenAIProvider};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create LLM provider
//! let provider = OpenAIProvider::new(std::env::var("OPENAI_API_KEY")?);
//!
//! // Build agent
//! let mut agent = Agent::builder()
//!     .name("Customer Support")
//!     .provider(Box::new(provider))
//!     .build()?;
//!
//! // Add behavioral guideline
//! let guideline = Guideline::new(
//!     GuidelineCondition::Literal("pricing".to_string()),
//!     GuidelineAction::template("Our pricing starts at $49/month."),
//!     10, // priority
//! );
//! agent.add_guideline(guideline).await?;
//!
//! // Process messages
//! let session_id = agent.create_session().await?;
//! let response = agent.process_message(
//!     session_id,
//!     "What is your pricing?".to_string()
//! ).await?;
//!
//! println!("Agent: {}", response.message);
//! # Ok(())
//! # }
//! ```
//!
//! ### Agent with Tool Integration
//!
//! ```no_run
//! use talk::{Agent, Tool, ToolResult, Guideline, GuidelineCondition, OpenAIProvider};
//! use std::collections::HashMap;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Define async tool
//! struct WeatherTool;
//!
//! #[async_trait::async_trait]
//! impl Tool for WeatherTool {
//!     fn id(&self) -> &talk::ToolId { unimplemented!() }
//!     fn name(&self) -> &str { "get_weather" }
//!     fn description(&self) -> &str { "Get current weather" }
//!     fn parameters(&self) -> &HashMap<String, talk::ParameterSchema> { unimplemented!() }
//!
//!     async fn execute(&self, params: HashMap<String, serde_json::Value>)
//!         -> talk::Result<ToolResult>
//!     {
//!         let city = params.get("city").and_then(|v| v.as_str()).unwrap_or("Unknown");
//!         Ok(ToolResult {
//!             output: serde_json::json!({"weather": format!("Sunny in {}", city)}),
//!             error: None,
//!             metadata: HashMap::new(),
//!         })
//!     }
//! }
//!
//! let provider = OpenAIProvider::new(std::env::var("OPENAI_API_KEY")?);
//! let mut agent = Agent::builder()
//!     .name("Weather Bot")
//!     .provider(Box::new(provider))
//!     .build()?;
//!
//! // Register tool
//! let tool_id = agent.add_tool(Box::new(WeatherTool)).await?;
//!
//! // Create guideline that uses the tool
//! let mut guideline = Guideline::new(
//!     GuidelineCondition::Regex("weather.*in (.+)".to_string()),
//!     GuidelineAction::llm_with_template("Let me check the weather."),
//!     10,
//! );
//! guideline.tools.push(tool_id);
//! agent.add_guideline(guideline).await?;
//!
//! // Process message
//! let session_id = agent.create_session().await?;
//! let response = agent.process_message(
//!     session_id,
//!     "What's the weather in Tokyo?".to_string()
//! ).await?;
//!
//! println!("Agent: {}", response.message);
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                    Agent                        │
//! │  ┌──────────────┐  ┌──────────────┐            │
//! │  │  Guidelines  │  │    Tools     │            │
//! │  │  - Literal   │  │  - Async     │            │
//! │  │  - Regex     │  │  - Timeout   │            │
//! │  │  - Priority  │  │  - Retry     │            │
//! │  └──────────────┘  └──────────────┘            │
//! │  ┌──────────────────────────────────┐          │
//! │  │       LLM Provider               │          │
//! │  │  - OpenAI  - Anthropic           │          │
//! │  └──────────────────────────────────┘          │
//! │  ┌──────────────────────────────────┐          │
//! │  │      Session Storage             │          │
//! │  │  - Memory  - Redis  - Postgres   │          │
//! │  └──────────────────────────────────┘          │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Module Overview
//!
//! - [`agent`]: Core agent implementation and builder
//! - [`guideline`]: Pattern matching and behavioral rules
//! - [`tool`]: Tool integration with async execution
//! - [`journey`]: Multi-step conversation state machines
//! - [`provider`]: LLM provider abstractions (OpenAI, Anthropic)
//! - [`storage`]: Session storage backends
//! - [`context`]: Conversation context and variables
//! - [`session`]: Session lifecycle management
//! - [`error`]: Error types and result aliases
//!
//! ## Examples
//!
//! See the `examples/` directory for complete examples:
//! - `simple_agent.rs` - Basic agent with guidelines
//! - `weather_agent.rs` - Agent with tool integration
//! - `onboarding_journey.rs` - Multi-step journey flows
//!
//! ## Performance
//!
//! - Agent response: <2s (excluding LLM latency)
//! - Tool overhead: <100ms
//! - Pattern matching: O(n) with SIMD acceleration
//! - Concurrent sessions: 1000+ without degradation
//!
//! ## License
//!
//! Licensed under either of Apache License 2.0 or MIT license at your option.

// Core type definitions
pub mod types;

// Error types
pub mod error;

// Context management
pub mod context;

// Session management
pub mod session;

// Provider abstraction
pub mod provider;

// Storage backends
pub mod storage;

// Guideline matching engine
pub mod guideline;

// Agent core
pub mod agent;

// Tool integration
pub mod tool;

// Journey system
pub mod journey;

// Embedding module (optional, requires semantic-matching feature)
pub mod embedding;

// Public API exports will be added as modules are implemented
pub use agent::{
    Agent, AgentBuilder, AgentConfig, AgentResponse, LogLevel, ResponseExplanation, ToolExecution,
};
pub use context::{Context, ContextVariable, Message, MessageRole, Validator};
pub use error::{AgentError, GuidelineError, JourneyError, Result, StorageError, ToolError};
pub use guideline::{
    DefaultGuidelineMatcher, Guideline, GuidelineAction, GuidelineCondition, GuidelineMatch,
    GuidelineMatcher, ParameterDef,
};
pub use journey::{
    DefaultJourneyManager, Journey, JourneyManager, JourneyState, JourneyStep, Transition,
    TransitionCondition,
};
pub use provider::{AnthropicProvider, LLMProvider, OpenAIProvider, ProviderConfig, StreamChunk};
pub use session::{Session, SessionStatus};
pub use storage::{memory::InMemorySessionStore, SessionStore};
pub use tool::{ParameterSchema, Tool, ToolRegistry, ToolResult};
pub use types::*;

// Semantic matching exports (only available with feature)
#[cfg(feature = "semantic-matching")]
pub use embedding::{cosine_similarity, SentenceEmbedder};
