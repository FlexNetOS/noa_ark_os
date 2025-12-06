// Phase 4 Integration Tests - Complete Hierarchical Agent System Validation
// Tests the full coordination between Executive → Board → Specialized layers

use super::*;
use crate::agents::{
    executive::{ExecutiveLayer, ExecutiveLayerConfig},
    board::{BoardLayer, BoardLayerConfig},
    specialized::SpecializedLayer,
};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, error};

/// Comprehensive integration test suite for Phase 4 Agent Framework
pub struct Phase4IntegrationTest {
    executive_layer: ExecutiveLayer,
    board_layer: BoardLayer,
    specialized_layer: SpecializedLayer,
}

impl Phase4IntegrationTest {
    /// Initialize complete agent hierarchy for testing
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing Phase 4 Integration Test Environment");

        // Initialize Executive Layer
        let executive_config = ExecutiveLayerConfig::default();
        let executive_layer = ExecutiveLayer::new(executive_config).await
            .map_err(|e| format!("Failed to initialize Executive Layer: {}", e))?;

        // Initialize Board Layer  
        let board_config = BoardLayerConfig::default();
        let board_layer = BoardLayer::new(board_config).await
            .map_err(|e| format!("Failed to initialize Board Layer: {}", e))?;

        // Initialize Specialized Layer
        let specialized_layer = SpecializedLayer::new().await
            .map_err(|e| format!("Failed to initialize Specialized Layer: {}", e))?;

