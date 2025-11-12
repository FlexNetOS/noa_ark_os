use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::engine::Engine;
use crate::graph::{CRCGraph, GraphNode, NodeKind};
use crate::ir::Lane;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Failed,
    Succeeded,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct JobPlan {
    pub name: String,
    pub graph: CRCGraph,
    pub checkpoint: PathBuf,
    pub retries: u32,
    pub backoff_secs: u64,
}

impl JobPlan {
    pub fn simple(name: impl Into<String>, checkpoint: impl Into<PathBuf>) -> Self {
        let mut graph = CRCGraph::new();
        let analyze = graph.add_node(GraphNode::new("analyze", NodeKind::Analyze, Lane::Fast));
        let decide = graph.add_node(GraphNode::new("decide", NodeKind::Decide, Lane::Fast));
        let persist = graph.add_node(GraphNode::new("persist", NodeKind::Persist, Lane::Deep));
        let _ = graph.add_edge(&analyze, &decide);
        let _ = graph.add_edge(&decide, &persist);
        Self {
            name: name.into(),
            graph,
            checkpoint: checkpoint.into(),
            retries: 2,
            backoff_secs: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub plan: JobPlan,
    pub state: JobState,
    pub attempts: u32,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct Orchestrator {
    queue: VecDeque<JobRecord>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, plan: JobPlan) {
        self.queue.push_back(JobRecord {
            plan,
            state: JobState::Queued,
            attempts: 0,
            last_error: None,
        });
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub async fn run_next(&mut self) -> Result<Option<JobRecord>> {
        if let Some(mut job) = self.queue.pop_front() {
            job.state = JobState::Running;
            let engine = Engine::new(job.plan.graph.clone());
            loop {
                match engine.run(&job.plan.checkpoint).await {
                    Ok(summary) => {
                        job.state = JobState::Succeeded;
                        job.last_error = None;
                        job.plan.graph = job.plan.graph.clone();
                        return Ok(Some(job));
                    }
                    Err(err) => {
                        job.attempts += 1;
                        job.last_error = Some(err.to_string());
                        if job.attempts > job.plan.retries {
                            job.state = JobState::Failed;
                            return Ok(Some(job));
                        }
                        sleep(Duration::from_secs(job.plan.backoff_secs)).await;
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn orchestrator_completes_job() {
        let mut orchestrator = Orchestrator::new();
        orchestrator.enqueue(JobPlan::simple("demo", "./out/ckpt/test"));
        let result = orchestrator.run_next().await.unwrap();
        assert!(matches!(result.unwrap().state, JobState::Succeeded));
    }
}
