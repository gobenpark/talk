//! Talk: A Rust library for creating controlled LLM agents
//!
//! Talk enables developers to create production-ready AI agents with predictable behavior
//! in under 50 lines of Rust code. The library provides behavioral guidelines, tool integration,
//! multi-step conversation journeys, and pluggable storage backends.
//!
//! # Quick Start
//!
//! ```ignore
//! use talk::{Agent, Guideline, GuidelineCondition, GuidelineAction};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Implementation examples will be available after Agent API is implemented
//!     Ok(())
//! }
//! ```

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

// Public API exports will be added as modules are implemented
pub use agent::{Agent, AgentBuilder, AgentConfig, AgentResponse, LogLevel, ResponseExplanation, ToolExecution};
pub use context::{Context, ContextVariable, Message, MessageRole, Validator};
pub use error::{AgentError, GuidelineError, JourneyError, Result, StorageError, ToolError};
pub use guideline::{
    DefaultGuidelineMatcher, Guideline, GuidelineAction, GuidelineCondition, GuidelineMatch,
    GuidelineMatcher, ParameterDef,
};
pub use provider::{LlmProvider, ProviderConfig, StreamChunk};
pub use session::{JourneyState, Session, SessionStatus};
pub use storage::{memory::InMemorySessionStore, SessionStore};
pub use types::*;
