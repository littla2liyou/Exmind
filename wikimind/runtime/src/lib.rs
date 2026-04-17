//! WikiMind Runtime — Agent · Memory · Tools

// ─── Agent ────────────────────────────────────────────────────────────────────
pub mod agent;
pub use agent::{
    conversation::{ApiClient, ApiRequest, AssistantEvent, ConversationRuntime, RuntimeError, ToolError, ToolExecutor},
    task_registry::{TaskRegistry, TaskStatus},
    worker_boot::Worker,
};

// ─── Memory / Context ───────────────────────────────────────────────────────
pub mod memory;
pub use memory::{
    compact::{compact_session, estimate_session_tokens, CompactionConfig, CompactionResult},
    session::{ContentBlock, ConversationMessage, MessageRole, Session, SessionCompaction, SessionFork, SessionPromptEntry},
    summary_compression::{compress_summary, compress_summary_text, SummaryCompressionBudget},
};

// ─── Tools ───────────────────────────────────────────────────────────────────
pub mod tools;
pub use tools::{file_ops, git_context};

// ─── Infra ──────────────────────────────────────────────────────────────────
pub mod infra;
pub use infra::usage::TokenUsage;

/// Test helper: serializes parallel tests that share env (cwd, etc.)
pub fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().expect("test env lock poisoned")
}
