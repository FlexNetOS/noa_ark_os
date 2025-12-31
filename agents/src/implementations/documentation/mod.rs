//! Documentation Agent
//!
//! Consumes pipeline outputs and keeps markdown artifacts, SOPs, and wiki pages
//! synchronized. The agent focuses on deterministic file generation so it can be
//! executed safely inside CI pipelines.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

fn pushln(buffer: &mut String, args: std::fmt::Arguments<'_>) -> Result<()> {
    buffer
        .write_fmt(args)
        .map_err(|err| Error::AgentError(err.to_string()))?;
    buffer.push('\n');
    Ok(())
}

fn push_blank(buffer: &mut String) -> Result<()> {
    buffer.push('\n');
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentApprovalRequirement {
    pub role: String,
    pub minimum_trust_score: f32,
    #[serde(default)]
    pub required_evidence_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentApprovalRecord {
    pub role: String,
    pub agent_id: String,
    pub trust_score: f32,
    #[serde(default)]
    pub evidence_tags: Vec<String>,
    #[serde(default)]
    pub evidence_references: Vec<String>,
    pub recorded_at: u64,
}
/// Structured output emitted by the documentation pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentationPipelineOutput {
    pub run_id: String,
    pub generated_at: DateTime<Utc>,
    pub diff_summary: String,
    pub approvals_required: Vec<AgentApprovalRequirement>,
    pub approvals_granted: Vec<AgentApprovalRecord>,
    pub services: Vec<ServiceDocumentation>,
    pub sops: Vec<SopDocumentation>,
    pub workflows: Vec<WorkflowDocumentation>,
}

impl DocumentationPipelineOutput {
    /// Parse output from a JSON payload produced by the pipeline.
    pub fn from_json_str(payload: &str) -> Result<Self> {
        serde_json::from_str(payload).map_err(|err| Error::ParseError(err.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceDocumentation {
    pub name: String,
    pub path: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SopDocumentation {
    pub name: String,
    pub path: String,
    pub last_reviewed: DateTime<Utc>,
    pub automation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowDocumentation {
    pub name: String,
    pub path: String,
    pub owner: String,
    pub verification: String,
}

/// Agent responsible for syncing documentation artifacts.
#[derive(Debug, Clone)]
pub struct DocumentationAgent {
    root: PathBuf,
}

impl DocumentationAgent {
    /// Create a new agent anchored at the provided workspace root.
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Convenience helper that assumes the current working directory is the workspace root.
    pub fn from_workspace_root() -> Self {
        Self::new(PathBuf::from("."))
    }

    /// Process documentation pipeline output and persist artifacts.
    pub fn process_pipeline_output(&self, output: &DocumentationPipelineOutput) -> Result<()> {
        self.write_docs_markdown(output)?;
        self.write_sop_status(output)?;
        self.write_wiki_site(output)?;
        self.write_runbook(output)?;
        Ok(())
    }

    fn write_docs_markdown(&self, output: &DocumentationPipelineOutput) -> Result<()> {
        let doc_dir = self.root.join("docs").join("documentation");
        fs::create_dir_all(&doc_dir)?;
        let mut body = String::new();

        pushln(&mut body, format_args!("# Documentation Sync Report"))?;
        push_blank(&mut body)?;
        pushln(&mut body, format_args!("- Run ID: `{}`", output.run_id))?;
        pushln(
            &mut body,
            format_args!("- Generated: {}", output.generated_at.to_rfc3339()),
        )?;
        pushln(
            &mut body,
            format_args!("- Diff Summary: {}", output.diff_summary),
        )?;
        let required_summary = if output.approvals_required.is_empty() {
            "none".to_string()
        } else {
            output
                .approvals_required
                .iter()
                .map(|req| {
                    let evidence = if req.required_evidence_tags.is_empty() {
                        "none".to_string()
                    } else {
                        req.required_evidence_tags.join(", ")
                    };
                    format!(
                        "{} (trust ≥ {:.2}, evidence: {})",
                        req.role, req.minimum_trust_score, evidence
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let granted_summary = if output.approvals_granted.is_empty() {
            "none".to_string()
        } else {
            output
                .approvals_granted
                .iter()
                .map(|grant| {
                    let references = if grant.evidence_references.is_empty() {
                        "none".to_string()
                    } else {
                        grant.evidence_references.join(", ")
                    };
                    format!(
                        "{}@{} (trust {:.2}, evidence: {})",
                        grant.role, grant.agent_id, grant.trust_score, references
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        pushln(
            &mut body,
            format_args!("- Approvals Required: {}", required_summary),
        )?;
        pushln(
            &mut body,
            format_args!("- Approvals Granted: {}", granted_summary),
        )?;
        push_blank(&mut body)?;

        pushln(&mut body, format_args!("## Services"))?;
        pushln(
            &mut body,
            format_args!("| Service | Path | Status | Summary |"),
        )?;
        pushln(&mut body, format_args!("| --- | --- | --- | --- |"))?;
        for svc in &output.services {
            pushln(
                &mut body,
                format_args!(
                    "| {} | `{}` | {} | {} |",
                    svc.name, svc.path, svc.status, svc.summary
                ),
            )?;
        }

        push_blank(&mut body)?;
        pushln(&mut body, format_args!("## Workflows"))?;
        pushln(
            &mut body,
            format_args!("| Workflow | Path | Owner | Verification |"),
        )?;
        pushln(&mut body, format_args!("| --- | --- | --- | --- |"))?;
        for flow in &output.workflows {
            pushln(
                &mut body,
                format_args!(
                    "| {} | `{}` | {} | {} |",
                    flow.name, flow.path, flow.owner, flow.verification
                ),
            )?;
        }

        fs::write(doc_dir.join("sync-report.md"), body)?;
        Ok(())
    }

    fn write_sop_status(&self, output: &DocumentationPipelineOutput) -> Result<()> {
        let sop_dir = self.root.join(".workspace").join("sop");
        fs::create_dir_all(&sop_dir)?;

        let mut body = String::new();
        pushln(&mut body, format_args!("# SOP Automation Status"))?;
        pushln(
            &mut body,
            format_args!(
                "Generated by documentation agent at {}",
                output.generated_at.to_rfc3339()
            ),
        )?;
        push_blank(&mut body)?;
        pushln(
            &mut body,
            format_args!("| SOP | Path | Last Reviewed | Automation |"),
        )?;
        pushln(&mut body, format_args!("| --- | --- | --- | --- |"))?;
        for sop in &output.sops {
            pushln(
                &mut body,
                format_args!(
                    "| {} | `{}` | {} | {} |",
                    sop.name,
                    sop.path,
                    sop.last_reviewed.to_rfc3339(),
                    sop.automation_status
                ),
            )?;
        }

        fs::write(sop_dir.join("automation-status.md"), body)?;
        Ok(())
    }

    fn write_wiki_site(&self, output: &DocumentationPipelineOutput) -> Result<()> {
        let wiki_dir = self.root.join("docs").join("wiki");
        fs::create_dir_all(&wiki_dir)?;

        // Index page
        let mut index = String::new();
        pushln(&mut index, format_args!("# NOA ARK OS Documentation Wiki"))?;
        pushln(
            &mut index,
            format_args!("Last generated: {}", output.generated_at.to_rfc3339()),
        )?;
        push_blank(&mut index)?;
        pushln(&mut index, format_args!("- [Services](services.md)"))?;
        pushln(&mut index, format_args!("- [Workflows](workflows.md)"))?;
        pushln(
            &mut index,
            format_args!("- [Standard Operating Procedures](sops.md)"),
        )?;
        pushln(&mut index, format_args!("- [Runbook](../runbook/index.md)"))?;
        fs::write(wiki_dir.join("index.md"), index)?;

        // Services page
        let mut services = String::new();
        pushln(&mut services, format_args!("# Services Documentation"))?;
        for svc in &output.services {
            push_blank(&mut services)?;
            pushln(&mut services, format_args!("## {}", svc.name))?;
            pushln(&mut services, format_args!("- Path: `{}`", svc.path))?;
            pushln(&mut services, format_args!("- Status: {}", svc.status))?;
            pushln(&mut services, format_args!("- Summary: {}", svc.summary))?;
        }
        fs::write(wiki_dir.join("services.md"), services)?;

        // Workflows page
        let mut workflows = String::new();
        pushln(&mut workflows, format_args!("# Workflow Catalogue"))?;
        for flow in &output.workflows {
            push_blank(&mut workflows)?;
            pushln(&mut workflows, format_args!("## {}", flow.name))?;
            pushln(&mut workflows, format_args!("- Path: `{}`", flow.path))?;
            pushln(&mut workflows, format_args!("- Owner: {}", flow.owner))?;
            pushln(
                &mut workflows,
                format_args!("- Verification: {}", flow.verification),
            )?;
        }
        fs::write(wiki_dir.join("workflows.md"), workflows)?;

        // SOP page
        let mut sops = String::new();
        pushln(&mut sops, format_args!("# SOP Directory"))?;
        for sop in &output.sops {
            push_blank(&mut sops)?;
            pushln(&mut sops, format_args!("## {}", sop.name))?;
            pushln(&mut sops, format_args!("- Path: `{}`", sop.path))?;
            pushln(
                &mut sops,
                format_args!("- Last Reviewed: {}", sop.last_reviewed.to_rfc3339()),
            )?;
            pushln(
                &mut sops,
                format_args!("- Automation: {}", sop.automation_status),
            )?;
        }
        fs::write(wiki_dir.join("sops.md"), sops)?;

        Ok(())
    }

    fn write_runbook(&self, output: &DocumentationPipelineOutput) -> Result<()> {
        let runbook_dir = self.root.join("docs").join("runbook");
        fs::create_dir_all(&runbook_dir)?;

        let mut index = String::new();
        pushln(&mut index, format_args!("# NOA ARK OS Runbook"))?;
        pushln(
            &mut index,
            format_args!("Generated: {}", output.generated_at.to_rfc3339()),
        )?;
        push_blank(&mut index)?;
        pushln(
            &mut index,
            format_args!(
                "This runbook is compiled from the current SOP library and is intended to be executable during operations handoffs."
            )
        )?;
        push_blank(&mut index)?;
        pushln(&mut index, format_args!("## Verification Matrix"))?;
        pushln(
            &mut index,
            format_args!("| SOP | Automation | Verification |"),
        )?;
        pushln(&mut index, format_args!("| --- | --- | --- |"))?;
        for sop in &output.sops {
            let verification = output
                .workflows
                .iter()
                .find(|w| w.name.eq_ignore_ascii_case(&sop.name))
                .map(|w| w.verification.clone())
                .unwrap_or_else(|| "Manual checklist".to_string());
            pushln(
                &mut index,
                format_args!(
                    "| {} | {} | {} |",
                    sop.name, sop.automation_status, verification
                ),
            )?;
        }

        push_blank(&mut index)?;
        pushln(&mut index, format_args!("## Operational Playbooks"))?;
        for sop in &output.sops {
            push_blank(&mut index)?;
            pushln(&mut index, format_args!("### {}", sop.name))?;
            pushln(&mut index, format_args!("- Source: `{}`", sop.path))?;
            pushln(
                &mut index,
                format_args!("- Last Reviewed: {}", sop.last_reviewed.to_rfc3339()),
            )?;
            pushln(
                &mut index,
                format_args!("- Automation: {}", sop.automation_status),
            )?;
            pushln(
                &mut index,
                format_args!(
                    "- Verification Workflow: {}",
                    output
                        .workflows
                        .iter()
                        .find(|w| w.name.eq_ignore_ascii_case(&sop.name))
                        .map(|w| w.path.clone())
                        .unwrap_or_else(|| "workflow/docs-refresh".to_string())
                ),
            )?;
        }

        fs::write(runbook_dir.join("index.md"), index)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn build_output() -> DocumentationPipelineOutput {
        DocumentationPipelineOutput {
            run_id: "run_123".to_string(),
            generated_at: Utc::now(),
            diff_summary: "Updated services/api and SOP library".to_string(),
            approvals_required: vec![AgentApprovalRequirement {
                role: "doc-lead".to_string(),
                minimum_trust_score: 0.7,
                required_evidence_tags: vec!["ledger:docs".to_string()],
            }],
            approvals_granted: vec![AgentApprovalRecord {
                role: "doc-lead".to_string(),
                agent_id: "agent-docs".to_string(),
                trust_score: 0.82,
                evidence_tags: vec!["ledger:docs".to_string()],
                evidence_references: vec!["ledger://docs/123".to_string()],
                recorded_at: Utc::now().timestamp() as u64,
            }],
            services: vec![ServiceDocumentation {
                name: "API Gateway".to_string(),
                path: "services/api".to_string(),
                status: "healthy".to_string(),
                summary: "Exposes REST APIs".to_string(),
            }],
            sops: vec![SopDocumentation {
                name: "Release Management".to_string(),
                path: ".workspace/sop/release.md".to_string(),
                last_reviewed: Utc::now(),
                automation_status: "auto".to_string(),
            }],
            workflows: vec![WorkflowDocumentation {
                name: "Release Management".to_string(),
                path: "workflow/merge".to_string(),
                owner: "operations".to_string(),
                verification: "ci:doc-refresh".to_string(),
            }],
        }
    }

    #[test]
    fn writes_expected_artifacts() {
        let tmp_dir = std::env::temp_dir().join(format!("docs-agent-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).unwrap();

        let agent = DocumentationAgent::new(&tmp_dir);
        let output = build_output();
        agent.process_pipeline_output(&output).unwrap();

        let wiki_index = fs::read_to_string(tmp_dir.join("docs/wiki/index.md")).unwrap();
        assert!(wiki_index.contains("NOA ARK OS Documentation Wiki"));

        let runbook = fs::read_to_string(tmp_dir.join("docs/runbook/index.md")).unwrap();
        assert!(runbook.contains("Verification Matrix"));

        let sop_status =
            fs::read_to_string(tmp_dir.join(".workspace/sop/automation-status.md")).unwrap();
        assert!(sop_status.contains("Release Management"));
    }
}
