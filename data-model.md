# Data Model Specification

## Overview

This document defines the complete data model for the Rust conversational agent library. Each entity includes Rust struct definitions, field descriptions, validation rules, relationships, state transitions, and JSON serialization examples.

---

## 1. Agent

### Description
A conversational AI agent instance with defined behaviors, tools, context management, and LLM provider integration.

### Rust Definition

```rust
pub struct Agent {
    /// Unique identifier for the agent
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// System prompt defining agent personality and behavior
    pub system_prompt: String,

    /// Registered guidelines (rules for behavior)
    pub guidelines: Vec<Guideline>,

    /// Available tools the agent can invoke
    pub tools: HashMap<String, Tool>,

    /// Optional journey definitions for multi-step flows
    pub journeys: HashMap<String, Journey>,

    /// Context variables to extract from conversations
    pub context_variables: Vec<ContextVariable>,

    /// LLM provider for generating responses
    pub provider: Box<dyn Provider>,

    /// Optional storage for session persistence
    pub storage: Option<Box<dyn Storage>>,

    /// Agent-level configuration
    pub config: AgentConfig,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

pub struct AgentConfig {
    /// Maximum conversation history to maintain
    pub max_history_length: usize,

    /// Default temperature for LLM calls
    pub temperature: f32,

    /// Maximum tokens per response
    pub max_tokens: usize,

    /// Tool execution timeout in seconds
    pub tool_timeout_secs: u64,

    /// Enable/disable automatic context extraction
    pub auto_extract_context: bool,

    /// Enable/disable journey tracking
    pub enable_journeys: bool,
}
```

### Field Descriptions

- **id**: Unique identifier (UUID recommended). Must be unique across all agents.
- **name**: Display name for the agent. Used in logs and UI.
- **system_prompt**: Instructions defining agent personality, role, and behavior patterns.
- **guidelines**: Ordered collection of behavioral rules. Processed by priority.
- **tools**: Map of tool name to Tool instance. Tools must have unique names.
- **journeys**: Map of journey name to Journey instance. Optional multi-step flows.
- **context_variables**: Variables to automatically extract from user messages.
- **provider**: LLM provider implementation (OpenAI, Anthropic, etc.).
- **storage**: Optional persistence layer for sessions and context.
- **config**: Agent-level configuration parameters.
- **created_at**: UTC timestamp of agent creation.
- **updated_at**: UTC timestamp of last modification.

### Validation Rules

1. **id**: Non-empty, valid UUID format recommended
2. **name**: 1-100 characters, non-empty
3. **system_prompt**: 1-10,000 characters
4. **tools**: Tool names must be unique, match regex `^[a-zA-Z][a-zA-Z0-9_]*$`
5. **journeys**: Journey names must be unique
6. **config.max_history_length**: 1-1000
7. **config.temperature**: 0.0-2.0
8. **config.max_tokens**: 1-100,000
9. **config.tool_timeout_secs**: 1-300

### Relationships

- **Has Many**: Guidelines (1:N)
- **Has Many**: Tools (1:N via HashMap)
- **Has Many**: Journeys (1:N via HashMap)
- **Has Many**: ContextVariables (1:N)
- **Uses One**: Provider (1:1, polymorphic)
- **Uses Zero or One**: Storage (0:1, polymorphic)

### JSON Serialization Example

