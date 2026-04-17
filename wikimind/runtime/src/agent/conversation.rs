//! Simplified conversation runtime for WikiMind MVP
//! This is a stub implementation - full multi-agent support to be added later

use crate::memory::session::{ConversationMessage, Session};
use std::fmt::{Display, Formatter};

/// Streamed events emitted while processing a single assistant turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantEvent {
    TextDelta(String),
    MessageStop,
}

/// Fully assembled request payload sent to the upstream model client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRequest {
    pub system_prompt: Vec<String>,
    pub messages: Vec<ConversationMessage>,
}

/// Minimal streaming API contract required by [`ConversationRuntime`].
pub trait ApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError>;
}

/// Trait implemented by tool dispatchers that execute model-requested tools.
pub trait ToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError>;
}

/// Error returned when a tool invocation fails locally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    message: String,
}

impl ToolError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ToolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ToolError {}

/// Error returned when a conversation turn cannot be completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

/// Simplified conversation runtime for MVP
pub struct ConversationRuntime {
    session: Session,
}

impl ConversationRuntime {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        let message_role = match role {
            "user" => crate::memory::session::MessageRole::User,
            "assistant" => crate::memory::session::MessageRole::Assistant,
            "system" => crate::memory::session::MessageRole::System,
            _ => crate::memory::session::MessageRole::User,
        };
        let message = ConversationMessage {
            role: message_role,
            blocks: vec![crate::memory::session::ContentBlock::Text {
                text: content.to_string(),
            }],
            usage: None,
        };
        self.session.messages.push(message);
    }

    pub fn run_turn(&mut self) -> Result<Vec<AssistantEvent>, RuntimeError> {
        // Stub: return a simple response
        Ok(vec![AssistantEvent::TextDelta("Hello from WikiMind!".to_string())])
    }

    pub fn session(&mut self) -> &mut Session {
        &mut self.session
    }
}
