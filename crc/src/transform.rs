use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub trait TransformPlan: Send + Sync {
    fn identifier(&self) -> &str;
    fn describe(&self) -> String;
    fn generate_diff(&self, root: &Path) -> Result<DiffArtifact>;
    fn apply(&self, root: &Path) -> Result<()>;
    fn rollback(&self, root: &Path) -> Result<()>;
}

pub trait Verifier: Send + Sync {
    fn name(&self) -> &str;
    fn verify(&self, root: &Path) -> Result<VerifierOutcome>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffArtifact {
    pub plan_id: String,
    pub diff: String,
    pub risk: RiskGrade,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskGrade {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierOutcome {
    pub name: String,
    pub passed: bool,
    pub details: Option<String>,
}

pub struct FileReplacePlan {
    pub id: String,
    pub target: PathBuf,
    pub replacement: String,
}

impl FileReplacePlan {
    pub fn new(
        id: impl Into<String>,
        target: impl Into<PathBuf>,
        replacement: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            replacement: replacement.into(),
        }
    }
}

impl TransformPlan for FileReplacePlan {
    fn identifier(&self) -> &str {
        &self.id
    }

    fn describe(&self) -> String {
        format!("replace {:?}", self.target)
    }

    fn generate_diff(&self, root: &Path) -> Result<DiffArtifact> {
        let path = root.join(&self.target);
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", path.display()));
        diff.push_str(&format!("+++ {}\n", path.display()));
        for line in existing.lines() {
            diff.push_str(&format!("-{}\n", line));
        }
        for line in self.replacement.lines() {
            diff.push_str(&format!("+{}\n", line));
        }
        Ok(DiffArtifact {
            plan_id: self.id.clone(),
            diff,
            risk: RiskGrade::Medium,
        })
    }

    fn apply(&self, root: &Path) -> Result<()> {
        let path = root.join(&self.target);
        std::fs::write(&path, &self.replacement)?;
        Ok(())
    }

    fn rollback(&self, root: &Path) -> Result<()> {
        let path = root.join(&self.target);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

pub struct DummyVerifier;

impl Verifier for DummyVerifier {
    fn name(&self) -> &str {
        "dummy"
    }

    fn verify(&self, _root: &Path) -> Result<VerifierOutcome> {
        Ok(VerifierOutcome {
            name: self.name().into(),
            passed: true,
            details: Some("no-op".into()),
        })
    }
}

pub fn execute_plan(
    plan: &dyn TransformPlan,
    verifiers: &[Box<dyn Verifier>],
    root: &Path,
    apply: bool,
) -> Result<Vec<VerifierOutcome>> {
    let diff = plan.generate_diff(root)?;
    if diff.risk == RiskGrade::High && apply {
        return Err(anyhow!("high risk diff requires override"));
    }
    if apply {
        plan.apply(root)?;
    }
    let mut outcomes = Vec::new();
    for verifier in verifiers {
        outcomes.push(verifier.verify(root)?);
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_verifier_passes() {
        let verifier = DummyVerifier;
        let result = verifier.verify(Path::new("."));
        assert!(result.unwrap().passed);
    }
}