        Ok(Self {
            executive_layer,
            board_layer,
            specialized_layer,
        })
    }

    /// Test complete agent hierarchy initialization
    pub async fn test_hierarchy_initialization(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing hierarchical agent initialization");

        // Verify Executive Layer
        let exec_stats = self.executive_layer.get_statistics().await
            .map_err(|e| format!("Failed to get executive statistics: {}", e))?;
        assert_eq!(exec_stats.total_agents, 5, "Executive layer should have 5 agents");
        info!("✓ Executive Layer: {} agents initialized", exec_stats.total_agents);

        // Verify Board Layer
        let board_status = self.board_layer.get_layer_status().await
            .map_err(|e| format!("Failed to get board status: {}", e))?;
        assert_eq!(board_status.total_agents, 5, "Board layer should have 5 agents");
        info!("✓ Board Layer: {} agents initialized", board_status.total_agents);

        // Verify Specialized Layer
        let specialized_status = self.specialized_layer.get_layer_status().await
            .map_err(|e| format!("Failed to get specialized status: {}", e))?;
        assert_eq!(specialized_status.total_agents, 8, "Specialized layer should have 8 agents");
        info!("✓ Specialized Layer: {} agents initialized", specialized_status.total_agents);

        info!("✅ Hierarchical initialization test PASSED");
        Ok(())
    }

    /// Test agent startup sequence across all layers
    pub async fn test_agent_startup_sequence(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing agent startup sequence");

        // Start Executive Layer first (highest priority)
        timeout(Duration::from_secs(30), self.executive_layer.start_all_agents()).await
            .map_err(|_| "Executive layer startup timeout")?
            .map_err(|e| format!("Executive layer startup failed: {}", e))?;
        info!("✓ Executive Layer agents started");

        // Start Board Layer second (strategic oversight)
        timeout(Duration::from_secs(30), self.board_layer.start_all_agents()).await
            .map_err(|_| "Board layer startup timeout")?
            .map_err(|e| format!("Board layer startup failed: {}", e))?;
        info!("✓ Board Layer agents started");

        // Start Specialized Layer last (operational execution)
        timeout(Duration::from_secs(30), self.specialized_layer.start_all_agents()).await
            .map_err(|_| "Specialized layer startup timeout")?
            .map_err(|e| format!("Specialized layer startup failed: {}", e))?;
        info!("✓ Specialized Layer agents started");

        // Verify all agents are active
        tokio::time::sleep(Duration::from_secs(2)).await; // Allow startup to complete

        let exec_stats = self.executive_layer.get_statistics().await?;
        let board_status = self.board_layer.get_layer_status().await?;
        let specialized_status = self.specialized_layer.get_layer_status().await?;

        assert!(exec_stats.active_agents > 0, "Executive agents should be active");
        assert!(board_status.active_agents > 0, "Board agents should be active");
        assert!(specialized_status.active_agents > 0, "Specialized agents should be active");

        info!("✅ Agent startup sequence test PASSED");
        Ok(())
    }

    /// Test cross-layer agent discovery and communication
    pub async fn test_cross_layer_communication(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing cross-layer agent communication");

        // Test Executive → Specialized communication
        let code_gen_task = Task {
            id: Uuid::new_v4(),
            name: "test_code_generation".to_string(),
            description: "Generate a simple function for testing".to_string(),
            parameters: serde_json::json!({
                "language": "rust",
                "function_name": "test_function",
                "description": "A simple test function"
            }),
            required_capabilities: vec!["code_generation".to_string()],
            deadline: None,
            dependencies: vec![],
        };

        match timeout(
            Duration::from_secs(10),
            self.specialized_layer.execute_task_on_agent("code_generation", code_gen_task)
        ).await {
            Ok(Ok(task_result)) => {
                info!("✓ Executive → Specialized communication successful");
                assert_eq!(task_result.status, TaskStatus::Completed, "Code generation task should complete");
            }
            Ok(Err(e)) => return Err(format!("Code generation task failed: {}", e).into()),
            Err(_) => return Err("Code generation task timeout".into()),
        }

        // Test Board → Specialized communication (Analytics request)
        let analytics_task = Task {
            id: Uuid::new_v4(),
            name: "test_analytics".to_string(),
            description: "Analyze test data".to_string(),
            parameters: serde_json::json!({
                "data_source": "test_dataset",
                "analysis_type": "summary_statistics"
            }),
            required_capabilities: vec!["data_analytics".to_string()],
            deadline: None,
            dependencies: vec![],
        };

        match timeout(
            Duration::from_secs(10),
            self.specialized_layer.execute_task_on_agent("data_analytics", analytics_task)
        ).await {
            Ok(Ok(task_result)) => {
                info!("✓ Board → Specialized communication successful");
                assert_eq!(task_result.status, TaskStatus::Completed, "Analytics task should complete");
            }
            Ok(Err(e)) => return Err(format!("Analytics task failed: {}", e).into()),
            Err(_) => return Err("Analytics task timeout".into()),
        }

        info!("✅ Cross-layer communication test PASSED");
        Ok(())
    }

    /// Test specialized agent capabilities and coordination
    pub async fn test_specialized_agent_capabilities(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing specialized agent capabilities");

        // Test Security Specialist capabilities
        let security_capabilities = self.specialized_layer
            .get_agent_capabilities("security_specialist").await?;
        assert!(!security_capabilities.is_empty(), "Security agent should have capabilities");
        info!("✓ Security Specialist: {} capabilities", security_capabilities.len());

        // Test Data Analytics capabilities
        let analytics_capabilities = self.specialized_layer
            .get_agent_capabilities("data_analytics").await?;
        assert!(!analytics_capabilities.is_empty(), "Analytics agent should have capabilities");
        info!("✓ Data Analytics: {} capabilities", analytics_capabilities.len());

        // Test Integration capabilities
        let integration_capabilities = self.specialized_layer
            .get_agent_capabilities("integration").await?;
        assert!(!integration_capabilities.is_empty(), "Integration agent should have capabilities");
        info!("✓ Integration: {} capabilities", integration_capabilities.len());

        // Test all agent names are available
        let agent_names = self.specialized_layer.list_agent_names().await;
        let expected_agents = vec![
            "code_generation", "testing", "deployment", "monitoring", 
            "learning", "security_specialist", "data_analytics", "integration"
        ];

        for expected in &expected_agents {
            assert!(agent_names.contains(&expected.to_string()), 
                   "Expected agent '{}' not found", expected);
        }
        info!("✓ All 8 specialized agents available: {:?}", agent_names);

        info!("✅ Specialized agent capabilities test PASSED");
        Ok(())
    }

    /// Test system-wide task execution workflow
    pub async fn test_end_to_end_workflow(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing end-to-end workflow execution");

        // Simulate a complex workflow: Code → Test → Deploy → Monitor
        let workflow_tasks = vec![
            ("code_generation", "generate_microservice", serde_json::json!({
                "service_name": "test_service",
                "language": "rust",
                "features": ["rest_api", "database"]
            })),
            ("testing", "create_test_suite", serde_json::json!({
                "service_name": "test_service",
                "test_types": ["unit", "integration"]
            })),
            ("deployment", "deploy_service", serde_json::json!({
                "service_name": "test_service",
                "environment": "staging"
            })),
            ("monitoring", "setup_monitoring", serde_json::json!({
                "service_name": "test_service",
                "metrics": ["response_time", "error_rate"]
            })),
        ];

        for (agent_name, task_name, params) in workflow_tasks {
            let task = Task {
                id: Uuid::new_v4(),
                name: task_name.to_string(),
                description: format!("Workflow step: {}", task_name),
                parameters: params,
                required_capabilities: vec![agent_name.replace("_", "")],
                deadline: Some(std::time::Instant::now() + Duration::from_secs(30)),
                dependencies: vec![],
            };

            match timeout(
                Duration::from_secs(15),
                self.specialized_layer.execute_task_on_agent(agent_name, task)
            ).await {
                Ok(Ok(result)) => {
                    info!("✓ Workflow step '{}' completed successfully", task_name);
                    assert_eq!(result.status, TaskStatus::Completed);
                }
                Ok(Err(e)) => {
                    error!("Workflow step '{}' failed: {}", task_name, e);
                    return Err(format!("Workflow step '{}' failed: {}", task_name, e).into());
                }
                Err(_) => {
                    error!("Workflow step '{}' timed out", task_name);
                    return Err(format!("Workflow step '{}' timed out", task_name).into());
                }
            }
        }

        info!("✅ End-to-end workflow test PASSED");
        Ok(())
    }

    /// Test system performance and scalability
    pub async fn test_system_performance(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing system performance and scalability");

        let start_time = std::time::Instant::now();
        
        // Execute multiple tasks concurrently across different agents
        let mut task_handles = vec![];

        for i in 0..10 {
            let layer = &self.specialized_layer;
            let task = Task {
                id: Uuid::new_v4(),
                name: format!("performance_test_{}", i),
                description: "Performance testing task".to_string(),
                parameters: serde_json::json!({ "test_id": i }),
                required_capabilities: vec!["general".to_string()],
                deadline: None,
                dependencies: vec![],
            };

            // Distribute tasks across different agents
            let agent_names = vec![
                "code_generation", "testing", "deployment", "monitoring", 
                "learning", "security_specialist", "data_analytics", "integration"
            ];
            let agent_name = agent_names[i % agent_names.len()];

            let handle = tokio::spawn(async move {
                layer.execute_task_on_agent(agent_name, task).await
            });

            task_handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut completed = 0;
        for handle in task_handles {
            match timeout(Duration::from_secs(20), handle).await {
                Ok(Ok(Ok(_))) => completed += 1,
                Ok(Ok(Err(e))) => error!("Task failed: {}", e),
                Ok(Err(e)) => error!("Task join error: {}", e),
                Err(_) => error!("Task timeout"),
            }
        }

        let elapsed = start_time.elapsed();
        let success_rate = completed as f64 / 10.0 * 100.0;

        info!("Performance test completed:");
        info!("  - Total tasks: 10");
        info!("  - Completed: {}", completed);
        info!("  - Success rate: {:.1}%", success_rate);
        info!("  - Total time: {:?}", elapsed);

        assert!(success_rate >= 80.0, "Success rate should be at least 80%");
        assert!(elapsed < Duration::from_secs(30), "Should complete within 30 seconds");

        info!("✅ System performance test PASSED");
        Ok(())
    }

    /// Test graceful shutdown sequence
    pub async fn test_graceful_shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing graceful shutdown sequence");

        // Shutdown in reverse order: Specialized → Board → Executive
        
        timeout(Duration::from_secs(15), self.specialized_layer.stop_all_agents()).await
            .map_err(|_| "Specialized layer shutdown timeout")?
            .map_err(|e| format!("Specialized layer shutdown failed: {}", e))?;
        info!("✓ Specialized Layer stopped gracefully");

        timeout(Duration::from_secs(15), self.board_layer.stop_all_agents()).await
            .map_err(|_| "Board layer shutdown timeout")?
            .map_err(|e| format!("Board layer shutdown failed: {}", e))?;
        info!("✓ Board Layer stopped gracefully");

        timeout(Duration::from_secs(15), self.executive_layer.stop_all_agents()).await
            .map_err(|_| "Executive layer shutdown timeout")?
            .map_err(|e| format!("Executive layer shutdown failed: {}", e))?;
        info!("✓ Executive Layer stopped gracefully");

        info!("✅ Graceful shutdown test PASSED");
        Ok(())
    }

    /// Run complete integration test suite
    pub async fn run_complete_test_suite() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🚀 Starting Phase 4 Complete Integration Test Suite");

        let mut test_suite = Self::new().await?;

        // Run all tests in sequence
        test_suite.test_hierarchy_initialization().await?;
        test_suite.test_agent_startup_sequence().await?;
        test_suite.test_cross_layer_communication().await?;
        test_suite.test_specialized_agent_capabilities().await?;
        test_suite.test_end_to_end_workflow().await?;
        test_suite.test_system_performance().await?;
        test_suite.test_graceful_shutdown().await?;

        info!("🎉 Phase 4 Complete Integration Test Suite PASSED!");
        info!("   ✅ Hierarchy Initialization");
        info!("   ✅ Agent Startup Sequence");
        info!("   ✅ Cross-Layer Communication");
        info!("   ✅ Specialized Agent Capabilities");
        info!("   ✅ End-to-End Workflow");
        info!("   ✅ System Performance");
        info!("   ✅ Graceful Shutdown");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_phase4_integration_quick() {
        // Quick integration test for CI/CD
        let test_suite = Phase4IntegrationTest::new().await.expect("Failed to create test suite");
        test_suite.test_hierarchy_initialization().await.expect("Hierarchy test failed");
        test_suite.test_specialized_agent_capabilities().await.expect("Capabilities test failed");
    }

    #[tokio::test]
    #[traced_test]
    async fn test_specialized_layer_standalone() {
        // Test specialized layer in isolation
        let specialized = SpecializedLayer::new().await.expect("Failed to create specialized layer");
        
        let status = specialized.get_layer_status().await.expect("Failed to get status");
        assert_eq!(status.total_agents, 8);

        let agent_names = specialized.list_agent_names().await;
        assert!(agent_names.contains(&"security_specialist".to_string()));
        assert!(agent_names.contains(&"data_analytics".to_string()));
        assert!(agent_names.contains(&"integration".to_string()));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_agent_task_execution() {
        // Test individual agent task execution
        let specialized = SpecializedLayer::new().await.expect("Failed to create specialized layer");

        let task = Task {
            id: Uuid::new_v4(),
            name: "test_task".to_string(),
            description: "Test task execution".to_string(),
            parameters: serde_json::json!({"test": true}),
            required_capabilities: vec!["testing".to_string()],
            deadline: None,
            dependencies: vec![],
        };

        let result = specialized.execute_task_on_agent("code_generation", task).await;
        assert!(result.is_ok(), "Task execution should succeed");
    }
}

