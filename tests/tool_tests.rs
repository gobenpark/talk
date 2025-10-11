//! Integration tests for Tool API
//!
//! Tests tool registration, execution, parameter validation, error handling,
//! and timeout behavior following TDD principles.

use talk::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

/// Mock tool that echoes input parameters
struct EchoTool {
    id: ToolId,
    parameters: HashMap<String, ParameterSchema>,
}

impl EchoTool {
    fn new() -> Self {
        let mut parameters = HashMap::new();
        parameters.insert(
            "message".to_string(),
            ParameterSchema {
                param_type: "string".to_string(),
                required: false,
                description: "Message to echo back".to_string(),
                default: Some(json!("Hello")),
            },
        );

        Self {
            id: ToolId::new(),
            parameters,
        }
    }
}

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn id(&self) -> &ToolId {
        &self.id
    }

    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the input parameters"
    }

    fn parameters(&self) -> &HashMap<String, ParameterSchema> {
        &self.parameters
    }

    async fn execute(
        &self,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        Ok(ToolResult {
            output: serde_json::to_value(parameters).unwrap(),
            error: None,
            metadata: HashMap::new(),
        })
    }
}

/// Mock tool that simulates slow execution
struct SlowTool {
    id: ToolId,
    delay: Duration,
    parameters: HashMap<String, ParameterSchema>,
}

