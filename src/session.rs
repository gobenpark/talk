//! Session management for agent conversations
//!
//! This module provides data structures for managing conversation sessions,
//! including session metadata, status tracking, and journey state.

use crate::context::Context;
use crate::types::{AgentId, JourneyId, SessionId, StepId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a conversation session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session is active and can process messages
    Active,
    /// Session is paused (temporarily inactive)
    Paused,
    /// Session has been completed
    Completed,
    /// Session has been terminated/cancelled
    Terminated,
}

/// Journey state within a session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyState {
    /// ID of the journey being executed
    pub journey_id: JourneyId,
    /// Current step in the journey
    pub current_step: StepId,
    /// When the journey was started
    pub started_at: DateTime<Utc>,
    /// When the journey was completed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Step history (ordered list of visited steps)
    pub step_history: Vec<StepId>,
}

impl JourneyState {
    /// Create a new journey state
    pub fn new(journey_id: JourneyId, initial_step: StepId) -> Self {
        Self {
            journey_id,
            current_step: initial_step,
            started_at: Utc::now(),
            completed_at: None,
            step_history: vec![initial_step],
        }
    }

    /// Move to the next step
    pub fn move_to_step(&mut self, step_id: StepId) {
        self.current_step = step_id;
        self.step_history.push(step_id);
    }

    /// Mark the journey as completed
    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
    }

    /// Check if the journey is completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
}

