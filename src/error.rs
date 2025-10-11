//! Error types for the Talk library
//!
//! This module provides comprehensive error types using thiserror for all Talk operations.

use crate::types::{GuidelineId, JourneyId, SessionId, StepId, ToolId};
use thiserror::Error;

/// Main error type for Talk library operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AgentError {
    /// LLM provider error
    #[error("LLM provider error: {0}")]
    LLMProvider(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    /// Session already exists
    #[error("Session already exists: {0}")]
    SessionAlreadyExists(SessionId),

    /// Guideline matching failed
    #[error("Guideline matching failed: {0}")]
    GuidelineMatch(String),

    /// Guideline not found
    #[error("Guideline not found: {0}")]
    GuidelineNotFound(GuidelineId),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(#[from] ToolError),

    /// Journey execution error
    #[error("Journey execution error: {0}")]
    JourneyExecution(#[from] JourneyError),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(ToolId),

    /// Tool already registered with same name
    #[error("Tool already registered: {0}")]
    ToolAlreadyRegistered(String),

    /// Tool execution failed
    #[error("Tool execution failed for {tool_name}: {reason}")]
    ToolExecutionFailed {
        tool_name: String,
        reason: String,
    },

    /// Invalid tool parameters
    #[error("Invalid tool parameters for {tool_name}: {reason}")]
    InvalidToolParameters {
        tool_name: String,
        reason: String,
    },

    /// Tool execution timeout
    #[error("Tool execution timeout for {tool_name} after {timeout:?}")]
    ToolTimeout {
        tool_name: String,
        timeout: std::time::Duration,
    },

    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Storage-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StorageError {
    /// Connection failed
    #[error("Storage connection failed: {0}")]
    Connection(String),

    /// Query failed
    #[error("Storage query failed: {0}")]
    Query(String),

    /// Serialization failed
    #[error("Storage serialization failed: {0}")]
    Serialization(String),

    /// Deserialization failed
    #[error("Storage deserialization failed: {0}")]
    Deserialization(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    /// Storage backend not available
    #[error("Storage backend not available: {0}")]
    BackendUnavailable(String),

    /// Internal storage error
    #[error("Internal storage error: {0}")]
    Internal(String),
}

/// Guideline-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GuidelineError {
    /// Invalid guideline condition
    #[error("Invalid guideline condition: {0}")]
    InvalidCondition(String),

    /// Invalid guideline action
    #[error("Invalid guideline action: {0}")]
    InvalidAction(String),

    /// Guideline compilation failed (for regex patterns)
    #[error("Guideline compilation failed: {0}")]
    CompilationFailed(String),

    /// Guideline not found
    #[error("Guideline not found: {0}")]
    NotFound(GuidelineId),

    /// Guideline already exists
    #[error("Guideline already exists: {0}")]
    AlreadyExists(GuidelineId),

    /// Multiple guidelines matched with same priority
    #[error("Multiple guidelines matched with same priority: {0:?}")]
    AmbiguousMatch(Vec<GuidelineId>),

    /// No guideline matched
    #[error("No guideline matched for input: {0}")]
    NoMatch(String),

    /// Internal guideline error
    #[error("Internal guideline error: {0}")]
    Internal(String),
}

/// Tool-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(ToolId),

    /// Tool already exists
    #[error("Tool already exists: {0}")]
    AlreadyExists(ToolId),

    /// Tool execution timeout
    #[error("Tool execution timeout after {timeout_ms}ms: {tool_id}")]
    Timeout { tool_id: ToolId, timeout_ms: u64 },

    /// Tool execution failed
    #[error("Tool execution failed for {tool_id}: {message}")]
    ExecutionFailed { tool_id: ToolId, message: String },

    /// Invalid tool parameters
    #[error("Invalid tool parameters for {tool_id}: {message}")]
    InvalidParameters { tool_id: ToolId, message: String },

    /// Tool output deserialization failed
    #[error("Tool output deserialization failed: {0}")]
    OutputDeserialization(String),

    /// Internal tool error
    #[error("Internal tool error: {0}")]
    Internal(String),
}

/// Journey-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JourneyError {
    /// Journey not found
    #[error("Journey not found: {0}")]
    NotFound(JourneyId),

    /// Journey already exists
    #[error("Journey already exists: {0}")]
    AlreadyExists(JourneyId),

    /// Journey step not found
    #[error("Journey step not found: {step_id} in journey {journey_id}")]
    StepNotFound {
        journey_id: JourneyId,
        step_id: StepId,
    },

    /// Invalid journey transition
    #[error(
        "Invalid journey transition from step {from_step} to {to_step} in journey {journey_id}"
    )]
    InvalidTransition {
        journey_id: JourneyId,
        from_step: StepId,
        to_step: StepId,
    },

    /// Journey already started
    #[error("Journey already started: {0}")]
    AlreadyStarted(JourneyId),

    /// Journey not started
    #[error("Journey not started: {0}")]
    NotStarted(JourneyId),

    /// Journey already completed
    #[error("Journey already completed: {0}")]
    AlreadyCompleted(JourneyId),

    /// Journey has no initial step
    #[error("Journey has no initial step: {0}")]
    NoInitialStep(JourneyId),

    /// Circular journey detected
    #[error("Circular journey detected: {0}")]
    CircularJourney(JourneyId),

    /// Internal journey error
    #[error("Internal journey error: {0}")]
    Internal(String),
}

