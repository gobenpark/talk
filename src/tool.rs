//! Tool integration for external API and function calls
//!
//! This module implements the tool system that allows agents to call external
//! APIs and functions during conversation processing.

use crate::error::{AgentError, Result};
use crate::types::ToolId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, info, trace, warn};

/// Parameter schema definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub param_type: String,
    pub required: bool,
    pub description: String,
    pub default: Option<serde_json::Value>,
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trait for tools that can be executed by the agent
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Unique identifier for this tool
    fn id(&self) -> &ToolId;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Description of what this tool does
    fn description(&self) -> &str;

    /// Parameter schema for this tool
    fn parameters(&self) -> &HashMap<String, ParameterSchema>;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult>;

    /// Validate parameters before execution
    fn validate_parameters(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        trace!(tool_name = %self.name(), "Validating tool parameters");

        let schema = self.parameters();

        // Check required parameters
        for (param_name, param_schema) in schema {
            if param_schema.required && !parameters.contains_key(param_name) {
                warn!(
                    tool_name = %self.name(),
                    param_name = %param_name,
                    "Missing required parameter"
                );
                return Err(AgentError::InvalidToolParameters {
                    tool_name: self.name().to_string(),
                    reason: format!("Missing required parameter: {}", param_name),
                });
            }
        }

        // Validate parameter types
        for (param_name, value) in parameters {
            if let Some(param_schema) = schema.get(param_name) {
                if !validate_type(value, &param_schema.param_type) {
                    warn!(
                        tool_name = %self.name(),
                        param_name = %param_name,
                        expected_type = %param_schema.param_type,
                        "Parameter type mismatch"
                    );
                    return Err(AgentError::InvalidToolParameters {
                        tool_name: self.name().to_string(),
                        reason: format!(
                            "Parameter '{}' has wrong type, expected {}",
                            param_name, param_schema.param_type
                        ),
                    });
                }
            }
        }

        debug!(
            tool_name = %self.name(),
            param_count = parameters.len(),
            "Parameter validation successful"
        );

        Ok(())
    }

    /// Apply default values to parameters
    fn apply_defaults(
        &self,
        parameters: &mut HashMap<String, serde_json::Value>,
    ) {
        let schema = self.parameters();

        for (param_name, param_schema) in schema {
            if !parameters.contains_key(param_name) {
                if let Some(ref default_value) = param_schema.default {
                    trace!(
                        tool_name = %self.name(),
                        param_name = %param_name,
                        "Applying default parameter value"
                    );
                    parameters.insert(param_name.clone(), default_value.clone());
                }
            }
        }
    }
}

/// Validate a JSON value against a type string
fn validate_type(value: &serde_json::Value, expected_type: &str) -> bool {
    use serde_json::Value;

    match expected_type {
        "string" => matches!(value, Value::String(_)),
        "number" => matches!(value, Value::Number(_)),
        "boolean" => matches!(value, Value::Bool(_)),
        "object" => matches!(value, Value::Object(_)),
        "array" => matches!(value, Value::Array(_)),
        "null" => matches!(value, Value::Null),
        _ => true, // Unknown types pass validation
    }
}

