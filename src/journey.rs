//! Conversation Journey System
//!
//! This module provides multi-step conversation flows with conditional transitions.
//!
//! # Example
//!
//! ```rust,no_run
//! use talk::{Journey, JourneyStep, Transition, TransitionCondition, StepId, JourneyId};
//! use chrono::Utc;
//!
//! let step1_id = StepId::new();
//! let step2_id = StepId::new();
//!
//! let journey = Journey {
//!     id: JourneyId::new(),
//!     name: "Onboarding".to_string(),
//!     description: "New user onboarding".to_string(),
//!     steps: vec![
//!         JourneyStep {
//!             id: step1_id,
//!             name: "Welcome".to_string(),
//!             prompt: "Welcome! What's your name?".to_string(),
//!             expected_response: Some(".*".to_string()),
//!             transitions: vec![Transition {
//!                 condition: TransitionCondition::Always,
//!                 next_step: step2_id,
//!             }],
//!             actions: vec![],
//!         },
//!         JourneyStep {
//!             id: step2_id,
//!             name: "Complete".to_string(),
//!             prompt: "Thank you!".to_string(),
//!             expected_response: None,
//!             transitions: vec![],
//!             actions: vec!["complete".to_string()],
//!         },
//!     ],
//!     initial_step: step1_id,
//!     current_step: None,
//!     created_at: Utc::now(),
//! };
//! ```

use crate::context::Context;
use crate::error::AgentError;
use crate::types::{JourneyId, SessionId, StepId};
use crate::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Multi-step conversation journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journey {
    /// Unique journey identifier
    pub id: JourneyId,

    /// Human-readable name
    pub name: String,

    /// Journey description
    pub description: String,

    /// All steps in this journey
    pub steps: Vec<JourneyStep>,

    /// ID of the first step
    pub initial_step: StepId,

    /// Current step (None if not started)
    pub current_step: Option<StepId>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Individual step within a journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    /// Unique step identifier
    pub id: StepId,

    /// Human-readable step name
    pub name: String,

    /// Prompt to display to user
    pub prompt: String,

    /// Expected response pattern (regex)
    pub expected_response: Option<String>,

    /// Possible transitions to next steps
    pub transitions: Vec<Transition>,

    /// Actions to execute when reaching this step
    pub actions: Vec<String>,
}

/// Transition from one step to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Condition that must be met for transition
    pub condition: TransitionCondition,

    /// Next step to transition to
    pub next_step: StepId,
}

/// Condition for transitioning between steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// Always transition
    Always,

    /// Transition if user message matches regex
    Match(String),

    /// Transition if context variable matches value
    ContextVariable { key: String, value: String },
}

/// Runtime state of a journey for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyState {
    /// Journey being executed
    pub journey_id: JourneyId,

    /// Current step in the journey
    pub current_step: StepId,

    /// Steps already completed
    pub completed_steps: Vec<StepId>,

    /// Whether journey is complete
    pub is_complete: bool,

    /// Additional state data
    pub metadata: HashMap<String, serde_json::Value>,

    /// When journey was started
    pub started_at: DateTime<Utc>,

    /// When journey was completed (if is_complete)
    pub completed_at: Option<DateTime<Utc>>,
}

