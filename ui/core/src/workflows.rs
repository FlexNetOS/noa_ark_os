use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Describes a stage within a workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStage {
    pub id: String,
    pub label: String,
    pub description: String,
}

impl WorkflowStage {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
        }
    }
}

/// Primary workflow entity consumed by the unified shell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workflow {
    pub id: String,
    pub label: String,
    pub description: String,
    pub stages: Vec<WorkflowStage>,
}

impl Workflow {
    pub fn builder(id: impl Into<String>) -> WorkflowBuilder {
        WorkflowBuilder::new(id)
    }
}

/// Builder used to describe workflow pipelines fluently.
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            workflow: Workflow {
                id: id.into(),
                label: String::new(),
                description: String::new(),
                stages: vec![],
            },
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.workflow.label = label.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.workflow.description = description.into();
        self
    }

    pub fn with_stage(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        self.workflow.stages.push(WorkflowStage::new(
            id.clone(),
            id.clone(),
            format!("Automatically generated stage {}", id),
        ));
        self
    }

    pub fn add_stage(mut self, stage: WorkflowStage) -> Self {
        self.workflow.stages.push(stage);
        self
    }

    pub fn finish(self) -> Workflow {
        self.workflow
    }
}

/// Runtime execution of a workflow triggered from the shell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowRun {
    pub workflow: Workflow,
    pub payload: serde_json::Value,
    pub triggered_at: u64,
}

impl WorkflowRun {
    pub fn new(workflow: Workflow, payload: serde_json::Value) -> Self {
        Self {
            workflow,
            payload,
            triggered_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Shared catalog used across modules and the chat workspace.
#[derive(Clone, Default)]
pub struct WorkflowCatalog {
    inner: Arc<RwLock<HashMap<String, Workflow>>>,
}

impl WorkflowCatalog {
    pub fn register(&self, workflow: Workflow) {
        self.inner
            .write()
            .expect("catalog poisoned")
            .insert(workflow.id.clone(), workflow);
    }

    pub fn merge(&self, other: &WorkflowCatalog) {
        let mut guard = self.inner.write().expect("catalog poisoned");
        for (id, workflow) in other.inner.read().expect("catalog poisoned").iter() {
            guard.insert(id.clone(), workflow.clone());
        }
    }

    pub fn get(&self, id: &str) -> Option<Workflow> {
        self.inner
            .read()
            .expect("catalog poisoned")
            .get(id)
            .cloned()
    }

    pub fn feature_explanation(&self, id: &str) -> String {
        if let Some(workflow) = self.get(id) {
            format!(
                "Workflow '{}' orchestrates {} stages for rapid delivery.",
                workflow.label,
                workflow.stages.len()
            )
        } else {
            format!(
                "Feature '{}' is not yet registered in the workflow catalog.",
                id
            )
        }
    }

    pub fn list(&self) -> Vec<Workflow> {
        self.inner
            .read()
            .expect("catalog poisoned")
            .values()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn workflow_builder_produces_expected_structure() {
        let workflow = Workflow::builder("deploy")
            .label("Deploy")
            .description("Deploy to production")
            .with_stage("plan")
            .with_stage("apply")
            .finish();

        assert_eq!(workflow.stages.len(), 2);
        assert_eq!(workflow.label, "Deploy");
    }

    #[test]
    fn workflow_catalog_can_retrieve_entries() {
        let catalog = WorkflowCatalog::default();
        catalog.register(Workflow::builder("test").with_stage("unit").finish());
        assert!(catalog.get("test").is_some());
    }

    #[test]
    fn workflow_run_embeds_payload() {
        let workflow = Workflow::builder("sandbox").with_stage("validate").finish();
        let run = WorkflowRun::new(workflow.clone(), json!({"feature": "search"}));
        assert_eq!(run.workflow.id, workflow.id);
    }
}