```json
{
  "id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "name": "Customer Support Agent",
  "system_prompt": "You are a helpful customer support agent. Be professional, empathetic, and solution-focused.",
  "guidelines": [
    {
      "id": "guideline_1",
      "priority": 100,
      "condition": "user mentions 'refund' or 'return'",
      "action": "Check order status and provide refund policy",
      "tools": ["check_order", "get_refund_policy"]
    }
  ],
  "tools": {
    "check_order": {
      "name": "check_order",
      "description": "Check order status by order ID",
      "parameters": {
        "type": "object",
        "properties": {
          "order_id": {"type": "string"}
        },
        "required": ["order_id"]
      }
    }
  },
  "journeys": {},
  "context_variables": [
    {
      "name": "user_name",
      "description": "Customer's name",
      "extraction_prompt": "Extract the customer's name if mentioned"
    }
  ],
  "config": {
    "max_history_length": 50,
    "temperature": 0.7,
    "max_tokens": 2048,
    "tool_timeout_secs": 30,
    "auto_extract_context": true,
    "enable_journeys": false
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

---

## 2. Guideline

### Description
A rule defining conditions for activation, actions to take, numeric priority for conflict resolution, and optionally associated tools.

### Rust Definition

```rust
pub struct Guideline {
    /// Unique identifier
    pub id: String,

    /// Numeric priority (higher = more important)
    pub priority: i32,

    /// Condition description (when to activate)
    pub condition: String,

    /// Action description (what to do)
    pub action: String,

    /// Optional tool names to invoke
    pub tools: Vec<String>,

    /// Optional context requirements
    pub required_context: Vec<String>,

    /// Optional journey this guideline belongs to
    pub journey_id: Option<String>,

    /// Optional journey step this guideline applies to
    pub journey_step: Option<String>,

    /// Enable/disable this guideline
    pub enabled: bool,

    /// Metadata for tracking and analytics
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}
```

### Field Descriptions

- **id**: Unique identifier for the guideline.
- **priority**: Numeric priority for conflict resolution. Higher values take precedence.
- **condition**: Natural language description of when this guideline activates.
- **action**: Natural language description of what the agent should do.
- **tools**: Names of tools to invoke when this guideline matches.
- **required_context**: Context variable names that must be present for activation.
- **journey_id**: If set, this guideline only applies within the specified journey.
- **journey_step**: If set, this guideline only applies at the specified step.
- **enabled**: Whether this guideline is active.
- **metadata**: Additional key-value data for tracking (e.g., category, tags).
- **created_at**: UTC timestamp of guideline creation.

### Validation Rules

1. **id**: Non-empty, unique within agent
2. **priority**: Integer, typical range -1000 to 1000
3. **condition**: 1-1000 characters, non-empty
4. **action**: 1-2000 characters, non-empty
5. **tools**: Each tool name must exist in agent's tools map
6. **required_context**: Each name must match a defined ContextVariable
7. **journey_id**: If set, must match an existing journey
8. **journey_step**: If set, journey_id must also be set

### Relationships

- **Belongs To**: Agent (N:1)
- **References**: Tools (N:N via tool names)
- **References**: ContextVariables (N:N via required_context)
- **Belongs To**: Journey (N:1, optional)

### Priority Conflict Resolution

When multiple guidelines match:
1. Sort by priority (descending)
2. Take top N guidelines (configurable)
3. Combine actions in priority order
4. Execute tools from all matched guidelines

### JSON Serialization Example

```json
{
  "id": "guideline_refund_policy",
  "priority": 100,
  "condition": "user asks about refunds, returns, or mentions 'money back'",
  "action": "Explain the 30-day refund policy and offer to check their order status",
  "tools": ["check_order", "get_refund_policy"],
  "required_context": ["order_id"],
  "journey_id": null,
  "journey_step": null,
  "enabled": true,
  "metadata": {
    "category": "customer_service",
    "tags": "refund,policy"
  },
  "created_at": "2025-01-15T10:30:00Z"
}
```

---

## 3. Tool

### Description
An async function that can be invoked by the agent to perform external operations (API calls, database queries, calculations, etc.).

### Rust Definition

```rust
pub struct Tool {
    /// Tool name (must be unique)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for parameters
    pub parameters: serde_json::Value,

    /// Async function implementation
    pub handler: Arc<dyn ToolHandler>,

    /// Execution timeout in seconds
    pub timeout_secs: u64,

    /// Whether this tool can fail gracefully
    pub allow_failure: bool,

