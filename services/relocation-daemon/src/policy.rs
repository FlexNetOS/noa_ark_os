use std::collections::HashSet;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::registry::FileEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDocument {
    pub version: String,
    #[serde(default)]
    pub defaults: PolicyDefaults,
    #[serde(default)]
    pub relocations: Vec<RelocationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyDefaults {
    #[serde(default)]
    pub retention: RetentionPolicy,
    #[serde(default)]
    pub naming: NamingRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionPolicy {
    pub max_versions: Option<usize>,
    pub ttl_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRules {
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(flatten)]
    pub pattern: NamingPattern,
}

impl Default for NamingRules {
    fn default() -> Self {
        Self {
            prefix: None,
            suffix: None,
            pattern: NamingPattern::Preserve,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum NamingPattern {
    Preserve,
    Timestamp { format: Option<String> },
    Incremental { width: Option<u32> },
}

impl NamingRules {
    pub fn format_name(&self, file_name: &str, counter: u64, now: DateTime<Utc>) -> String {
        let core = match &self.pattern {
            NamingPattern::Preserve => file_name.to_string(),
            NamingPattern::Timestamp { format } => {
                let fmt = format.as_deref().unwrap_or("%Y%m%dT%H%M%SZ");
                now.format(fmt).to_string()
            }
            NamingPattern::Incremental { width } => match width {
                Some(width) if *width > 0 => {
                    format!("{:0width$}", counter, width = *width as usize)
                }
                _ => counter.to_string(),
            },
        };

        format!(
            "{}{}{}",
            self.prefix.as_deref().unwrap_or(""),
            core,
            self.suffix.as_deref().unwrap_or("")
        )
    }

    pub fn uses_counter(&self) -> bool {
        matches!(self.pattern, NamingPattern::Incremental { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleFilters {
    pub min_size: Option<u64>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationRule {
    pub name: String,
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default)]
    pub retention: Option<RetentionPolicy>,
    #[serde(default)]
    pub naming: Option<NamingRules>,
    #[serde(default)]
    pub filters: RuleFilters,
}

#[derive(Debug, Clone)]
pub struct RuntimePolicy {
    pub defaults: PolicyDefaults,
    pub rules: Vec<RuntimeRule>,
}

#[derive(Debug, Clone)]
pub struct RuntimeRule {
    pub name: String,
    pub matcher: PathPattern,
    pub destination: String,
    pub requires_approval: bool,
    pub retention: Option<RetentionPolicy>,
    pub naming: Option<NamingRules>,
    pub filters: RuleFilters,
}

impl PolicyDocument {
    pub fn into_runtime(self) -> Result<RuntimePolicy> {
        if self.version.trim().is_empty() {
            return Err(anyhow!("policy version must be provided"));
        }

        let mut rules = Vec::with_capacity(self.relocations.len());
        for rule in self.relocations {
            let matcher = PathPattern::new(&rule.source)
                .with_context(|| format!("invalid glob expression in rule {}", rule.name))?;

            rules.push(RuntimeRule {
                name: rule.name,
                matcher,
                destination: rule.destination,
                requires_approval: rule.requires_approval,
                retention: rule.retention,
                naming: rule.naming,
                filters: rule.filters,
            });
        }

        Ok(RuntimePolicy {
            defaults: self.defaults,
            rules,
        })
    }
}

impl RuntimeRule {
    pub fn matches(&self, entry: &FileEntry) -> bool {
        if !self.matcher.is_match(entry.path()) {
            return false;
        }

        if let Some(min_size) = self.filters.min_size {
            if entry.size < min_size {
                return false;
            }
        }

        if !self.filters.labels.is_empty() {
            let labels: HashSet<&str> = entry.labels.iter().map(|s| s.as_str()).collect();
            if !self
                .filters
                .labels
                .iter()
                .all(|required| labels.contains(required.as_str()))
            {
                return false;
            }
        }

        true
    }

    pub fn naming<'a>(&'a self, defaults: &'a NamingRules) -> &'a NamingRules {
        self.naming.as_ref().unwrap_or(defaults)
    }

    pub fn retention<'a>(&'a self, defaults: &'a RetentionPolicy) -> &'a RetentionPolicy {
        self.retention.as_ref().unwrap_or(defaults)
    }

    pub fn file_name(&self, entry: &FileEntry) -> Option<String> {
        Path::new(entry.path())
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PathPattern {
    original: String,
}

impl PathPattern {
    pub fn new(pattern: &str) -> Result<Self> {
        if pattern.is_empty() {
            return Err(anyhow!("pattern cannot be empty"));
        }
        Ok(Self {
            original: pattern.to_string(),
        })
    }

    pub fn is_match(&self, candidate: &str) -> bool {
        wildcard_match(self.original.as_bytes(), candidate.as_bytes())
    }
}

fn wildcard_match(pattern: &[u8], text: &[u8]) -> bool {
    let mut p = 0;
    let mut t = 0;
    let mut star_idx: Option<usize> = None;
    let mut match_idx = 0;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star_idx = Some(p);
            match_idx = t;
            p += 1;
        } else if let Some(star_pos) = star_idx {
            p = star_pos + 1;
            match_idx += 1;
            t = match_idx;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}

impl RuntimePolicy {
    pub fn rules(&self) -> &[RuntimeRule] {
        &self.rules
    }

    pub fn defaults(&self) -> &PolicyDefaults {
        &self.defaults
    }
}