/// Type alias for Talk library Result
pub type Result<T> = std::result::Result<T, AgentError>;

/// Type alias for Storage Result
pub type StorageResult<T> = std::result::Result<T, StorageError>;

/// Type alias for Guideline Result
pub type GuidelineResult<T> = std::result::Result<T, GuidelineError>;

/// Type alias for Tool Result
pub type ToolResult<T> = std::result::Result<T, ToolError>;

/// Type alias for Journey Result
pub type JourneyResult<T> = std::result::Result<T, JourneyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_error_display() {
        let session_id = SessionId::new();
        let err = AgentError::SessionNotFound(session_id);
        let display = format!("{}", err);
        assert!(display.contains("Session not found"));
        assert!(display.contains(&session_id.to_string()));
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::Connection("connection refused".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Storage connection failed"));
        assert!(display.contains("connection refused"));
    }

    #[test]
    fn test_guideline_error_display() {
        let guideline_id = GuidelineId::new();
        let err = GuidelineError::NotFound(guideline_id);
        let display = format!("{}", err);
        assert!(display.contains("Guideline not found"));
        assert!(display.contains(&guideline_id.to_string()));
    }

    #[test]
    fn test_tool_error_timeout_display() {
        let tool_id = ToolId::new();
        let err = ToolError::Timeout {
            tool_id,
            timeout_ms: 5000,
        };
        let display = format!("{}", err);
        assert!(display.contains("Tool execution timeout"));
        assert!(display.contains("5000ms"));
        assert!(display.contains(&tool_id.to_string()));
    }

    #[test]
    fn test_tool_error_execution_failed_display() {
        let tool_id = ToolId::new();
        let err = ToolError::ExecutionFailed {
            tool_id,
            message: "API call failed".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Tool execution failed"));
        assert!(display.contains("API call failed"));
        assert!(display.contains(&tool_id.to_string()));
    }

    #[test]
    fn test_journey_error_step_not_found_display() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let err = JourneyError::StepNotFound {
            journey_id,
            step_id,
        };
        let display = format!("{}", err);
        assert!(display.contains("Journey step not found"));
        assert!(display.contains(&journey_id.to_string()));
        assert!(display.contains(&step_id.to_string()));
    }

    #[test]
    fn test_journey_error_invalid_transition_display() {
        let journey_id = JourneyId::new();
        let from_step = StepId::new();
        let to_step = StepId::new();
        let err = JourneyError::InvalidTransition {
            journey_id,
            from_step,
            to_step,
        };
        let display = format!("{}", err);
        assert!(display.contains("Invalid journey transition"));
        assert!(display.contains(&journey_id.to_string()));
        assert!(display.contains(&from_step.to_string()));
        assert!(display.contains(&to_step.to_string()));
    }

    #[test]
    fn test_error_conversion_storage_to_agent() {
        let storage_err = StorageError::Connection("test".to_string());
        let agent_err: AgentError = storage_err.into();
        assert!(matches!(agent_err, AgentError::Storage(_)));
    }

    #[test]
    fn test_error_conversion_tool_to_agent() {
        let tool_err = ToolError::NotFound(ToolId::new());
        let agent_err: AgentError = tool_err.into();
        assert!(matches!(agent_err, AgentError::ToolExecution(_)));
    }

    #[test]
    fn test_error_conversion_journey_to_agent() {
        let journey_err = JourneyError::NotFound(JourneyId::new());
        let agent_err: AgentError = journey_err.into();
        assert!(matches!(agent_err, AgentError::JourneyExecution(_)));
    }

    #[test]
    fn test_result_type_aliases() {
        fn returns_result() -> Result<()> {
            Ok(())
        }

        fn returns_storage_result() -> StorageResult<()> {
            Ok(())
        }

        fn returns_guideline_result() -> GuidelineResult<()> {
            Ok(())
        }

        fn returns_tool_result() -> ToolResult<()> {
            Ok(())
        }

        fn returns_journey_result() -> JourneyResult<()> {
            Ok(())
        }

        assert!(returns_result().is_ok());
        assert!(returns_storage_result().is_ok());
        assert!(returns_guideline_result().is_ok());
        assert!(returns_tool_result().is_ok());
        assert!(returns_journey_result().is_ok());
    }
}
