//! Journey API Tests (T066-T070)
//!
//! Following TDD: Write tests FIRST, ensure they FAIL, then implement.
//!
//! Test Coverage:
//! - T066: start_journey - Journey initialization
//! - T067: process_step - Step progression based on transitions
//! - T068: get_state - State retrieval
//! - T069: end_journey - Journey termination
//! - T070: add_journey - Journey registration

use talk::{
    Agent, AgentConfig, Journey, JourneyId, JourneyManager, JourneyState, JourneyStep,
    LogLevel, OpenAIProvider, SessionId, StepId, Transition, TransitionCondition,
};
use std::collections::HashMap;
use std::time::Duration;

// Helper: Create a test agent with OpenAI provider
fn create_test_agent() -> Agent {
    Agent::builder()
        .name("Test Journey Agent")
        .description("Agent for testing journey functionality")
        .provider(Box::new(
            OpenAIProvider::new("test-api-key")
                .with_model("gpt-3.5-turbo")
                .with_temperature(0.7),
        ))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: LogLevel::Info,
        })
        .build()
        .expect("Failed to build test agent")
}

// Helper: Create a simple two-step onboarding journey
fn create_onboarding_journey() -> Journey {
    let step1_id = StepId::new();
    let step2_id = StepId::new();

    Journey {
        id: JourneyId::new(),
        name: "Onboarding".to_string(),
        description: "New user onboarding flow".to_string(),
        steps: vec![
            JourneyStep {
                id: step1_id,
                name: "Welcome".to_string(),
                prompt: "Welcome! What's your name?".to_string(),
                expected_response: Some(".*".to_string()), // Accept any response
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: step2_id,
                }],
                actions: vec![],
            },
            JourneyStep {
                id: step2_id,
                name: "Confirm".to_string(),
                prompt: "Nice to meet you! Ready to start?".to_string(),
                expected_response: None,
                transitions: vec![],
                actions: vec!["complete_onboarding".to_string()],
            },
        ],
        initial_step: step1_id,
        current_step: None,
        created_at: chrono::Utc::now(),
    }
}

// Helper: Create a journey with conditional transitions
fn create_conditional_journey() -> Journey {
    let step1_id = StepId::new();
    let step2_yes_id = StepId::new();
    let step2_no_id = StepId::new();

    Journey {
        id: JourneyId::new(),
        name: "Conditional Flow".to_string(),
        description: "Journey with conditional transitions".to_string(),
        steps: vec![
            JourneyStep {
                id: step1_id,
                name: "Question".to_string(),
                prompt: "Do you want to continue? (yes/no)".to_string(),
                expected_response: Some(r"(?i)(yes|no)".to_string()),
                transitions: vec![
                    Transition {
                        condition: TransitionCondition::Match(r"(?i)yes".to_string()),
                        next_step: step2_yes_id,
                    },
                    Transition {
                        condition: TransitionCondition::Match(r"(?i)no".to_string()),
                        next_step: step2_no_id,
                    },
                ],
                actions: vec![],
            },
            JourneyStep {
                id: step2_yes_id,
                name: "Affirmative".to_string(),
                prompt: "Great! Let's proceed.".to_string(),
                expected_response: None,
                transitions: vec![],
                actions: vec!["proceed".to_string()],
            },
            JourneyStep {
                id: step2_no_id,
                name: "Negative".to_string(),
                prompt: "Okay, we'll stop here.".to_string(),
                expected_response: None,
                transitions: vec![],
                actions: vec!["cancel".to_string()],
            },
        ],
        initial_step: step1_id,
        current_step: None,
        created_at: chrono::Utc::now(),
    }
}

