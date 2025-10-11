//! Live Weather Agent Example with Real API
//!
//! This example demonstrates how to create an agent with a tool that
//! fetches real weather data from OpenWeatherMap API.
//!
//! Prerequisites:
//! 1. Set one of these LLM provider API keys:
//!    - OPENAI_API_KEY (for OpenAI GPT-3.5)
//!    - ANTHROPIC_API_KEY (for Claude Sonnet 4.5)
//! 2. Set OPENWEATHER_API_KEY environment variable
//!    Get free API key at: https://openweathermap.org/api
//!
//! Run with: cargo run --example weather_agent_live

use std::collections::HashMap;
use std::time::Duration;
use talk::{
    Agent, AgentConfig, AnthropicProvider, Guideline, GuidelineAction, GuidelineCondition,
    LLMProvider, OpenAIProvider, ParameterSchema, Tool, ToolResult,
};

/// Real weather tool that fetches data from OpenWeatherMap API
struct LiveWeatherTool {
    id: talk::ToolId,
    api_key: String,
    client: reqwest::Client,
    parameters: HashMap<String, ParameterSchema>,
}

impl LiveWeatherTool {
    fn new(api_key: String) -> Self {
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
            api_key,
            client: reqwest::Client::new(),
            parameters,
        }
    }
}

#[async_trait::async_trait]
impl Tool for LiveWeatherTool {
    fn id(&self) -> &talk::ToolId {
        &self.id
    }

    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather information for a city using OpenWeatherMap API"
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

        println!("   🌐 Fetching weather data for: {}", city);