    /// Retry configuration
    pub retry_config: Option<RetryConfig>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}

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

pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Delay between retries in milliseconds
    pub delay_ms: u64,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f32,
}
```

### Field Descriptions

- **name**: Unique identifier for the tool. Used in guideline references.
- **description**: Human-readable description for LLM tool selection.
- **parameters**: JSON Schema defining expected parameters.
- **handler**: Async function implementation via trait object.
- **timeout_secs**: Maximum execution time before timeout.
- **allow_failure**: If true, failures won't halt conversation.
- **retry_config**: Optional retry behavior for transient failures.
- **metadata**: Additional tracking data (e.g., cost, category).

### Validation Rules

1. **name**: Match regex `^[a-zA-Z][a-zA-Z0-9_]*$`, 1-50 characters
2. **description**: 1-500 characters
3. **parameters**: Valid JSON Schema with type: "object"
4. **timeout_secs**: 1-300
5. **retry_config.max_attempts**: 1-10
6. **retry_config.delay_ms**: 10-60000
7. **retry_config.backoff_multiplier**: 1.0-10.0

### Relationships

- **Belongs To**: Agent (N:1)
- **Referenced By**: Guidelines (N:N)

### State Transitions

Tool execution states:
1. **Pending**: Queued for execution
2. **Running**: Currently executing
3. **Success**: Completed successfully
4. **Failed**: Execution failed
5. **Timeout**: Exceeded timeout_secs
6. **Retrying**: Retry attempt in progress

### JSON Serialization Example

```json
{
  "name": "check_order",
  "description": "Retrieve order details by order ID",
  "parameters": {
    "type": "object",
    "properties": {
      "order_id": {
        "type": "string",
        "description": "Unique order identifier"
      }
    },
    "required": ["order_id"]
  },
  "timeout_secs": 30,
  "allow_failure": false,
  "retry_config": {
    "max_attempts": 3,
    "delay_ms": 1000,
    "backoff_multiplier": 2.0
  },
  "metadata": {
    "category": "order_management",
    "cost": "low"
  }
}
```

---

## 4. Journey

### Description
A multi-step interaction flow with state transitions and step-specific behaviors. Journeys guide users through complex processes.

### Rust Definition

```rust
pub struct Journey {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the journey purpose
    pub description: String,

    /// Ordered steps in the journey
    pub steps: Vec<JourneyStep>,

    /// Initial step ID
    pub initial_step: String,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

pub struct JourneyStep {
    /// Step identifier (unique within journey)
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what happens in this step
    pub description: String,

    /// Step-specific guidelines
    pub guidelines: Vec<String>,

    /// Required context to complete this step
    pub required_context: Vec<String>,

    /// Possible transitions to other steps
    pub transitions: Vec<JourneyTransition>,

    /// Whether this is a terminal step
    pub is_terminal: bool,
}

pub struct JourneyTransition {
    /// Target step ID
    pub to_step: String,

    /// Condition for this transition
    pub condition: String,

    /// Priority if multiple transitions match
    pub priority: i32,
}

pub struct JourneyState {
    /// Journey ID
    pub journey_id: String,

    /// Current step ID
    pub current_step: String,

    /// Collected context during journey
    pub context: HashMap<String, serde_json::Value>,

    /// Journey started timestamp
    pub started_at: DateTime<Utc>,

    /// Last step transition timestamp
    pub last_transition_at: DateTime<Utc>,