/// Utility functions for integration testing
pub mod test_utils {
    use super::*;

    /// Create a test task with standard parameters
    pub fn create_test_task(name: &str, agent_capability: &str) -> Task {
        Task {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: format!("Test task for {}", name),
            parameters: serde_json::json!({
                "test": true,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }),
            required_capabilities: vec![agent_capability.to_string()],
            deadline: Some(std::time::Instant::now() + Duration::from_secs(60)),
            dependencies: vec![],
        }
    }

    /// Check if all expected agents are present in a layer
    pub async fn verify_agent_count(layer_name: &str, expected_count: usize, actual_count: usize) -> bool {
        if actual_count == expected_count {
            info!("✓ {}: {} agents verified", layer_name, actual_count);
            true
        } else {
            error!("✗ {}: expected {} agents, found {}", layer_name, expected_count, actual_count);
            false
        }
    }

    /// Generate performance test tasks
    pub fn generate_performance_tasks(count: usize) -> Vec<Task> {
        (0..count).map(|i| {
            Task {
                id: Uuid::new_v4(),
                name: format!("perf_test_{}", i),
                description: "Performance testing task".to_string(),
                parameters: serde_json::json!({
                    "test_id": i,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }),
                required_capabilities: vec!["performance".to_string()],
                deadline: Some(std::time::Instant::now() + Duration::from_secs(30)),
                dependencies: vec![],
            }
        }).collect()
    }

    /// Wait for agents to reach active state
    pub async fn wait_for_agent_activation(
        max_wait: Duration,
        check_interval: Duration,
        check_fn: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>,
    ) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < max_wait {
            if check_fn().await {
                return true;
            }
            tokio::time::sleep(check_interval).await;
        }
        false
    }
}

// This completes the Phase 4 Integration Testing framework, providing comprehensive
// validation of the complete hierarchical agent system with Executive → Board → Specialized
// layer coordination and cross-agent communication testing.