/// A conversation session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for the session
    pub id: SessionId,
    /// ID of the agent managing this session
    pub agent_id: AgentId,
    /// Current status of the session
    pub status: SessionStatus,
    /// Conversation context (messages and variables)
    pub context: Context,
    /// Active journey state (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey_state: Option<JourneyState>,
    /// Session metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// When the session expires (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Session {
    /// Create a new session
    pub fn new(agent_id: AgentId) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            agent_id,
            status: SessionStatus::Active,
            context: Context::new(),
            journey_state: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    /// Create a new session with custom context
    pub fn with_context(agent_id: AgentId, context: Context) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            agent_id,
            status: SessionStatus::Active,
            context,
            journey_state: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    /// Set session expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Add metadata to the session
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Update the session's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
        self.touch();
    }

    /// Resume the session
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
            self.touch();
        }
    }

    /// Complete the session
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.touch();
    }

    /// Terminate the session
    pub fn terminate(&mut self) {
        self.status = SessionStatus::Terminated;
        self.touch();
    }

    /// Start a journey in this session
    pub fn start_journey(&mut self, journey_id: JourneyId, initial_step: StepId) {
        self.journey_state = Some(JourneyState::new(journey_id, initial_step));
        self.touch();
    }

    /// Get the current journey state
    pub fn get_journey_state(&self) -> Option<&JourneyState> {
        self.journey_state.as_ref()
    }

    /// Get mutable journey state
    pub fn get_journey_state_mut(&mut self) -> Option<&mut JourneyState> {
        self.journey_state.as_mut()
    }

    /// Complete the active journey
    pub fn complete_journey(&mut self) {
        if let Some(ref mut journey) = self.journey_state {
            journey.complete();
            self.touch();
        }
    }

    /// Clear the journey state
    pub fn clear_journey(&mut self) {
        self.journey_state = None;
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Message;
    use chrono::Duration;

    #[test]
    fn test_session_status_serialization() {
        let status = SessionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let deserialized: SessionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_journey_state_creation() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let state = JourneyState::new(journey_id, step_id);

        assert_eq!(state.journey_id, journey_id);
        assert_eq!(state.current_step, step_id);
        assert_eq!(state.step_history.len(), 1);
        assert!(!state.is_completed());
    }

    #[test]
    fn test_journey_state_move_to_step() {
        let journey_id = JourneyId::new();
        let step1 = StepId::new();
        let step2 = StepId::new();
        let step3 = StepId::new();

        let mut state = JourneyState::new(journey_id, step1);
        state.move_to_step(step2);
        state.move_to_step(step3);

        assert_eq!(state.current_step, step3);
        assert_eq!(state.step_history.len(), 3);
        assert_eq!(state.step_history[0], step1);
        assert_eq!(state.step_history[1], step2);
        assert_eq!(state.step_history[2], step3);
    }

    #[test]
    fn test_journey_state_complete() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let mut state = JourneyState::new(journey_id, step_id);

        assert!(!state.is_completed());
        state.complete();
        assert!(state.is_completed());
        assert!(state.completed_at.is_some());
    }

    #[test]
    fn test_session_creation() {
        let agent_id = AgentId::new();
        let session = Session::new(agent_id);

        assert_eq!(session.agent_id, agent_id);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.context.messages.is_empty());
        assert!(session.journey_state.is_none());
        assert!(session.metadata.is_empty());
    }

    #[test]
    fn test_session_with_context() {
        let agent_id = AgentId::new();
        let mut context = Context::new();
        context.add_message(Message::user("Hello"));

        let session = Session::with_context(agent_id, context.clone());

        assert_eq!(session.context.messages.len(), 1);
    }

    #[test]
    fn test_session_with_expiration() {
        let agent_id = AgentId::new();
        let expires_at = Utc::now() + Duration::hours(24);
        let session = Session::new(agent_id).with_expiration(expires_at);

        assert!(session.expires_at.is_some());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expiration() {
        let agent_id = AgentId::new();
        let expires_at = Utc::now() - Duration::hours(1); // Expired 1 hour ago
        let session = Session::new(agent_id).with_expiration(expires_at);

        assert!(session.is_expired());
    }

    #[test]
    fn test_session_with_metadata() {
        let agent_id = AgentId::new();
        let session = Session::new(agent_id)
            .with_metadata("user_id", serde_json::json!("user123"))
            .with_metadata("language", serde_json::json!("en"));

        assert_eq!(session.metadata.len(), 2);
        assert_eq!(
            session.metadata.get("user_id"),
            Some(&serde_json::json!("user123"))
        );
    }

    #[test]
    fn test_session_lifecycle() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);

        assert!(session.is_active());

        session.pause();
        assert_eq!(session.status, SessionStatus::Paused);
        assert!(!session.is_active());

        session.resume();
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.is_active());

        session.complete();
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(!session.is_active());
    }

    #[test]
    fn test_session_terminate() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);

        session.terminate();
        assert_eq!(session.status, SessionStatus::Terminated);
        assert!(!session.is_active());
    }

    #[test]
    fn test_session_touch() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);

        let initial_updated_at = session.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.touch();
        assert!(session.updated_at > initial_updated_at);
    }

    #[test]
    fn test_session_journey_lifecycle() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);
        let journey_id = JourneyId::new();
        let step_id = StepId::new();

        assert!(session.get_journey_state().is_none());

        session.start_journey(journey_id, step_id);
        assert!(session.get_journey_state().is_some());
        assert_eq!(session.get_journey_state().unwrap().journey_id, journey_id);

        session.complete_journey();
        assert!(session.get_journey_state().unwrap().is_completed());

        session.clear_journey();
        assert!(session.get_journey_state().is_none());
    }

    #[test]
    fn test_session_journey_state_mut() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);
        let journey_id = JourneyId::new();
        let step1 = StepId::new();
        let step2 = StepId::new();

        session.start_journey(journey_id, step1);

        if let Some(journey) = session.get_journey_state_mut() {
            journey.move_to_step(step2);
        }

        assert_eq!(session.get_journey_state().unwrap().current_step, step2);
    }

    #[test]
    fn test_session_serialization() {
        let agent_id = AgentId::new();
        let session = Session::new(agent_id).with_metadata("test", serde_json::json!("value"));

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.agent_id, deserialized.agent_id);
        assert_eq!(session.status, deserialized.status);
        assert_eq!(session.metadata, deserialized.metadata);
    }

    #[test]
    fn test_session_with_journey_serialization() {
        let agent_id = AgentId::new();
        let mut session = Session::new(agent_id);
        let journey_id = JourneyId::new();
        let step_id = StepId::new();

        session.start_journey(journey_id, step_id);

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert!(deserialized.journey_state.is_some());
        assert_eq!(
            deserialized.journey_state.as_ref().unwrap().journey_id,
            journey_id
        );
    }
}
