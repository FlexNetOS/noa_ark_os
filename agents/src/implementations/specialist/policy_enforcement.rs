//! Policy Enforcement Specialist Agent (pe-ssp)
//!
//! Provides an always-on specialist that enforces AGENT.md controls across the
//! agent factory. The agent keeps track of subject-specific sub-agents so every
//! automation pathway keeps the 4-D method, Truth Gate, and smoke-test loops in
//! sync.

use crate::unified_types::*;
use crate::{AgentFactory, AgentId, Result};
use chrono::Utc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Registry manifest tracking detector to fixer routing for this agent.
pub const DETECTOR_HOOKS_MANIFEST: &str = ".workspace/registry/pe_ssp.detector_hooks.json";

/// Lightweight specification describing a policy-focused sub-agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicySubAgentSpec {
    pub code: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub responsibilities: Vec<&'static str>,
}

pub struct PolicyEnforcementAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    sub_agents: Vec<PolicySubAgentSpec>,
}

impl PolicyEnforcementAgent {
    pub fn new() -> Self {
        let timestamp = Utc::now().to_rfc3339();
        Self {
            metadata: AgentMetadata {
                id: Uuid::new_v4(),
                agent_id: "pe-ssp".to_string(),
                name: "Policy Enforcement Subject Specialist".to_string(),
                layer: AgentLayer::L4Operations,
                category: AgentCategory::Governance,
                agent_type: AgentType::Worker,
                language: AgentLanguage::Rust,
                description: "Guardrail specialist enforcing AGENT.md".to_string(),
                role: "Policy Enforcement".to_string(),
                purpose:
                    "Maintain 4-D discipline, Truth Gate artifacts, and automated smoke checks"
                        .to_string(),
                state: AgentState::Created,
                health_status: HealthStatus::Healthy,
                parent_id: Some("agent-factory-governor".to_string()),
                escalation_to: Some("cecca-governance".to_string()),
                stack: Some("policy-enforcement".to_string()),
                capabilities: vec![
                    "policy-enforcement".to_string(),
                    "4-D-audit".to_string(),
                    "truth-gate-verification".to_string(),
                    "smoke-test-loop".to_string(),
                ],
                tools: vec![
                    "claims.table.md".to_string(),
                    "evidence.ledger.json".to_string(),
                    "truth_gate.checklist.md".to_string(),
                ],
                tags: vec![
                    "policy".to_string(),
                    "enforcement".to_string(),
                    "pe-ssp".to_string(),
                ],
                inputs: vec![
                    "task-requests".to_string(),
                    "automation-gap-report".to_string(),
                    "detector-alerts".to_string(),
                ],
                outputs: vec![
                    "compliance-report".to_string(),
                    "truth-gate-artifacts".to_string(),
                    "sub-agent-directives".to_string(),
                ],
                dependencies: vec![
                    "AGENT.md".to_string(),
                    "scripts/full_stack_launch.sh".to_string(),
                    "registry/tooling.catalog.json".to_string(),
                ],
                cpu_min: "1".to_string(),
                ram_min: "1GB".to_string(),
                disk_min: "500MB".to_string(),
                autonomy_level: "orchestrated".to_string(),
                disposable: false,
                issues_identified: Vec::new(),
                repair_recommendations: Vec::new(),
                created_at: Some(timestamp.clone()),
                last_updated: Some(timestamp),
                version: Some("1.0.0".to_string()),
            },
            state: RwLock::new(AgentState::Created),
            sub_agents: Self::default_sub_agents(),
        }
    }

    /// Initialize the agent and mark it ready for orchestration.
    pub async fn initialize(&self) -> Result<()> {
        let mut writer = self.state.write().await;
        if matches!(*writer, AgentState::Created | AgentState::Initializing) {
            *writer = AgentState::Ready;
        }
        Ok(())
    }

    /// Return a clone of the metadata for registry insertion.
    pub fn metadata_snapshot(&self) -> AgentMetadata {
        self.metadata.clone()
    }

    /// Provide the manifest path used to coordinate detector-to-fixer hooks.
    pub fn detector_manifest_path() -> PathBuf {
        PathBuf::from(DETECTOR_HOOKS_MANIFEST)
    }

    /// Ensure the specialist is materialized inside the agent factory runtime.
    pub fn ensure_factory_registration(&self, factory: &AgentFactory) -> Result<AgentId> {
        if let Some(agent) = factory
            .list_agents()
            .into_iter()
            .find(|agent| agent.name == self.metadata.agent_id)
        {
            return Ok(agent.id);
        }

        let runtime_id = factory.create_agent(
            self.metadata.agent_id.clone(),
            self.metadata.agent_type.clone(),
            self.metadata.language.clone(),
            false,
        )?;
        factory.update_state(&runtime_id, AgentState::Ready)?;
        Ok(runtime_id)
    }

    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    pub fn sub_agents(&self) -> &[PolicySubAgentSpec] {
        &self.sub_agents
    }

    pub fn find_sub_agent(&self, code: &str) -> Option<&PolicySubAgentSpec> {
        self.sub_agents.iter().find(|spec| spec.code == code)
    }

    fn default_sub_agents() -> Vec<PolicySubAgentSpec> {
        vec![
            PolicySubAgentSpec {
                code: "4-D_policy-enforcement_pe-ssp01",
                name: "4-D Compliance Lead",
                description:
                    "Ensures every task records the Deconstruct→Diagnose→Develop→Deliver plan",
                responsibilities: vec![
                    "Capture 4-D checklists",
                    "Verify task inputs and constraints",
                    "Publish mitigation notes to Evidence Ledger",
                ],
            },
            PolicySubAgentSpec {
                code: "Evidence/Truth Gate_pe-ssp-01",
                name: "Truth Gate Steward",
                description:
                    "Maintains claims.table, evidence ledger, and truth gate checklist artifacts",
                responsibilities: vec![
                    "Collect smoke-test logs",
                    "Record SHA-256 hashes",
                    "Emit RESULT blocks with Pass A/B/C status",
                ],
            },
            PolicySubAgentSpec {
                code: "smoke_pe-ssp01-sub01",
                name: "Smoke Test Sentinel",
                description:
                    "Runs offline-first smoke tests + launcher health probes for each change",
                responsibilities: vec![
                    "Invoke scripts/full_stack_launch.sh --prepare-only",
                    "Report gateway + kernel readiness",
                    "Escalate failures back to policy specialist",
                ],
            },
            PolicySubAgentSpec {
                code: "self-check_pe-ssp01-sub01",
                name: "Self-Check Loop",
                description: "Keeps continuous self-check + adversarial probes running in parallel",
                responsibilities: vec![
                    "Schedule Pass A/B/C verification",
                    "Mirror detector findings into registry",
                    "Trigger follow-up disposable agents when gaps persist",
                ],
            },
        ]
    }
}

impl Default for PolicyEnforcementAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initializes_and_exposes_sub_agents() {
        let agent = PolicyEnforcementAgent::new();
        assert_eq!(agent.metadata().agent_id, "pe-ssp");
        let subs = agent.sub_agents();
        assert!(subs
            .iter()
            .any(|spec| spec.code == "4-D_policy-enforcement_pe-ssp01"));
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }

    #[tokio::test]
    async fn finds_truth_gate_sub_agent() {
        let agent = PolicyEnforcementAgent::new();
        let spec = agent
            .find_sub_agent("Evidence/Truth Gate_pe-ssp-01")
            .expect("truth gate agent present");
        assert!(spec
            .responsibilities
            .iter()
            .any(|item| item.contains("SHA-256")));
    }
}