        // Call OpenWeatherMap API
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        println!("   🔍 DEBUG: URL = {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => {
                            // Extract weather information
                            let temp = data["main"]["temp"].as_f64().unwrap_or(0.0);
                            let temp_f = temp * 9.0 / 5.0 + 32.0; // Convert to Fahrenheit
                            let condition = data["weather"][0]["description"]
                                .as_str()
                                .unwrap_or("Unknown");
                            let humidity = data["main"]["humidity"].as_i64().unwrap_or(0);
                            let wind_speed = data["wind"]["speed"].as_f64().unwrap_or(0.0);
                            let wind_speed_mph = wind_speed * 2.237; // Convert m/s to mph

                            let weather_data = serde_json::json!({
                                "city": city,
                                "temperature_c": format!("{:.1}°C", temp),
                                "temperature_f": format!("{:.1}°F", temp_f),
                                "condition": condition,
                                "humidity": format!("{}%", humidity),
                                "wind_speed": format!("{:.1} mph", wind_speed_mph)
                            });

                            println!("   ✅ Weather data fetched successfully");

                            Ok(ToolResult {
                                output: weather_data,
                                error: None,
                                metadata: HashMap::new(),
                            })
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to parse weather data: {}", e);
                            println!("   ❌ {}", error_msg);
                            Ok(ToolResult {
                                output: serde_json::json!({}),
                                error: Some(error_msg),
                                metadata: HashMap::new(),
                            })
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Could not read body".to_string());
                    let error_msg =
                        format!("Weather API returned error: {} - Body: {}", status, body);
                    println!("   ❌ {}", error_msg);
                    Ok(ToolResult {
                        output: serde_json::json!({}),
                        error: Some(error_msg),
                        metadata: HashMap::new(),
                    })
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch weather data: {}", e);
                println!("   ❌ {}", error_msg);
                Ok(ToolResult {
                    output: serde_json::json!({}),
                    error: Some(error_msg),
                    metadata: HashMap::new(),
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌤️  Live Weather Agent Example");
    println!("================================\n");

    // Check for OpenWeather API key
    let openweather_api_key = match std::env::var("OPENWEATHER_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ OPENWEATHER_API_KEY not set!");
            println!("\n📝 To run this example:");
            println!("   1. Sign up at https://openweathermap.org/api");
            println!("   2. Get your free API key");
            println!("   3. Set environment variable:");
            println!("      export OPENWEATHER_API_KEY=your_api_key_here");
            println!("   4. Also set OPENAI_API_KEY for the LLM");
            return Ok(());
        }
    };

    // Check for LLM provider API keys (OpenAI or Anthropic)
    let provider: Box<dyn LLMProvider> = if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        println!("✅ Using OpenAI GPT-4o");
        Box::new(
            OpenAIProvider::new(openai_key)
                .with_model("gpt-4o")
                .with_temperature(0.7),
        )
    } else if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
        println!("✅ Using Anthropic Claude Sonnet 4.5");
        Box::new(
            AnthropicProvider::new(anthropic_key)
                .with_model("claude-sonnet-4-5-20250929")
                .with_temperature(0.7),
        )
    } else {
        println!("❌ No LLM API key found!");
        println!("\n📝 Set one of these environment variables:");
        println!("   Option 1 - OpenAI:");
        println!("      export OPENAI_API_KEY=your_openai_api_key_here");
        println!("\n   Option 2 - Anthropic (Claude):");
        println!("      export ANTHROPIC_API_KEY=your_anthropic_api_key_here");
        println!("\n   Get API keys:");
        println!("      OpenAI: https://platform.openai.com/");
        println!("      Anthropic: https://console.anthropic.com/");
        return Ok(());
    };

    println!("✅ API keys configured\n");

    // Create agent with configuration
    let mut agent = Agent::builder()
        .name("Weather Assistant")
        .description("A helpful assistant that provides real-time weather information")
        .provider(provider)
        .config(AgentConfig {
            max_context_messages: 100,
            default_tool_timeout: Duration::from_secs(30),
            enable_explainability: true,
            log_level: talk::LogLevel::Info,
        })
        .build()?;

    // Create and register the live weather tool
    let weather_tool = Box::new(LiveWeatherTool::new(openweather_api_key));
    let tool_id = agent.add_tool(weather_tool).await?;

    println!("✅ Live weather tool registered\n");

    // Create a guideline that uses the weather tool
    let weather_guideline = Guideline {
        id: talk::GuidelineId::new(),
        condition: GuidelineCondition::Regex(r"(?i)weather.*in\s+(\w+)".to_string()),
        action: GuidelineAction {
            response_template: "Let me check the current weather for you.".to_string(),
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

    // Test cases to validate guideline accuracy
    println!("🧪 Testing Guideline Accuracy\n");
    println!("Legend:");
    println!("  ✅ MATCH    = Guideline matched correctly");
    println!("  ❌ NO MATCH = Guideline did not match");
    println!("  🔧 TOOL     = Weather tool executed");
    println!("  💬 FALLBACK = Used fallback response (no tool)\n");

    let test_cases = vec![
        // Should MATCH - Standard weather queries
        ("What's the weather in Seoul?", true, "Standard weather query"),
        ("How's the weather in London?", true, "Alternative phrasing"),
        ("Tell me about the weather in Tokyo", true, "Natural language"),
        ("weather in Paris", true, "Minimal phrasing"),
        ("Check weather in Berlin", true, "Command style"),

        // Should NOT MATCH - Non-weather queries
        ("Tell me about Seoul", false, "City info, not weather"),
        ("What's the population of London?", false, "Different topic"),
        ("Is Paris a nice city?", false, "General question"),
        ("weather", false, "Missing city name"),

        // Edge cases
        ("What's the temperature in Madrid?", false, "Temperature not 'weather'"),
        ("weather forecast in Rome", true, "Contains 'weather in'"),
    ];

    let mut correct_matches = 0;
    let mut total_cases = test_cases.len();

    for (i, (user_message, should_match, description)) in test_cases.iter().enumerate() {
        if i > 0 {
            println!("\n{}", "─".repeat(70));
        }

        println!("\n📝 Test Case {}: {}", i + 1, description);
        println!("👤 User: \"{}\"", user_message);
        println!("🎯 Expected: {} guideline", if *should_match { "MATCH" } else { "NO MATCH" });

        let response = agent
            .process_message(session_id, user_message.to_string())
            .await?;

        let did_use_tool = !response.tools_used.is_empty();
        let guideline_matched = did_use_tool; // Tool execution indicates guideline match

        // Determine test result
        let test_passed = guideline_matched == *should_match;
        if test_passed {
            correct_matches += 1;
        }

        // Display result
        if guideline_matched {
            println!("📊 Result: ✅ MATCH + 🔧 TOOL");
        } else {
            println!("📊 Result: ❌ NO MATCH + 💬 FALLBACK");
        }

        println!("   Test: {}", if test_passed { "✅ PASS" } else { "❌ FAIL" });
        println!("🤖 Agent: {}", response.message);

        if !response.tools_used.is_empty() {
            println!("\n   🔧 Tools executed:");
            for tool_exec in &response.tools_used {
                println!("      - {} (took {:?})", tool_exec.tool_id, tool_exec.duration);
            }
        }
    }

    // Summary
    println!("\n{}", "=".repeat(70));
    println!("\n📊 GUIDELINE ACCURACY REPORT");
    println!("   Total Tests: {}", total_cases);
    println!("   Passed: {}", correct_matches);
    println!("   Failed: {}", total_cases - correct_matches);
    println!("   Accuracy: {:.1}%", (correct_matches as f32 / total_cases as f32) * 100.0);
    println!("\n{}", "=".repeat(70));

    // End the session
    println!("\n{}", "─".repeat(60));
    agent.end_session(&session_id).await?;
    println!("\n✅ Session ended");

    Ok(())
}
