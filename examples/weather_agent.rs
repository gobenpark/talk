//! Weather Agent Example
//!
//! This example demonstrates how to create an agent with a custom tool
//! that can fetch weather information.
//!
//! Run with: cargo run --example weather_agent

use std::collections::HashMap;
use std::time::Duration;
use talk::{
    Agent, AgentConfig, Guideline, GuidelineAction, GuidelineCondition, OpenAIProvider,
    ParameterSchema, Tool, ToolResult,
};

/// A simple weather tool that returns mock weather data
struct WeatherTool {
    id: talk::ToolId,
    parameters: HashMap<String, ParameterSchema>,
}

impl WeatherTool {
    fn new() -> Self {
        let mut parameters = HashMap::new();
        parameters.insert(
            "city".to_string(),
            ParameterSchema {
                param_type: "string".to_string(),
                required: true,
                description: "The city to get weather for".to_string(),
                default: None,
            },
        );

        Self {
            id: talk::ToolId::new(),
            parameters,
        }
    }
}

#[async_trait::async_trait]
impl Tool for WeatherTool {
    fn id(&self) -> &talk::ToolId {
        &self.id
    }

    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather information for a city"
    }

    fn parameters(&self) -> &HashMap<String, ParameterSchema> {
        &self.parameters
    }

    async fn execute(
        &self,
        parameters: HashMap<String, serde_json::Value>,
    ) -> talk::Result<ToolResult> {
        // Extract city parameter
        let city = parameters
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        // Return mock weather data
        let weather_data = serde_json::json!({
            "city": city,
            "temperature": "72°F",
            "condition": "Sunny",
            "humidity": "45%",
            "wind_speed": "10 mph"
        });

        Ok(ToolResult {
            output: weather_data,
            error: None,
            metadata: HashMap::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌤️  Weather Agent Example");
    println!("=======================\n");

    // Create OpenAI provider
    // Note: Set OPENAI_API_KEY environment variable
    let provider = match std::env::var("OPENAI_API_KEY") {
        Ok(api_key) => OpenAIProvider::new(api_key)
            .with_model("gpt-3.5-turbo")
            .with_temperature(0.7),
        Err(_) => {
            println!("⚠️  OPENAI_API_KEY not set. Using mock provider.");
            println!("   Set OPENAI_API_KEY to use real OpenAI API.\n");
            return Ok(());
        }
    };

    // Create agent with configuration
    let mut agent = Agent::builder()
        .name("Weather Assistant")
        .description("A helpful assistant that provides weather information")
        .provider(Box::new(provider))
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: talk::LogLevel::Info,
        })
        .build()?;

    // Create and register the weather tool
    let weather_tool = Box::new(WeatherTool::new());
    let tool_id = agent.add_tool(weather_tool).await?;

    println!("✅ Weather tool registered with ID: {}\n", tool_id);

    // Create a guideline that uses the weather tool
    let weather_guideline = Guideline {
        id: talk::GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"(?i)weather.*in\s+(\w+)".to_string()),
        action: GuidelineAction {
            response_template: "Let me check the weather for you.".to_string(),
            requires_llm: true,
            parameters: vec!["city".to_string()],
        },
        priority: 10,
        tools: vec![tool_id],
        parameters: HashMap::new(),
        created_at: chrono::Utc::now(),
    };

    agent.add_guideline(weather_guideline).await?;

    println!("✅ Weather guideline registered\n");

    // Create a session
    let session_id = agent.create_session().await?;
    println!("📝 Created session: {}\n", session_id);

    // Example conversations
    let test_messages = vec![
        "What's the weather in Seattle?",
        "How's the weather in Paris?",
        "Tell me about the weather in Tokyo",
    ];

    for user_message in test_messages {
        println!("👤 User: {}", user_message);

        let response = agent
            .process_message(session_id, user_message.to_string())
            .await?;

        println!("🤖 Agent: {}", response.message);

        if !response.tools_used.is_empty() {
            println!("   🔧 Tools used: {} tool(s)", response.tools_used.len());
            for tool_exec in &response.tools_used {
                println!(
                    "      - Tool ID: {} (took {:?})",
                    tool_exec.tool_id, tool_exec.duration
                );
            }
        }

        if let Some(explanation) = response.explanation {
            println!("   📊 Confidence: {:.2}%", explanation.confidence * 100.0);
        }

        println!();
    }

    // End the session
    agent.end_session(&session_id).await?;
    println!("✅ Session ended");

    Ok(())
}