/// T066: Test start_journey
///
/// Requirement: Journey must be initialized with initial step set as current step
///
/// Acceptance Criteria:
/// - Journey state is created
/// - Current step is set to journey's initial_step
/// - Journey state is stored in session context
#[tokio::test]
async fn test_start_journey() {
    let mut agent = create_test_agent();
    let journey = create_onboarding_journey();
    let journey_id = journey.id;
    let initial_step_id = journey.initial_step;

    // Register journey
    agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Create session
    let session_id = agent.create_session().await.expect("Failed to create session");

    // Start journey
    let state = agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to start journey");

    // Verify journey state
    assert_eq!(state.journey_id, journey_id);
    assert_eq!(state.current_step, initial_step_id);
    assert_eq!(state.completed_steps.len(), 0);
    assert!(!state.is_complete);

    // Verify journey state is stored in session
    let retrieved_state = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get journey state");

    assert!(retrieved_state.is_some());
    let retrieved_state = retrieved_state.unwrap();
    assert_eq!(retrieved_state.journey_id, journey_id);
    assert_eq!(retrieved_state.current_step, initial_step_id);
}

/// T067: Test process_step
///
/// Requirement: Journey must progress through steps based on transitions
///
/// Acceptance Criteria:
/// - Current step is updated based on transition conditions
/// - Step actions are recorded in journey state
/// - Completed steps are tracked
/// - Final step marks journey as complete
#[tokio::test]
async fn test_process_step() {
    let mut agent = create_test_agent();
    let journey = create_onboarding_journey();
    let journey_id = journey.id;
    let step1_id = journey.steps[0].id;
    let step2_id = journey.steps[1].id;

    // Register journey
    agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Create session and start journey
    let session_id = agent.create_session().await.expect("Failed to create session");
    agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to start journey");

    // Process first step with user message
    let next_step = agent
        .process_journey_step(&session_id, "My name is Alice")
        .await
        .expect("Failed to process step");

    // Verify transition to step 2
    assert_eq!(next_step.id, step2_id);
    assert_eq!(next_step.name, "Confirm");

    // Verify journey state updated
    let state = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get journey state")
        .expect("Journey state not found");

    assert_eq!(state.current_step, step2_id);
    assert_eq!(state.completed_steps.len(), 1);
    assert!(state.completed_steps.contains(&step1_id));
    assert!(!state.is_complete);

    // Process final step (no transitions)
    let final_step = agent
        .process_journey_step(&session_id, "Yes, I'm ready!")
        .await
        .expect("Failed to process final step");

    // Verify journey completion
    assert_eq!(final_step.id, step2_id); // Still on step2 but marked complete

    let final_state = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get final state")
        .expect("Journey state not found");

    assert_eq!(final_state.completed_steps.len(), 2);
    assert!(final_state.completed_steps.contains(&step2_id));
    assert!(final_state.is_complete);
}

/// T068: Test get_state
///
/// Requirement: Journey state must be retrievable at any point
///
/// Acceptance Criteria:
/// - Returns None if no journey active
/// - Returns current state if journey active
/// - State includes journey_id, current_step, completed_steps, is_complete
#[tokio::test]
async fn test_get_state() {
    let mut agent = create_test_agent();
    let journey = create_onboarding_journey();
    let journey_id = journey.id;

    // Register journey
    agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Create session
    let session_id = agent.create_session().await.expect("Failed to create session");

    // Get state before starting journey - should be None
    let state_before = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get state");
    assert!(state_before.is_none());

    // Start journey
    agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to start journey");

    // Get state after starting - should exist
    let state_after = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get state");
    assert!(state_after.is_some());

    let state = state_after.unwrap();
    assert_eq!(state.journey_id, journey_id);
    assert_eq!(state.completed_steps.len(), 0);
    assert!(!state.is_complete);
}

/// T069: Test end_journey
///
/// Requirement: Journey must be cleanly terminable
///
/// Acceptance Criteria:
/// - Journey state is removed from session
/// - Subsequent get_state returns None
/// - Journey can be restarted after ending
#[tokio::test]
async fn test_end_journey() {
    let mut agent = create_test_agent();
    let journey = create_onboarding_journey();
    let journey_id = journey.id;

    // Register journey
    agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Create session and start journey
    let session_id = agent.create_session().await.expect("Failed to create session");
    agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to start journey");

    // Verify journey is active
    let state_before = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get state");
    assert!(state_before.is_some());

    // End journey
    agent
        .end_journey(&session_id)
        .await
        .expect("Failed to end journey");

    // Verify journey state is removed
    let state_after = agent
        .get_journey_state(&session_id)
        .await
        .expect("Failed to get state");
    assert!(state_after.is_none());

    // Verify journey can be restarted
    let restarted_state = agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to restart journey");

    assert_eq!(restarted_state.journey_id, journey_id);
    assert_eq!(restarted_state.completed_steps.len(), 0);
}