    /// Step history
    pub step_history: Vec<StepHistoryEntry>,
}

pub struct StepHistoryEntry {
    pub step_id: String,
    pub entered_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
}
```

### Field Descriptions

**Journey:**
- **id**: Unique identifier for the journey.
- **name**: Display name for the journey.
- **description**: Purpose and overview of the journey.
- **steps**: Ordered collection of steps in the journey.
- **initial_step**: ID of the first step to start from.
- **metadata**: Additional tracking data.
- **created_at**: UTC timestamp of journey creation.

**JourneyStep:**
- **id**: Step identifier (unique within journey).
- **name**: Display name for the step.
- **description**: What happens in this step.
- **guidelines**: IDs of guidelines specific to this step.
- **required_context**: Context variables needed to complete this step.
- **transitions**: Possible next steps based on conditions.
- **is_terminal**: If true, journey ends at this step.

**JourneyTransition:**
- **to_step**: Target step ID to transition to.
- **condition**: Natural language condition for transition.
- **priority**: Priority if multiple transitions match.

**JourneyState:**
- **journey_id**: Reference to the journey definition.
- **current_step**: Current position in the journey.
- **context**: Data collected during the journey.
- **started_at**: When the journey began.
- **last_transition_at**: Last step transition time.
- **step_history**: Complete history of step transitions.

### Validation Rules

1. **journey.id**: Non-empty, unique within agent
2. **journey.name**: 1-100 characters
3. **journey.description**: 1-1000 characters
4. **journey.initial_step**: Must reference an existing step
5. **step.id**: Unique within journey
6. **step.transitions**: All to_step values must reference existing steps
7. **step.guidelines**: All IDs must reference existing guidelines
8. **transition.priority**: Integer, typical range -100 to 100

### State Transitions

Journey lifecycle:
1. **Not Started**: Journey exists but not active
2. **Active**: User is progressing through steps
3. **Completed**: User reached a terminal step
4. **Abandoned**: No activity for timeout period
5. **Failed**: Error occurred during journey

Step transitions:
1. Evaluate all transitions from current step
2. Sort by priority (descending)
3. Check conditions using LLM
4. Execute first matching transition
5. Update JourneyState

### JSON Serialization Example

```json
{
  "id": "onboarding_journey",
  "name": "New User Onboarding",
  "description": "Guide new users through account setup",
  "steps": [
    {
      "id": "welcome",
      "name": "Welcome",
      "description": "Greet user and explain onboarding process",
      "guidelines": ["guideline_welcome"],
      "required_context": [],
      "transitions": [
        {
          "to_step": "collect_name",
          "condition": "user is ready to continue",
          "priority": 10
        }
      ],
      "is_terminal": false
    },
    {
      "id": "collect_name",
      "name": "Collect Name",
      "description": "Ask for and store user's name",
      "guidelines": ["guideline_ask_name"],
      "required_context": ["user_name"],
      "transitions": [
        {
          "to_step": "collect_email",
          "condition": "name is collected",
          "priority": 10
        }
      ],
      "is_terminal": false
    },
    {
      "id": "collect_email",
      "name": "Collect Email",
      "description": "Ask for and validate email address",
      "guidelines": ["guideline_ask_email"],
      "required_context": ["user_email"],
      "transitions": [
        {
          "to_step": "complete",
          "condition": "email is valid",
          "priority": 10
        }
      ],
      "is_terminal": false
    },
    {
      "id": "complete",
      "name": "Onboarding Complete",
      "description": "Thank user and confirm account creation",
      "guidelines": ["guideline_onboarding_complete"],
      "required_context": [],
      "transitions": [],
      "is_terminal": true
    }
  ],
  "initial_step": "welcome",
  "metadata": {
    "category": "onboarding",
    "version": "1.0"
  },
  "created_at": "2025-01-15T10:30:00Z"
}
```

---

## 5. Context

### Description
Session-specific data including conversation history and dynamic variables extracted from user interactions.

### Rust Definition

```rust
pub struct Context {
    /// Session identifier
    pub session_id: String,

    /// Conversation history
    pub messages: Vec<Message>,

    /// Dynamic context variables
    pub variables: HashMap<String, ContextValue>,

    /// Optional journey state
    pub journey_state: Option<JourneyState>,

    /// Session metadata
    pub metadata: HashMap<String, String>,

