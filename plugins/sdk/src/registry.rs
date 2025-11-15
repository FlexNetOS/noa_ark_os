use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Canonical registry describing every CLI/automation tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistry {
    pub schema_version: String,
    pub generated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolDescriptor>,
}

impl ToolRegistry {
    /// Load a registry definition from disk.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let mut file = File::open(path_ref)
            .with_context(|| format!("failed to open tool registry at {}", path_ref.display()))?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .with_context(|| format!("failed to read tool registry at {}", path_ref.display()))?;
        let registry: ToolRegistry = serde_json::from_str(&buffer).with_context(|| {
            format!(
                "failed to parse tool registry JSON from {}. Ensure it matches the noa-plugin-sdk schema",
                path_ref.display()
            )
        })?;
        Ok(registry)
    }

    /// Return all tools scoped to the provided category label.
    pub fn tools_for_category<'a>(&'a self, category: &str) -> Vec<&'a ToolDescriptor> {
        let normalized = category.to_ascii_lowercase();
        self.tools
            .iter()
            .filter(|tool| tool.category.eq_ignore_ascii_case(&normalized))
            .collect()
    }

    /// Find a tool by id, alias, or CLI command string.
    pub fn find_tool<'a>(&'a self, reference: &str) -> Option<&'a ToolDescriptor> {
        let reference_lower = reference.to_ascii_lowercase();
        self.tools.iter().find(|tool| {
            tool.id.to_ascii_lowercase() == reference_lower
                || tool.aliases.iter().any(|alias| alias.to_ascii_lowercase() == reference_lower)
                || tool
                    .cli_mappings
                    .iter()
                    .any(|mapping| mapping.command.join(" ").to_ascii_lowercase() == reference_lower)
        })
    }

    /// Validate that the registry contains at least one entry for each category provided.
    pub fn ensure_categories(&self, categories: &[&str]) -> Result<()> {
        for category in categories {
            if self.tools_for_category(category).is_empty() {
                anyhow::bail!("tool registry missing required category: {}", category);
            }
        }
        Ok(())
    }
}

/// A single tool capability exposed by the NOA toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub id: String,
    pub name: String,
    pub category: String,
    pub layer: String,
    pub summary: String,
    pub description: String,
    pub owner: String,
    pub maturity: String,
    pub lifecycle: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<ParameterSpec>,
    #[serde(default)]
    pub side_effects: Vec<SideEffectSpec>,
    pub budgets: BudgetSpec,
    #[serde(default)]
    pub outputs: Vec<OutputSpec>,
    #[serde(default)]
    pub cli_mappings: Vec<CliMapping>,
    pub api: ApiSurface,
    pub automation: AutomationSurface,
}

impl ToolDescriptor {
    /// Determine whether the descriptor belongs to a category label (case-insensitive).
    pub fn is_category(&self, category: &str) -> bool {
        self.category.eq_ignore_ascii_case(category)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub description: String,
    pub data_type: String,
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffectSpec {
    pub impact: String,
    pub scope: String,
    pub description: String,
    #[serde(default)]
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSpec {
    pub cpu_millis: u32,
    pub memory_mebibytes: u32,
    pub storage_mebibytes: u32,
    pub max_duration_seconds: u32,
    pub network_class: NetworkClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkClass {
    Offline,
    Internal,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    pub format: String,
    pub description: String,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliMapping {
    pub command: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSurface {
    pub grpc: String,
    pub rest: RestSurface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestSurface {
    pub method: String,
    pub path: String,
    pub operation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationSurface {
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub approvals_required: Vec<String>,
    #[serde(default)]
    pub runbooks: Vec<String>,
}
