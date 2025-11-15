use std::env;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use noa_agents::implementations::documentation::{
    AgentApprovalRequirement, DocumentationAgent, DocumentationPipelineOutput,
    ServiceDocumentation, SopDocumentation, WorkflowDocumentation,
};
use uuid::Uuid;

use noa_core::kernel::{self, AiControlLoop};

fn main() {
    if let Err(err) = run() {
        eprintln!("documentation_sync failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut pipeline_path: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pipeline-output" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--pipeline-output requires a value")?;
                pipeline_path = Some(value);
            }
            other => return Err(format!("Unknown argument: {other}").into()),
        }
    }

    let pipeline_path = pipeline_path.ok_or("--pipeline-output is required")?;
    let payload = fs::read_to_string(&pipeline_path)?;

    let directive = kernel::handle()
        .map(|handle| handle.machine_directive())
        .unwrap_or_default();

    println!(
        "[documentation_sync] machine remediation in effect (confidence {:.2}): {}",
        directive.confidence, directive.rationale
    );

    let output = DocumentationPipelineOutput::from_json_str(&payload)
        .unwrap_or_else(|_| build_output_from_diff(&payload));

    let agent = DocumentationAgent::from_workspace_root();
    agent.process_pipeline_output(&output)?;

    Ok(())
}

fn build_output_from_diff(diff_summary: &str) -> DocumentationPipelineOutput {
    let generated_at = Utc::now();
    let services = collect_services();
    let sops = collect_sops(generated_at);
    let workflows = collect_workflows();

    DocumentationPipelineOutput {
        run_id: format!("doc-sync-{}", Uuid::new_v4()),
        generated_at,
        diff_summary: diff_summary.trim().to_string(),
        approvals_required: vec![
            AgentApprovalRequirement {
                role: "doc-lead".to_string(),
                minimum_trust_score: 0.7,
                required_evidence_tags: vec!["ledger:docs".to_string()],
            },
            AgentApprovalRequirement {
                role: "release-manager".to_string(),
                minimum_trust_score: 0.8,
                required_evidence_tags: vec!["ledger:release".to_string()],
            },
        ],
        approvals_granted: Vec::new(),
        services,
        sops,
        workflows,
    }
}

fn collect_services() -> Vec<ServiceDocumentation> {
    let services_root = Path::new("services");
    if !services_root.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(services_root) {
        for entry in read_dir.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .replace('_', " ")
                        .to_title_case();
                    let path = format!("services/{}", entry.file_name().to_string_lossy());
                    entries.push(ServiceDocumentation {
                        name,
                        path,
                        status: "pending-refresh".to_string(),
                        summary: "Awaiting documentation agent sync".to_string(),
                    });
                }
            }
        }
    }

    if entries.is_empty() {
        entries.push(ServiceDocumentation {
            name: "Core Platform".to_string(),
            path: "services/core".to_string(),
            status: "pending-refresh".to_string(),
            summary: "Core platform documentation placeholder".to_string(),
        });
    }

    entries
}

fn collect_sops(now: DateTime<Utc>) -> Vec<SopDocumentation> {
    let sop_root = Path::new(".workspace/sop");
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(sop_root) {
        for entry in read_dir.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("md") {
                let name = entry
                    .file_name()
                    .to_string_lossy()
                    .replace('-', " ")
                    .replace('_', " ")
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_string();
                let path = format!(".workspace/sop/{}", entry.file_name().to_string_lossy());
                let last_reviewed = entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .map(|ts| DateTime::<Utc>::from(ts))
                    .unwrap_or(now);
                entries.push(SopDocumentation {
                    name,
                    path,
                    last_reviewed,
                    automation_status: "auto".to_string(),
                });
            }
        }
    }

    if entries.is_empty() {
        entries.push(SopDocumentation {
            name: "Documentation".to_string(),
            path: ".workspace/sop/development.md".to_string(),
            last_reviewed: now,
            automation_status: "manual".to_string(),
        });
    }

    entries
}

fn collect_workflows() -> Vec<WorkflowDocumentation> {
    let workflow_root = Path::new("workflow");
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(workflow_root) {
        for entry in read_dir.flatten() {
            if entry.path().is_dir() {
                let name = entry
                    .file_name()
                    .to_string_lossy()
                    .replace('-', " ")
                    .replace('_', " ")
                    .to_string();
                let path = format!("workflow/{}", entry.file_name().to_string_lossy());
                entries.push(WorkflowDocumentation {
                    name,
                    path,
                    owner: "operations".to_string(),
                    verification: "automated-checks".to_string(),
                });
            }
        }
    }

    if entries.is_empty() {
        entries.push(WorkflowDocumentation {
            name: "docs refresh".to_string(),
            path: "workflow/docs-refresh".to_string(),
            owner: "operations".to_string(),
            verification: "automated-checks".to_string(),
        });
    }

    entries
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for String {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl ToTitleCase for std::borrow::Cow<'_, str> {
    fn to_title_case(&self) -> String {
        self.to_string().to_title_case()
    }
}
