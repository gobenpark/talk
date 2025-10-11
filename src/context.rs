//! Context management for agent conversations
//!
//! This module provides data structures for managing conversation context,
//! messages, and context variables extracted from user inputs.

use crate::types::MessageId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the system
    System,
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
    /// Tool execution result
    Tool,
}

/// A single message in the conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
    /// Optional metadata about the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp when the message was created
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::System,
            content: content.into(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::User,
            content: content.into(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::Assistant,
            content: content.into(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new tool message
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            role: MessageRole::Tool,
            content: content.into(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }
}

/// Validator for context variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Validator {
    /// String type with optional pattern
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    /// Integer type with optional range
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// Float type with optional range
    Float {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// Boolean type
    Boolean,
    /// Email address
    Email,
    /// URL
    Url,
    /// Date in ISO 8601 format
    Date,
    /// DateTime in ISO 8601 format
    DateTime,
    /// Enum with allowed values
    Enum { allowed_values: Vec<String> },
}

/// A context variable extracted from user input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextVariable {
    /// Name of the variable
    pub name: String,
    /// Value of the variable
    pub value: serde_json::Value,
    /// Optional validator for the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator: Option<Validator>,
    /// Source message ID where this variable was extracted
    pub source_message_id: MessageId,
    /// Timestamp when the variable was extracted
    pub extracted_at: DateTime<Utc>,
}

impl ContextVariable {
    /// Create a new context variable
    pub fn new(
        name: impl Into<String>,
        value: serde_json::Value,
        source_message_id: MessageId,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            validator: None,
            source_message_id,
            extracted_at: Utc::now(),
        }
    }

    /// Add a validator to the context variable
    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Validate the variable value against its validator
    pub fn validate(&self) -> Result<(), String> {
        match &self.validator {
            None => Ok(()),
            Some(Validator::String {
                pattern,
                min_length,
                max_length,
            }) => {
                let s = self
                    .value
                    .as_str()
                    .ok_or_else(|| "Value is not a string".to_string())?;

                if let Some(min) = min_length {
                    if s.len() < *min {
                        return Err(format!(
                            "String length {} is less than minimum {}",
                            s.len(),
                            min
                        ));
                    }
                }

                if let Some(max) = max_length {
                    if s.len() > *max {
                        return Err(format!("String length {} exceeds maximum {}", s.len(), max));
                    }
                }

                if let Some(pat) = pattern {
                    let re = regex::Regex::new(pat)
                        .map_err(|e| format!("Invalid regex pattern: {}", e))?;
                    if !re.is_match(s) {
                        return Err(format!("String does not match pattern: {}", pat));
                    }
                }

                Ok(())
            }
            Some(Validator::Integer { min, max }) => {
                let i = self
                    .value
                    .as_i64()
                    .ok_or_else(|| "Value is not an integer".to_string())?;

                if let Some(min_val) = min {
                    if i < *min_val {
                        return Err(format!("Integer {} is less than minimum {}", i, min_val));
                    }
                }

                if let Some(max_val) = max {
                    if i > *max_val {
                        return Err(format!("Integer {} exceeds maximum {}", i, max_val));
                    }
                }

                Ok(())
            }
            Some(Validator::Float { min, max }) => {
                let f = self
                    .value
                    .as_f64()
                    .ok_or_else(|| "Value is not a float".to_string())?;

                if let Some(min_val) = min {
                    if f < *min_val {
                        return Err(format!("Float {} is less than minimum {}", f, min_val));
                    }
                }

                if let Some(max_val) = max {
                    if f > *max_val {
                        return Err(format!("Float {} exceeds maximum {}", f, max_val));
                    }
                }

                Ok(())
            }
            Some(Validator::Boolean) => {
                self.value
                    .as_bool()
                    .ok_or_else(|| "Value is not a boolean".to_string())?;
                Ok(())
            }
            Some(Validator::Email) => {
                let s = self
                    .value
                    .as_str()
                    .ok_or_else(|| "Value is not a string".to_string())?;

                // Simple email validation
                if !s.contains('@') || !s.contains('.') {
                    return Err("Invalid email format".to_string());
                }

                Ok(())
            }
            Some(Validator::Url) => {
                let s = self
                    .value
                    .as_str()
                    .ok_or_else(|| "Value is not a string".to_string())?;

                // Simple URL validation
                if !s.starts_with("http://") && !s.starts_with("https://") {
                    return Err(
                        "Invalid URL format (must start with http:// or https://)".to_string()
                    );
                }

                Ok(())
            }
            Some(Validator::Date) | Some(Validator::DateTime) => {
                let s = self
                    .value
                    .as_str()
                    .ok_or_else(|| "Value is not a string".to_string())?;

                // Validate ISO 8601 format
                chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| format!("Invalid date/datetime format: {}", e))?;

                Ok(())
            }
            Some(Validator::Enum { allowed_values }) => {
                let s = self
                    .value
                    .as_str()
                    .ok_or_else(|| "Value is not a string".to_string())?;

                if !allowed_values.contains(&s.to_string()) {
                    return Err(format!(
                        "Value '{}' not in allowed values: {:?}",
                        s, allowed_values
                    ));
                }

                Ok(())
            }
        }
    }
}

