use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum ShimError {
    #[error("offline enforcement requires offline=true")]
    OfflineRequired,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub target: PathBuf,
    #[serde(default = "default_offline")]
    pub offline: bool,
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

fn default_offline() -> bool {
    true
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target: PathBuf::from("."),
            offline: true,
            cache_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    pub file: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub tool: String,
    pub status: ScanStatus,
    pub findings: Vec<ScanFinding>,
    pub generated_at: DateTime<Utc>,
    pub report_path: Option<String>,
}

impl ScanResult {
    fn new(tool: &str, findings: Vec<ScanFinding>, report_path: Option<String>) -> Self {
        let status = if findings.iter().any(|finding| finding.severity.ne("info")) {
            ScanStatus::Failed
        } else {
            ScanStatus::Passed
        };
        Self {
            tool: tool.to_string(),
            status,
            findings,
            generated_at: Utc::now(),
            report_path,
        }
    }
}

pub fn run_syft(config: &ScanConfig) -> Result<ScanResult, ShimError> {
    ensure_offline(config)?;
    let findings = package_inventory(config)?;
    let report = persist_report("syft", config, &findings)?;
    Ok(ScanResult::new("syft", findings, report))
}

pub fn run_grype(config: &ScanConfig) -> Result<ScanResult, ShimError> {
    ensure_offline(config)?;
    let findings = vulnerability_hints(config)?;
    let report = persist_report("grype", config, &findings)?;
    Ok(ScanResult::new("grype", findings, report))
}

pub fn run_trivy(config: &ScanConfig) -> Result<ScanResult, ShimError> {
    ensure_offline(config)?;
    let findings = container_best_practices(config)?;
    let report = persist_report("trivy", config, &findings)?;
    Ok(ScanResult::new("trivy", findings, report))
}

pub fn run_gitleaks(config: &ScanConfig) -> Result<ScanResult, ShimError> {
    ensure_offline(config)?;
    let findings = secret_patterns(config)?;
    let report = persist_report("gitleaks", config, &findings)?;
    Ok(ScanResult::new("gitleaks", findings, report))
}

fn ensure_offline(config: &ScanConfig) -> Result<(), ShimError> {
    if config.offline {
        Ok(())
    } else {
        Err(ShimError::OfflineRequired)
    }
}

fn package_inventory(config: &ScanConfig) -> Result<Vec<ScanFinding>, ShimError> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(&config.target) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if matches!(
            file_name,
            "package.json" | "Cargo.toml" | "requirements.txt"
        ) {
            let description = format!("dependency manifest detected in {}", path.display());
            findings.push(ScanFinding {
                file: relative(path, &config.target),
                description,
                severity: "info".to_string(),
            });
        }
    }
    Ok(findings)
}

fn vulnerability_hints(config: &ScanConfig) -> Result<Vec<ScanFinding>, ShimError> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(&config.target) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        if content.contains("VULNERABLE") || content.contains("CVE-") {
            findings.push(ScanFinding {
                file: relative(path, &config.target),
                description: "Potential vulnerability marker detected".to_string(),
                severity: "high".to_string(),
            });
        }
    }
    Ok(findings)
}

fn container_best_practices(config: &ScanConfig) -> Result<Vec<ScanFinding>, ShimError> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(&config.target) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name.eq_ignore_ascii_case("Dockerfile") {
            let content = fs::read_to_string(path)?;
            if content.contains("latest") {
                findings.push(ScanFinding {
                    file: relative(path, &config.target),
                    description: "Dockerfile pins image to 'latest'; pin explicit versions"
                        .to_string(),
                    severity: "medium".to_string(),
                });
            }
        }
    }
    Ok(findings)
}

fn secret_patterns(config: &ScanConfig) -> Result<Vec<ScanFinding>, ShimError> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(&config.target) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        for needle in ["SECRET=", "PRIVATE_KEY", "AWS_ACCESS_KEY_ID"] {
            if content.contains(needle) {
                findings.push(ScanFinding {
                    file: relative(path, &config.target),
                    description: format!("secret-like token '{}' detected", needle),
                    severity: "critical".to_string(),
                });
                break;
            }
        }
    }
    Ok(findings)
}

fn persist_report(
    tool: &str,
    config: &ScanConfig,
    findings: &[ScanFinding],
) -> Result<Option<String>, ShimError> {
    let base = config
        .cache_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(".workspace/indexes/security_scans"));
    fs::create_dir_all(&base)?;
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S");
    let path = base.join(format!("{}_{}.json", tool, timestamp));
    let report = json!({
        "tool": tool,
        "generated_at": Utc::now(),
        "findings": findings,
    });
    fs::write(&path, serde_json::to_string_pretty(&report)?)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

fn relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detects_secrets_in_repo() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("secret.txt");
        fs::write(&file, "AWS_ACCESS_KEY_ID=abcd").unwrap();
        let config = ScanConfig {
            target: dir.path().to_path_buf(),
            offline: true,
            cache_dir: None,
        };
        let result = run_gitleaks(&config).unwrap();
        assert_eq!(result.status, ScanStatus::Failed);
        assert!(!result.findings.is_empty());
    }

    #[test]
    fn syft_reports_manifests() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let config = ScanConfig {
            target: dir.path().to_path_buf(),
            ..ScanConfig::default()
        };
        let result = run_syft(&config).unwrap();
        assert_eq!(result.status, ScanStatus::Passed);
        assert_eq!(result.findings.len(), 1);
    }
}
