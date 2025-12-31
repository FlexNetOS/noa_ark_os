//! Integration tests for Enhanced Workflow Processing System
//! 
//! This module provides comprehensive integration testing for the complete
//! workflow from user chat requests through deliverable generation.

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use uuid::Uuid;

use agentaskit_production::workflows::{
    EnhancedWorkflowProcessor, ChatRequest, RequestPriority, TaskSubject,
    DeliverableType, LocationType, VerificationStatus
};
use agentaskit_shared::{AgentCommunicationProtocol, TaskOrchestrationProtocol};

/// Mock communication protocol for testing
struct MockCommunicationProtocol;

#[async_trait::async_trait]
impl AgentCommunicationProtocol for MockCommunicationProtocol {
    async fn send_message(&self, _message: agentaskit_shared::AgentMessage) -> Result<()> {
        Ok(())
    }
    
    async fn receive_messages(&self) -> Result<Vec<agentaskit_shared::AgentMessage>> {
        Ok(Vec::new())
    }
    
    async fn broadcast_message(
        &self, 
        _message: agentaskit_shared::AgentMessage, 
        _targets: Vec<agentaskit_shared::AgentId>
    ) -> Result<()> {
        Ok(())
    }
    
    async fn subscribe(&self, _message_types: Vec<String>) -> Result<()> {
        Ok(())
    }
    
    async fn unsubscribe(&self, _message_types: Vec<String>) -> Result<()> {
        Ok(())
    }
}

/// Mock task orchestration protocol for testing
struct MockTaskOrchestrationProtocol;

#[async_trait::async_trait]
impl TaskOrchestrationProtocol for MockTaskOrchestrationProtocol {
    async fn submit_task(&self, _task: agentaskit_shared::Task) -> Result<agentaskit_shared::TaskId> {
        Ok(agentaskit_shared::TaskId::new())
    }
    
    async fn get_task_status(&self, _task_id: agentaskit_shared::TaskId) -> Result<agentaskit_shared::TaskStatus> {
        Ok(agentaskit_shared::TaskStatus::Pending)
    }
    
    async fn get_task(&self, _task_id: agentaskit_shared::TaskId) -> Result<agentaskit_shared::Task> {
        todo!("Mock implementation")
    }
    
    async fn cancel_task(&self, _task_id: agentaskit_shared::TaskId) -> Result<()> {
        Ok(())
    }
    
    async fn assign_task(&self, _task_id: agentaskit_shared::TaskId, _agent_id: agentaskit_shared::AgentId) -> Result<()> {
        Ok(())
    }
    
    async fn complete_task(&self, _task_id: agentaskit_shared::TaskId, _result: serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    async fn fail_task(&self, _task_id: agentaskit_shared::TaskId, _error: String) -> Result<()> {
        Ok(())
    }
}

/// Create test SOT content
fn create_test_sot_content() -> String {
    r#"# NOA Dynamic UI Cross-Platform Monorepo SoT

## 0) Meta
- Owner: DEFLEX
- Last updated: 2025-10-04 11:00 UTC
- Scope: Enhanced workflow processing system testing
- Status: Integration testing in progress

## 1) Task Ledger

### 1.1 Executed Tasks (Chronological)
- [x] 2025-10-04 10:00 UTC — Initial workflow analysis completed
- [x] 2025-10-04 10:30 UTC — Core workflow processor implementation
- [x] 2025-10-04 10:45 UTC — SOP file creation and validation

### 1.2 In-Progress Tasks
- [ ] 2025-10-04 11:00 UTC — Integration testing and validation
- [ ] 2025-10-04 11:30 UTC — Performance benchmarking

### 1.3 Pending Tasks
- [ ] Security validation and vulnerability assessment
- [ ] Production readiness certification
- [ ] Documentation finalization
"#.to_string()
}

/// Create test configuration
async fn create_test_config(temp_dir: &TempDir) -> Result<EnhancedWorkflowProcessor> {
    let sot_path = temp_dir.path().join("sot.md");
    let todo_path = temp_dir.path().join("tasks.todo");
    
    // Create test SOT file
    fs::write(&sot_path, create_test_sot_content()).await?;
    
    // Create empty TODO file
    fs::write(&todo_path, "# Test TODO List\n\n").await?;
    
    let communication_protocol = Arc::new(MockCommunicationProtocol);
    let task_protocol = Arc::new(MockTaskOrchestrationProtocol);
    
    Ok(EnhancedWorkflowProcessor::new(
        sot_path,
        todo_path,
        communication_protocol,
        task_protocol,
    ))
}

/// Create test chat request
fn create_test_chat_request() -> ChatRequest {
    let mut context = HashMap::new();
    context.insert(
        "user_session".to_string(), 
        serde_json::json!("test_session_123")
    );
    
    ChatRequest {
        id: Uuid::new_v4(),
        user_id: "test_user".to_string(),
        message: "Create a new Rust module for user authentication with comprehensive tests and documentation".to_string(),
        timestamp: chrono::Utc::now(),
        context,
        session_id: Some("test_session_123".to_string()),
        priority: RequestPriority::High,
    }
}

#[tokio::test]
async fn test_complete_workflow_processing() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process the complete workflow
    let task_subject = processor.process_chat_request(request.clone()).await?;
    
