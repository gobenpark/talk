//! Flight Booking Journey Example
//!
//! This example demonstrates how to create and use conversation journeys
//! with the Talk library. It implements a flight booking flow that:
//! 1. Asks for destination preference
//! 2. Conditionally branches based on customer clarity
//! 3. Suggests destinations if customer is unsure
//! 4. Collects travel dates
//! 5. Searches for flights (tool state)
//! 6. Confirms and completes booking
//!
//! This mirrors the Parlant journey example with fork states and conditional transitions.
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

    println!("🚀 Flight Booking Journey Example");
    println!("===================================\n");

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
        .name("Flight Booking Agent")
        .description("An AI travel agent that helps customers book flights through a guided journey.")
        .provider(Box::new(
            OpenAIProvider::new(api_key)
                .with_model("gpt-4")
                .with_temperature(0.7),
        ))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: LogLevel::Info,
        })
        .build()?;

    // Define journey steps for flight booking
    let ask_destination_id = StepId::new();
    let check_destination_fork_id = StepId::new();
    let suggest_destinations_id = StepId::new();
    let ask_dates_id = StepId::new();
    let search_flights_id = StepId::new();
    let confirm_booking_id = StepId::new();

    // Create flight booking journey
    let flight_booking_journey = Journey {
        id: talk::JourneyId::new(),
        name: "Book Flight".to_string(),
        description: "Guided journey for booking flights with conditional branching".to_string(),
        steps: vec![
            // Step 1: Ask destination
            JourneyStep {
                id: ask_destination_id,
                name: "Ask Destination".to_string(),
                prompt: "✈️ I'd love to help you book a flight! Where would you like to go?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: check_destination_fork_id,
                }],
                actions: vec!["extract_destination".to_string()],
            },
            // Step 2: Fork - Check if destination is clear
            JourneyStep {
                id: check_destination_fork_id,
                name: "Check Destination Fork".to_string(),
                prompt: "Let me check if I understand your destination...".to_string(),
                expected_response: None,
                transitions: vec![
                    // If destination is unclear, suggest options
                    Transition {
                        condition: TransitionCondition::Match("(not sure|don't know|maybe|any)".to_string()),
                        next_step: suggest_destinations_id,
                    },
                    // If destination is clear, proceed to dates
                    Transition {
                        condition: TransitionCondition::Always,
                        next_step: ask_dates_id,
                    },
                ],
                actions: vec!["validate_destination".to_string()],
            },
            // Step 3a: Suggest destinations (branch for unclear destination)
            JourneyStep {
                id: suggest_destinations_id,
                name: "Suggest Destinations".to_string(),
                prompt: "No problem! Here are some popular destinations:\n• Paris 🇫🇷\n• Tokyo 🇯🇵\n• New York 🇺🇸\n\nWhich one sounds interesting?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: ask_dates_id,
                }],
                actions: vec!["store_destination".to_string()],
            },
            // Step 4: Ask travel dates (merge point)
            JourneyStep {
                id: ask_dates_id,
                name: "Ask Dates".to_string(),
                prompt: "Great choice! When would you like to travel? Please provide departure and return dates.".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: search_flights_id,
                }],
                actions: vec!["extract_dates".to_string()],
            },
            // Step 5: Search flights (Tool state)
            JourneyStep {
                id: search_flights_id,
                name: "Search Flights".to_string(),
                prompt: "🔍 Searching for available flights...".to_string(),
                expected_response: None,
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: confirm_booking_id,
                }],
                actions: vec!["search_flights_tool".to_string()],
            },
            // Step 6: Confirm booking (final step)
            JourneyStep {
                id: confirm_booking_id,
                name: "Confirm Booking".to_string(),
                prompt: "I found some great options! Here are the details:\n\n✈️ Flight: [Details]\n📅 Dates: [Dates]\n💰 Price: $XXX\n\nWould you like to proceed with this booking?".to_string(),
                expected_response: Some("(yes|confirm|book|proceed)".to_string()),
                transitions: vec![], // Final step
                actions: vec!["confirm_booking_tool".to_string()],
            },
        ],
        initial_step: ask_destination_id,
        current_step: None,
        created_at: chrono::Utc::now(),
    };

    // Register the journey with the agent
    let journey_id = agent.add_journey(flight_booking_journey).await?;
    println!("✅ Flight booking journey registered\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("📝 Created session: {}\n", session_id);

    // Start the journey
    let state = agent.start_journey(&session_id, &journey_id).await?;
    println!("🎬 Journey started!");
    println!("   Current Step: {:?}", state.current_step);
    println!("   Is Complete: {}\n", state.is_complete);

    // Simulate user responses through the flight booking journey
    let user_responses = vec![
        "I'd like to go to Paris",           // Clear destination
        "Leaving June 15th, returning June 22nd",  // Dates
        "",                                   // Auto-transition for search
        "Yes, please book it!",              // Confirmation
    ];

    for (i, response) in user_responses.iter().enumerate() {
        println!("─────────────────────────────────────────────");
        println!("단계 {}: 사용자 응답", i + 1);
        println!("─────────────────────────────────────────────\n");

        println!("👤 사용자: {}", response);

        // Process the journey step to get transition logic
        let next_step = agent
            .process_journey_step(&session_id, response)
            .await?;

        println!("\n📍 Journey 단계: {}", next_step.name);

        // Get real LLM response using the journey prompt as context
        let llm_response = if !response.is_empty() {
            // Process message through LLM with journey context
            agent.process_message(session_id.clone(), response.to_string()).await?
        } else {
            // For empty responses (auto-transitions), use the journey prompt
            talk::AgentResponse {
                message: next_step.prompt.clone(),
                matched_guideline: None,
                tools_used: vec![],
                journey_step: Some(next_step.id),
                context_updates: std::collections::HashMap::new(),
                explanation: None,
            }
        };

        println!("💬 에이전트: {}", llm_response.message);

        // Get updated journey state
        let current_state = agent.get_journey_state(&session_id).await?;

        if let Some(state) = current_state {
            println!("\n📊 Journey 상태:");
            println!("   현재 단계: {}", next_step.name);
            println!("   완료된 단계: {}", state.completed_steps.len());
            println!("   완료 여부: {}", state.is_complete);

            if !next_step.actions.is_empty() {
                println!("   액션: {:?}", next_step.actions);
            }

            if state.is_complete {
                println!("\n✅ Journey가 성공적으로 완료되었습니다!");
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
    println!("Journey Name: Book Flight");
    println!("Total Steps: 6 (with conditional branching)");
    println!("Outcome: Successfully guided customer through flight booking\n");

    println!("💡 Key Concepts Demonstrated:");
    println!("   • Creating multi-step journeys with JourneyStep");
    println!("   • Conditional transitions (MessageMatches)");
    println!("   • Fork states for branching logic");
    println!("   • Merge points where branches reconverge");
    println!("   • Tool states for external operations");
    println!("   • Tracking journey state and progress");
    println!("   • Processing user responses through steps");
    println!("   • Journey completion detection\n");

    Ok(())
}
