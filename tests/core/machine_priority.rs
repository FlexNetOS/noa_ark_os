#![cfg(test)]

use noa_agents::implementations::orchestrator::{
    AgentOrchestrator, AgentTaskType, ExecutionRoute, TaskPriority, TaskStatus,
};
use noa_core::kernel::{self, AiControlLoop};
use noa_core::metrics::{self, TelemetrySnapshot};
use tokio::runtime::Runtime;

#[test]
fn kernel_exposes_machine_directive() {
    if kernel::is_running() {
        kernel::shutdown();
    }

    let handle = kernel::init().expect("kernel should initialise");
    metrics::record(TelemetrySnapshot::now(0.55, 0.48, 6, 6, 4));

    let snapshot = handle.agent_health_snapshot();
    let directive = snapshot.directive();
    assert!(
        directive.prefer_machine(),
        "machine remediation must be preferred"
    );
    assert!(
        directive.confidence >= 0.6,
        "directive confidence should be elevated"
    );

    kernel::shutdown();
}

#[test]
fn orchestrator_defaults_to_machine_route() {
    if kernel::is_running() {
        kernel::shutdown();
    }

    kernel::init().expect("kernel should initialise");
    metrics::record(TelemetrySnapshot::now(0.72, 0.70, 12, 18, 14));

    let runtime = Runtime::new().expect("tokio runtime");
    runtime.block_on(async {
        let orchestrator = AgentOrchestrator::new();
        let task_id = orchestrator
            .submit_task(
                AgentTaskType::Monitoring,
                "Evaluate remediation stream",
                TaskPriority::Critical,
            )
            .await
            .expect("submit task");

        let task = orchestrator.get_task(task_id).await.expect("task exists");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.execution_route, ExecutionRoute::Machine);
        let result = task.result.expect("result present");
        assert!(result.success);
        assert_eq!(result.execution_route, ExecutionRoute::Machine);
    });

    kernel::shutdown();
}
