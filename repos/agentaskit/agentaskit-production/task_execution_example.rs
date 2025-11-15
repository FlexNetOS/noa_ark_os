use agentaskit_production::workflows::{
    ChatRequest, EnhancedWorkflowProcessor, RequestPriority
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create the original user chat request
    let chat_request = ChatRequest {
        id: Uuid::new_v4(),
        user_id: "user_001".to_string(),
        message: "Are the any dublicate, unused, or files with no value add. Make production ready. Cross-refrence archives for anthing missing or overlooked e.g. (.todo and .sop at project root) (hooks, sandboxes, sbom...)".to_string(),
        timestamp: Utc::now(),
        context: HashMap::new(),
        session_id: Some("production_analysis_session".to_string()),
        priority: RequestPriority::High,
    };

    // Initialize the workflow processor
    let processor = EnhancedWorkflowProcessor::new(
        "agentaskit-production/core/src/orchestration/workflows.sop".into(),
        "agentaskit-production/core/src/orchestration/tasks.todo".into(),
    ).await?;

    // Process the chat request through the 4D method
    println!("🔄 Processing chat request through 4D Method Task Framework...");
    
    let task_subject = processor.process_chat_request(chat_request).await?;
    
    println!("✅ Task processed successfully:");
    println!("📋 Task ID: {}", task_subject.id);
    println!("📝 Title: {}", task_subject.title);
    println!("🔍 Description: {}", task_subject.description);
    
    // Display 4D Method breakdown
    println!("\n🎯 4D Method Breakdown:");
    println!("1. DECONSTRUCT: {}", task_subject.deconstruct.core_intent);
    println!("2. DIAGNOSE: Complexity: {:?}", task_subject.diagnose.complexity_assessment);
    println!("3. DEVELOP: Type: {:?}", task_subject.develop.request_type);
    println!("4. DELIVER: {} execution steps", task_subject.deliver.execution_plan.len());
    
    // Show deliverables and target locations
    println!("\n📦 Deliverables:");
    for deliverable in &task_subject.deliverables {
        println!("  - {}: {}", deliverable.name, deliverable.description);
        println!("    Target: {}", deliverable.target_location.path.display());
    }
    
    println!("\n✅ Task execution framework properly utilized!");
    
    Ok(())
}