impl SlowTool {
    fn new(delay: Duration) -> Self {
        Self {
            id: ToolId::new(),
            delay,
            parameters: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for SlowTool {
    fn id(&self) -> &ToolId {
        &self.id
    }

    fn name(&self) -> &str {
        "slow"
    }

    fn description(&self) -> &str {
        "A tool that takes time to execute"
    }

    fn parameters(&self) -> &HashMap<String, ParameterSchema> {
        &self.parameters
    }

    async fn execute(
        &self,
        _parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        tokio::time::sleep(self.delay).await;

        Ok(ToolResult {
            output: json!({"status": "completed"}),
            error: None,
            metadata: HashMap::new(),
        })
    }
}

/// Mock tool that always fails
struct FailingTool {
    id: ToolId,
    parameters: HashMap<String, ParameterSchema>,
}

impl FailingTool {
    fn new() -> Self {
        Self {
            id: ToolId::new(),
            parameters: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for FailingTool {
    fn id(&self) -> &ToolId {
        &self.id
    }

    fn name(&self) -> &str {
        "failing"
    }

    fn description(&self) -> &str {
        "A tool that always fails"
    }

    fn parameters(&self) -> &HashMap<String, ParameterSchema> {
        &self.parameters
    }

    async fn execute(
        &self,
        _parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        Err(AgentError::ToolExecutionFailed {
            tool_name: "failing".to_string(),
            reason: "Simulated failure".to_string(),
        })
    }
}

// ============================================================================
// T047: Tool Registration and Management Tests
// ============================================================================

#[tokio::test]
async fn test_register_tool() {
    let tool_registry = ToolRegistry::new();
    let echo_tool = EchoTool::new();

    let tool_id = tool_registry
        .register(Box::new(echo_tool))
        .await
        .expect("Failed to register tool");

    assert!(tool_registry.get(&tool_id).await.is_some());
}

#[tokio::test]
async fn test_register_duplicate_tool_fails() {
    let tool_registry = ToolRegistry::new();
    let echo_tool1 = EchoTool::new();
    let echo_tool2 = EchoTool::new();

    tool_registry
        .register(Box::new(echo_tool1))
        .await
        .expect("First registration should succeed");

    let result = tool_registry
        .register(Box::new(echo_tool2))
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::ToolAlreadyRegistered(_)
    ));
}

#[tokio::test]
async fn test_unregister_tool() {
    let tool_registry = ToolRegistry::new();
    let echo_tool = EchoTool::new();

    let tool_id = tool_registry
        .register(Box::new(echo_tool))
        .await
        .expect("Failed to register tool");

    tool_registry
        .unregister(&tool_id)
        .await
        .expect("Failed to unregister tool");

    assert!(tool_registry.get(&tool_id).await.is_none());
}

#[tokio::test]
async fn test_list_tools() {
    let tool_registry = ToolRegistry::new();
    let echo_tool = EchoTool::new();
    let slow_tool = SlowTool::new(Duration::from_millis(100));

    tool_registry.register(Box::new(echo_tool)).await.unwrap();
    tool_registry.register(Box::new(slow_tool)).await.unwrap();

    let tools = tool_registry.list().await;
    assert_eq!(tools.len(), 2);
}

#[tokio::test]
async fn test_get_tool_by_id() {
    let tool_registry = ToolRegistry::new();
    let echo_tool = EchoTool::new();

    let tool_id = tool_registry
        .register(Box::new(echo_tool))
        .await
        .unwrap();

    let retrieved = tool_registry.get(&tool_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "echo");
}

// ============================================================================
// T048: Tool Execution Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_execute_tool_basic() {
    let tool_registry = ToolRegistry::new();
    let echo_tool = EchoTool::new();

    let tool_id = tool_registry
        .register(Box::new(echo_tool))
        .await
        .unwrap();

    let mut params = HashMap::new();
    params.insert("message".to_string(), json!("Hello"));

    let result = tool_registry
        .execute(&tool_id, params)
        .await
        .expect("Tool execution failed");

    assert!(result.error.is_none());
    assert_eq!(result.output["message"], "Hello");
}

#[tokio::test]
async fn test_execute_tool_multiple_times() {
    let echo_tool = EchoTool::new();
    let tool_registry = ToolRegistry::new();

    let tool_id = tool_registry
        .register(Box::new(echo_tool))
        .await
        .unwrap();

    // Execute multiple times
    for i in 0..5 {
        let mut params = HashMap::new();
        params.insert("count".to_string(), json!(i));

        let result = tool_registry.execute(&tool_id, params).await.unwrap();

        // Verify output matches input
        assert_eq!(result.output["count"], i);
        assert!(result.error.is_none());
    }
}

#[tokio::test]
async fn test_execute_nonexistent_tool_fails() {
    let tool_registry = ToolRegistry::new();
    let fake_id = ToolId::new();

    let params = HashMap::new();
    let result = tool_registry.execute(&fake_id, params).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::ToolNotFound(_)
    ));
}

// ============================================================================
// T049: Tool Parameter Validation Tests
// ============================================================================

#[tokio::test]
async fn test_validate_required_parameters() {
    // Create a tool with required parameters
    struct RequiredParamTool {
        id: ToolId,
        parameters: HashMap<String, ParameterSchema>,
    }

    let mut param_schema = HashMap::new();
    param_schema.insert(
        "required_field".to_string(),
        ParameterSchema {
            param_type: "string".to_string(),
            required: true,
            description: "A required parameter".to_string(),
            default: None,
        },
    );

    #[async_trait::async_trait]
    impl Tool for RequiredParamTool {
        fn id(&self) -> &ToolId {
            &self.id
        }

        fn name(&self) -> &str {
            "required_param_tool"
        }

        fn description(&self) -> &str {
            "A tool with required parameters"
        }

        fn parameters(&self) -> &HashMap<String, ParameterSchema> {
            &self.parameters
        }

        async fn execute(
            &self,
            parameters: HashMap<String, serde_json::Value>,
        ) -> Result<ToolResult> {
            Ok(ToolResult {
                output: serde_json::to_value(parameters).unwrap(),
                error: None,
                metadata: HashMap::new(),
            })
        }
    }

    let tool = RequiredParamTool {
        id: ToolId::new(),
        parameters: param_schema,
    };

    let tool_registry = ToolRegistry::new();
    let tool_id = tool_registry.register(Box::new(tool)).await.unwrap();

    // Attempt to execute without required parameter
    let params = HashMap::new();
    let result = tool_registry.execute(&tool_id, params).await;

    // This should fail validation
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::InvalidToolParameters { .. }
    ));
}

#[tokio::test]
async fn test_validate_parameter_types() {
    // Create a tool expecting specific types
    struct TypedParamTool {
        id: ToolId,
        parameters: HashMap<String, ParameterSchema>,
    }

    let mut param_schema = HashMap::new();
    param_schema.insert(
        "number_field".to_string(),
        ParameterSchema {
            param_type: "number".to_string(),
            required: true,
            description: "A numeric parameter".to_string(),
            default: None,
        },
    );

    #[async_trait::async_trait]
    impl Tool for TypedParamTool {
        fn id(&self) -> &ToolId {
            &self.id
        }

        fn name(&self) -> &str {
            "typed_param_tool"
        }

        fn description(&self) -> &str {
            "A tool with typed parameters"
        }

        fn parameters(&self) -> &HashMap<String, ParameterSchema> {
            &self.parameters
        }

        async fn execute(
            &self,
            parameters: HashMap<String, serde_json::Value>,
        ) -> Result<ToolResult> {
            Ok(ToolResult {
                output: serde_json::to_value(parameters).unwrap(),
                error: None,
                metadata: HashMap::new(),
            })
        }
    }

    let tool = TypedParamTool {
        id: ToolId::new(),
        parameters: param_schema,
    };

    let tool_registry = ToolRegistry::new();
    let tool_id = tool_registry.register(Box::new(tool)).await.unwrap();

    // Attempt to pass wrong type (string instead of number)
    let mut params = HashMap::new();
    params.insert("number_field".to_string(), json!("not a number"));

    let result = tool_registry.execute(&tool_id, params).await;

    // This should fail validation
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::InvalidToolParameters { .. }
    ));
}

