//! Customer Complaint Handling Journey Example
//!
//! This example demonstrates a customer service journey that handles complaints through:
//! 1. Identifies complaint type (product, service, delivery)
//! 2. Assesses urgency level (urgent/normal)
//! 3. Conditional branching based on urgency
//! 4. Collects detailed information
//! 5. Proposes solution
//! 6. Confirms customer satisfaction
//!
//! Run with: cargo run --example customer_complaint

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

    println!("📞 Customer Complaint Handling Journey");
    println!("======================================\n");


    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "your-api-key-here".to_string());

    if api_key == "your-api-key-here" {
        println!("⚠️  Note: Set OPENAI_API_KEY environment variable for real LLM responses\n");
    }

    // Create customer service agent
    let mut agent = Agent::builder()
        .name("Customer Service Agent")
        .description("An empathetic AI agent that handles customer complaints efficiently.")
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

    // Define journey steps for complaint handling
    let greet_customer_id = StepId::new();
    let identify_issue_id = StepId::new();
    let assess_urgency_id = StepId::new();
    let urgent_escalation_id = StepId::new();
    let collect_details_id = StepId::new();
    let propose_solution_id = StepId::new();
    let confirm_satisfaction_id = StepId::new();

    // Create complaint handling journey
    let complaint_journey = Journey {
        id: talk::JourneyId::new(),
        name: "Handle Customer Complaint".to_string(),
        description: "Guided journey for handling customer complaints with urgency-based branching".to_string(),
        steps: vec![
            // Step 1: Greet customer
            JourneyStep {
                id: greet_customer_id,
                name: "Greet Customer".to_string(),
                prompt: "👋 Hello! I'm here to help resolve your issue. I understand this can be frustrating, and I'll do my best to assist you. Could you please tell me what happened?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: identify_issue_id,
                }],
                actions: vec![],
            },
            // Step 2: Identify complaint type
            JourneyStep {
                id: identify_issue_id,
                name: "Identify Issue Type".to_string(),
                prompt: "I understand. Let me help you with this. Is this related to:\n• Product quality issue\n• Service problem\n• Delivery/shipping issue\n• Billing concern\n• Other".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: assess_urgency_id,
                }],
                actions: vec!["categorize_complaint".to_string()],
            },
            // Step 3: Fork - Assess urgency
            JourneyStep {
                id: assess_urgency_id,
                name: "Assess Urgency".to_string(),
                prompt: "Thank you for that information. How urgent is this issue for you?".to_string(),
                expected_response: None,
                transitions: vec![
                    // If urgent, escalate immediately
                    Transition {
                        condition: TransitionCondition::Match("(urgent|emergency|asap|immediately|critical)".to_string()),
                        next_step: urgent_escalation_id,
                    },
                    // If normal, continue with standard flow
                    Transition {
                        condition: TransitionCondition::Always,
                        next_step: collect_details_id,
                    },
                ],
                actions: vec!["evaluate_urgency".to_string()],
            },
            // Step 4a: Urgent escalation (branch for urgent cases)
            JourneyStep {
                id: urgent_escalation_id,
                name: "Urgent Escalation".to_string(),
                prompt: "🚨 I understand this is urgent. I'm connecting you with a senior support specialist right away. They will reach out to you within the next 15 minutes. In the meantime, could you provide your contact information and a brief summary?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: confirm_satisfaction_id,
                }],
                actions: vec!["escalate_to_specialist".to_string(), "create_urgent_ticket".to_string()],
            },
            // Step 5: Collect detailed information (merge point)
            JourneyStep {
                id: collect_details_id,
                name: "Collect Details".to_string(),
                prompt: "I'd like to help resolve this for you right away. Could you please provide:\n• Order number or account ID (if applicable)\n• When did this issue occur?\n• Have you tried any steps to resolve it?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: propose_solution_id,
                }],
                actions: vec!["extract_details".to_string()],
            },
            // Step 6: Propose solution
            JourneyStep {
                id: propose_solution_id,
                name: "Propose Solution".to_string(),
                prompt: "🔧 Based on your situation, here's what I can do:\n\n• [Solution Option 1]\n• [Solution Option 2]\n• [Solution Option 3]\n\nWhich option works best for you? Or would you like me to explore other alternatives?".to_string(),
                expected_response: Some(".*".to_string()),
                transitions: vec![Transition {
                    condition: TransitionCondition::Always,
                    next_step: confirm_satisfaction_id,
                }],
                actions: vec!["generate_solution".to_string(), "create_ticket".to_string()],
            },
            // Step 7: Confirm satisfaction (final step)
            JourneyStep {
                id: confirm_satisfaction_id,
                name: "Confirm Satisfaction".to_string(),
                prompt: "✅ I've documented everything and created a ticket for tracking. Is there anything else I can help you with today? Your feedback helps us improve our service.".to_string(),
                expected_response: Some("(yes|no|thanks|thank you|all set|nothing else)".to_string()),
                transitions: vec![], // Final step
                actions: vec!["record_satisfaction".to_string(), "close_ticket".to_string()],
            },
        ],
        initial_step: greet_customer_id,
        current_step: None,
        created_at: chrono::Utc::now(),
    };

    // Register the journey with the agent
    let journey_id = agent.add_journey(complaint_journey).await?;
    println!("✅ Complaint handling journey registered\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("📝 Created session: {}\n", session_id);

    // Start the journey
    let state = agent.start_journey(&session_id, &journey_id).await?;
    println!("🎬 Journey started!");
    println!("   Current Step: {:?}", state.current_step);
    println!("   Is Complete: {}\n", state.is_complete);

    // Simulate user responses - Normal urgency case
    println!("=== Scenario 1: Normal Urgency ===\n");
    let normal_responses = vec![
        "My product arrived damaged",
        "Order #12345. It's a product quality issue",  // Provide order number as requested
        "It's not urgent, but I'd like it resolved",
        "Received yesterday, haven't tried anything yet",
        "Option 1 sounds good",
        "No, that's all. Thank you!",
    ];

    for (i, response) in normal_responses.iter().enumerate() {
        println!("─────────────────────────────────────────────");
        println!("Step {}: User Response", i + 1);
        println!("─────────────────────────────────────────────\n");

        println!("👤 Customer: {}", response);

        // Process the journey step
        let next_step = agent
            .process_journey_step(&session_id, response)
            .await?;

        println!("\n📍 Journey Step: {}", next_step.name);

        // Get real LLM response
        let llm_response = if !response.is_empty() {
            agent.process_message(session_id.clone(), response.to_string()).await?
        } else {
            talk::AgentResponse {
                message: next_step.prompt.clone(),
                matched_guideline: None,
                tools_used: vec![],
                journey_step: Some(next_step.id),
                context_updates: std::collections::HashMap::new(),
                explanation: None,
            }
        };

        println!("💬 Agent: {}", llm_response.message);

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
                println!("\n✅ Complaint handling completed successfully!");
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
    println!("Journey Name: Handle Customer Complaint");
    println!("Total Steps: 7 (with urgency-based branching)");
    println!("Scenario: Normal urgency case with standard resolution flow\n");

    println!("💡 Key Features Demonstrated:");
    println!("   • Empathetic customer service interactions");
    println!("   • Complaint categorization and urgency assessment");
    println!("   • Conditional branching (urgent vs. normal)");
    println!("   • Two different resolution paths:");
    println!("     - Urgent: Immediate escalation to specialist");
    println!("     - Normal: Standard resolution with solution proposals");
    println!("   • Ticket creation and tracking");
    println!("   • Customer satisfaction confirmation");
    println!("   • Complete audit trail of the interaction\n");

    // Demonstrate urgent case flow
    println!("\n=== What happens in an URGENT case? ===");
    println!("If customer says 'urgent' or 'emergency':");
    println!("  Greet → Identify → Assess Urgency → 🚨 URGENT ESCALATION → Satisfaction");
    println!("  (Skips standard detail collection and solution proposal)");
    println!("  Immediately creates urgent ticket and connects to specialist\n");

    Ok(())
}
