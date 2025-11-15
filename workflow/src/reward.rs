use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RewardError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardInputs {
    pub coverage: f64,
    pub flake_rate: f64,
    pub token_ratio: f64,
    pub rollback_count: u32,
}

impl Default for RewardInputs {
    fn default() -> Self {
        Self {
            coverage: 1.0,
            flake_rate: 0.0,
            token_ratio: 1.0,
            rollback_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardAgentSnapshot {
    pub agent: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardDelta {
    pub timestamp: String,
    pub workflow_id: String,
    pub goal_id: String,
    pub coverage_delta: f64,
    pub flake_delta: f64,
    pub token_delta: f64,
    pub rollback_delta: f64,
    pub total_reward: f64,
    pub inputs: RewardInputs,
    pub agents: Vec<RewardAgentSnapshot>,
}

impl RewardDelta {
    fn new(
        workflow_id: &str,
        goal_id: &str,
        inputs: RewardInputs,
        agents: Vec<RewardAgentSnapshot>,
        config: &RewardConfig,
    ) -> Self {
        let coverage_delta = ((inputs.coverage - config.coverage_target) * config.coverage_weight)
            .clamp(-config.coverage_weight, config.coverage_weight);
        let flake_delta = ((config.flake_target - inputs.flake_rate) * config.flake_weight)
            .clamp(-config.flake_weight, config.flake_weight);
        let token_delta = ((config.token_target - inputs.token_ratio) * config.token_weight)
            .clamp(-config.token_weight, config.token_weight);
        let rollback_delta = -((inputs.rollback_count as f64) * config.rollback_weight);
        let total_reward = coverage_delta + flake_delta + token_delta + rollback_delta;

        Self {
            timestamp: Utc::now().to_rfc3339(),
            workflow_id: workflow_id.to_string(),
            goal_id: goal_id.to_string(),
            coverage_delta,
            flake_delta,
            token_delta,
            rollback_delta,
            total_reward,
            inputs,
            agents,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentStanding {
    pub total_reward: f64,
    pub recent_rewards: VecDeque<f64>,
    pub penalties: u32,
    pub last_update: String,
}

impl Default for AgentStanding {
    fn default() -> Self {
        Self {
            total_reward: 0.0,
            recent_rewards: VecDeque::new(),
            penalties: 0,
            last_update: "1970-01-01T00:00:00Z".to_string(),
        }
    }
}

impl AgentStanding {
    pub fn recent_average(&self) -> f64 {
        if self.recent_rewards.is_empty() {
            0.0
        } else {
            self.recent_rewards.iter().copied().sum::<f64>() / self.recent_rewards.len() as f64
        }
    }

    fn push_reward(&mut self, reward: f64, window: usize) {
        self.total_reward += reward;
        if reward < 0.0 {
            self.penalties = self.penalties.saturating_add(1);
        }
        if window > 0 {
            if self.recent_rewards.len() == window {
                self.recent_rewards.pop_front();
            }
            self.recent_rewards.push_back(reward);
        }
        self.last_update = Utc::now().to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardConfig {
    pub coverage_target: f64,
    pub flake_target: f64,
    pub token_target: f64,
    pub rollback_weight: f64,
    pub coverage_weight: f64,
    pub flake_weight: f64,
    pub token_weight: f64,
    pub failure_penalty: f64,
    pub gating_threshold: f64,
    pub gating_recent_threshold: f64,
    pub trailing_window: usize,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            coverage_target: 0.85,
            flake_target: 0.02,
            token_target: 1.0,
            rollback_weight: 1.25,
            coverage_weight: 2.5,
            flake_weight: 1.5,
            token_weight: 1.0,
            failure_penalty: 1.0,
            gating_threshold: -5.0,
            gating_recent_threshold: -0.5,
            trailing_window: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentStandingSummary {
    pub agent: String,
    pub total_reward: f64,
    pub recent_average: f64,
    pub penalties: u32,
    pub requires_manual_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardReport {
    pub total_entries: usize,
    pub average_reward: f64,
    pub coverage_average: f64,
    pub flake_average: f64,
    pub token_average: f64,
    pub rollback_average: f64,
    pub best_agents: Vec<AgentStandingSummary>,
    pub riskiest_agents: Vec<AgentStandingSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentApprovalStatus {
    pub agent: String,
    pub requires_manual_approval: bool,
    pub reason: Option<String>,
    pub standing: AgentStanding,
}

impl AgentApprovalStatus {
    pub fn approved(&self) -> bool {
        !self.requires_manual_approval
    }
}

#[derive(Debug, Clone)]
pub struct RewardScorekeeper {
    config: RewardConfig,
    history_path: PathBuf,
    history: Vec<RewardDelta>,
    standings: HashMap<String, AgentStanding>,
}

impl RewardScorekeeper {
    pub fn new(history_path: PathBuf) -> Result<Self, RewardError> {
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !history_path.exists() {
            fs::write(&history_path, "[]")?;
        }
        let raw = fs::read_to_string(&history_path)?;
        let history = if raw.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str::<Vec<RewardDelta>>(&raw)?
        };
        let mut scorekeeper = Self {
            config: RewardConfig::default(),
            history_path,
            history,
            standings: HashMap::new(),
        };
        scorekeeper.rebuild_standings();
        Ok(scorekeeper)
    }

    pub fn record(
        &mut self,
        goal_id: &str,
        workflow_id: &str,
        inputs: RewardInputs,
        agents: &[RewardAgentSnapshot],
    ) -> RewardDelta {
        let delta = RewardDelta::new(workflow_id, goal_id, inputs, agents.to_vec(), &self.config);
        self.history.push(delta.clone());
        self.update_standings(&delta);
        delta
    }

    pub fn history(&self) -> &[RewardDelta] {
        &self.history
    }

    pub fn standings(&self) -> &HashMap<String, AgentStanding> {
        &self.standings
    }

    pub fn save(&self) -> Result<(), RewardError> {
        let payload = serde_json::to_string_pretty(&self.history)?;
        fs::write(&self.history_path, payload)?;
        Ok(())
    }

    pub fn requires_manual_approval(&self, agent: &str) -> bool {
        let standing = self.standings.get(agent).cloned().unwrap_or_default();
        self.requires_manual_approval_for(&standing)
    }

    pub fn approval_status(&self, agent: &str) -> AgentApprovalStatus {
        let standing = self.standings.get(agent).cloned().unwrap_or_default();
        let requires_manual_approval = self.requires_manual_approval_for(&standing);
        let reason = if requires_manual_approval {
            Some(format!(
                "Reward total {:.2} or recent trend {:.2} below threshold",
                standing.total_reward,
                standing.recent_average()
            ))
        } else {
            None
        };
        AgentApprovalStatus {
            agent: agent.to_string(),
            requires_manual_approval,
            reason,
            standing,
        }
    }

    pub fn generate_report(&self) -> RewardReport {
        let mut reward_sum = 0.0;
        let mut coverage_sum = 0.0;
        let mut flake_sum = 0.0;
        let mut token_sum = 0.0;
        let mut rollback_sum = 0.0;

        for delta in &self.history {
            reward_sum += delta.total_reward;
            coverage_sum += delta.inputs.coverage;
            flake_sum += delta.inputs.flake_rate;
            token_sum += delta.inputs.token_ratio;
            rollback_sum += delta.inputs.rollback_count as f64;
        }

        let entries = self.history.len();
        let divisor = entries.max(1) as f64;

        let mut summaries: Vec<AgentStandingSummary> = self
            .standings
            .iter()
            .map(|(agent, standing)| AgentStandingSummary {
                agent: agent.clone(),
                total_reward: standing.total_reward,
                recent_average: standing.recent_average(),
                penalties: standing.penalties,
                requires_manual_approval: self.requires_manual_approval_for(standing),
            })
            .collect();

        summaries.sort_by(|a, b| {
            b.total_reward
                .partial_cmp(&a.total_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_agents = summaries.iter().take(5).cloned().collect();

        let mut riskiest = summaries;
        riskiest.sort_by(|a, b| {
            a.total_reward
                .partial_cmp(&b.total_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let riskiest_agents = riskiest.iter().take(5).cloned().collect();

        RewardReport {
            total_entries: entries,
            average_reward: reward_sum / divisor,
            coverage_average: coverage_sum / divisor,
            flake_average: flake_sum / divisor,
            token_average: token_sum / divisor,
            rollback_average: rollback_sum / divisor,
            best_agents,
            riskiest_agents,
        }
    }

    pub fn flagged_agents(&self) -> Vec<AgentStandingSummary> {
        self.standings
            .iter()
            .filter_map(|(agent, standing)| {
                if self.requires_manual_approval_for(standing) {
                    Some(AgentStandingSummary {
                        agent: agent.clone(),
                        total_reward: standing.total_reward,
                        recent_average: standing.recent_average(),
                        penalties: standing.penalties,
                        requires_manual_approval: true,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn rebuild_standings(&mut self) {
        self.standings.clear();
        for delta in &self.history {
            self.update_standings(delta);
        }
    }

    fn update_standings(&mut self, delta: &RewardDelta) {
        if delta.agents.is_empty() {
            return;
        }
        let agent_count = delta.agents.len() as f64;
        let shared_reward = delta.total_reward / agent_count;
        for agent in &delta.agents {
            let entry = self
                .standings
                .entry(agent.agent.clone())
                .or_insert_with(AgentStanding::default);
            let mut reward = shared_reward;
            if !agent.success {
                reward -= self.config.failure_penalty;
            }
            entry.push_reward(reward, self.config.trailing_window);
        }
    }

    fn requires_manual_approval_for(&self, standing: &AgentStanding) -> bool {
        standing.total_reward < self.config.gating_threshold
            || standing.recent_average() < self.config.gating_recent_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_agents(success: bool) -> Vec<RewardAgentSnapshot> {
        vec![RewardAgentSnapshot {
            agent: "agent-a".to_string(),
            success,
        }]
    }

    #[test]
    fn penalises_flaky_runs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("reward_history.json");
        let mut keeper = RewardScorekeeper::new(path).unwrap();

        let good_inputs = RewardInputs {
            coverage: 0.94,
            flake_rate: 0.01,
            token_ratio: 0.8,
            rollback_count: 0,
        };
        keeper.record("goal", "wf", good_inputs, &sample_agents(true));
        assert!(!keeper.requires_manual_approval("agent-a"));

        let bad_inputs = RewardInputs {
            coverage: 0.42,
            flake_rate: 0.55,
            token_ratio: 1.7,
            rollback_count: 2,
        };
        keeper.record("goal", "wf", bad_inputs.clone(), &sample_agents(false));
        keeper.record("goal", "wf", bad_inputs, &sample_agents(false));
        assert!(keeper.requires_manual_approval("agent-a"));
    }

    #[test]
    fn improvements_clear_penalties() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("reward_history.json");
        let mut keeper = RewardScorekeeper::new(path).unwrap();

        let bad_inputs = RewardInputs {
            coverage: 0.3,
            flake_rate: 0.6,
            token_ratio: 1.8,
            rollback_count: 3,
        };
        keeper.record("goal", "wf", bad_inputs.clone(), &sample_agents(false));
        keeper.record("goal", "wf", bad_inputs, &sample_agents(false));
        assert!(keeper.requires_manual_approval("agent-a"));

        let good_inputs = RewardInputs {
            coverage: 0.95,
            flake_rate: 0.01,
            token_ratio: 0.7,
            rollback_count: 0,
        };
        for _ in 0..10 {
            keeper.record("goal", "wf", good_inputs.clone(), &sample_agents(true));
        }
        assert!(!keeper.requires_manual_approval("agent-a"));

        let report = keeper.generate_report();
        assert!(report.total_entries >= 8);
        assert!(report.best_agents.iter().any(|a| a.agent == "agent-a"));
    }
}
