//! Specialist Agents Module
//!
//! Specialist-level agents (L4 Operations layer)
//! Domain-specific expertise and task execution

pub mod code_generation;
pub mod data_analytics;
pub mod deployment;
pub mod integration;
pub mod learning;
pub mod monitoring;
pub mod policy_enforcement;
pub mod security;
pub mod testing;

// Re-export for convenience
pub use code_generation::CodeGenerationAgent;
pub use data_analytics::DataAnalyticsAgent;
pub use deployment::DeploymentAgent;
pub use integration::IntegrationAgent;
pub use learning::LearningAgent;
pub use monitoring::MonitoringAgent;
pub use policy_enforcement::PolicyEnforcementAgent;
pub use security::SecurityAgent;
pub use testing::TestingAgent;
