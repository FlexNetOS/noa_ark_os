use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;
use noa_abi::{ExecutionPlan, ExecutionResult, AgentResult, Task, Queue};

/// Parallel execution engine that respects resource limits
pub struct ParallelExecutor {
    max_workers: usize,
}

impl ParallelExecutor {
    pub fn new(max_workers: usize) -> Self {
        Self { max_workers }
    }

    /// Execute a plan with parallel task execution
    pub async fn execute_plan<F, Fut>(
        &self,
        plan: ExecutionPlan,
        executor_fn: F,
    ) -> Result<ExecutionResult>
    where
        F: Fn(Task, Queue, serde_json::Value) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<AgentResult>> + Send + 'static,
    {
        let started = chrono::Utc::now().to_rfc3339();
        let semaphore = Arc::new(Semaphore::new(self.max_workers));
        let mut handles = Vec::new();

        // Collect all tasks with their queue context
        let mut task_queue_pairs = Vec::new();
        for queue in &plan.queues {
            for task in &queue.tasks {
                task_queue_pairs.push((task.clone(), queue.clone()));
            }
        }

        // Spawn tasks with concurrency control
        for (task, queue) in task_queue_pairs {
            let permit = semaphore.clone().acquire_owned().await?;
            let executor_fn = executor_fn.clone();
            let hooks = queue.hooks.clone().unwrap_or_default();

            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task completes
                executor_fn(task, queue, hooks).await
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    results.push(AgentResult {
                        task: "unknown".to_string(),
                        queue: "unknown".to_string(),
                        agent: "executor".to_string(),
                        status: "error".to_string(),
                        evidence: None,
                        error: Some(e.to_string()),
                        commands: None,
                    });
                }
                Err(e) => {
                    results.push(AgentResult {
                        task: "unknown".to_string(),
                        queue: "unknown".to_string(),
                        agent: "executor".to_string(),
                        status: "error".to_string(),
                        evidence: None,
                        error: Some(format!("Task panicked: {}", e)),
                        commands: None,
                    });
                }
            }
        }

        let ended = chrono::Utc::now().to_rfc3339();
        let all_ok = results.iter().all(|r| matches!(r.status.as_str(), "ok" | "noop"));

        Ok(ExecutionResult {
            status: if all_ok { "ok".to_string() } else { "error".to_string() },
            started,
            ended,
            results,
        })
    }

    /// Execute tasks sequentially (for debugging or when parallelism is not desired)
    pub async fn execute_sequential<F, Fut>(
        &self,
        plan: ExecutionPlan,
        executor_fn: F,
    ) -> Result<ExecutionResult>
    where
        F: Fn(Task, Queue, serde_json::Value) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<AgentResult>> + Send,
    {
        let started = chrono::Utc::now().to_rfc3339();
        let mut results = Vec::new();

        for queue in &plan.queues {
            for task in &queue.tasks {
                let hooks = queue.hooks.clone().unwrap_or_default();
                match executor_fn(task.clone(), queue.clone(), hooks).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        results.push(AgentResult {
                            task: task.id.clone(),
                            queue: queue.name.clone(),
                            agent: "executor".to_string(),
                            status: "error".to_string(),
                            evidence: None,
                            error: Some(e.to_string()),
                            commands: None,
                        });
                    }
                }
            }
        }

        let ended = chrono::Utc::now().to_rfc3339();
        let all_ok = results.iter().all(|r| matches!(r.status.as_str(), "ok" | "noop"));

        Ok(ExecutionResult {
            status: if all_ok { "ok".to_string() } else { "error".to_string() },
            started,
            ended,
            results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_abi::{Task, Queue};
    use serde_json::json;

    #[tokio::test]
    async fn test_sequential_execution() {
        let executor = ParallelExecutor::new(4);
        
        let plan = ExecutionPlan {
            version: 1,
            source: "test".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            queues: vec![
                Queue {
                    name: "Test Queue".to_string(),
                    tasks: vec![
                        Task {
                            id: "TASK-001".to_string(),
                            title: "Test Task 1".to_string(),
                            layer: "execution".to_string(),
                            queue: "Test Queue".to_string(),
                            metadata: json!({}),
                        },
                        Task {
                            id: "TASK-002".to_string(),
                            title: "Test Task 2".to_string(),
                            layer: "execution".to_string(),
                            queue: "Test Queue".to_string(),
                            metadata: json!({}),
                        },
                    ],
                    hooks: None,
                },
            ],
        };

        let executor_fn = |task: Task, queue: Queue, _hooks: serde_json::Value| async move {
            Ok(AgentResult {
                task: task.id,
                queue: queue.name,
                agent: "test".to_string(),
                status: "ok".to_string(),
                evidence: None,
                error: None,
                commands: None,
            })
        };

        let result = executor.execute_sequential(plan, executor_fn).await.unwrap();
        
        assert_eq!(result.status, "ok");
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].task, "TASK-001");
        assert_eq!(result.results[1].task, "TASK-002");
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let executor = ParallelExecutor::new(2);
        
        let plan = ExecutionPlan {
            version: 1,
            source: "test".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            queues: vec![
                Queue {
                    name: "Test Queue".to_string(),
                    tasks: vec![
                        Task {
                            id: "TASK-001".to_string(),
                            title: "Test Task 1".to_string(),
                            layer: "execution".to_string(),
                            queue: "Test Queue".to_string(),
                            metadata: json!({}),
                        },
                        Task {
                            id: "TASK-002".to_string(),
                            title: "Test Task 2".to_string(),
                            layer: "execution".to_string(),
                            queue: "Test Queue".to_string(),
                            metadata: json!({}),
                        },
                    ],
                    hooks: None,
                },
            ],
        };

        let executor_fn = |task: Task, queue: Queue, _hooks: serde_json::Value| async move {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            Ok(AgentResult {
                task: task.id,
                queue: queue.name,
                agent: "test".to_string(),
                status: "ok".to_string(),
                evidence: None,
                error: None,
                commands: None,
            })
        };

        let result = executor.execute_plan(plan, executor_fn).await.unwrap();
        
        assert_eq!(result.status, "ok");
        assert_eq!(result.results.len(), 2);
        
        // Results might be in different order due to parallel execution
        let task_ids: Vec<&str> = result.results.iter().map(|r| r.task.as_str()).collect();
        assert!(task_ids.contains(&"TASK-001"));
        assert!(task_ids.contains(&"TASK-002"));
    }
}
