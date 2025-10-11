//! Onboarding Journey Example
//!
//! This example demonstrates how to create and use conversation journeys
//! with the Talk library. It implements a simple onboarding flow that:
//! 1. Welcomes the user and asks for their name
//! 2. Asks what they want to accomplish
//! 3. Confirms and completes onboarding
//!
//! Run with: cargo run --example onboarding_journey

use talk::{
    Agent, AgentConfig, Journey, JourneyStep, LogLevel, OpenAIProvider, StepId, Transition,
    TransitionCondition,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Onboarding Journey Example");
    println!("================================\n");

    // Set up OpenAI API key (you can also use Anthropic)
    // For this example, we'll use a mock key since it's just a demonstration
    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "your-api-key-here".to_string());

    if api_key == "your-api-key-here" {
        println!("⚠️  Note: Set OPENAI_API_KEY environment variable for real LLM responses");
        println!("    This example will still demonstrate the journey system structure\n");
    }

    // Create agent with OpenAI provider
    let mut agent = Agent::builder()
        .name("Onboarding Agent")
        .description("Agent that guides users through onboarding")
        .provider(Box::new(
            OpenAIProvider::new(api_key)
                .with_model("gpt-3.5-turbo")
                .with_temperature(0.7),
        ))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: LogLevel::Info,
        })
        .build()?;

    // Define journey steps
    let step1_id = StepId::new();
    let step2_id = StepId::new();
    let step3_id = StepId::new();

    // Create onboarding journey
    let onboarding_journey = Journey {
        id: talk::JourneyId::new(),
        name: "User Onboarding".to_string(),
        description: "Guided onboarding flow for new users".to_string(),
        steps: vec![
            // Step 1: Welcome and get name
            JourneyStep {
                id: step1_id,
                name: "Welcome".to_string(),
                prompt: "Welcome to our platform! 👋 What's your name?".to_string(),
                expected_response: Some(".*".to_string()), // Accept any response
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: step2_id,
                }],
                actions: vec!["store_name".to_string()],
            },
            // Step 2: Ask about goals
            JourneyStep {
                id: step2_id,
                name: "Goals".to_string(),
                prompt:
                    "Nice to meet you! What are you hoping to accomplish with our platform today?"
                        .to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: step3_id,
                }],
                actions: vec!["store_goals".to_string()],
            },
            // Step 3: Confirmation (final step with no transitions)
            JourneyStep {
                id: step3_id,
                name: "Complete".to_string(),
                prompt: "Perfect! You're all set. We'll help you achieve your goals. 🎉"
                    .to_string(),
                expected_response: None,
                transitions: vec![], // No transitions = final step
                actions: vec!["complete_onboarding".to_string()],
            },
        ],
        initial_step: step1_id,
        current_step: None,
        created_at: chrono::Utc::now(),
    };

    // Register the journey with the agent
    let journey_id = agent.add_journey(onboarding_journey).await?;
    println!("✅ Onboarding journey registered\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("📝 Created session: {}\n", session_id);

    // Start the journey
    let state = agent.start_journey(&session_id, &journey_id).await?;
    println!("🎬 Journey started!");
    println!("   Current Step: {:?}", state.current_step);
    println!("   Is Complete: {}\n", state.is_complete);

    // Simulate user responses
    let user_responses = vec![
        "My name is Alice",
        "I want to learn how to use the Talk library for building AI agents",
        "Thank you!",
    ];

    for (i, response) in user_responses.iter().enumerate() {
        println!("─────────────────────────────────────────────");
        println!("Step {}: User Response", i + 1);
        println!("─────────────────────────────────────────────\n");

        println!("👤 User: {}", response);

        // Process the journey step
        let next_step = agent
            .process_journey_step(&session_id, response)
            .await?;

        println!("\n📍 Journey Step: {}", next_step.name);
        println!("💬 Agent: {}", next_step.prompt);

        // Get updated journey state
        let current_state = agent.get_journey_state(&session_id).await?;

        if let Some(state) = current_state {
            println!("\n📊 Journey Status:");
            println!("   Current Step: {}", next_step.name);
            println!("   Completed Steps: {}", state.completed_steps.len());
            println!("   Is Complete: {}", state.is_complete);

            if !next_step.actions.is_empty() {
                println!("   Actions: {:?}", next_step.actions);
            }

            if state.is_complete {
                println!("\n✅ Journey completed successfully!");
                break;
            }
        }

        println!();
    }

    // End the session
    println!("\n─────────────────────────────────────────────");
    agent.end_session(&session_id).await?;
    println!("✅ Session ended\n");

    // Summary
    println!("📋 Summary");
    println!("─────────────────────────────────────────────");
    println!("Journey Name: User Onboarding");
    println!("Total Steps: 3");
    println!("Outcome: Successfully guided user through onboarding\n");

    println!("💡 Key Concepts Demonstrated:");
    println!("   • Creating multi-step journeys with JourneyStep");
    println!("   • Defining transitions between steps");
    println!("   • Tracking journey state and progress");
    println!("   • Processing user responses through steps");
    println!("   • Conditional transitions (TransitionCondition)");
    println!("   • Journey completion detection\n");

    Ok(())
}
