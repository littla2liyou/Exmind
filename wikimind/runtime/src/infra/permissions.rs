//! Permissions stub - simplified for WikiMind

use std::path::PathBuf;

/// Permission context stub
pub struct PermissionContext;

impl PermissionContext {
    pub fn new(_cwd: PathBuf) -> Self {
        Self
    }

    pub fn check_permission(&self, _permission: &str) -> bool {
        true
    }
}

/// Permission outcome
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionOutcome {
    Allow,
    Deny,
    Prompt,
}

/// Permission policy stub
pub struct PermissionPolicy;

impl PermissionPolicy {
    pub fn default_policy() -> Self {
        Self
    }

    pub fn evaluate(&self, _context: &PermissionContext, _permission: &str) -> PermissionOutcome {
        PermissionOutcome::Allow
    }
}

/// Permission prompter trait
pub trait PermissionPrompter {
    fn prompt(&self, message: &str) -> bool;
}

/// Default permission prompter implementation
pub struct DefaultPermissionPrompter;

impl PermissionPrompter for DefaultPermissionPrompter {
    fn prompt(&self, _message: &str) -> bool {
        true
    }
}