/// T070: Test add_journey
///
/// Requirement: Journey must be registrable with the agent
///
/// Acceptance Criteria:
/// - Journey is stored and retrievable by ID
/// - Journey ID is returned
/// - Journey validation (no circular dependencies, valid initial_step)
#[tokio::test]
async fn test_add_journey() {
    let mut agent = create_test_agent();
    let journey = create_onboarding_journey();
    let journey_id = journey.id;
    let journey_name = journey.name.clone();

    // Add journey
    let returned_id = agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Verify returned ID matches
    assert_eq!(returned_id, journey_id);

    // Verify journey can be started (implicitly tests it's stored)
    let session_id = agent.create_session().await.expect("Failed to create session");
    let state = agent
        .start_journey(&session_id, &journey_id)
        .await
        .expect("Failed to start registered journey");

    assert_eq!(state.journey_id, journey_id);
}

/// Additional Test: Conditional transitions
///
/// Requirement: Journey must support conditional transitions based on user input
///
/// Acceptance Criteria:
/// - Match conditions route to correct next step
/// - ContextVariable conditions access session context
#[tokio::test]
async fn test_conditional_transitions() {
    let mut agent = create_test_agent();
    let journey = create_conditional_journey();
    let journey_id = journey.id;
    let step_yes_id = journey.steps[1].id;
    let step_no_id = journey.steps[2].id;

    // Register journey
    agent
        .add_journey(journey)
        .await
        .expect("Failed to add journey");

    // Test YES path
    let session_yes = agent.create_session().await.expect("Failed to create session");
    agent
        .start_journey(&session_yes, &journey_id)
        .await
        .expect("Failed to start journey");

    let next_step_yes = agent
        .process_journey_step(&session_yes, "yes")
        .await
        .expect("Failed to process step");

    assert_eq!(next_step_yes.id, step_yes_id);
    assert_eq!(next_step_yes.name, "Affirmative");

    // Test NO path
    let session_no = agent.create_session().await.expect("Failed to create session");
    agent
        .start_journey(&session_no, &journey_id)
        .await
        .expect("Failed to start journey");

    let next_step_no = agent
        .process_journey_step(&session_no, "no")
        .await
        .expect("Failed to process step");

    assert_eq!(next_step_no.id, step_no_id);
    assert_eq!(next_step_no.name, "Negative");
}

/// Additional Test: Journey validation
///
/// Requirement: Invalid journeys must be rejected
///
/// Acceptance Criteria:
/// - Reject journeys with invalid initial_step
/// - Reject journeys with circular dependencies
/// - Reject journeys with unreachable steps
#[tokio::test]
async fn test_journey_validation() {
    let mut agent = create_test_agent();

    // Test 1: Invalid initial_step
    let invalid_initial_step = Journey {
        id: JourneyId::new(),
        name: "Invalid Initial Step".to_string(),
        description: "Journey with non-existent initial step".to_string(),
        steps: vec![JourneyStep {
            id: StepId::new(),
            name: "Step 1".to_string(),
            prompt: "Prompt".to_string(),
            expected_response: None,
            transitions: vec![],
            actions: vec![],
        }],
        initial_step: StepId::new(), // Different ID, not in steps
        current_step: None,
        created_at: chrono::Utc::now(),
    };

    let result = agent.add_journey(invalid_initial_step).await;
    assert!(result.is_err());

    // Test 2: Circular dependency (step1 -> step2 -> step1)
    let step1_id = StepId::new();
    let step2_id = StepId::new();

    let circular_journey = Journey {
        id: JourneyId::new(),
        name: "Circular Journey".to_string(),
        description: "Journey with circular dependency".to_string(),
        steps: vec![
            JourneyStep {
                id: step1_id,
                name: "Step 1".to_string(),
                prompt: "First step".to_string(),
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
                prompt: "Second step".to_string(),
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
        created_at: chrono::Utc::now(),
    };

    let result = agent.add_journey(circular_journey).await;
    assert!(result.is_err());
}
