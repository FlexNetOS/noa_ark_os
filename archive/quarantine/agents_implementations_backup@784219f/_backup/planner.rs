use async_trait::async_trait;
use config::AppConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Plan {
    pub steps: Vec<String>,
}

#[async_trait]
pub trait Planner: Send + Sync {
    async fn plan(&self, goal: &str, input: &Value) -> anyhow::Result<Plan>;
}

/// Default: heuristic steps based on goal
pub struct HeuristicPlanner;

#[async_trait]
impl Planner for HeuristicPlanner {
    async fn plan(&self, goal: &str, input: &Value) -> anyhow::Result<Plan> {
        let steps = match goal.to_ascii_lowercase().as_str() {
            "digest" => vec![
                "extract text content".into(),
                "retrieve relevant context".into(),
                "analyze with AI inference".into(),
                "compose structured summary".into(),
            ],
            "search" => vec![
                "parse search query".into(),
                "generate query embedding".into(),
                "search vector database".into(),
                "rank and filter results".into(),
            ],
            "infer" | "generate" => vec![
                "prepare prompt".into(),
                "call inference service".into(),
                "post-process output".into(),
                "return generated text".into(),
            ],
            goal if goal.contains("rag") || goal.contains("augment") => vec![
                "understand user query".into(),
                "retrieve relevant documents".into(),
                "augment prompt with context".into(),
                "generate contextual response".into(),
            ],
            _ => vec![
                "analyze request".into(),
                "determine approach".into(),
                "execute workflow".into(),
                "compose response".into(),
            ],
        };

        // Add input-specific context to planning
        if let Some(text) = input.get("text").and_then(|t| t.as_str()) {
            if text.len() > 1000 {
                let mut enhanced_steps = steps;
                enhanced_steps.insert(1, "chunk large text".into());
                return Ok(Plan {
                    steps: enhanced_steps,
                });
            }
        }

        Ok(Plan { steps })
    }
}

pub async fn execute_plan(
    _http: &Client,
    goal: &str,
    input: &Value,
    _cfg: &Arc<AppConfig>,
) -> anyhow::Result<Value> {
    let planner = HeuristicPlanner;
    let plan = planner.plan(goal, input).await?;

    let summary = format!(
        "Executed goal '{}' with {} planned steps.",
        goal,
        plan.steps.len()
    );

    Ok(json!({
        "plan": plan.steps,
        "summary": summary,
        "input": input,
    }))
}

/// Rig-powered planner (enabled with --features rig)
#[cfg(feature = "rig")]
pub struct RigPlanner;

#[cfg(feature = "rig")]
#[async_trait]
impl Planner for RigPlanner {
    async fn plan(&self, goal: &str, input: &Value) -> anyhow::Result<Plan> {
        // TODO: Replace with real Rig calls once dependency is pinned
        // Example: let plan = rig_core::planner::create_plan(goal, input).await?;

        tracing::info!("Using Rig planner for goal: {}", goal);

        let mut steps = vec![
            "rig: analyze request context".into(),
            "rig: plan multi-step workflow".into(),
            "rig: coordinate service calls".into(),
            "rig: synthesize final result".into(),
        ];

        // Rig-specific goal handling
        match goal {
            "digest" => {
                steps.insert(1, "rig: semantic analysis".into());
                steps.insert(3, "rig: extract key insights".into());
            }
            "search" => {
                steps.insert(1, "rig: query optimization".into());
                steps.insert(3, "rig: relevance scoring".into());
            }
            "infer" => {
                steps.insert(1, "rig: prompt engineering".into());
                steps.insert(3, "rig: output validation".into());
            }
            _ => {
                steps[0] = format!("rig: custom workflow for {}", goal);
            }
        }

        Ok(Plan { steps })
    }
}
