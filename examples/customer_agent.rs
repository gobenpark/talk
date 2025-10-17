use talk::{
    Agent, LLMProvider, OpenAIProvider,
    Guideline, GuidelineCondition, GuidelineAction,
    GuidelineId,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup LLM provider
    let provider: Box<dyn LLMProvider> = if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        Box::new(
            OpenAIProvider::new(openai_key)
                .with_model("gpt-4")
        )
    } else {
        return Err("OPENAI_API_KEY not found".into());
    };

    // Create agent with comprehensive system instructions
    let mut agent = Agent::builder()
        .name("customer")
        .description(
            "A helpful multilingual customer service assistant. \
            Handle pricing inquiries, cancellation requests, and support questions. \
            Be concise and professional."
        )
        .provider(provider)
        .build()?;

    println!("Agent created successfully!");
    println!("Using LLM for all queries (language-independent)");

    // Add LLM-powered guideline for pricing (any language)
    let pricing_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Tell the user: Our pricing starts at $49/month for basic, $99/month for professional, and custom pricing for enterprise. Ask if they need more details.".to_string(),
            requires_llm: true,  // LLM will understand and respond in any language
            parameters: vec![],
        },
        priority: 10,
        tools: vec![],
        parameters: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(pricing_guideline).await?;
    println!("Added guideline: pricing query → LLM with template");

    // Add LLM-powered guideline for cancellation (any language)
    let cancel_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("cancel".to_string()),
        action: GuidelineAction {
            response_template: "Tell the user: To cancel your subscription, please contact support@example.com or visit your account settings. Be empathetic and ask if there's anything we can improve.".to_string(),
            requires_llm: true,  // LLM handles any language
            parameters: vec![],
        },
        priority: 15,
        tools: vec![],
        parameters: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(cancel_guideline).await?;
    println!("Added guideline: cancellation query → LLM with template");

    // Add LLM-powered guideline for support hours (any language)
    let hours_guideline = Guideline {
        id: GuidelineId::new(),
        condition: GuidelineCondition::Literal("hours".to_string()),
        action: GuidelineAction {
            response_template: "Tell the user: Our support team is available Monday-Friday, 9 AM to 6 PM EST. Offer to help with anything else.".to_string(),
            requires_llm: true,  // LLM handles any language
            parameters: vec![],
        },
        priority: 5,
        tools: vec![],
        parameters: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(hours_guideline).await?;
    println!("Added guideline: support hours query → LLM with template");

    // Create a session and test
    let session_id = agent.create_session().await?;
    println!("\nSession created: {}", session_id);

    // Test messages in multiple languages
    let test_messages = vec![
        "What is your pricing?",
        "가격이 얼마인가요?",
        "I want to cancel my subscription",
        "구독을 취소하고 싶어요",
        "What are your support hours?",
        "지원 시간이 언제인가요?",
        "How do I get started?",
    ];

    println!("\n--- Testing LLM-Powered Guidelines (Multilingual) ---");
    for msg in test_messages {
        println!("\nUser: {}", msg);
        match agent.process_message(session_id, msg.to_string()).await {
            Ok(response) => {
                println!("Agent: {}", response.message);
                println!("{:?}",response.matched_guideline)
            },
            Err(e) => println!("Error: {}", e),
        }
    }


    Ok(())
}
