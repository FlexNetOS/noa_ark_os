use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ark_os_core::*;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn bench_agent_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("agent_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let security_manager = SecurityManager::new().await.unwrap();
            let agent_manager = AgentManager::new(black_box(10), &security_manager).await.unwrap();
            black_box(agent_manager)
        })
    });
}

fn bench_agent_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("agent_scaling");
    
    for agent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("agents", agent_count), agent_count, |b, &agent_count| {
            b.to_async(&rt).iter(|| async move {
                let security_manager = SecurityManager::new().await.unwrap();
                let agent_manager = AgentManager::new(black_box(agent_count), &security_manager).await.unwrap();
                agent_manager.start().await.unwrap();
                black_box(agent_manager)
            })
        });
    }
    
    group.finish();
}

fn bench_task_submission(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (orchestrator, _security_manager) = rt.block_on(async {
        let security_manager = SecurityManager::new().await.unwrap();
        let message_broker = MessageBroker::new().await.unwrap();
        let metrics_collector = MetricsCollector::new().await.unwrap();
        let agent_manager = AgentManager::new(100, &security_manager).await.unwrap();
        
        let orchestrator = OrchestratorEngine::new(
            agent_manager,
            message_broker,
            metrics_collector,
        ).await.unwrap();
        
        orchestrator.start("supervised".to_string()).await.unwrap();
        (orchestrator, security_manager)
    });
    
    c.bench_function("task_submission", |b| {
        b.to_async(&rt).iter(|| async {
            let task = create_task(
                "benchmark-task".to_string(),
                "Performance benchmark task".to_string(),
                TaskType::Processing,
                Priority::Normal,
                vec!["task_execution".to_string()],
                serde_json::json!({"benchmark": true}),
            );
            
            black_box(orchestrator.submit_task(task).await.unwrap())
        })
    });
}

fn bench_security_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let security_manager = rt.block_on(async {
        SecurityManager::new().await.unwrap()
    });
    
    c.bench_function("token_issuance", |b| {
        b.to_async(&rt).iter(|| async {
            let agent_id = Uuid::new_v4();
            let capabilities = vec![
                Capability::TaskExecution,
                Capability::DataAccess,
            ];
            
            black_box(security_manager.issue_token(agent_id, capabilities).await.unwrap())
        })
    });
    
    c.bench_function("access_check", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let agent_id = Uuid::new_v4();
                    let capabilities = vec![Capability::TaskExecution];
                    let token = security_manager.issue_token(agent_id, capabilities).await.unwrap();
                    (agent_id, token)
                })
            },
            |(agent_id, _token)| async move {
                black_box(security_manager.check_access(
                    agent_id,
                    "test-resource",
                    &Capability::TaskExecution,
                ).await.unwrap())
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_agent_creation,
    bench_agent_scaling,
    bench_task_submission,
    bench_security_operations
);
criterion_main!(benches);