    /// Session created timestamp
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,
}

pub struct Message {
    /// Message identifier
    pub id: String,

    /// Role: "user", "assistant", "system", "tool"
    pub role: String,

    /// Message content
    pub content: String,

    /// Optional tool call information
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Optional tool result
    pub tool_result: Option<ToolResult>,

    /// Message timestamp
    pub timestamp: DateTime<Utc>,

    /// Message metadata
    pub metadata: HashMap<String, String>,
}

pub struct ToolCall {
    /// Tool call identifier
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,
}

pub struct ContextValue {
    /// Variable name
    pub name: String,

    /// Variable value (JSON)
    pub value: serde_json::Value,

    /// When this value was extracted/updated
    pub extracted_at: DateTime<Utc>,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Source message ID
    pub source_message_id: Option<String>,
}
```

### Field Descriptions

**Context:**
- **session_id**: Unique identifier for this conversation session.
- **messages**: Chronological conversation history.
- **variables**: Named values extracted from conversation.
- **journey_state**: If in a journey, tracks current position and context.
- **metadata**: Additional session tracking data (e.g., user_id, channel).
- **created_at**: Session start time.
- **last_activity_at**: Most recent message timestamp.

**Message:**
- **id**: Unique message identifier.
- **role**: Message sender ("user", "assistant", "system", "tool").
- **content**: Message text content.
- **tool_calls**: If assistant requested tool execution.
- **tool_result**: If this is a tool response message.
- **timestamp**: When message was created.
- **metadata**: Additional tracking data.

**ContextValue:**
- **name**: Variable name (matches ContextVariable definition).
- **value**: Extracted value as JSON.
- **extracted_at**: Extraction timestamp.
- **confidence**: Confidence score from LLM extraction.
- **source_message_id**: Which message this was extracted from.

### Validation Rules

1. **session_id**: Non-empty, UUID recommended
2. **messages**: Chronologically ordered
3. **message.role**: Must be one of ["user", "assistant", "system", "tool"]
4. **variables**: Keys must match defined ContextVariable names
5. **context_value.confidence**: 0.0-1.0
6. **journey_state**: If present, journey_id must reference existing journey

### Relationships

- **Belongs To**: Session (1:1)
- **Contains**: Messages (1:N)
- **Contains**: ContextValues (1:N)
- **References**: Journey (0:1 via JourneyState)

### State Transitions

Session lifecycle:
1. **Active**: Recent activity within timeout
2. **Idle**: No activity but within session TTL
3. **Expired**: Exceeded session TTL
4. **Archived**: Persisted to storage, not in memory

### JSON Serialization Example

```json
{
  "session_id": "session_660e8400-e29b-41d4-a716-446655440000",
  "messages": [
    {
      "id": "msg_1",
      "role": "user",
      "content": "Hi, I need help with my order #12345",
      "tool_calls": null,
      "tool_result": null,
      "timestamp": "2025-01-15T14:30:00Z",
      "metadata": {}
    },
    {
      "id": "msg_2",
      "role": "assistant",
      "content": "I'll check your order status for you.",
      "tool_calls": [
        {
          "id": "call_1",
          "name": "check_order",
          "arguments": {"order_id": "12345"}
        }
      ],
      "tool_result": null,
      "timestamp": "2025-01-15T14:30:01Z",
      "metadata": {}
    }
  ],
  "variables": {
    "order_id": {
      "name": "order_id",
      "value": "12345",
      "extracted_at": "2025-01-15T14:30:00Z",
      "confidence": 0.95,
      "source_message_id": "msg_1"
    }
  },
  "journey_state": null,
  "metadata": {
    "user_id": "user_123",
    "channel": "web_chat"
  },
  "created_at": "2025-01-15T14:30:00Z",
  "last_activity_at": "2025-01-15T14:30:01Z"
}
```

---

## 6. ContextVariable

### Description
A named piece of data that is automatically extracted from conversations using LLM analysis.

### Rust Definition

```rust
pub struct ContextVariable {
    /// Variable name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Expected data type
    pub data_type: VariableDataType,