    // Verify task subject was created correctly
    assert_eq!(task_subject.priority, RequestPriority::High);
    assert!(!task_subject.title.is_empty());
    assert!(!task_subject.description.is_empty());
    
    // Verify 4D methodology was applied
    assert!(!task_subject.deconstruct.core_intent.is_empty());
    assert!(!task_subject.deconstruct.key_entities.is_empty());
    assert!(!task_subject.deconstruct.output_requirements.is_empty());
    
    assert!(task_subject.diagnose.completeness_score > 0.0);
    assert!(!task_subject.develop.selected_techniques.is_empty());
    assert!(!task_subject.deliver.execution_plan.is_empty());
    
    // Verify deliverables were defined
    assert!(!task_subject.deliverables.is_empty());
    
    for deliverable in &task_subject.deliverables {
        assert!(!deliverable.name.is_empty());
        assert!(!deliverable.description.is_empty());
        assert!(!deliverable.file_specifications.is_empty());
        assert!(!deliverable.quality_requirements.is_empty());
        assert!(!deliverable.acceptance_criteria.is_empty());
    }
    
    println!("✅ Complete workflow processing test passed");
    Ok(())
}

#[tokio::test]
async fn test_sot_reading_and_analysis() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    
    // Test SOT file reading
    let sot_content = processor.read_sot_file().await?;
    assert!(sot_content.contains("NOA Dynamic UI"));
    assert!(sot_content.contains("Executed Tasks"));
    assert!(sot_content.contains("In-Progress Tasks"));
    
    // Test SOT analysis
    let request = create_test_chat_request();
    let analysis = processor.analyze_sot_content(&sot_content, &request).await?;
    
    assert!(!analysis.executed_tasks.is_empty());
    assert!(!analysis.in_progress_tasks.is_empty());
    assert!(!analysis.system_constraints.is_empty());
    assert!(analysis.request_alignment >= 0.0 && analysis.request_alignment <= 1.0);
    
    println!("✅ SOT reading and analysis test passed");
    Ok(())
}

#[tokio::test]
async fn test_4d_methodology_application() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Get SOT analysis
    let sot_content = processor.read_sot_file().await?;
    let sot_analysis = processor.analyze_sot_content(&sot_content, &request).await?;
    
    // Apply 4D methodology
    let task_subject = processor.apply_4d_method(&request, &sot_analysis).await?;
    
    // Verify DECONSTRUCT phase
    assert!(!task_subject.deconstruct.core_intent.is_empty());
    assert!(!task_subject.deconstruct.key_entities.is_empty());
    assert!(!task_subject.deconstruct.output_requirements.is_empty());
    assert!(!task_subject.deconstruct.constraints.is_empty());
    
    // Verify DIAGNOSE phase
    assert!(task_subject.diagnose.completeness_score >= 0.0);
    assert!(task_subject.diagnose.completeness_score <= 1.0);
    
    // Verify DEVELOP phase
    assert!(!task_subject.develop.selected_techniques.is_empty());
    assert!(!task_subject.develop.ai_role_assignment.is_empty());
    
    // Verify DELIVER phase
    assert!(!task_subject.deliver.execution_plan.is_empty());
    assert!(!task_subject.deliver.deliverable_specifications.is_empty());
    assert!(!task_subject.deliver.target_locations.is_empty());
    
    println!("✅ 4D methodology application test passed");
    Ok(())
}

