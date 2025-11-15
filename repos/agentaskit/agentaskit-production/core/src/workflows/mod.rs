//! Enhanced Workflow Processing System
//! 
//! This module implements the complete workflow from user chat requests through
//! AI model processing, SOT reading, TODO updating with 4D method application,
//! and deliverable definition with target locations.

pub mod seven_phase;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::agents::{AgentId, AgentMessage, Task, TaskStatus};
use crate::shared::{AgentCommunicationProtocol, TaskOrchestrationProtocol};

/// Enhanced chat request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub id: Uuid,
    pub user_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, serde_json::Value>,
    pub session_id: Option<String>,
    pub priority: RequestPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task subject with 4D method application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubject {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub deconstruct: DeconstructPhase,
    pub diagnose: DiagnosePhase,
    pub develop: DevelopPhase,
    pub deliver: DeliverPhase,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: TaskStatus,
    pub priority: RequestPriority,
    pub assigned_agents: Vec<AgentId>,
    pub deliverables: Vec<Deliverable>,
}

/// 4D Method Phase 1: Deconstruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeconstructPhase {
    pub core_intent: String,
    pub key_entities: Vec<String>,
    pub context_analysis: String,
    pub output_requirements: Vec<String>,
    pub constraints: Vec<String>,
    pub provided_vs_missing: HashMap<String, bool>,
}

/// 4D Method Phase 2: Diagnose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosePhase {
    pub clarity_gaps: Vec<String>,
    pub ambiguity_points: Vec<String>,
    pub specificity_level: SpecificityLevel,
    pub completeness_score: f32,
    pub structure_needs: Vec<String>,
    pub complexity_assessment: ComplexityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecificityLevel {
    Vague,
    Moderate,
    Specific,
    Precise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    HighlyComplex,
}

/// 4D Method Phase 3: Develop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopPhase {
    pub request_type: RequestType,
    pub selected_techniques: Vec<OptimizationTechnique>,
    pub ai_role_assignment: String,
    pub context_enhancement: String,
    pub logical_structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    Creative,
    Technical,
    Educational,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationTechnique {
    MultiPerspective,
    ToneEmphasis,
    ConstraintBased,
    PrecisionFocus,
    FewShotExamples,
    ClearStructure,
    ChainOfThought,
    SystematicFrameworks,
}

/// 4D Method Phase 4: Deliver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverPhase {
    pub execution_plan: Vec<ExecutionStep>,
    pub verification_protocol: VerificationProtocol,
    pub deliverable_specifications: Vec<Deliverable>,
    pub target_locations: Vec<TargetLocation>,
    pub timeline: ExecutionTimeline,
}

/// Execution step with verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: Uuid,
    pub name: String,
    pub description: String,
    pub dependencies: Vec<Uuid>,
    pub assigned_agents: Vec<AgentId>,
    pub estimated_duration: chrono::Duration,
    pub verification_criteria: Vec<String>,
    pub artifacts: Vec<String>,
}

/// Triple-verification protocol implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationProtocol {
    pub pass_a_self_check: VerificationPass,
    pub pass_b_independent: VerificationPass,
    pub pass_c_adversarial: VerificationPass,
    pub evidence_ledger: EvidenceLedger,
    pub truth_gate_requirements: TruthGateRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPass {
    pub name: String,
    pub criteria: Vec<String>,
    pub tests: Vec<String>,
    pub status: VerificationStatus,
    pub timestamp: Option<DateTime<Utc>>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Passed,
    Failed,
    RequiresReview,
}

/// Evidence ledger for truth verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLedger {
    pub files: HashMap<String, String>, // path -> SHA-256 hash
    pub data_sources: Vec<DataSource>,
    pub external_references: Vec<ExternalReference>,
    pub mathematics: Vec<MathematicalProof>,
    pub tests: Vec<TestEvidence>,
    pub verification_results: Vec<VerificationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub origin: String,
    pub timestamp: DateTime<Utc>,
    pub validation_method: String,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    pub author: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub url: Option<String>,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalProof {
    pub formula: String,
    pub inputs: Vec<String>,
    pub steps: Vec<String>,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvidence {
    pub command: String,
    pub full_log: String,
    pub exit_code: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub pass_type: String,
    pub outcome: String,
    pub discrepancies: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Truth gate requirements checklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthGateRequirements {
    pub artifact_presence: bool,
    pub smoke_test_passed: bool,
    pub spec_match_verified: bool,
    pub limits_documented: bool,
    pub hashes_provided: bool,
    pub gap_scan_complete: bool,
    pub triple_verification_complete: bool,
}

/// Deliverable specification with target location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub deliverable_type: DeliverableType,
    pub target_location: TargetLocation,
    pub file_specifications: Vec<FileSpecification>,
    pub quality_requirements: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliverableType {
    SourceCode,
    Documentation,
    Configuration,
    TestSuite,
    BuildArtifact,
    Deployment,
    Report,
    Analysis,
}

/// Target location specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLocation {
    pub location_type: LocationType,
    pub base_path: PathBuf,
    pub relative_path: String,
    pub filename_pattern: Option<String>,
    pub organization_rules: Vec<String>,
    pub backup_locations: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    ProductionDirectory,  // agentaskit-production
    DocsSubdirectory,    // ~/docs
    TestDirectory,       // tests/
    ConfigDirectory,     // configs/
    ScriptsDirectory,    // scripts/
    ArchiveDirectory,    // archive/
    TempDirectory,       // temp/
}

/// File specification for deliverables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSpecification {
    pub filename: String,
    pub file_type: String,
    pub size_limits: Option<(u64, u64)>, // min, max bytes
    pub format_requirements: Vec<String>,
    pub encoding: String,
    pub permissions: Option<String>,
}