    /// LLM prompt for extraction
    pub extraction_prompt: String,

    /// Whether this variable is required
    pub required: bool,

    /// Validation rules
    pub validation: Option<ValidationRules>,

    /// Default value if not extracted
    pub default_value: Option<serde_json::Value>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

pub enum VariableDataType {
    String,
    Number,
    Boolean,
    Date,
    Array,
    Object,
}

pub struct ValidationRules {
    /// Regex pattern for string validation
    pub pattern: Option<String>,

    /// Minimum value for numbers
    pub min: Option<f64>,

    /// Maximum value for numbers
    pub max: Option<f64>,

    /// Minimum length for strings/arrays
    pub min_length: Option<usize>,

    /// Maximum length for strings/arrays
    pub max_length: Option<usize>,

    /// Enum of allowed values
    pub allowed_values: Option<Vec<serde_json::Value>>,
}
```

### Field Descriptions

- **name**: Unique identifier for the variable.
- **description**: Human-readable description of what this variable represents.
- **data_type**: Expected type of the extracted value.
- **extraction_prompt**: LLM prompt used to extract this variable from messages.
- **required**: Whether this variable must be present for certain operations.
- **validation**: Optional validation rules to apply to extracted values.
- **default_value**: Fallback value if extraction fails.
- **metadata**: Additional tracking data.

### Validation Rules

1. **name**: Match regex `^[a-z][a-z0-9_]*$`, 1-50 characters
2. **description**: 1-500 characters
3. **extraction_prompt**: 1-1000 characters
4. **validation.pattern**: Valid regex if provided
5. **validation.min**: Less than or equal to max
6. **validation.max**: Greater than or equal to min
7. **validation.min_length**: Less than or equal to max_length
8. **validation.max_length**: Greater than or equal to min_length
9. **default_value**: Must match data_type

### Relationships

- **Belongs To**: Agent (N:1)
- **Referenced By**: Guidelines (N:N via required_context)
- **Referenced By**: JourneySteps (N:N via required_context)
- **Stored In**: Context (N:N via variables HashMap)

### JSON Serialization Example

```json
{
  "name": "order_id",
  "description": "Customer's order identifier",
  "data_type": "String",
  "extraction_prompt": "Extract the order ID or order number from the user's message. It typically follows patterns like '#12345' or 'order 12345'.",
  "required": false,
  "validation": {
    "pattern": "^[0-9]{5,10}$",
    "min": null,
    "max": null,
    "min_length": 5,
    "max_length": 10,
    "allowed_values": null
  },
  "default_value": null,
  "metadata": {
    "category": "order_management"
  }
}
```

---

## 7. Session

### Description
A conversation instance with its own context, state, and history. Sessions encapsulate all data for a single user conversation.

### Rust Definition

```rust
pub struct Session {
    /// Session identifier
    pub id: String,

    /// Agent identifier
    pub agent_id: String,

    /// Session context (history, variables, journey)
    pub context: Context,

    /// Session state
    pub state: SessionState,

    /// Session configuration
    pub config: SessionConfig,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,

    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum SessionState {
    /// Active conversation
    Active,

    /// Inactive but within TTL
    Idle,

    /// Waiting for user input
    AwaitingInput,

    /// Waiting for tool execution
    AwaitingTool,

    /// Session has ended
    Completed,

    /// Session exceeded TTL
    Expired,
}

pub struct SessionConfig {
    /// Maximum session duration in seconds
    pub ttl_secs: u64,

    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,

    /// Maximum messages to retain in memory
    pub max_messages: usize,

    /// Enable automatic context extraction
    pub auto_extract: bool,