/// Context for a conversation session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// Message history
    pub messages: Vec<Message>,
    /// Extracted context variables
    pub variables: HashMap<String, ContextVariable>,
    /// Maximum number of messages to keep in context
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
}

fn default_max_messages() -> usize {
    100
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            variables: HashMap::new(),
            max_messages: default_max_messages(),
        }
    }

    /// Create a new context with a custom max message limit
    pub fn with_max_messages(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            variables: HashMap::new(),
            max_messages,
        }
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);

        // Trim old messages if we exceed the limit
        if self.messages.len() > self.max_messages {
            let excess = self.messages.len() - self.max_messages;
            self.messages.drain(0..excess);
        }
    }

    /// Add a context variable
    pub fn add_variable(&mut self, variable: ContextVariable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    /// Get a context variable by name
    pub fn get_variable(&self, name: &str) -> Option<&ContextVariable> {
        self.variables.get(name)
    }

    /// Get all messages with a specific role
    pub fn messages_by_role(&self, role: MessageRole) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.role == role).collect()
    }

    /// Get the most recent message
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Clear all messages and variables
    pub fn clear(&mut self) {
        self.messages.clear();
        self.variables.clear();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert!(msg.metadata.is_none());
    }

    #[test]
    fn test_message_with_metadata() {
        let msg =
            Message::assistant("Response").with_metadata("confidence", serde_json::json!(0.95));
        assert!(msg.metadata.is_some());
        assert_eq!(
            msg.metadata.unwrap().get("confidence"),
            Some(&serde_json::json!(0.95))
        );
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::system("System prompt");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_context_variable_creation() {
        let msg_id = MessageId::new();
        let var = ContextVariable::new("name", serde_json::json!("Alice"), msg_id);
        assert_eq!(var.name, "name");
        assert_eq!(var.value, serde_json::json!("Alice"));
        assert_eq!(var.source_message_id, msg_id);
    }

    #[test]
    fn test_context_variable_string_validator() {
        let msg_id = MessageId::new();
        let var = ContextVariable::new("name", serde_json::json!("Alice"), msg_id).with_validator(
            Validator::String {
                pattern: None,
                min_length: Some(3),
                max_length: Some(10),
            },
        );

        assert!(var.validate().is_ok());

        let short_var = ContextVariable::new("name", serde_json::json!("Al"), msg_id)
            .with_validator(Validator::String {
                pattern: None,
                min_length: Some(3),
                max_length: Some(10),
            });

        assert!(short_var.validate().is_err());
    }

    #[test]
    fn test_context_variable_integer_validator() {
        let msg_id = MessageId::new();
        let var = ContextVariable::new("age", serde_json::json!(25), msg_id).with_validator(
            Validator::Integer {
                min: Some(0),
                max: Some(120),
            },
        );

        assert!(var.validate().is_ok());

        let invalid_var = ContextVariable::new("age", serde_json::json!(150), msg_id)
            .with_validator(Validator::Integer {
                min: Some(0),
                max: Some(120),
            });

        assert!(invalid_var.validate().is_err());
    }

    #[test]
    fn test_context_variable_enum_validator() {
        let msg_id = MessageId::new();
        let var = ContextVariable::new("color", serde_json::json!("red"), msg_id).with_validator(
            Validator::Enum {
                allowed_values: vec!["red".to_string(), "green".to_string(), "blue".to_string()],
            },
        );

        assert!(var.validate().is_ok());

        let invalid_var = ContextVariable::new("color", serde_json::json!("yellow"), msg_id)
            .with_validator(Validator::Enum {
                allowed_values: vec!["red".to_string(), "green".to_string(), "blue".to_string()],
            });

        assert!(invalid_var.validate().is_err());
    }

    #[test]
    fn test_context_creation() {
        let ctx = Context::new();
        assert_eq!(ctx.messages.len(), 0);
        assert_eq!(ctx.variables.len(), 0);
        assert_eq!(ctx.max_messages, 100);
    }

    #[test]
    fn test_context_add_message() {
        let mut ctx = Context::new();
        ctx.add_message(Message::user("Hello"));
        ctx.add_message(Message::assistant("Hi there"));

        assert_eq!(ctx.messages.len(), 2);
        assert_eq!(ctx.messages[0].content, "Hello");
        assert_eq!(ctx.messages[1].content, "Hi there");
    }

    #[test]
    fn test_context_max_messages() {
        let mut ctx = Context::with_max_messages(3);

        for i in 0..5 {
            ctx.add_message(Message::user(format!("Message {}", i)));
        }

        assert_eq!(ctx.messages.len(), 3);
        assert_eq!(ctx.messages[0].content, "Message 2");
        assert_eq!(ctx.messages[2].content, "Message 4");
    }

    #[test]
    fn test_context_add_variable() {
        let mut ctx = Context::new();
        let msg_id = MessageId::new();
        let var = ContextVariable::new("name", serde_json::json!("Alice"), msg_id);

        ctx.add_variable(var.clone());

        assert_eq!(ctx.variables.len(), 1);
        assert_eq!(ctx.get_variable("name"), Some(&var));
    }

    #[test]
    fn test_context_messages_by_role() {
        let mut ctx = Context::new();
        ctx.add_message(Message::user("User message 1"));
        ctx.add_message(Message::assistant("Assistant message 1"));
        ctx.add_message(Message::user("User message 2"));

        let user_messages = ctx.messages_by_role(MessageRole::User);
        assert_eq!(user_messages.len(), 2);

        let assistant_messages = ctx.messages_by_role(MessageRole::Assistant);
        assert_eq!(assistant_messages.len(), 1);
    }

    #[test]
    fn test_context_last_message() {
        let mut ctx = Context::new();
        assert!(ctx.last_message().is_none());

        ctx.add_message(Message::user("First"));
        ctx.add_message(Message::user("Second"));

        assert_eq!(ctx.last_message().unwrap().content, "Second");
    }

    #[test]
    fn test_context_clear() {
        let mut ctx = Context::new();
        ctx.add_message(Message::user("Hello"));
        ctx.add_variable(ContextVariable::new(
            "name",
            serde_json::json!("Alice"),
            MessageId::new(),
        ));

        ctx.clear();

        assert_eq!(ctx.messages.len(), 0);
        assert_eq!(ctx.variables.len(), 0);
    }

    #[test]
    fn test_context_serialization() {
        let mut ctx = Context::new();
        ctx.add_message(Message::user("Hello"));
        ctx.add_variable(ContextVariable::new(
            "name",
            serde_json::json!("Alice"),
            MessageId::new(),
        ));

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: Context = serde_json::from_str(&json).unwrap();

        assert_eq!(ctx.messages.len(), deserialized.messages.len());
        assert_eq!(ctx.variables.len(), deserialized.variables.len());
    }
}
