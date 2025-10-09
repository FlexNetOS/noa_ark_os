//! Board Finance Agent - Financial Oversight
//! 
//! Simplified working version
//! Monitors financial performance and provides oversight

use crate::unified_types::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Board Finance Agent - Financial oversight and reporting
/// 
/// Responsible for:
/// - Financial performance monitoring
/// - Budget oversight and compliance
/// - Financial risk assessment
/// - Revenue and expense tracking
pub struct FinanceAgent {
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
    financial_data: Arc<RwLock<FinancialData>>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceAgentConfig {
    pub reporting_interval: u64, // seconds
    pub risk_threshold: f64,
}

impl Default for FinanceAgentConfig {
    fn default() -> Self {
        Self {
            reporting_interval: 3600, // Hourly
            risk_threshold: 0.75,
        }
    }
}

/// Financial data
#[derive(Debug, Default)]
struct FinancialData {
    metrics: FinancialMetrics,
    budget: BudgetInfo,
    risks: Vec<FinancialRisk>,
}

/// Financial metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
    pub revenue: f64,
    pub expenses: f64,
    pub profit_margin: f64,
    pub cash_flow: f64,
    pub budget_utilization: f64,
}

/// Budget information
#[derive(Debug, Default, Clone)]
struct BudgetInfo {
    pub total_budget: f64,
    pub allocated: f64,
    pub spent: f64,
    pub remaining: f64,
}

/// Financial risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialRisk {
    pub risk_id: Uuid,
    pub risk_type: String,
    pub severity: f64,
    pub description: String,
}

/// Financial report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReport {
    pub report_id: Uuid,
    pub metrics: FinancialMetrics,
    pub summary: String,
    pub risks: Vec<FinancialRisk>,
    pub recommendations: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl FinanceAgent {
    pub fn new() -> Self {
        let metadata = AgentMetadata {
            id: Uuid::new_v4(),
            agent_id: "finance-agent".to_string(),
            name: "Finance Board Agent".to_string(),
            layer: AgentLayer::L2Reasoning,
            category: AgentCategory::Governance,
            agent_type: AgentType::Master,
            language: AgentLanguage::Rust,
            description: "Financial Oversight Agent - Budget and performance monitoring".to_string(),
            role: "Board Finance".to_string(),
            purpose: "Monitor financial performance and ensure fiscal responsibility".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: Some("noa-commander".to_string()),
            stack: None,
            capabilities: vec![
                "financial-monitoring".to_string(),
                "budget-oversight".to_string(),
                "risk-assessment".to_string(),
                "financial-reporting".to_string(),
            ],
            tools: vec![],
            tags: vec!["board".to_string(), "finance".to_string()],
            inputs: vec!["financial-data".to_string()],
            outputs: vec!["financial-report".to_string()],
            dependencies: vec![],
            cpu_min: "0.5".to_string(),
            ram_min: "512MB".to_string(),
            disk_min: "100MB".to_string(),
            autonomy_level: "guided".to_string(),
            disposable: false,
            issues_identified: vec![],
            repair_recommendations: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            version: Some("1.0.0".to_string()),
        };
        
        Self {
            metadata,
            state: RwLock::new(AgentState::Created),
            financial_data: Arc::new(RwLock::new(FinancialData::default())),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        *self.state.write().await = AgentState::Initializing;
        
        // Initialize financial tracking
        let mut data = self.financial_data.write().await;
        data.budget = BudgetInfo {
            total_budget: 1000000.0,
            allocated: 800000.0,
            spent: 600000.0,
            remaining: 400000.0,
        };
        
        *self.state.write().await = AgentState::Ready;
        tracing::info!("Finance Agent initialized");
        Ok(())
    }
    
    pub async fn generate_report(&self) -> Result<FinancialReport> {
        let data = self.financial_data.read().await;
        
        Ok(FinancialReport {
            report_id: Uuid::new_v4(),
            metrics: data.metrics.clone(),
            summary: "Financial performance is stable".to_string(),
            risks: data.risks.clone(),
            recommendations: vec!["Continue monitoring budget utilization".to_string()],
            generated_at: chrono::Utc::now(),
        })
    }
    
    pub fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }
    
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }
}

impl Default for FinanceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_finance_agent() {
        let agent = FinanceAgent::new();
        assert_eq!(agent.metadata().name, "Finance Board Agent");
    }
    
    #[tokio::test]
    async fn test_initialize() {
        let mut agent = FinanceAgent::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }
    
    #[tokio::test]
    async fn test_generate_report() {
        let mut agent = FinanceAgent::new();
        agent.initialize().await.unwrap();
        let report = agent.generate_report().await.unwrap();
        assert!(!report.summary.is_empty());
    }
}