/// Execution timeline for task management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    pub start_time: DateTime<Utc>,
    pub estimated_end_time: DateTime<Utc>,
    pub milestones: Vec<Milestone>,
    pub critical_path: Vec<Uuid>, // step IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub target_date: DateTime<Utc>,
    pub dependencies: Vec<Uuid>,
    pub deliverables: Vec<Uuid>,
}

/// Enhanced workflow processor
pub struct EnhancedWorkflowProcessor {
    sot_path: PathBuf,
    todo_path: PathBuf,
    communication_protocol: Arc<dyn AgentCommunicationProtocol + Send + Sync>,
    task_protocol: Arc<dyn TaskOrchestrationProtocol + Send + Sync>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskSubject>>>,
    pending_requests: Arc<RwLock<HashMap<Uuid, ChatRequest>>>,
}

impl EnhancedWorkflowProcessor {
    pub fn new(
        sot_path: PathBuf,
        todo_path: PathBuf,
        communication_protocol: Arc<dyn AgentCommunicationProtocol + Send + Sync>,
        task_protocol: Arc<dyn TaskOrchestrationProtocol + Send + Sync>,
    ) -> Self {
        Self {
            sot_path,
            todo_path,
            communication_protocol,
            task_protocol,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Process user chat request through complete workflow
    /// Process chat request through either enhanced workflow or 7-phase workflow
    /// depending on request content and system configuration
    pub async fn process_chat_request(&self, request: ChatRequest) -> Result<TaskSubject> {
        // Check if this is a 7-phase workflow request
        if self.is_seven_phase_request(&request) {
            // Use the 7-phase orchestrator
            let orchestrator = seven_phase::SevenPhaseOrchestrator::new().await?;
            return orchestrator.execute_workflow(request).await;
        }
        
        // Fall back to enhanced workflow processing
        self.process_enhanced_workflow(request).await
    }

    /// Check if request requires 7-phase workflow processing
    fn is_seven_phase_request(&self, request: &ChatRequest) -> bool {
        let message = request.message.to_lowercase();
        message.contains("7-phase") || 
        message.contains("928-agent") || 
        message.contains("performance optimization") ||
        message.contains("triple cross-reference") ||
        request.priority == RequestPriority::Critical
    }

    /// Process using enhanced workflow (original implementation)
    async fn process_enhanced_workflow(&self, request: ChatRequest) -> Result<TaskSubject> { {
        // Store pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request.id, request.clone());
        }

        // Step 1: Read and analyze SOT
        let sot_content = self.read_sot_file().await?;
        let sot_analysis = self.analyze_sot_content(&sot_content, &request).await?;

        // Step 2: Apply 4D methodology
        let task_subject = self.apply_4d_method(&request, &sot_analysis).await?;

        // Step 3: Update TODO with task subjects
        self.update_todo_file(&task_subject).await?;

        // Step 4: Define deliverables and target locations
        self.define_deliverables_and_targets(&task_subject).await?;

        // Step 5: Store active task
        {
            let mut active = self.active_tasks.write().await;
            active.insert(task_subject.id, task_subject.clone());
        }

        // Step 6: Initiate agent orchestration
        self.initiate_agent_orchestration(&task_subject).await?;

        Ok(task_subject)
    }

    /// Read SOT file content
    async fn read_sot_file(&self) -> Result<String> {
        let content = fs::read_to_string(&self.sot_path).await?;
        Ok(content)
    }

    /// Analyze SOT content in context of user request
    async fn analyze_sot_content(&self, sot_content: &str, request: &ChatRequest) -> Result<SOTAnalysis> {
        // Parse SOT content to extract:
        // - Executed tasks
        // - In-progress tasks  
        // - Pending tasks
        // - System constraints
        // - Available resources
        
        let executed_tasks = self.extract_executed_tasks(sot_content).await?;
        let in_progress_tasks = self.extract_in_progress_tasks(sot_content).await?;
        let system_constraints = self.extract_system_constraints(sot_content).await?;
        
        Ok(SOTAnalysis {
            executed_tasks,
            in_progress_tasks,
            system_constraints,
            last_updated: Utc::now(),
            request_alignment: self.assess_request_alignment(request, sot_content).await?,
        })
    }

    /// Apply 4D methodology to create task subject
    async fn apply_4d_method(&self, request: &ChatRequest, sot_analysis: &SOTAnalysis) -> Result<TaskSubject> {
        // Phase 1: DECONSTRUCT
        let deconstruct = self.deconstruct_request(request, sot_analysis).await?;
        
        // Phase 2: DIAGNOSE  
        let diagnose = self.diagnose_requirements(&deconstruct, request).await?;
        
        // Phase 3: DEVELOP
        let develop = self.develop_approach(&diagnose, &deconstruct).await?;
        
        // Phase 4: DELIVER
        let deliver = self.design_delivery(&develop, &diagnose, &deconstruct).await?;

        Ok(TaskSubject {
            id: Uuid::new_v4(),
            title: self.generate_task_title(request, &deconstruct).await?,
            description: self.generate_task_description(request, &deconstruct).await?,
            deconstruct,
            diagnose,
            develop,
            deliver,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: TaskStatus::Pending,
            priority: request.priority.clone(),
            assigned_agents: Vec::new(),
            deliverables: Vec::new(),
        })
    }

    /// Update TODO file with new task subject
    async fn update_todo_file(&self, task_subject: &TaskSubject) -> Result<()> {
        // Read existing TODO content
        let existing_content = if self.todo_path.exists() {
            fs::read_to_string(&self.todo_path).await.unwrap_or_default()
        } else {
            String::new()
        };

        // Generate TODO entry
        let todo_entry = self.generate_todo_entry(task_subject).await?;
        
        // Append new entry
        let updated_content = if existing_content.is_empty() {
            format!("# AgentAsKit TODO List\n\n{}", todo_entry)
        } else {
            format!("{}\n\n{}", existing_content, todo_entry)
        };

        // Write updated content
        fs::write(&self.todo_path, updated_content).await?;

        Ok(())
    }

    /// Define deliverables and target locations
    async fn define_deliverables_and_targets(&self, task_subject: &TaskSubject) -> Result<Vec<Deliverable>> {
        let mut deliverables = Vec::new();

        // Generate deliverables based on task type and requirements
        for output_req in &task_subject.deconstruct.output_requirements {
            let deliverable = self.create_deliverable_specification(
                output_req,
                &task_subject.develop.request_type,
                &task_subject.priority
            ).await?;
            
            deliverables.push(deliverable);
        }

        // Add standard deliverables based on truth gate requirements
        deliverables.extend(self.create_standard_deliverables(task_subject).await?);

        Ok(deliverables)
    }

    /// Create deliverable specification with target location
    async fn create_deliverable_specification(
        &self,
        output_requirement: &str,
        request_type: &RequestType,
        priority: &RequestPriority,
    ) -> Result<Deliverable> {
        let deliverable_type = self.determine_deliverable_type(output_requirement).await?;
        let target_location = self.determine_target_location(&deliverable_type, priority).await?;
        
        Ok(Deliverable {
            id: Uuid::new_v4(),
            name: self.generate_deliverable_name(output_requirement).await?,
            description: output_requirement.to_string(),
            deliverable_type,
            target_location,
            file_specifications: self.generate_file_specifications(output_requirement).await?,
            quality_requirements: self.generate_quality_requirements(request_type).await?,
            acceptance_criteria: self.generate_acceptance_criteria(output_requirement).await?,
            dependencies: Vec::new(),
        })
    }

    /// Determine target location based on production structure preference
    async fn determine_target_location(
        &self,
        deliverable_type: &DeliverableType,
        priority: &RequestPriority,
    ) -> Result<TargetLocation> {
        let location_type = match deliverable_type {
            DeliverableType::SourceCode => LocationType::ProductionDirectory,
            DeliverableType::Documentation => LocationType::DocsSubdirectory,
            DeliverableType::Configuration => LocationType::ConfigDirectory,
            DeliverableType::TestSuite => LocationType::TestDirectory,
            DeliverableType::BuildArtifact => LocationType::ProductionDirectory,
            DeliverableType::Deployment => LocationType::ScriptsDirectory,
            DeliverableType::Report => LocationType::DocsSubdirectory,
            DeliverableType::Analysis => LocationType::DocsSubdirectory,
        };

        let base_path = match location_type {
            LocationType::ProductionDirectory => {
                PathBuf::from("agentaskit-production")
            }
            LocationType::DocsSubdirectory => {
                PathBuf::from("docs")
            }
            LocationType::TestDirectory => {
                PathBuf::from("agentaskit-production/tests")
            }
            LocationType::ConfigDirectory => {
                PathBuf::from("agentaskit-production/configs")
            }
            LocationType::ScriptsDirectory => {
                PathBuf::from("agentaskit-production/scripts")
            }
            LocationType::ArchiveDirectory => {
                PathBuf::from("archive")
            }
            LocationType::TempDirectory => {
                PathBuf::from("temp")
            }
        };

        Ok(TargetLocation {
            location_type,
            base_path,
            relative_path: self.generate_relative_path(deliverable_type).await?,
            filename_pattern: self.generate_filename_pattern(deliverable_type).await?,
            organization_rules: self.get_organization_rules(&location_type).await?,
            backup_locations: self.get_backup_locations(&location_type).await?,
        })
    }

    /// Initiate agent orchestration for task execution
    async fn initiate_agent_orchestration(&self, task_subject: &TaskSubject) -> Result<()> {
        // Create orchestration tasks for each execution step
        for step in &task_subject.deliver.execution_plan {
            let task = Task {
                id: step.step_id,
                name: step.name.clone(),
                description: step.description.clone(),
                task_type: self.determine_task_type(&step.name).await?,
                priority: self.convert_priority(&task_subject.priority).await?,
                required_capabilities: self.extract_required_capabilities(&step.description).await?,
                parameters: self.generate_task_parameters(step).await?,
                deadline: Some(Utc::now() + step.estimated_duration),
                created_at: Utc::now(),
                status: TaskStatus::Pending,
                assigned_agent: None,
                result: None,
                metadata: HashMap::new(),
            };

            // Submit task for orchestration
            let task_id = self.task_protocol.submit_task(task).await?;
            
            // Send notification to communication protocol
            let message = AgentMessage::TaskAssignment {
                id: uuid::Uuid::new_v4(),
                task_id,
                from: AgentId::new(), // System agent
                to: AgentId::new(),   // Will be assigned by orchestrator
                task: task,
                deadline: Some(Utc::now() + step.estimated_duration),
            };
            
            self.communication_protocol.send_message(message).await?;
        }

        Ok(())
    }

    // Implementation helper methods
    async fn extract_executed_tasks(&self, sot_content: &str) -> Result<Vec<String>> {
        // Parse executed tasks from SOT markdown content
        // Look for "### 1.1 Executed Tasks (Chronological)" section
        let lines: Vec<&str> = sot_content.lines().collect();
        let mut executed_tasks = Vec::new();
        let mut in_executed_section = false;
        
        for line in lines {
            if line.contains("### 1.1 Executed Tasks") {
                in_executed_section = true;
                continue;
            }
            if line.starts_with("### ") && in_executed_section {
                break;
            }
            if in_executed_section && line.starts_with("- [x]") {
                executed_tasks.push(line.to_string());
            }
        }
        
        Ok(executed_tasks)
    }

    async fn extract_in_progress_tasks(&self, sot_content: &str) -> Result<Vec<String>> {
        // Parse in-progress tasks from SOT content
        let lines: Vec<&str> = sot_content.lines().collect();
        let mut in_progress_tasks = Vec::new();
        let mut in_progress_section = false;
        
        for line in lines {
            if line.contains("### 1.2 In-Progress Tasks") {
                in_progress_section = true;
                continue;
            }
            if line.starts_with("### ") && in_progress_section {
                break;
            }
            if in_progress_section && line.starts_with("- [") {
                in_progress_tasks.push(line.to_string());
            }
        }
        
        Ok(in_progress_tasks)
    }

    async fn extract_system_constraints(&self, sot_content: &str) -> Result<Vec<String>> {
        // Extract system constraints and metadata
        let mut constraints = Vec::new();
        
        // Add standard constraints from production structure preference
        constraints.push("Primary production codebase must reside in agentaskit-production directory".to_string());
        constraints.push("All artifacts must be organized in ~/docs subdirectory".to_string());
        constraints.push("Only sot.md allowed at root level".to_string());
        constraints.push("Triple-verification protocol mandatory for all claims".to_string());
        constraints.push("Heal, Don't Harm principle must be followed".to_string());
        
        Ok(constraints)
    }

    // Additional helper method implementations would continue here...
    // [Implementation continues with remaining helper methods]
}

// Supporting structures and traits
#[derive(Debug, Clone)]
pub struct SOTAnalysis {
    pub executed_tasks: Vec<String>,
    pub in_progress_tasks: Vec<String>,  
    pub system_constraints: Vec<String>,
    pub last_updated: DateTime<Utc>,
    pub request_alignment: f32, // 0.0 to 1.0 score
}

// Re-export for external use
pub use crate::agents::{Task, TaskStatus, AgentId, AgentMessage};