impl JourneyState {
    /// Create a new journey state
    pub fn new(journey_id: JourneyId, initial_step: StepId) -> Self {
        Self {
            journey_id,
            current_step: initial_step,
            completed_steps: Vec::new(),
            is_complete: false,
            metadata: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Mark a step as completed
    pub fn complete_step(&mut self, step_id: StepId) {
        if !self.completed_steps.contains(&step_id) {
            self.completed_steps.push(step_id);
        }
    }

    /// Mark journey as complete
    pub fn mark_complete(&mut self) {
        self.is_complete = true;
        self.completed_at = Some(Utc::now());
    }
}

/// Trait for managing conversation journeys
#[async_trait]
pub trait JourneyManager: Send + Sync {
    /// Start a journey - returns initial state
    async fn start_journey(
        &self,
        _session_id: &SessionId,
        journey_id: &JourneyId,
    ) -> Result<JourneyState>;

    /// Progress to the next step based on user message
    /// Takes journey_id and current_step_id instead of looking up session state
    async fn process_step(
        &self,
        journey_id: &JourneyId,
        current_step_id: StepId,
        message: &str,
    ) -> Result<JourneyStep>;

    /// Register a new journey
    async fn add_journey(&mut self, journey: Journey) -> Result<JourneyId>;

    /// Get a journey by ID
    fn get_journey(&self, journey_id: &JourneyId) -> Option<&Journey>;
}

/// Default implementation of JourneyManager
pub struct DefaultJourneyManager {
    /// Registered journeys
    journeys: HashMap<JourneyId, Journey>,
}

impl DefaultJourneyManager {
    /// Create a new journey manager
    pub fn new() -> Self {
        Self {
            journeys: HashMap::new(),
        }
    }

    /// Validate a journey's structure
    fn validate_journey(&self, journey: &Journey) -> Result<()> {
        // Check initial_step exists in steps
        let initial_step_exists = journey.steps.iter().any(|s| s.id == journey.initial_step);
        if !initial_step_exists {
            return Err(AgentError::Journey(format!(
                "Initial step {:?} not found in journey steps",
                journey.initial_step
            )));
        }

        // Check all transition targets exist
        let step_ids: HashSet<StepId> = journey.steps.iter().map(|s| s.id).collect();
        for step in &journey.steps {
            for transition in &step.transitions {
                if !step_ids.contains(&transition.next_step) {
                    return Err(AgentError::Journey(format!(
                        "Transition target {:?} not found in journey steps",
                        transition.next_step
                    )));
                }
            }
        }

        // Check for circular dependencies
        self.check_circular_dependencies(journey)?;

        Ok(())
    }

    /// Check for circular dependencies in journey
    fn check_circular_dependencies(&self, _journey: &Journey) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn has_cycle(
            step_id: StepId,
            journey: &Journey,
            visited: &mut HashSet<StepId>,
            rec_stack: &mut HashSet<StepId>,
        ) -> bool {
            if rec_stack.contains(&step_id) {
                return true; // Cycle detected
            }

            if visited.contains(&step_id) {
                return false;
            }

            visited.insert(step_id);
            rec_stack.insert(step_id);

            // Find step and check its transitions
            if let Some(step) = journey.steps.iter().find(|s| s.id == step_id) {
                for transition in &step.transitions {
                    if has_cycle(transition.next_step, journey, visited, rec_stack) {
                        return true;
                    }
                }
            }

            rec_stack.remove(&step_id);
            false
        }

        if has_cycle(_journey.initial_step, _journey, &mut visited, &mut rec_stack) {
            return Err(AgentError::Journey(
                "Circular dependency detected in journey".to_string(),
            ));
        }

        Ok(())
    }

    /// Find a step by ID in a journey
    fn find_step<'a>(&self, journey: &'a Journey, step_id: StepId) -> Option<&'a JourneyStep> {
        journey.steps.iter().find(|s| s.id == step_id)
    }

    /// Evaluate a transition condition
    fn evaluate_condition(
        &self,
        condition: &TransitionCondition,
        message: &str,
        _context: &Context,
    ) -> Result<bool> {
        match condition {
            TransitionCondition::Always => Ok(true),

            TransitionCondition::Match(pattern) => {
                let regex = Regex::new(pattern)
                    .map_err(|e| AgentError::Journey(format!("Invalid regex pattern: {}", e)))?;
                Ok(regex.is_match(message))
            }

            TransitionCondition::ContextVariable { key, value } => {
                // Get value from context
                let context_value = _context
                    .variables
                    .get(key)
                    .and_then(|v| v.value.as_str())
                    .unwrap_or("");
                Ok(context_value == value)
            }
        }
    }

    /// Find the next step based on transitions
    async fn find_next_step(
        &self,
        _journey: &Journey,
        current_step: &JourneyStep,
        message: &str,
        context: &Context,
    ) -> Result<Option<StepId>> {
        for transition in &current_step.transitions {
            if self.evaluate_condition(&transition.condition, message, context)? {
                debug!(
                    step_id = ?current_step.id,
                    next_step = ?transition.next_step,
                    "Transition condition matched"
                );
                return Ok(Some(transition.next_step));
            }
        }

        // No transition matched
        Ok(None)
    }
}

impl Default for DefaultJourneyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JourneyManager for DefaultJourneyManager {
    async fn start_journey(
        &self,
        session_id: &SessionId,
        journey_id: &JourneyId,
    ) -> Result<JourneyState> {
        let journey = self
            .journeys
            .get(journey_id)
            .ok_or_else(|| AgentError::Journey(format!("Journey {:?} not found", journey_id)))?;

        info!(
            session_id = ?session_id,
            journey_id = ?journey_id,
            journey_name = %journey.name,
            "Starting journey"
        );

        let state = JourneyState::new(*journey_id, journey.initial_step);
        Ok(state)
    }

