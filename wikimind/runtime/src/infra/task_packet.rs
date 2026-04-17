//! Task packet stub - simplified for WikiMind

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskPacket {
    pub objective: String,
    pub scope: String,
    pub repo: String,
    pub branch_policy: String,
    pub acceptance_tests: Vec<String>,
    pub commit_policy: String,
    pub reporting_contract: String,
    pub escalation_policy: String,
}

#[derive(Debug, Clone)]
pub struct TaskPacketValidationError {
    pub message: String,
}

/// Wrapper type for validated packet
pub struct ValidatedPacket(TaskPacket);

impl ValidatedPacket {
    pub fn into_inner(self) -> TaskPacket {
        self.0
    }
}

pub fn validate_packet(packet: &TaskPacket) -> Result<ValidatedPacket, TaskPacketValidationError> {
    Ok(ValidatedPacket(TaskPacket {
        objective: packet.objective.clone(),
        scope: packet.scope.clone(),
        repo: packet.repo.clone(),
        branch_policy: packet.branch_policy.clone(),
        acceptance_tests: packet.acceptance_tests.clone(),
        commit_policy: packet.commit_policy.clone(),
        reporting_contract: packet.reporting_contract.clone(),
        escalation_policy: packet.escalation_policy.clone(),
    }))
}