/// Registry for managing tools
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<ToolId, Arc<dyn Tool>>>>,
    tools_by_name: Arc<RwLock<HashMap<String, ToolId>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        info!("Creating new tool registry");
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tools_by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new tool
    pub async fn register(&self, tool: Box<dyn Tool>) -> Result<ToolId> {
        let tool_id = *tool.id();
        let tool_name = tool.name().to_string();

        info!(
            tool_id = %tool_id,
            tool_name = %tool_name,
            "Registering tool"
        );

        let mut tools = self.tools.write().await;
        let mut tools_by_name = self.tools_by_name.write().await;

        // Check if tool with same name already exists
        if tools_by_name.contains_key(&tool_name) {
            warn!(
                tool_name = %tool_name,
                "Attempted to register duplicate tool"
            );
            return Err(AgentError::ToolAlreadyRegistered(tool_name));
        }

        tools.insert(tool_id, Arc::from(tool));
        tools_by_name.insert(tool_name.clone(), tool_id);

        debug!(
            tool_id = %tool_id,
            tool_name = %tool_name,
            total_tools = tools.len(),
            "Tool registered successfully"
        );

        Ok(tool_id)
    }

    /// Unregister a tool by ID
    pub async fn unregister(&self, tool_id: &ToolId) -> Result<()> {
        info!(tool_id = %tool_id, "Unregistering tool");

        let mut tools = self.tools.write().await;
        let mut tools_by_name = self.tools_by_name.write().await;

        if let Some(tool) = tools.remove(tool_id) {
            let tool_name = tool.name().to_string();
            tools_by_name.remove(&tool_name);

            debug!(
                tool_id = %tool_id,
                tool_name = %tool_name,
                remaining_tools = tools.len(),
                "Tool unregistered successfully"
            );

            Ok(())
        } else {
            warn!(tool_id = %tool_id, "Attempted to unregister unknown tool");
            Err(AgentError::ToolNotFound(*tool_id))
        }
    }

    /// Get a tool by ID
    pub async fn get(&self, tool_id: &ToolId) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(tool_id).cloned()
    }

    /// Get a tool by name
    pub async fn get_by_name(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools_by_name = self.tools_by_name.read().await;
        let tool_id = tools_by_name.get(name)?;

        let tools = self.tools.read().await;
        tools.get(tool_id).cloned()
    }

    /// List all registered tools
    pub async fn list(&self) -> Vec<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Execute a tool by ID with parameters
    pub async fn execute(
        &self,
        tool_id: &ToolId,
        mut parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        info!(
            tool_id = %tool_id,
            param_count = parameters.len(),
            "Executing tool"
        );

        let tool = self
            .get(tool_id)
            .await
            .ok_or_else(|| AgentError::ToolNotFound(*tool_id))?;

        // Apply default values
        tool.apply_defaults(&mut parameters);

        // Validate parameters
        tool.validate_parameters(&parameters)?;

        // Execute tool
        let result = tool.execute(parameters).await?;

        debug!(
            tool_id = %tool_id,
            tool_name = %tool.name(),
            has_error = result.error.is_some(),
            "Tool execution completed"
        );

        Ok(result)
    }

    /// Execute a tool with a timeout
    pub async fn execute_with_timeout(
        &self,
        tool_id: &ToolId,
        parameters: HashMap<String, serde_json::Value>,
        timeout_duration: Duration,
    ) -> Result<ToolResult> {
        info!(
            tool_id = %tool_id,
            timeout_secs = timeout_duration.as_secs(),
            "Executing tool with timeout"
        );

        match timeout(timeout_duration, self.execute(tool_id, parameters)).await {
            Ok(result) => result,
            Err(_) => {
                warn!(
                    tool_id = %tool_id,
                    timeout_secs = timeout_duration.as_secs(),
                    "Tool execution timed out"
                );

                let tool = self.get(tool_id).await;
                let tool_name = tool.map(|t| t.name().to_string()).unwrap_or_else(|| "unknown".to_string());

                Err(AgentError::ToolTimeout {
                    tool_name,
                    timeout: timeout_duration,
                })
            }
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool {
        id: ToolId,
        parameters: HashMap<String, ParameterSchema>,
    }

    impl TestTool {
        fn new() -> Self {
            let mut parameters = HashMap::new();
            parameters.insert(
                "message".to_string(),
                ParameterSchema {
                    param_type: "string".to_string(),
                    required: true,
                    description: "A test message".to_string(),
                    default: None,
                },
            );

            Self {
                id: ToolId::new(),
                parameters,
            }
        }
    }

    #[async_trait::async_trait]
    impl Tool for TestTool {
        fn id(&self) -> &ToolId {
            &self.id
        }

        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "A test tool"
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

    #[tokio::test]
    async fn test_parameter_validation_missing_required() {
        let tool = TestTool::new();
        let params = HashMap::new();

        let result = tool.validate_parameters(&params);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_validation_type_mismatch() {
        let tool = TestTool::new();
        let mut params = HashMap::new();
        params.insert("message".to_string(), serde_json::json!(123)); // number instead of string

        let result = tool.validate_parameters(&params);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_validation_success() {
        let tool = TestTool::new();
        let mut params = HashMap::new();
        params.insert("message".to_string(), serde_json::json!("Hello"));

        let result = tool.validate_parameters(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_type_validation() {
        assert!(validate_type(&serde_json::json!("hello"), "string"));
        assert!(validate_type(&serde_json::json!(123), "number"));
        assert!(validate_type(&serde_json::json!(true), "boolean"));
        assert!(validate_type(&serde_json::json!({}), "object"));
        assert!(validate_type(&serde_json::json!([]), "array"));
        assert!(validate_type(&serde_json::json!(null), "null"));

        assert!(!validate_type(&serde_json::json!(123), "string"));
        assert!(!validate_type(&serde_json::json!("hello"), "number"));
    }
}