    async fn process_step(
        &self,
        journey_id: &JourneyId,
        current_step_id: StepId,
        message: &str,
    ) -> Result<JourneyStep> {
        let journey = self
            .journeys
            .get(journey_id)
            .ok_or_else(|| AgentError::Journey("Journey not found".to_string()))?;

        let current_step = self
            .find_step(journey, current_step_id)
            .ok_or_else(|| AgentError::Journey("Current step not found".to_string()))?;

        debug!(
            journey_id = ?journey_id,
            current_step = ?current_step.id,
            step_name = %current_step.name,
            message = %message,
            "Processing journey step"
        );

        // Create a temporary context for evaluation
        let context = Context::new();

        // Find next step
        let next_step_id = self
            .find_next_step(journey, current_step, message, &context)
            .await?;

        if let Some(next_id) = next_step_id {
            let next_step = self
                .find_step(journey, next_id)
                .ok_or_else(|| AgentError::Journey("Next step not found".to_string()))?;

            info!(
                journey_id = ?journey_id,
                from_step = ?current_step.id,
                to_step = ?next_step.id,
                "Journey step transition"
            );

            Ok(next_step.clone())
        } else {
            // No transition - this is the final step
            debug!(
                journey_id = ?journey_id,
                current_step = ?current_step.id,
                "No transitions available - journey complete"
            );

            Ok(current_step.clone())
        }
    }

    async fn add_journey(&mut self, journey: Journey) -> Result<JourneyId> {
        // Validate journey structure
        self.validate_journey(&journey)?;

        let journey_id = journey.id;
        info!(
            journey_id = ?journey_id,
            journey_name = %journey.name,
            step_count = journey.steps.len(),
            "Registering journey"
        );

        self.journeys.insert(journey_id, journey);
        Ok(journey_id)
    }

    fn get_journey(&self, journey_id: &JourneyId) -> Option<&Journey> {
        self.journeys.get(journey_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journey_state_new() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let state = JourneyState::new(journey_id, step_id);

        assert_eq!(state.journey_id, journey_id);
        assert_eq!(state.current_step, step_id);
        assert_eq!(state.completed_steps.len(), 0);
        assert!(!state.is_complete);
    }

    #[test]
    fn test_journey_state_complete_step() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let mut state = JourneyState::new(journey_id, step_id);

        state.complete_step(step_id);
        assert_eq!(state.completed_steps.len(), 1);
        assert!(state.completed_steps.contains(&step_id));

        // Completing same step again should not duplicate
        state.complete_step(step_id);
        assert_eq!(state.completed_steps.len(), 1);
    }

    #[test]
    fn test_journey_state_mark_complete() {
        let journey_id = JourneyId::new();
        let step_id = StepId::new();
        let mut state = JourneyState::new(journey_id, step_id);

        assert!(!state.is_complete);
        assert!(state.completed_at.is_none());

        state.mark_complete();
        assert!(state.is_complete);
        assert!(state.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_validate_journey_invalid_initial_step() {
        let manager = DefaultJourneyManager::new();
        let journey = Journey {
            id: JourneyId::new(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            steps: vec![JourneyStep {
                id: StepId::new(),
                name: "Step 1".to_string(),
                prompt: "Prompt".to_string(),
                expected_response: None,
                transitions: vec![],
                actions: vec![],
            }],
            initial_step: StepId::new(), // Different ID
            current_step: None,
            created_at: Utc::now(),
        };

        let result = manager.validate_journey(&journey);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_journey_circular_dependency() {
        let manager = DefaultJourneyManager::new();
        let step1_id = StepId::new();
        let step2_id = StepId::new();

        let journey = Journey {
            id: JourneyId::new(),
            name: "Circular".to_string(),
            description: "Test".to_string(),
            steps: vec![
                JourneyStep {
                    id: step1_id,
                    name: "Step 1".to_string(),
                    prompt: "Prompt".to_string(),
                    expected_response: None,
                    transitions: vec![Transition {
                        condition: TransitionCondition::Always,
                        next_step: step2_id,
                    }],
                    actions: vec![],
                },
                JourneyStep {
                    id: step2_id,
                    name: "Step 2".to_string(),
                    prompt: "Prompt".to_string(),
                    expected_response: None,
                    transitions: vec![Transition {
                        condition: TransitionCondition::Always,
                        next_step: step1_id, // Circular!
                    }],
                    actions: vec![],
                },
            ],
            initial_step: step1_id,
            current_step: None,
            created_at: Utc::now(),
        };

        let result = manager.validate_journey(&journey);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_evaluate_condition_always() {
        let manager = DefaultJourneyManager::new();
        let context = Context::new();
        let condition = TransitionCondition::Always;

        let result = manager.evaluate_condition(&condition, "any message", &context);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_evaluate_condition_match() {
        let manager = DefaultJourneyManager::new();
        let context = Context::new();
        let condition = TransitionCondition::Match(r"(?i)yes".to_string());

        let result_yes = manager.evaluate_condition(&condition, "YES", &context);
        assert!(result_yes.is_ok());
        assert!(result_yes.unwrap());

        let result_no = manager.evaluate_condition(&condition, "no", &context);
        assert!(result_no.is_ok());
        assert!(!result_no.unwrap());
    }
}