    /// Enable journey tracking
    pub enable_journeys: bool,
}
```

### Field Descriptions

**Session:**
- **id**: Unique identifier for this session.
- **agent_id**: Reference to the agent handling this session.
- **context**: Session-specific context (messages, variables, journey state).
- **state**: Current session state.
- **config**: Session-specific configuration.
- **created_at**: Session creation time.
- **last_activity_at**: Most recent message or activity.
- **expires_at**: Optional absolute expiration time.

**SessionConfig:**
- **ttl_secs**: Maximum session lifetime in seconds.
- **idle_timeout_secs**: Seconds of inactivity before marking idle.
- **max_messages**: Maximum conversation history to retain.
- **auto_extract**: Whether to automatically extract context variables.
- **enable_journeys**: Whether to track journey state.

### Validation Rules

1. **id**: Non-empty, UUID recommended, unique globally
2. **agent_id**: Must reference an existing agent
3. **config.ttl_secs**: 60-86400 (1 minute to 1 day)
4. **config.idle_timeout_secs**: 30-3600 (30 seconds to 1 hour)
5. **config.max_messages**: 10-1000
6. **expires_at**: If set, must be in the future

### Relationships

- **Belongs To**: Agent (N:1)
- **Has One**: Context (1:1)
- **May Reference**: Journey (0:1 via Context.journey_state)

### State Transitions

```
Active → Idle (idle_timeout_secs exceeded)
Active → AwaitingInput (waiting for user)
Active → AwaitingTool (tool executing)
AwaitingInput → Active (user message received)
AwaitingTool → Active (tool execution complete)
Active → Completed (user or agent ends session)
Idle → Expired (ttl_secs exceeded)
Any → Expired (expires_at reached)
```

### JSON Serialization Example

```json
{
  "id": "session_770e8400-e29b-41d4-a716-446655440000",
  "agent_id": "agent_550e8400-e29b-41d4-a716-446655440000",
  "context": {
    "session_id": "session_770e8400-e29b-41d4-a716-446655440000",
    "messages": [],
    "variables": {},
    "journey_state": null,
    "metadata": {
      "user_id": "user_456",
      "channel": "mobile_app"
    },
    "created_at": "2025-01-15T15:00:00Z",
    "last_activity_at": "2025-01-15T15:00:00Z"
  },
  "state": "Active",
  "config": {
    "ttl_secs": 3600,
    "idle_timeout_secs": 300,
    "max_messages": 100,
    "auto_extract": true,
    "enable_journeys": true
  },
  "created_at": "2025-01-15T15:00:00Z",
  "last_activity_at": "2025-01-15T15:00:00Z",
  "expires_at": "2025-01-15T16:00:00Z"
}
```

---

## 8. GuidelineMatch

### Description
Result of evaluating a message against guidelines, including relevance score, matched parameters, and extracted context.

### Rust Definition

```rust
pub struct GuidelineMatch {
    /// Matched guideline ID
    pub guideline_id: String,

    /// Guideline priority
    pub priority: i32,

    /// Relevance score (0.0-1.0)
    pub relevance_score: f32,

    /// Matched condition
    pub condition: String,

    /// Recommended action
    pub action: String,

    /// Tools to invoke
    pub tools: Vec<String>,

    /// Extracted parameters for tools
    pub tool_parameters: HashMap<String, serde_json::Value>,

    /// Required context that was matched
    pub matched_context: HashMap<String, serde_json::Value>,

    /// Confidence in this match
    pub confidence: f32,

    /// Reasoning for the match
    pub reasoning: Option<String>,

    /// Timestamp of evaluation
    pub evaluated_at: DateTime<Utc>,
}

pub struct GuidelineMatchResult {
    /// All matches found
    pub matches: Vec<GuidelineMatch>,

    /// Top N matches by priority and score
    pub top_matches: Vec<GuidelineMatch>,

    /// Combined action from top matches
    pub combined_action: Option<String>,

    /// All tools to execute
    pub tools_to_execute: Vec<ToolExecution>,