#[tokio::test]
async fn test_optional_parameters_with_defaults() {
    // Create a tool with optional parameters having defaults
    struct DefaultParamTool {
        id: ToolId,
        parameters: HashMap<String, ParameterSchema>,
    }

    let mut param_schema = HashMap::new();
    param_schema.insert(
        "optional_field".to_string(),
        ParameterSchema {
            param_type: "string".to_string(),
            required: false,
            description: "An optional parameter".to_string(),
            default: Some(json!("default_value")),
        },
    );

    #[async_trait::async_trait]
    impl Tool for DefaultParamTool {
        fn id(&self) -> &ToolId {
            &self.id
        }

        fn name(&self) -> &str {
            "default_param_tool"
        }

        fn description(&self) -> &str {
            "A tool with default parameters"
        }

        fn parameters(&self) -> &HashMap<String, ParameterSchema> {
            &self.parameters
        }

        async fn execute(
            &self,
            parameters: HashMap<String, serde_json::Value>,
        ) -> Result<ToolResult> {
            Ok(ToolResult {
                output: serde_json::to_value(parameters).unwrap(),
                error: None,
                metadata: HashMap::new(),
            })
        }
    }

    let tool = DefaultParamTool {
        id: ToolId::new(),
        parameters: param_schema,
    };

    let tool_registry = ToolRegistry::new();
    let tool_id = tool_registry.register(Box::new(tool)).await.unwrap();

    // Execute without providing optional parameter
    let params = HashMap::new();
    let result = tool_registry.execute(&tool_id, params).await.unwrap();

    // Should use default value
    assert_eq!(result.output["optional_field"], "default_value");
    assert!(result.error.is_none());
}

// ============================================================================
// T050: Tool Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_tool_execution_failure() {
    let tool_registry = ToolRegistry::new();
    let failing_tool = FailingTool::new();

    let tool_id = tool_registry
        .register(Box::new(failing_tool))
        .await
        .unwrap();

    let params = HashMap::new();
    let result = tool_registry.execute(&tool_id, params).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::ToolExecutionFailed { .. }
    ));
}

#[tokio::test]
async fn test_tool_error_contains_context() {
    let tool_registry = ToolRegistry::new();
    let failing_tool = FailingTool::new();

    let tool_id = tool_registry
        .register(Box::new(failing_tool))
        .await
        .unwrap();

    let params = HashMap::new();
    let result = tool_registry.execute(&tool_id, params).await;

    match result {
        Err(AgentError::ToolExecutionFailed { tool_name, reason }) => {
            assert_eq!(tool_name, "failing");
            assert!(reason.contains("Simulated failure"));
        }
        _ => panic!("Expected ToolExecutionFailed error"),
    }
}

// ============================================================================
// T051: Tool Timeout Behavior Tests
// ============================================================================

#[tokio::test]
async fn test_tool_execution_timeout() {
    let tool_registry = ToolRegistry::new();
    let slow_tool = SlowTool::new(Duration::from_secs(5));

    let tool_id = tool_registry
        .register(Box::new(slow_tool))
        .await
        .unwrap();

    let params = HashMap::new();

    // Set a short timeout (1 second)
    let timeout = Duration::from_secs(1);

    let result = tool_registry
        .execute_with_timeout(&tool_id, params, timeout)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AgentError::ToolTimeout { .. }
    ));
}

#[tokio::test]
async fn test_tool_execution_within_timeout() {
    let tool_registry = ToolRegistry::new();
    let slow_tool = SlowTool::new(Duration::from_millis(100));

    let tool_id = tool_registry
        .register(Box::new(slow_tool))
        .await
        .unwrap();

    let params = HashMap::new();

    // Set a generous timeout (5 seconds)
    let timeout = Duration::from_secs(5);

    let result = tool_registry
        .execute_with_timeout(&tool_id, params, timeout)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_default_timeout_from_config() {
    // This test demonstrates that AgentConfig has default_tool_timeout
    // The actual integration with Agent will be tested when tools are integrated with Agent (T055)

    let config = AgentConfig {
        max_context_messages: 100,
        default_tool_timeout: Duration::from_secs(2),
        enable_explainability: true,
        log_level: LogLevel::Info,
    };

    // Verify config has the timeout configured
    assert_eq!(config.default_tool_timeout, Duration::from_secs(2));
}
