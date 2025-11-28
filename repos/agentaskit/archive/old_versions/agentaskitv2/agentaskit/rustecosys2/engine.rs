use crate::production_config::ProductionConfig;
use crate::progress::ProgressTracker;
use crate::scheduler::{ExecutionPlan, SchedulerConfig, TaskScheduler};
use crate::task::Task;

/// Engine-level configuration derived from global settings.
#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    pub scheduler: SchedulerConfig,
    pub max_workers: usize,
}

impl EngineConfig {
    pub fn from_production(config: &ProductionConfig) -> Self {
        let optimization = config.optimization_features();
        let max_workers = if optimization.auto_scaling {
            32
        } else if optimization.distributed_processing {
            16
        } else if optimization.multi_device {
            8
        } else {
            4
        };

        let mut scheduler = SchedulerConfig::default();
        scheduler.max_concurrency = max_workers;

        Self {
            scheduler,
            max_workers,
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            scheduler: SchedulerConfig::default(),
            max_workers: SchedulerConfig::default().max_concurrency,
        }
    }
}

/// Coordinates scheduling and progress reporting for the orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestrationEngine {
    production: ProductionConfig,
    scheduler: TaskScheduler,
    tracker: ProgressTracker,
    config: EngineConfig,
}

impl OrchestrationEngine {
    pub fn new(production: ProductionConfig, config: EngineConfig) -> Self {
        let scheduler = TaskScheduler::new(config.scheduler);
        Self {
            production,
            scheduler,
            tracker: ProgressTracker::new(),
            config,
        }
    }

    pub fn from_production(production: ProductionConfig) -> Self {
        let config = EngineConfig::from_production(&production);
        Self::new(production, config)
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub fn production(&self) -> &ProductionConfig {
        &self.production
    }

    pub fn tracker(&self) -> &ProgressTracker {
        &self.tracker
    }

    pub fn tracker_mut(&mut self) -> &mut ProgressTracker {
        &mut self.tracker
    }

    pub fn schedule(&self, tasks: &[Task]) -> ExecutionPlan {
        self.scheduler.build_plan(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_core::config::NoaConfig;
    use std::io::Write;
    use tempfile::NamedTempFile;

    use crate::task::{
        ExecutionContext, ResourceRequirements, Task, TaskDefinition, TaskPriority, TaskStatus,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn write_config(contents: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(contents.as_bytes()).expect("write config");
        file.flush().expect("flush");
        file
    }

    fn make_task(name: &str, priority: TaskPriority) -> Task {
        Task {
            id: Uuid::new_v4(),
            definition: TaskDefinition {
                name: name.to_string(),
                description: String::new(),
                category: "test".into(),
                tags: vec![],
                priority,
                resources: ResourceRequirements::default(),
                dependencies: vec![],
                context: ExecutionContext::Shell {
                    command: "echo".into(),
                    args: vec![name.to_string()],
                    working_dir: None,
                    env: Default::default(),
                },
                metadata: Default::default(),
                max_retries: 0,
                parallel_safe: true,
            },
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            retry_count: 0,
            results: None,
            parent_task: None,
            child_tasks: vec![],
        }
    }

    #[test]
    fn optimization_features_shape_engine_config() {
        let file = write_config(
            r#"
			[feature_toggles.optimization_auto_scaling]
			enabled = true
			"#,
        );

        let noa = NoaConfig::load_from_path(file.path()).expect("load config");
        let production = ProductionConfig::from_noa_config(noa);
        let engine = OrchestrationEngine::from_production(production);

        assert_eq!(engine.config().max_workers, 32);
        assert_eq!(engine.config().scheduler.max_concurrency, 32);
    }

    #[test]
    fn scheduler_limits_tasks_by_concurrency() {
        let file = write_config(
            r#"
			[feature_toggles.optimization_multi_device]
			enabled = true
			"#,
        );

        let noa = NoaConfig::load_from_path(file.path()).expect("load config");
        let production = ProductionConfig::from_noa_config(noa);
        let engine = OrchestrationEngine::from_production(production);

        let tasks = vec![
            make_task("a", TaskPriority::Low),
            make_task("b", TaskPriority::Normal),
            make_task("c", TaskPriority::High),
            make_task("d", TaskPriority::Critical),
            make_task("e", TaskPriority::Normal),
        ];

        let plan = engine.schedule(&tasks);
        assert!(plan.ordered_tasks().len() <= engine.config().max_workers);
    }
}
