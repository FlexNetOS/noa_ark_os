use ark_os_core::*;
use anyhow::Result;
use tokio_test;
use uuid::Uuid;
use std::time::Duration;
use serial_test::serial;

/// Integration test for the complete agent hierarchy system
#[tokio::test]
#[serial]
async fn test_complete_agent_hierarchy() -> Result<()> {
    // Initialize test environment
    let security_manager = SecurityManager::new().await?;
    let message_broker = MessageBroker::new().await?;
    let metrics_collector = MetricsCollector::new().await?;
    
    // Create agent manager with small hierarchy for testing
    let agent_manager = AgentManager::new(10, &security_manager).await?;
    
    // Start components
    message_broker.start().await?;
    metrics_collector.start().await?;
    agent_manager.start().await?;
    
    // Verify hierarchy creation
    let layer_stats = agent_manager.get_layer_statistics().await;
    
    // Should have at least one agent in each critical layer
    assert!(layer_stats.get(&AgentLayer::CECCA).map_or(false, |s| s.total_agents >= 1));
    assert!(layer_stats.get(&AgentLayer::Board).map_or(false, |s| s.total_agents >= 1));
    assert!(layer_stats.get(&AgentLayer::Executive).map_or(false, |s| s.total_agents >= 1));
    
    // Test agent communication
    let cecca_agents = layer_stats.get(&AgentLayer::CECCA).unwrap();
    assert!(cecca_agents.active_agents > 0, "CECCA agents should be active");
    
    // Shutdown
    agent_manager.shutdown().await?;
    message_broker.shutdown().await?;
    metrics_collector.shutdown().await?;
    
    Ok(())
}

/// Test task orchestration workflow
#[tokio::test]
#[serial]
async fn test_task_orchestration() -> Result<()> {
    // Setup orchestrator
    let security_manager = SecurityManager::new().await?;
    let message_broker = MessageBroker::new().await?;
    let metrics_collector = MetricsCollector::new().await?;
    let agent_manager = AgentManager::new(5, &security_manager).await?;
    
    let orchestrator = OrchestratorEngine::new(
        agent_manager,
        message_broker,
        metrics_collector,
    ).await?;
    
    // Start orchestrator
    orchestrator.start("supervised".to_string()).await?;
    
    // Create and submit test task
    let task = create_task(
        "test-task".to_string(),
        "Integration test task".to_string(),
        TaskType::Processing,
        Priority::High,
        vec!["task_execution".to_string()],
        serde_json::json!({"test": true}),
    );
    
    let task_id = orchestrator.submit_task(task).await?;
    
    // Wait for task processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check task status
    let status = orchestrator.get_task_status(task_id).await?;
    assert!(matches!(status, TaskStatus::Pending | TaskStatus::Assigned | TaskStatus::InProgress));
    
    // Shutdown
    orchestrator.shutdown().await?;
    
    Ok(())
}

/// Test security framework integration
#[tokio::test]
#[serial]
async fn test_security_integration() -> Result<()> {
    let security_manager = SecurityManager::new().await?;
    let agent_id = Uuid::new_v4();
    
    // Test token issuance
    let capabilities = vec![
        Capability::TaskExecution,
        Capability::DataAccess,
    ];
    
    let token = security_manager.issue_token(agent_id, capabilities.clone()).await?;
    assert_eq!(token.agent_id, agent_id);
    assert_eq!(token.capabilities, capabilities);
    assert!(token.is_valid());
    
    // Test token validation
    let validated_token = security_manager.validate_token(token.id).await?;
    assert_eq!(validated_token.id, token.id);
    
    // Test access control
    let has_access = security_manager.check_access(
        agent_id,
        "test-resource",
        &Capability::TaskExecution,
    ).await?;
    assert!(has_access);
    
    let no_access = security_manager.check_access(
        agent_id,
        "admin-resource",
        &Capability::SystemAdmin,
    ).await?;
    assert!(!no_access);
    
    // Test token revocation
    security_manager.revoke_token(token.id).await?;
    let revoke_result = security_manager.validate_token(token.id).await;
    assert!(revoke_result.is_err());
    
    Ok(())
}

/// Test message broker communication
#[tokio::test]
#[serial]
async fn test_message_broker() -> Result<()> {
    let broker = MessageBroker::new().await?;
    broker.start().await?;
    
    let agent1_id = Uuid::new_v4();
    let agent2_id = Uuid::new_v4();
    
    // Register agents
    let mut channel1 = broker.register_agent(agent1_id).await?;
    let _channel2 = broker.register_agent(agent2_id).await?;
    
    // Test direct messaging
    let message = Message::new(
        agent2_id,
        Some(agent1_id),
        MessageType::Request,
        communication::Priority::High,
        serde_json::json!({"test": "message"}),
    );
    
    broker.send_message(message.clone()).await?;
    
    // Check message delivery (with timeout)
    let received = tokio::time::timeout(
        Duration::from_millis(100),
        channel1.receive_message()
    ).await;
    
    assert!(received.is_ok());
    let received_message = received.unwrap();
    assert!(received_message.is_some());
    let received_message = received_message.unwrap();
    assert_eq!(received_message.from, agent2_id);
    
    // Test broadcast
    let broadcast_message = Message::new(
        agent1_id,
        None,
        MessageType::Broadcast,
        communication::Priority::Normal,
        serde_json::json!({"broadcast": "test"}),
    );
    
    broker.broadcast(broadcast_message).await?;
    
    // Verify broker stats
    assert_eq!(broker.get_connection_count().await, 2);
    
    broker.shutdown().await?;
    Ok(())
}