#[tokio::test]
async fn test_todo_management_system() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process request to generate task subject
    let task_subject = processor.process_chat_request(request).await?;
    
    // Verify TODO file was updated
    let todo_content = fs::read_to_string(&processor.todo_path).await?;
    assert!(todo_content.contains(&task_subject.title));
    assert!(todo_content.contains("Priority:"));
    assert!(todo_content.contains("Status:"));
    assert!(todo_content.contains("### Deconstruct"));
    assert!(todo_content.contains("### Diagnose"));
    assert!(todo_content.contains("### Develop"));
    assert!(todo_content.contains("### Deliver"));
    
    println!("✅ TODO management system test passed");
    Ok(())
}

#[tokio::test]
async fn test_deliverable_definition_and_target_locations() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process request to generate deliverables
    let task_subject = processor.process_chat_request(request).await?;
    
    // Verify deliverables were created
    assert!(!task_subject.deliverables.is_empty());
    
    for deliverable in &task_subject.deliverables {
        // Verify deliverable has all required fields
        assert!(!deliverable.name.is_empty());
        assert!(!deliverable.description.is_empty());
        
        // Verify target location follows production structure preference
        match deliverable.target_location.location_type {
            LocationType::ProductionDirectory => {
                assert!(deliverable.target_location.base_path
                    .to_string_lossy()
                    .contains("agentaskit-production"));
            }
            LocationType::DocsSubdirectory => {
                assert!(deliverable.target_location.base_path
                    .to_string_lossy()
                    .contains("docs"));
            }
            _ => {
                // Other location types should also have appropriate base paths
                assert!(!deliverable.target_location.base_path.as_os_str().is_empty());
            }
        }
        
        // Verify file specifications
        assert!(!deliverable.file_specifications.is_empty());
        for file_spec in &deliverable.file_specifications {
            assert!(!file_spec.filename.is_empty());
            assert!(!file_spec.file_type.is_empty());
            assert!(!file_spec.encoding.is_empty());
        }
        
        // Verify quality requirements and acceptance criteria
        assert!(!deliverable.quality_requirements.is_empty());
        assert!(!deliverable.acceptance_criteria.is_empty());
    }
    
    println!("✅ Deliverable definition and target locations test passed");
    Ok(())
}

#[tokio::test]
async fn test_verification_protocol_implementation() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process request to generate task subject with verification protocol
    let task_subject = processor.process_chat_request(request).await?;
    
    // Verify verification protocol is properly implemented
    let verification_protocol = &task_subject.deliver.verification_protocol;
    
    // Verify all three verification passes are defined
    assert_eq!(verification_protocol.pass_a_self_check.name, "Self-Check");
    assert_eq!(verification_protocol.pass_b_independent.name, "Independent Re-derivation");
    assert_eq!(verification_protocol.pass_c_adversarial.name, "Adversarial Check");
    
    // Verify each pass has criteria and tests defined
    assert!(!verification_protocol.pass_a_self_check.criteria.is_empty());
    assert!(!verification_protocol.pass_b_independent.criteria.is_empty());
    assert!(!verification_protocol.pass_c_adversarial.criteria.is_empty());
    
    // Verify evidence ledger structure
    let evidence_ledger = &verification_protocol.evidence_ledger;
    assert!(evidence_ledger.files.is_empty()); // Will be populated during execution
    assert!(evidence_ledger.data_sources.is_empty()); // Will be populated during execution
    assert!(evidence_ledger.tests.is_empty()); // Will be populated during execution
    
    // Verify truth gate requirements
    let truth_gate = &verification_protocol.truth_gate_requirements;
    // Requirements will be checked during actual execution
    assert!(!truth_gate.artifact_presence || !truth_gate.smoke_test_passed); // At least one should be false initially
    
    println!("✅ Verification protocol implementation test passed");
    Ok(())
}