    /// Evaluation time in milliseconds
    pub evaluation_time_ms: u64,
}

pub struct ToolExecution {
    /// Tool name
    pub tool_name: String,

    /// Tool parameters
    pub parameters: serde_json::Value,

    /// Execution priority
    pub priority: i32,

    /// Source guideline
    pub guideline_id: String,
}
```

### Field Descriptions

**GuidelineMatch:**
- **guideline_id**: ID of the matched guideline.
- **priority**: Priority of the matched guideline.
- **relevance_score**: How relevant this guideline is (0.0-1.0).
- **condition**: The condition that was matched.
- **action**: The action to take.
- **tools**: Tool names to invoke.
- **tool_parameters**: Extracted parameters for each tool.
- **matched_context**: Context variables that satisfied requirements.
- **confidence**: Confidence in this match (0.0-1.0).
- **reasoning**: Optional explanation of why this matched.
- **evaluated_at**: When this evaluation occurred.

**GuidelineMatchResult:**
- **matches**: All guidelines that matched above threshold.
- **top_matches**: Top N matches by combined priority and relevance.
- **combined_action**: Merged action text from top matches.
- **tools_to_execute**: Deduplicated tools with parameters.
- **evaluation_time_ms**: Time taken to evaluate all guidelines.

### Validation Rules

1. **relevance_score**: 0.0-1.0
2. **confidence**: 0.0-1.0
3. **tool_parameters**: Must conform to tool's parameter schema
4. **matched_context**: Keys must match ContextVariable names
5. **evaluation_time_ms**: Positive integer

### Relationships

- **References**: Guideline (N:1)
- **References**: Tools (N:N via tool names)
- **References**: ContextVariables (N:N via matched_context)
- **Created By**: Session message processing

### Matching Algorithm

1. Filter enabled guidelines
2. If in journey, filter by journey_id and current step
3. Check required_context availability
4. Evaluate condition relevance using LLM (returns 0.0-1.0)
5. Filter matches below relevance threshold (default 0.3)
6. Sort by priority (desc), then relevance_score (desc)
7. Take top N matches (configurable, default 3)
8. Extract tool parameters from user message
9. Combine actions and deduplicate tools

### JSON Serialization Example

```json
{
  "guideline_id": "guideline_refund_policy",
  "priority": 100,
  "relevance_score": 0.92,
  "condition": "user asks about refunds, returns, or mentions 'money back'",
  "action": "Explain the 30-day refund policy and offer to check their order status",
  "tools": ["check_order", "get_refund_policy"],
  "tool_parameters": {
    "check_order": {
      "order_id": "12345"
    },
    "get_refund_policy": {}
  },
  "matched_context": {
    "order_id": "12345"
  },
  "confidence": 0.88,
  "reasoning": "User explicitly mentioned 'refund' and provided order number",
  "evaluated_at": "2025-01-15T14:35:00Z"
}
```

---

## Entity Relationship Diagram

```
Agent (1) ----< (N) Guideline
  |                     |
  |                     |
  |--- (N) Tool         |
  |--- (N) Journey      |
  |--- (N) ContextVariable
  |                     |
  |                     v
  +--< (N) Session ---> Context
            |             |
            |             +--- (N) Message
            |             +--- (N) ContextValue
            |             +--- (0..1) JourneyState
            |
            v
       GuidelineMatch (ephemeral, created during processing)
```

---

## Summary

This data model provides a comprehensive foundation for building a conversational AI agent library in Rust with:

- **Modularity**: Clear separation of concerns (Agent, Guideline, Tool, Journey)
- **Flexibility**: Extensible through metadata and trait-based providers
- **Type Safety**: Strong typing with validation rules
- **State Management**: Clear state transitions for sessions and journeys
- **Persistence**: JSON serialization for storage and API integration
- **Scalability**: Designed for async execution and concurrent sessions

Each entity includes detailed documentation, validation rules, relationships, and real-world examples for implementation guidance.
