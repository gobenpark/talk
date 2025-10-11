# API Contracts Specification

## Overview

This document defines all API contracts for the Rust conversational agent library. Each API includes trait/struct definitions, method signatures with documentation, input/output types, error handling, and usage examples.

---

## Table of Contents

1. [Agent API](#1-agent-api)
2. [Guideline API](#2-guideline-api)
3. [Tool API](#3-tool-api)
4. [Journey API](#4-journey-api)
5. [Provider API](#5-provider-api)
6. [Storage API](#6-storage-api)
7. [Error Types](#7-error-types)

---

## 1. Agent API

### Overview
The Agent API provides methods for creating, configuring, and processing messages through conversational AI agents.

### Trait Definition

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Core agent interface for conversational AI
#[async_trait]
pub trait AgentApi: Send + Sync {
    /// Create a new agent instance
    ///
    /// # Arguments
    /// * `config` - Agent configuration including name, system prompt, and settings
    ///
    /// # Returns
    /// * `Result<Agent, AgentError>` - Created agent or error
    ///
    /// # Errors
    /// * `AgentError::InvalidConfig` - Invalid configuration parameters
    /// * `AgentError::ProviderError` - Failed to initialize LLM provider
    async fn create_agent(&self, config: CreateAgentRequest) -> Result<Agent, AgentError>;

    /// Get an agent by ID
    ///
    /// # Arguments
    /// * `agent_id` - Unique agent identifier
    ///
    /// # Returns
    /// * `Result<Agent, AgentError>` - Agent instance or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    async fn get_agent(&self, agent_id: &str) -> Result<Agent, AgentError>;

    /// Update agent configuration
    ///
    /// # Arguments
    /// * `agent_id` - Agent to update
    /// * `updates` - Configuration changes
    ///
    /// # Returns
    /// * `Result<Agent, AgentError>` - Updated agent or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::InvalidConfig` - Invalid update parameters
    async fn update_agent(
        &self,
        agent_id: &str,
        updates: UpdateAgentRequest,
    ) -> Result<Agent, AgentError>;

    /// Delete an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent to delete
    ///
    /// # Returns
    /// * `Result<(), AgentError>` - Success or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::InUse` - Agent has active sessions
    async fn delete_agent(&self, agent_id: &str) -> Result<(), AgentError>;

    /// Process a user message and generate a response
    ///
    /// # Arguments
    /// * `agent_id` - Agent to use for processing
    /// * `request` - Message and session information
    ///
    /// # Returns
    /// * `Result<ProcessMessageResponse, AgentError>` - Response and updated context
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::SessionError` - Session state error
    /// * `AgentError::ProviderError` - LLM provider error
    /// * `AgentError::ToolError` - Tool execution failed
    async fn process_message(
        &self,
        agent_id: &str,
        request: ProcessMessageRequest,
    ) -> Result<ProcessMessageResponse, AgentError>;

    /// Register a new guideline for the agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent to add guideline to
    /// * `guideline` - Guideline definition
    ///
    /// # Returns
    /// * `Result<Guideline, AgentError>` - Created guideline or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::InvalidGuideline` - Invalid guideline definition
    async fn add_guideline(
        &self,
        agent_id: &str,
        guideline: CreateGuidelineRequest,
    ) -> Result<Guideline, AgentError>;

    /// Register a new tool for the agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent to add tool to
    /// * `tool` - Tool definition and handler
    ///
    /// # Returns
    /// * `Result<Tool, AgentError>` - Created tool or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::InvalidTool` - Invalid tool definition
    /// * `AgentError::DuplicateName` - Tool name already exists
    async fn add_tool(
        &self,
        agent_id: &str,
        tool: CreateToolRequest,
    ) -> Result<Tool, AgentError>;

    /// Register a new journey for the agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent to add journey to
    /// * `journey` - Journey definition
    ///
    /// # Returns
    /// * `Result<Journey, AgentError>` - Created journey or error
    ///
    /// # Errors
    /// * `AgentError::NotFound` - Agent does not exist
    /// * `AgentError::InvalidJourney` - Invalid journey definition
    async fn add_journey(
        &self,
        agent_id: &str,
        journey: CreateJourneyRequest,
    ) -> Result<Journey, AgentError>;

    /// List all agents
    ///
    /// # Arguments
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// * `Result<Vec<Agent>, AgentError>` - List of agents or error
    async fn list_agents(&self, filter: Option<AgentFilter>) -> Result<Vec<Agent>, AgentError>;
}
```

### Request/Response Types

```rust
/// Request to create a new agent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateAgentRequest {
    /// Agent name
    pub name: String,

    /// System prompt
    pub system_prompt: String,

    /// LLM provider configuration
    pub provider_config: ProviderConfig,

    /// Optional storage configuration
    pub storage_config: Option<StorageConfig>,

    /// Agent configuration
    pub config: AgentConfig,
}

/// Request to update an agent
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UpdateAgentRequest {
    /// New name (if changing)
    pub name: Option<String>,

    /// New system prompt (if changing)
    pub system_prompt: Option<String>,

    /// New configuration (if changing)
    pub config: Option<AgentConfig>,
}

/// Request to process a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessMessageRequest {
    /// Session ID (creates new session if not provided)
    pub session_id: Option<String>,

    /// User message
    pub message: String,

    /// Optional session metadata
    pub metadata: Option<HashMap<String, String>>,

    /// Optional context variables to inject
    pub context_variables: Option<HashMap<String, serde_json::Value>>,
}

/// Response from processing a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessMessageResponse {
    /// Session ID
    pub session_id: String,

    /// Agent's response message
    pub message: String,

    /// Tools that were executed
    pub tool_results: Vec<ToolExecutionResult>,

    /// Updated context variables
    pub context_variables: HashMap<String, ContextValue>,

    /// Current journey state (if in a journey)
    pub journey_state: Option<JourneyState>,

    /// Matched guidelines
    pub matched_guidelines: Vec<GuidelineMatch>,

    /// Processing metadata
    pub metadata: ProcessingMetadata,
}

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionResult {
    /// Tool name
    pub tool_name: String,

    /// Execution success
    pub success: bool,

    /// Result data
    pub result: serde_json::Value,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Processing metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingMetadata {
    /// Total processing time in milliseconds
    pub total_time_ms: u64,

    /// LLM call time in milliseconds
    pub llm_time_ms: u64,

    /// Guideline matching time in milliseconds
    pub guideline_matching_time_ms: u64,

    /// Tool execution time in milliseconds
    pub tool_execution_time_ms: u64,

    /// Number of LLM calls made
    pub llm_calls: u32,

    /// Total tokens used
    pub tokens_used: u32,
}

/// Agent filter criteria
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AgentFilter {
    /// Filter by name pattern
    pub name_pattern: Option<String>,

    /// Filter by creation date range
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,

    /// Pagination
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
```

### Usage Example

```rust
use talk::{AgentApi, CreateAgentRequest, ProcessMessageRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent API instance
    let agent_api = AgentApiImpl::new();

    // Create a new agent
    let agent = agent_api.create_agent(CreateAgentRequest {
        name: "Customer Support Agent".to_string(),
        system_prompt: "You are a helpful customer support agent.".to_string(),
        provider_config: ProviderConfig::OpenAI {
            api_key: env::var("OPENAI_API_KEY")?,
            model: "gpt-4".to_string(),
        },
        storage_config: None,
        config: AgentConfig {
            max_history_length: 50,
            temperature: 0.7,
            max_tokens: 2048,
            tool_timeout_secs: 30,
            auto_extract_context: true,
            enable_journeys: false,
        },
    }).await?;

    println!("Created agent: {}", agent.id);

    // Process a message
    let response = agent_api.process_message(
        &agent.id,
        ProcessMessageRequest {
            session_id: None, // Creates new session
            message: "Hi, I need help with my order #12345".to_string(),
            metadata: None,
            context_variables: None,
        },
    ).await?;

    println!("Agent response: {}", response.message);
    println!("Session ID: {}", response.session_id);
    println!("Extracted context: {:?}", response.context_variables);

    Ok(())
}
```

---

## 2. Guideline API

### Overview
The Guideline API provides methods for registering, prioritizing, and evaluating behavioral rules for agents.

### Trait Definition

```rust
/// Guideline management and matching interface
#[async_trait]
pub trait GuidelineApi: Send + Sync {
    /// Create a new guideline
    ///
    /// # Arguments
    /// * `agent_id` - Agent this guideline belongs to
    /// * `request` - Guideline definition
    ///
    /// # Returns
    /// * `Result<Guideline, GuidelineError>` - Created guideline or error
    ///
    /// # Errors
    /// * `GuidelineError::InvalidCondition` - Invalid condition syntax
    /// * `GuidelineError::InvalidAction` - Invalid action syntax
    /// * `GuidelineError::ToolNotFound` - Referenced tool doesn't exist
    async fn create_guideline(
        &self,
        agent_id: &str,
        request: CreateGuidelineRequest,
    ) -> Result<Guideline, GuidelineError>;

    /// Get a guideline by ID
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `guideline_id` - Guideline ID
    ///
    /// # Returns
    /// * `Result<Guideline, GuidelineError>` - Guideline or error
    ///
    /// # Errors
    /// * `GuidelineError::NotFound` - Guideline does not exist
    async fn get_guideline(
        &self,
        agent_id: &str,
        guideline_id: &str,
    ) -> Result<Guideline, GuidelineError>;

    /// Update a guideline
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `guideline_id` - Guideline ID
    /// * `updates` - Fields to update
    ///
    /// # Returns
    /// * `Result<Guideline, GuidelineError>` - Updated guideline or error
    ///
    /// # Errors
    /// * `GuidelineError::NotFound` - Guideline does not exist
    /// * `GuidelineError::InvalidUpdate` - Invalid update parameters
    async fn update_guideline(
        &self,
        agent_id: &str,
        guideline_id: &str,
        updates: UpdateGuidelineRequest,
    ) -> Result<Guideline, GuidelineError>;

    /// Delete a guideline
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `guideline_id` - Guideline ID
    ///
    /// # Returns
    /// * `Result<(), GuidelineError>` - Success or error
    ///
    /// # Errors
    /// * `GuidelineError::NotFound` - Guideline does not exist
    async fn delete_guideline(
        &self,
        agent_id: &str,
        guideline_id: &str,
    ) -> Result<(), GuidelineError>;

    /// List guidelines for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// * `Result<Vec<Guideline>, GuidelineError>` - List of guidelines or error
    async fn list_guidelines(
        &self,
        agent_id: &str,
        filter: Option<GuidelineFilter>,
    ) -> Result<Vec<Guideline>, GuidelineError>;

    /// Match guidelines against a message
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `request` - Message and context for matching
    ///
    /// # Returns
    /// * `Result<GuidelineMatchResult, GuidelineError>` - Matched guidelines or error
    ///
    /// # Errors
    /// * `GuidelineError::ProviderError` - LLM provider error during matching
    async fn match_guidelines(
        &self,
        agent_id: &str,
        request: MatchGuidelinesRequest,
    ) -> Result<GuidelineMatchResult, GuidelineError>;

    /// Enable or disable a guideline
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `guideline_id` - Guideline ID
    /// * `enabled` - New enabled state
    ///
    /// # Returns
    /// * `Result<Guideline, GuidelineError>` - Updated guideline or error
    async fn set_guideline_enabled(
        &self,
        agent_id: &str,
        guideline_id: &str,
        enabled: bool,
    ) -> Result<Guideline, GuidelineError>;

    /// Reorder guideline priority
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `guideline_id` - Guideline ID
    /// * `new_priority` - New priority value
    ///
    /// # Returns
    /// * `Result<Guideline, GuidelineError>` - Updated guideline or error
    async fn set_guideline_priority(
        &self,
        agent_id: &str,
        guideline_id: &str,
        new_priority: i32,
    ) -> Result<Guideline, GuidelineError>;
}
```

### Request/Response Types

```rust
/// Request to create a guideline
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateGuidelineRequest {
    /// Numeric priority (higher = more important)
    pub priority: i32,

    /// Condition description (when to activate)
    pub condition: String,

    /// Action description (what to do)
    pub action: String,

    /// Optional tool names to invoke
    pub tools: Option<Vec<String>>,

    /// Optional required context variables
    pub required_context: Option<Vec<String>>,

    /// Optional journey association
    pub journey_id: Option<String>,

    /// Optional journey step association
    pub journey_step: Option<String>,

    /// Metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to update a guideline
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UpdateGuidelineRequest {
    pub priority: Option<i32>,
    pub condition: Option<String>,
    pub action: Option<String>,
    pub tools: Option<Vec<String>>,
    pub required_context: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to match guidelines
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchGuidelinesRequest {
    /// User message to match against
    pub message: String,

    /// Current context variables
    pub context: HashMap<String, serde_json::Value>,

    /// Current journey state (if applicable)
    pub journey_state: Option<JourneyState>,

    /// Minimum relevance score threshold (0.0-1.0)
    pub relevance_threshold: Option<f32>,

    /// Maximum number of matches to return
    pub max_matches: Option<usize>,
}

/// Guideline filter criteria
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GuidelineFilter {
    /// Filter by enabled state
    pub enabled: Option<bool>,

    /// Filter by journey
    pub journey_id: Option<String>,

    /// Filter by priority range
    pub min_priority: Option<i32>,
    pub max_priority: Option<i32>,

    /// Filter by tool association
    pub has_tool: Option<String>,

    /// Sort order
    pub sort_by: Option<GuidelineSortBy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GuidelineSortBy {
    PriorityDesc,
    PriorityAsc,
    CreatedAtDesc,
    CreatedAtAsc,
}
```

### Usage Example

```rust
use talk::{GuidelineApi, CreateGuidelineRequest, MatchGuidelinesRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let guideline_api = GuidelineApiImpl::new();
    let agent_id = "agent_123";

    // Create a guideline
    let guideline = guideline_api.create_guideline(
        agent_id,
        CreateGuidelineRequest {
            priority: 100,
            condition: "user asks about refunds or returns".to_string(),
            action: "Explain the 30-day refund policy and offer to check order status".to_string(),
            tools: Some(vec!["check_order".to_string(), "get_refund_policy".to_string()]),
            required_context: Some(vec!["order_id".to_string()]),
            journey_id: None,
            journey_step: None,
            metadata: None,
        },
    ).await?;

    println!("Created guideline: {}", guideline.id);

    // Match guidelines against a message
    let matches = guideline_api.match_guidelines(
        agent_id,
        MatchGuidelinesRequest {
            message: "I want to return my order #12345".to_string(),
            context: HashMap::from([
                ("order_id".to_string(), json!("12345")),
            ]),
            journey_state: None,
            relevance_threshold: Some(0.3),
            max_matches: Some(3),
        },
    ).await?;

    println!("Matched {} guidelines", matches.matches.len());
    for m in matches.top_matches {
        println!("  - {} (priority: {}, score: {:.2})",
            m.guideline_id, m.priority, m.relevance_score);
    }

    Ok(())
}
```

---

## 3. Tool API

### Overview
The Tool API provides methods for registering async functions with timeouts and managing tool execution.

### Trait Definition

```rust
/// Tool registration and execution interface
#[async_trait]
pub trait ToolApi: Send + Sync {
    /// Register a new tool
    ///
    /// # Arguments
    /// * `agent_id` - Agent to register tool with
    /// * `request` - Tool definition and handler
    ///
    /// # Returns
    /// * `Result<Tool, ToolError>` - Registered tool or error
    ///
    /// # Errors
    /// * `ToolError::InvalidName` - Tool name doesn't match requirements
    /// * `ToolError::DuplicateName` - Tool name already exists
    /// * `ToolError::InvalidSchema` - Invalid parameter schema
    async fn register_tool(
        &self,
        agent_id: &str,
        request: RegisterToolRequest,
    ) -> Result<Tool, ToolError>;

    /// Get a tool by name
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `tool_name` - Tool name
    ///
    /// # Returns
    /// * `Result<Tool, ToolError>` - Tool or error
    ///
    /// # Errors
    /// * `ToolError::NotFound` - Tool does not exist
    async fn get_tool(&self, agent_id: &str, tool_name: &str) -> Result<Tool, ToolError>;

    /// Update a tool
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `tool_name` - Tool name
    /// * `updates` - Fields to update
    ///
    /// # Returns
    /// * `Result<Tool, ToolError>` - Updated tool or error
    ///
    /// # Errors
    /// * `ToolError::NotFound` - Tool does not exist
    async fn update_tool(
        &self,
        agent_id: &str,
        tool_name: &str,
        updates: UpdateToolRequest,
    ) -> Result<Tool, ToolError>;

    /// Unregister a tool
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `tool_name` - Tool name
    ///
    /// # Returns
    /// * `Result<(), ToolError>` - Success or error
    ///
    /// # Errors
    /// * `ToolError::NotFound` - Tool does not exist
    /// * `ToolError::InUse` - Tool is referenced by guidelines
    async fn unregister_tool(&self, agent_id: &str, tool_name: &str) -> Result<(), ToolError>;

    /// Execute a tool
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `tool_name` - Tool to execute
    /// * `parameters` - Tool parameters
    ///
    /// # Returns
    /// * `Result<ToolResult, ToolError>` - Execution result or error
    ///
    /// # Errors
    /// * `ToolError::NotFound` - Tool does not exist
    /// * `ToolError::InvalidParameters` - Parameters don't match schema
    /// * `ToolError::ExecutionFailed` - Tool execution failed
    /// * `ToolError::Timeout` - Tool exceeded timeout
    async fn execute_tool(
        &self,
        agent_id: &str,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> Result<ToolResult, ToolError>;

    /// List tools for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    ///
    /// # Returns
    /// * `Result<Vec<Tool>, ToolError>` - List of tools or error
    async fn list_tools(&self, agent_id: &str) -> Result<Vec<Tool>, ToolError>;

    /// Validate tool parameters against schema
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `tool_name` - Tool name
    /// * `parameters` - Parameters to validate
    ///
    /// # Returns
    /// * `Result<bool, ToolError>` - Validation result or error
    async fn validate_parameters(
        &self,
        agent_id: &str,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> Result<bool, ToolError>;
}

/// Tool handler trait for async execution
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with given parameters
    ///
    /// # Arguments
    /// * `params` - Tool parameters (must match schema)
    ///
    /// # Returns
    /// * `Result<ToolResult, ToolError>` - Execution result or error
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### Request/Response Types

```rust
/// Request to register a tool
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterToolRequest {
    /// Tool name (unique identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for parameters
    pub parameters: serde_json::Value,

    /// Execution timeout in seconds
    pub timeout_secs: Option<u64>,

    /// Whether failures are allowed
    pub allow_failure: Option<bool>,

    /// Optional retry configuration
    pub retry_config: Option<RetryConfig>,

    /// Metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to update a tool
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UpdateToolRequest {
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
    pub allow_failure: Option<bool>,
    pub retry_config: Option<RetryConfig>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    /// Success indicator
    pub success: bool,

    /// Result data (JSON)
    pub data: serde_json::Value,

    /// Optional message for the agent
    pub message: Option<String>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Retry configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial delay between retries in milliseconds
    pub delay_ms: u64,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f32,
}
```

### Usage Example

```rust
use talk::{ToolApi, ToolHandler, RegisterToolRequest, ToolResult};
use async_trait::async_trait;

// Define a custom tool handler
struct OrderCheckerTool {
    api_client: OrderApiClient,
}

#[async_trait]
impl ToolHandler for OrderCheckerTool {
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError> {
        let order_id = params["order_id"]
            .as_str()
            .ok_or(ToolError::InvalidParameters("order_id required".into()))?;

        let start = Instant::now();

        match self.api_client.get_order(order_id).await {
            Ok(order) => Ok(ToolResult {
                success: true,
                data: serde_json::to_value(order)?,
                message: Some(format!("Order {} found", order_id)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            }),
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tool_api = ToolApiImpl::new();
    let agent_id = "agent_123";

    // Register the tool
    let tool = tool_api.register_tool(
        agent_id,
        RegisterToolRequest {
            name: "check_order".to_string(),
            description: "Check order status by order ID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "order_id": {
                        "type": "string",
                        "description": "Order identifier"
                    }
                },
                "required": ["order_id"]
            }),
            timeout_secs: Some(30),
            allow_failure: Some(false),
            retry_config: Some(RetryConfig {
                max_attempts: 3,
                delay_ms: 1000,
                backoff_multiplier: 2.0,
            }),
            metadata: None,
        },
    ).await?;

    println!("Registered tool: {}", tool.name);

    // Execute the tool
    let result = tool_api.execute_tool(
        agent_id,
        "check_order",
        json!({ "order_id": "12345" }),
    ).await?;

    println!("Tool result: {:?}", result);

    Ok(())
}
```

---

## 4. Journey API

### Overview
The Journey API provides methods for defining multi-step flows with state tracking and step transitions.

### Trait Definition

```rust
/// Journey management and state tracking interface
#[async_trait]
pub trait JourneyApi: Send + Sync {
    /// Create a new journey
    ///
    /// # Arguments
    /// * `agent_id` - Agent this journey belongs to
    /// * `request` - Journey definition
    ///
    /// # Returns
    /// * `Result<Journey, JourneyError>` - Created journey or error
    ///
    /// # Errors
    /// * `JourneyError::InvalidDefinition` - Invalid journey structure
    /// * `JourneyError::InvalidStep` - Invalid step configuration
    /// * `JourneyError::InvalidTransition` - Invalid transition
    async fn create_journey(
        &self,
        agent_id: &str,
        request: CreateJourneyRequest,
    ) -> Result<Journey, JourneyError>;

    /// Get a journey by ID
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `journey_id` - Journey ID
    ///
    /// # Returns
    /// * `Result<Journey, JourneyError>` - Journey or error
    ///
    /// # Errors
    /// * `JourneyError::NotFound` - Journey does not exist
    async fn get_journey(
        &self,
        agent_id: &str,
        journey_id: &str,
    ) -> Result<Journey, JourneyError>;

    /// Update a journey
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `journey_id` - Journey ID
    /// * `updates` - Fields to update
    ///
    /// # Returns
    /// * `Result<Journey, JourneyError>` - Updated journey or error
    async fn update_journey(
        &self,
        agent_id: &str,
        journey_id: &str,
        updates: UpdateJourneyRequest,
    ) -> Result<Journey, JourneyError>;

    /// Delete a journey
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `journey_id` - Journey ID
    ///
    /// # Returns
    /// * `Result<(), JourneyError>` - Success or error
    ///
    /// # Errors
    /// * `JourneyError::NotFound` - Journey does not exist
    /// * `JourneyError::InUse` - Journey has active sessions
    async fn delete_journey(&self, agent_id: &str, journey_id: &str)
        -> Result<(), JourneyError>;

    /// Start a journey for a session
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `journey_id` - Journey to start
    /// * `session_id` - Session to attach journey to
    ///
    /// # Returns
    /// * `Result<JourneyState, JourneyError>` - Initial journey state or error
    ///
    /// # Errors
    /// * `JourneyError::NotFound` - Journey does not exist
    /// * `JourneyError::AlreadyActive` - Session already has active journey
    async fn start_journey(
        &self,
        agent_id: &str,
        journey_id: &str,
        session_id: &str,
    ) -> Result<JourneyState, JourneyError>;

    /// Transition to next step in journey
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `session_id` - Session ID
    /// * `request` - Transition request with message and context
    ///
    /// # Returns
    /// * `Result<JourneyState, JourneyError>` - Updated journey state or error
    ///
    /// # Errors
    /// * `JourneyError::NoActiveJourney` - Session has no active journey
    /// * `JourneyError::NoValidTransition` - No transition matches
    async fn transition_journey(
        &self,
        agent_id: &str,
        session_id: &str,
        request: TransitionJourneyRequest,
    ) -> Result<JourneyState, JourneyError>;

    /// Get current journey state for a session
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// * `Result<Option<JourneyState>, JourneyError>` - Journey state or None
    async fn get_journey_state(
        &self,
        agent_id: &str,
        session_id: &str,
    ) -> Result<Option<JourneyState>, JourneyError>;

    /// Complete (end) a journey
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// * `Result<JourneyState, JourneyError>` - Final journey state or error
    async fn complete_journey(
        &self,
        agent_id: &str,
        session_id: &str,
    ) -> Result<JourneyState, JourneyError>;

    /// List journeys for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    ///
    /// # Returns
    /// * `Result<Vec<Journey>, JourneyError>` - List of journeys or error
    async fn list_journeys(&self, agent_id: &str) -> Result<Vec<Journey>, JourneyError>;
}
```

### Request/Response Types

```rust
/// Request to create a journey
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateJourneyRequest {
    /// Journey name
    pub name: String,

    /// Journey description
    pub description: String,

    /// Journey steps
    pub steps: Vec<CreateJourneyStepRequest>,

    /// Initial step ID
    pub initial_step: String,

    /// Metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to create a journey step
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateJourneyStepRequest {
    /// Step ID
    pub id: String,

    /// Step name
    pub name: String,

    /// Step description
    pub description: String,

    /// Guideline IDs for this step
    pub guidelines: Option<Vec<String>>,

    /// Required context variables
    pub required_context: Option<Vec<String>>,

    /// Possible transitions
    pub transitions: Vec<CreateTransitionRequest>,

    /// Is this a terminal step?
    pub is_terminal: bool,
}

/// Request to create a transition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTransitionRequest {
    /// Target step ID
    pub to_step: String,

    /// Condition for transition
    pub condition: String,

    /// Priority if multiple match
    pub priority: i32,
}

/// Request to update a journey
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UpdateJourneyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Option<Vec<CreateJourneyStepRequest>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to transition journey
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransitionJourneyRequest {
    /// User message that may trigger transition
    pub message: String,

    /// Current context variables
    pub context: HashMap<String, serde_json::Value>,
}
```

### Usage Example

```rust
use talk::{JourneyApi, CreateJourneyRequest, CreateJourneyStepRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let journey_api = JourneyApiImpl::new();
    let agent_id = "agent_123";

    // Create a multi-step onboarding journey
    let journey = journey_api.create_journey(
        agent_id,
        CreateJourneyRequest {
            name: "User Onboarding".to_string(),
            description: "Guide new users through account setup".to_string(),
            steps: vec![
                CreateJourneyStepRequest {
                    id: "welcome".to_string(),
                    name: "Welcome".to_string(),
                    description: "Greet user".to_string(),
                    guidelines: Some(vec!["guideline_welcome".to_string()]),
                    required_context: None,
                    transitions: vec![
                        CreateTransitionRequest {
                            to_step: "collect_name".to_string(),
                            condition: "user is ready to continue".to_string(),
                            priority: 10,
                        }
                    ],
                    is_terminal: false,
                },
                CreateJourneyStepRequest {
                    id: "collect_name".to_string(),
                    name: "Collect Name".to_string(),
                    description: "Ask for user's name".to_string(),
                    guidelines: Some(vec!["guideline_ask_name".to_string()]),
                    required_context: Some(vec!["user_name".to_string()]),
                    transitions: vec![
                        CreateTransitionRequest {
                            to_step: "complete".to_string(),
                            condition: "name is collected".to_string(),
                            priority: 10,
                        }
                    ],
                    is_terminal: false,
                },
                CreateJourneyStepRequest {
                    id: "complete".to_string(),
                    name: "Complete".to_string(),
                    description: "Onboarding complete".to_string(),
                    guidelines: None,
                    required_context: None,
                    transitions: vec![],
                    is_terminal: true,
                },
            ],
            initial_step: "welcome".to_string(),
            metadata: None,
        },
    ).await?;

    println!("Created journey: {}", journey.id);

    // Start journey for a session
    let session_id = "session_456";
    let state = journey_api.start_journey(agent_id, &journey.id, session_id).await?;
    println!("Started journey at step: {}", state.current_step);

    // Transition to next step
    let new_state = journey_api.transition_journey(
        agent_id,
        session_id,
        TransitionJourneyRequest {
            message: "Yes, I'm ready!".to_string(),
            context: HashMap::new(),
        },
    ).await?;
    println!("Transitioned to step: {}", new_state.current_step);

    Ok(())
}
```

---

## 5. Provider API

### Overview
The Provider API defines a trait for LLM integration, allowing different providers (OpenAI, Anthropic, etc.) to be used interchangeably.

### Trait Definition

```rust
/// LLM provider interface for generating responses
#[async_trait]
pub trait Provider: Send + Sync {
    /// Generate a completion based on messages
    ///
    /// # Arguments
    /// * `request` - Completion request with messages and configuration
    ///
    /// # Returns
    /// * `Result<CompletionResponse, ProviderError>` - Generated response or error
    ///
    /// # Errors
    /// * `ProviderError::ApiError` - Provider API error
    /// * `ProviderError::InvalidRequest` - Invalid request parameters
    /// * `ProviderError::RateLimited` - Rate limit exceeded
    /// * `ProviderError::Timeout` - Request timeout
    async fn complete(&self, request: CompletionRequest)
        -> Result<CompletionResponse, ProviderError>;

    /// Generate a structured completion (function calling)
    ///
    /// # Arguments
    /// * `request` - Completion request with tools
    ///
    /// # Returns
    /// * `Result<StructuredCompletionResponse, ProviderError>` - Response with tool calls
    ///
    /// # Errors
    /// * `ProviderError::ApiError` - Provider API error
    /// * `ProviderError::ToolsNotSupported` - Provider doesn't support tools
    async fn complete_with_tools(
        &self,
        request: CompletionRequest,
    ) -> Result<StructuredCompletionResponse, ProviderError>;

    /// Extract structured data from text
    ///
    /// # Arguments
    /// * `request` - Extraction request with prompt and schema
    ///
    /// # Returns
    /// * `Result<serde_json::Value, ProviderError>` - Extracted data or error
    async fn extract(
        &self,
        request: ExtractionRequest,
    ) -> Result<serde_json::Value, ProviderError>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get provider capabilities
    fn capabilities(&self) -> ProviderCapabilities;
}

/// Provider capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderCapabilities {
    /// Supports function/tool calling
    pub supports_tools: bool,

    /// Supports structured output
    pub supports_structured_output: bool,

    /// Supports vision/image inputs
    pub supports_vision: bool,

    /// Maximum context length in tokens
    pub max_context_tokens: usize,

    /// Maximum output tokens
    pub max_output_tokens: usize,
}
```

### Request/Response Types

```rust
/// Request for LLM completion
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionRequest {
    /// Conversation messages
    pub messages: Vec<Message>,

    /// Optional system prompt override
    pub system_prompt: Option<String>,

    /// Available tools for function calling
    pub tools: Option<Vec<ToolDefinition>>,

    /// Temperature (0.0-2.0)
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,

    /// Stop sequences
    pub stop: Option<Vec<String>>,

    /// Additional provider-specific parameters
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Tool definition for function calling
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Parameter schema
    pub parameters: serde_json::Value,
}

/// Response from LLM completion
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionResponse {
    /// Generated message content
    pub content: String,

    /// Token usage
    pub usage: TokenUsage,

    /// Provider-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response with tool calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredCompletionResponse {
    /// Optional message content
    pub content: Option<String>,

    /// Tool calls requested by LLM
    pub tool_calls: Vec<ToolCall>,

    /// Token usage
    pub usage: TokenUsage,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenUsage {
    /// Input tokens
    pub prompt_tokens: u32,

    /// Output tokens
    pub completion_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,
}

/// Request for structured extraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionRequest {
    /// Text to extract from
    pub text: String,

    /// Extraction prompt
    pub prompt: String,

    /// Expected output schema
    pub schema: serde_json::Value,

    /// Temperature
    pub temperature: Option<f32>,
}
```

### Configuration Types

```rust
/// Provider configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProviderConfig {
    OpenAI {
        api_key: String,
        model: String,
        organization: Option<String>,
        base_url: Option<String>,
    },
    Anthropic {
        api_key: String,
        model: String,
    },
    Custom {
        provider_type: String,
        config: HashMap<String, serde_json::Value>,
    },
}
```

### Usage Example

```rust
use talk::{Provider, CompletionRequest, Message};

// Implement a custom provider
struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        // Call OpenAI API
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": request.messages,
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens,
            }))
            .send()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let data: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(CompletionResponse {
            content: data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            usage: TokenUsage {
                prompt_tokens: data["usage"]["prompt_tokens"].as_u64().unwrap() as u32,
                completion_tokens: data["usage"]["completion_tokens"].as_u64().unwrap() as u32,
                total_tokens: data["usage"]["total_tokens"].as_u64().unwrap() as u32,
            },
            metadata: HashMap::new(),
        })
    }

    async fn complete_with_tools(
        &self,
        request: CompletionRequest,
    ) -> Result<StructuredCompletionResponse, ProviderError> {
        // Implementation with tools
        todo!()
    }

    async fn extract(
        &self,
        request: ExtractionRequest,
    ) -> Result<serde_json::Value, ProviderError> {
        // Implementation for extraction
        todo!()
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_tools: true,
            supports_structured_output: true,
            supports_vision: true,
            max_context_tokens: 128000,
            max_output_tokens: 4096,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenAIProvider {
        api_key: env::var("OPENAI_API_KEY")?,
        model: "gpt-4".to_string(),
        client: reqwest::Client::new(),
    };

    let response = provider.complete(CompletionRequest {
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                ..Default::default()
            }
        ],
        system_prompt: Some("You are a helpful assistant".to_string()),
        tools: None,
        temperature: Some(0.7),
        max_tokens: Some(2048),
        stop: None,
        extra: None,
    }).await?;

    println!("Response: {}", response.content);
    println!("Tokens used: {}", response.usage.total_tokens);

    Ok(())
}
```

---

## 6. Storage API

### Overview
The Storage API defines a trait for session persistence, allowing different storage backends (in-memory, Redis, PostgreSQL, etc.) to be used.

### Trait Definition

```rust
/// Storage interface for session persistence
#[async_trait]
pub trait Storage: Send + Sync {
    /// Save a session
    ///
    /// # Arguments
    /// * `session` - Session to save
    ///
    /// # Returns
    /// * `Result<(), StorageError>` - Success or error
    ///
    /// # Errors
    /// * `StorageError::ConnectionError` - Storage connection failed
    /// * `StorageError::WriteError` - Failed to write data
    async fn save_session(&self, session: &Session) -> Result<(), StorageError>;

    /// Load a session by ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID to load
    ///
    /// # Returns
    /// * `Result<Option<Session>, StorageError>` - Session or None if not found
    ///
    /// # Errors
    /// * `StorageError::ConnectionError` - Storage connection failed
    /// * `StorageError::ReadError` - Failed to read data
    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, StorageError>;

    /// Delete a session
    ///
    /// # Arguments
    /// * `session_id` - Session ID to delete
    ///
    /// # Returns
    /// * `Result<(), StorageError>` - Success or error
    async fn delete_session(&self, session_id: &str) -> Result<(), StorageError>;

    /// List sessions for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// * `Result<Vec<Session>, StorageError>` - List of sessions or error
    async fn list_sessions(
        &self,
        agent_id: &str,
        filter: Option<SessionFilter>,
    ) -> Result<Vec<Session>, StorageError>;

    /// Save context for a session
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    /// * `context` - Context to save
    ///
    /// # Returns
    /// * `Result<(), StorageError>` - Success or error
    async fn save_context(&self, session_id: &str, context: &Context)
        -> Result<(), StorageError>;

    /// Load context for a session
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// * `Result<Option<Context>, StorageError>` - Context or None
    async fn load_context(&self, session_id: &str) -> Result<Option<Context>, StorageError>;

    /// Clean up expired sessions
    ///
    /// # Returns
    /// * `Result<usize, StorageError>` - Number of sessions deleted or error
    async fn cleanup_expired_sessions(&self) -> Result<usize, StorageError>;

    /// Health check
    ///
    /// # Returns
    /// * `Result<bool, StorageError>` - Health status
    async fn health_check(&self) -> Result<bool, StorageError>;
}
```

### Filter Types

```rust
/// Session filter criteria
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SessionFilter {
    /// Filter by state
    pub state: Option<SessionState>,

    /// Filter by creation date range
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,

    /// Filter by last activity range
    pub active_after: Option<DateTime<Utc>>,
    pub active_before: Option<DateTime<Utc>>,

    /// Pagination
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
```

### Usage Example

```rust
use talk::{Storage, Session, Context};
use async_trait::async_trait;

// Implement in-memory storage
struct InMemoryStorage {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    contexts: Arc<RwLock<HashMap<String, Context>>>,
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn save_session(&self, session: &Session) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, StorageError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);

        let mut contexts = self.contexts.write().await;
        contexts.remove(session_id);

        Ok(())
    }

    async fn list_sessions(
        &self,
        agent_id: &str,
        filter: Option<SessionFilter>,
    ) -> Result<Vec<Session>, StorageError> {
        let sessions = self.sessions.read().await;
        let mut results: Vec<Session> = sessions
            .values()
            .filter(|s| s.agent_id == agent_id)
            .cloned()
            .collect();

        // Apply filters
        if let Some(f) = filter {
            if let Some(state) = f.state {
                results.retain(|s| s.state == state);
            }
            if let Some(after) = f.created_after {
                results.retain(|s| s.created_at >= after);
            }
            // ... more filters
        }

        Ok(results)
    }

    async fn save_context(
        &self,
        session_id: &str,
        context: &Context,
    ) -> Result<(), StorageError> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(session_id.to_string(), context.clone());
        Ok(())
    }

    async fn load_context(&self, session_id: &str) -> Result<Option<Context>, StorageError> {
        let contexts = self.contexts.read().await;
        Ok(contexts.get(session_id).cloned())
    }

    async fn cleanup_expired_sessions(&self) -> Result<usize, StorageError> {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|_, session| {
            if let Some(expires_at) = session.expires_at {
                expires_at > now
            } else {
                true
            }
        });

        Ok(initial_count - sessions.len())
    }

    async fn health_check(&self) -> Result<bool, StorageError> {
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = InMemoryStorage {
        sessions: Arc::new(RwLock::new(HashMap::new())),
        contexts: Arc::new(RwLock::new(HashMap::new())),
    };

    // Save a session
    let session = Session {
        id: "session_123".to_string(),
        agent_id: "agent_456".to_string(),
        // ... other fields
    };
    storage.save_session(&session).await?;

    // Load it back
    let loaded = storage.load_session("session_123").await?;
    println!("Loaded session: {:?}", loaded);

    // Cleanup expired sessions
    let cleaned = storage.cleanup_expired_sessions().await?;
    println!("Cleaned up {} sessions", cleaned);

    Ok(())
}
```

---

## 7. Error Types

### Overview
Comprehensive error types for all API operations.

### Error Definitions

```rust
use thiserror::Error;

/// Agent API errors
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Agent is currently in use and cannot be deleted")]
    InUse,

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),

    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),

    #[error("Guideline error: {0}")]
    InvalidGuideline(String),

    #[error("Invalid tool: {0}")]
    InvalidTool(String),

    #[error("Invalid journey: {0}")]
    InvalidJourney(String),

    #[error("Duplicate name: {0}")]
    DuplicateName(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}

/// Guideline API errors
#[derive(Error, Debug)]
pub enum GuidelineError {
    #[error("Guideline not found: {0}")]
    NotFound(String),

    #[error("Invalid condition: {0}")]
    InvalidCondition(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid update: {0}")]
    InvalidUpdate(String),

    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),
}

/// Tool API errors
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid tool name: {0}")]
    InvalidName(String),

    #[error("Duplicate tool name: {0}")]
    DuplicateName(String),

    #[error("Invalid parameter schema: {0}")]
    InvalidSchema(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool execution timeout after {0}s")]
    Timeout(u64),

    #[error("Tool is in use and cannot be deleted")]
    InUse,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Journey API errors
#[derive(Error, Debug)]
pub enum JourneyError {
    #[error("Journey not found: {0}")]
    NotFound(String),

    #[error("Invalid journey definition: {0}")]
    InvalidDefinition(String),

    #[error("Invalid step: {0}")]
    InvalidStep(String),

    #[error("Invalid transition: {0}")]
    InvalidTransition(String),

    #[error("Journey is in use and cannot be deleted")]
    InUse,

    #[error("Journey already active for session")]
    AlreadyActive,

    #[error("No active journey for session")]
    NoActiveJourney,

    #[error("No valid transition found")]
    NoValidTransition,

    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),
}

/// Provider API errors
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider API error: {0}")]
    ApiError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limited, retry after {0}s")]
    RateLimited(u64),

    #[error("Request timeout")]
    Timeout,

    #[error("Tools not supported by this provider")]
    ToolsNotSupported,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

/// Storage API errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage connection error: {0}")]
    ConnectionError(String),

    #[error("Read error: {0}")]
    ReadError(String),

    #[error("Write error: {0}")]
    WriteError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

---

## Summary

This API contracts specification provides:

1. **Agent API**: Complete agent lifecycle management and message processing
2. **Guideline API**: Behavioral rule registration, prioritization, and matching
3. **Tool API**: Async function registration with timeout and retry support
4. **Journey API**: Multi-step flow definition and state tracking
5. **Provider API**: Pluggable LLM provider integration
6. **Storage API**: Pluggable session persistence
7. **Error Types**: Comprehensive error handling across all APIs

Each API includes:
- Trait definitions with async methods
- Complete request/response types
- Detailed documentation
- Error handling
- Practical usage examples

The design emphasizes:
- **Type Safety**: Strong Rust typing throughout
- **Async/Await**: Modern async Rust patterns
- **Trait-Based**: Pluggable providers and storage
- **Error Handling**: Comprehensive error types with thiserror
- **Extensibility**: Easy to add new implementations
- **Documentation**: Rich inline documentation for all methods