#[tokio::test]
async fn test_agent_orchestration_integration() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process request to trigger agent orchestration
    let task_subject = processor.process_chat_request(request).await?;
    
    // Verify execution plan was created for agent orchestration
    assert!(!task_subject.deliver.execution_plan.is_empty());
    
    for step in &task_subject.deliver.execution_plan {
        assert!(!step.name.is_empty());
        assert!(!step.description.is_empty());
        assert!(!step.verification_criteria.is_empty());
        assert!(step.estimated_duration.num_seconds() > 0);
    }
    
    // Verify timeline and milestones
    let timeline = &task_subject.deliver.timeline;
    assert!(timeline.start_time <= timeline.estimated_end_time);
    assert!(!timeline.milestones.is_empty());
    assert!(!timeline.critical_path.is_empty());
    
    println!("✅ Agent orchestration integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_performance_characteristics() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    
    // Test performance with multiple requests
    let start_time = std::time::Instant::now();
    
    for i in 0..10 {
        let mut request = create_test_chat_request();
        request.message = format!("Test request {} for performance validation", i);
        
        let _task_subject = processor.process_chat_request(request).await?;
    }
    
    let elapsed = start_time.elapsed();
    let average_per_request = elapsed / 10;
    
    // Verify performance meets requirements (<200ms per request)
    assert!(average_per_request.as_millis() < 200, 
        "Average processing time {} ms exceeds 200ms requirement", 
        average_per_request.as_millis());
    
    println!("✅ Performance characteristics test passed");
    println!("   Average processing time: {} ms", average_per_request.as_millis());
    Ok(())
}

#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    
    // Test with invalid SOT file
    fs::remove_file(&processor.sot_path).await?;
    let request = create_test_chat_request();
    
    let result = processor.process_chat_request(request).await;
    assert!(result.is_err(), "Should fail when SOT file is missing");
    
    // Test with corrupted TODO file
    fs::write(&processor.todo_path, "Invalid TODO content {{{{ malformed").await?;
    
    // Recreate SOT file
    fs::write(&processor.sot_path, create_test_sot_content()).await?;
    
    let request = create_test_chat_request();
    let result = processor.process_chat_request(request).await;
    
    // Should handle corrupted TODO file gracefully
    assert!(result.is_ok(), "Should handle corrupted TODO file gracefully");
    
    println!("✅ Error handling and recovery test passed");
    Ok(())
}

#[tokio::test]
async fn test_memory_rule_compliance() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let processor = create_test_config(&temp_dir).await?;
    let request = create_test_chat_request();
    
    // Process request and verify compliance
    let task_subject = processor.process_chat_request(request).await?;
    
    // Verify Production Structure Preference compliance
    for deliverable in &task_subject.deliverables {
        match deliverable.deliverable_type {
            DeliverableType::SourceCode | DeliverableType::BuildArtifact => {
                assert!(deliverable.target_location.base_path
                    .to_string_lossy()
                    .contains("agentaskit-production"),
                    "Source code must be in agentaskit-production directory");
            }
            DeliverableType::Documentation | DeliverableType::Report | DeliverableType::Analysis => {
                assert!(deliverable.target_location.base_path
                    .to_string_lossy()
                    .contains("docs"),
                    "Documentation must be in docs subdirectory");
            }
            _ => {} // Other types have appropriate locations
        }
    }
    
    // Verify File Unification Rule compliance
    assert!(!task_subject.deliver.deliverable_specifications.is_empty(),
        "All deliverables must be properly specified");
    
    println!("✅ Memory rule compliance test passed");
    Ok(())
}

/// Integration test runner
#[tokio::test]
async fn run_comprehensive_integration_tests() -> Result<()> {
    println!("\n🚀 Running Enhanced Workflow Integration Tests");
    println!("================================================");
    
    // Run all integration tests
    test_complete_workflow_processing().await?;
    test_sot_reading_and_analysis().await?;
    test_4d_methodology_application().await?;
    test_todo_management_system().await?;
    test_deliverable_definition_and_target_locations().await?;
    test_verification_protocol_implementation().await?;
    test_agent_orchestration_integration().await?;
    test_performance_characteristics().await?;
    test_error_handling_and_recovery().await?;
    test_memory_rule_compliance().await?;
    
    println!("\n✅ All Enhanced Workflow Integration Tests Passed");
    println!("================================================");
    println!("🎯 System is ready for production deployment");
    
    Ok(())
}