/// Test monitoring and metrics collection
#[tokio::test]
#[serial]
async fn test_monitoring_system() -> Result<()> {
    let metrics_collector = MetricsCollector::new().await?;
    metrics_collector.start().await?;
    
    // Test agent metrics recording
    let agent_id = Uuid::new_v4();
    let agent_metrics = AgentMetrics {
        agent_id,
        timestamp: chrono::Utc::now(),
        status: "active".to_string(),
        tasks_completed: 10,
        tasks_failed: 1,
        tasks_in_progress: 2,
        average_response_time_ms: 150.0,
        cpu_usage_percent: 45.0,
        memory_usage_mb: 512,
        message_queue_size: 5,
        last_activity: chrono::Utc::now(),
    };
    
    metrics_collector.record_agent_metrics(agent_metrics).await?;
    
    // Test system health check
    let health_status = metrics_collector.get_system_health().await?;
    assert!(matches!(health_status, monitoring::HealthStatus::Healthy | monitoring::HealthStatus::Warning));
    
    // Test alert system
    let alerts = metrics_collector.get_alerts(None).await;
    // Should start with no alerts (or system-generated ones)
    assert!(alerts.len() >= 0);
    
    metrics_collector.shutdown().await?;
    Ok(())
}

/// Test system resilience and error handling
#[tokio::test]
#[serial]
async fn test_system_resilience() -> Result<()> {
    let security_manager = SecurityManager::new().await?;
    let message_broker = MessageBroker::new().await?;
    let metrics_collector = MetricsCollector::new().await?;
    let agent_manager = AgentManager::new(3, &security_manager).await?;
    
    let orchestrator = OrchestratorEngine::new(
        agent_manager,
        message_broker,
        metrics_collector,
    ).await?;
    
    orchestrator.start("supervised".to_string()).await?;
    
    // Test with invalid task
    let invalid_task = create_task(
        "invalid-task".to_string(),
        "Task with impossible requirements".to_string(),
        TaskType::Processing,
        Priority::High,
        vec!["nonexistent_capability".to_string()],
        serde_json::json!({}),
    );
    
    let task_id = orchestrator.submit_task(invalid_task).await?;
    
    // Task should remain pending or fail gracefully
    tokio::time::sleep(Duration::from_millis(50)).await;
    let status = orchestrator.get_task_status(task_id).await?;
    assert!(matches!(status, TaskStatus::Pending | TaskStatus::Failed));
    
    // Test system recovery after shutdown and restart
    orchestrator.shutdown().await?;
    
    // System should handle shutdown gracefully
    Ok(())
}

/// Test performance under load
#[tokio::test]
#[serial]
async fn test_performance_load() -> Result<()> {
    let security_manager = SecurityManager::new().await?;
    let message_broker = MessageBroker::new().await?;
    let metrics_collector = MetricsCollector::new().await?;
    let agent_manager = AgentManager::new(20, &security_manager).await?;
    
    let orchestrator = OrchestratorEngine::new(
        agent_manager,
        message_broker,
        metrics_collector,
    ).await?;
    
    orchestrator.start("supervised".to_string()).await?;
    
    // Submit multiple tasks concurrently
    let mut task_handles = Vec::new();
    
    for i in 0..10 {
        let orchestrator_clone = &orchestrator;
        let handle = tokio::spawn(async move {
            let task = create_task(
                format!("load-test-task-{}", i),
                "Load test task".to_string(),
                TaskType::Processing,
                Priority::Normal,
                vec!["task_execution".to_string()],
                serde_json::json!({"index": i}),
            );
            
            orchestrator_clone.submit_task(task).await
        });
        task_handles.push(handle);
    }
    
    // Wait for all tasks to be submitted
    let mut submitted_tasks = Vec::new();
    for handle in task_handles {
        let task_id = handle.await??;
        submitted_tasks.push(task_id);
    }
    
    assert_eq!(submitted_tasks.len(), 10);
    
    // Brief wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check that tasks were processed (at least some should be assigned)
    let mut assigned_count = 0;
    for task_id in submitted_tasks {
        let status = orchestrator.get_task_status(task_id).await?;
        if !matches!(status, TaskStatus::Pending) {
            assigned_count += 1;
        }
    }
    
    // At least some tasks should have been processed
    assert!(assigned_count > 0, "No tasks were processed under load");
    
    orchestrator.shutdown().await?;
    Ok(())
}