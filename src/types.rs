//! Common type definitions used throughout the Talk library
//!
//! This module provides newtype wrappers around UUID for type-safe identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an Agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(Uuid);

impl AgentId {
    /// Create a new random AgentId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AgentId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Guideline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GuidelineId(Uuid);

impl GuidelineId {
    /// Create a new random GuidelineId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for GuidelineId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GuidelineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for GuidelineId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(Uuid);

impl ToolId {
    /// Create a new random ToolId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ToolId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Journey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JourneyId(Uuid);

impl JourneyId {
    /// Create a new random JourneyId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for JourneyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JourneyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for JourneyId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Journey Step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(Uuid);

impl StepId {
    /// Create a new random StepId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for StepId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random SessionId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a Message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Create a new random MessageId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1, id2, "AgentIds should be unique");
    }

    #[test]
    fn test_agent_id_display() {
        let id = AgentId::new();
        let display_str = format!("{}", id);
        assert!(
            !display_str.is_empty(),
            "Display string should not be empty"
        );
    }

    #[test]
    fn test_agent_id_serialization() {
        let id = AgentId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: AgentId = serde_json::from_str(&json).unwrap();
        assert_eq!(
            id, deserialized,
            "AgentId should serialize and deserialize correctly"
        );
    }

    #[test]
    fn test_guideline_id_creation() {
        let id1 = GuidelineId::new();
        let id2 = GuidelineId::new();
        assert_ne!(id1, id2, "GuidelineIds should be unique");
    }

    #[test]
    fn test_tool_id_creation() {
        let id1 = ToolId::new();
        let id2 = ToolId::new();
        assert_ne!(id1, id2, "ToolIds should be unique");
    }

    #[test]
    fn test_journey_id_creation() {
        let id1 = JourneyId::new();
        let id2 = JourneyId::new();
        assert_ne!(id1, id2, "JourneyIds should be unique");
    }

    #[test]
    fn test_step_id_creation() {
        let id1 = StepId::new();
        let id2 = StepId::new();
        assert_ne!(id1, id2, "StepIds should be unique");
    }

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2, "SessionIds should be unique");
    }

    #[test]
    fn test_message_id_creation() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2, "MessageIds should be unique");
    }

    #[test]
    fn test_all_ids_from_uuid() {
        let uuid = Uuid::new_v4();

        let agent_id = AgentId::from(uuid);
        assert_eq!(agent_id.as_uuid(), &uuid);

        let guideline_id = GuidelineId::from(uuid);
        assert_eq!(guideline_id.as_uuid(), &uuid);

        let tool_id = ToolId::from(uuid);
        assert_eq!(tool_id.as_uuid(), &uuid);

        let journey_id = JourneyId::from(uuid);
        assert_eq!(journey_id.as_uuid(), &uuid);

        let step_id = StepId::from(uuid);
        assert_eq!(step_id.as_uuid(), &uuid);

        let session_id = SessionId::from(uuid);
        assert_eq!(session_id.as_uuid(), &uuid);

        let message_id = MessageId::from(uuid);
        assert_eq!(message_id.as_uuid(), &uuid);
    }
}
