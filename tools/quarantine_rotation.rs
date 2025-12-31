use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const DEFAULT_QUARANTINE_ROOT: &str = "archive/quarantine";
const DEFAULT_ARCHIVE_ROOT: &str = "archive";
const LEDGER_NAME: &str = "ledger.json";

#[derive(Debug, Deserialize)]
struct StatusMetadata {
    #[serde(default)]
    quarantined_at: Option<String>,
}

pub fn run() -> Result<()> {
    let quarantine_root = env::var("QUARANTINE_ROOT").unwrap_or_else(|_| DEFAULT_QUARANTINE_ROOT.to_string());
    let archive_root = env::var("ARCHIVE_ROOT").unwrap_or_else(|_| DEFAULT_ARCHIVE_ROOT.to_string());

    let root_path = Path::new(&quarantine_root);
    if !root_path.exists() {
        return Ok(());
    }

    let mut relocated = 0usize;
    for entry in fs::read_dir(root_path).with_context(|| format!("failed to read {quarantine_root}"))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let bundle_path = entry.path();
        if should_skip_bundle(&bundle_path)? {
            continue;
        }
        if relocate_bundle(&bundle_path, &archive_root, &quarantine_root)? {
            relocated += 1;
        }
    }

    if relocated > 0 {
        println!("Relocated {relocated} quarantine bundle(s).");
    }

    Ok(())
}

fn should_skip_bundle(bundle_path: &Path) -> Result<bool> {
    let status = load_status(bundle_path)?;
    let Some(status) = status else { return Ok(true); };
    let Some(quarantined_at) = status.quarantined_at else {
        return Ok(true);
    };
    let parsed = chrono::DateTime::parse_from_rfc3339(&quarantined_at)
        .or_else(|_| chrono::DateTime::parse_from_str(&quarantined_at, "%Y-%m-%dT%H:%M:%SZ"))
        .with_context(|| format!("unable to parse quarantined_at in {}", bundle_path.display()))?;
    let age = Utc::now() - parsed.with_timezone(&Utc);
    Ok(age < Duration::days(90))
}

fn load_status(bundle_path: &Path) -> Result<Option<StatusMetadata>> {
    let status_path = bundle_path.join("status.yaml");
    if !status_path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&status_path)
        .with_context(|| format!("failed to read {}", status_path.display()))?;
    let parsed: StatusMetadata = serde_yaml::from_str(&data)
        .with_context(|| format!("invalid YAML in {}", status_path.display()))?;
    Ok(Some(parsed))
}

fn relocate_bundle(bundle_path: &Path, archive_root: &str, quarantine_root: &str) -> Result<bool> {
    let bundle_name = bundle_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("invalid bundle name for {}", bundle_path.display()))?;
    let now = Utc::now();
    let year = now.format("%Y");
    let month = now.format("%m");
    let archive_month_dir = Path::new(archive_root).join(year.to_string()).join(month.to_string());
    let final_dir = archive_month_dir.join("quarantine");
    fs::create_dir_all(&final_dir)
        .with_context(|| format!("failed to create {}", final_dir.display()))?;

    let destination = final_dir.join(bundle_name);
    if destination.exists() {
        return Ok(false);
    }

    fs::rename(bundle_path, &destination)
        .with_context(|| format!("failed to move {} to {}", bundle_path.display(), destination.display()))?;

    if let Err(err) = (|| -> Result<()> {
        let tar_path = final_dir.join(format!("{bundle_name}.tar.zst"));
        create_archive(&destination, &tar_path)?;
        let sha = compute_sha(&tar_path)?;
        update_ledger(
            &archive_month_dir.join(LEDGER_NAME),
            bundle_name,
            &destination,
            &tar_path,
            &sha,
            quarantine_root,
        )?;
        Ok(())
    })() {
        // Attempt to roll back the move on failure.
        let _ = fs::rename(&destination, bundle_path);
        return Err(err);
    }
    Ok(true)
}

fn create_archive(source_dir: &Path, archive_path: &Path) -> Result<()> {
    let parent = source_dir
        .parent()
        .ok_or_else(|| anyhow!("missing parent for {}", source_dir.display()))?;
    let dir_name = source_dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("invalid directory name for {}", source_dir.display()))?;
    if archive_path.exists() {
        fs::remove_file(archive_path)
            .with_context(|| format!("failed to remove existing {}", archive_path.display()))?;
    }
    let status = Command::new("tar")
        .arg("--zstd")
        .arg("-cf")
        .arg(archive_path)
        .arg("-C")
        .arg(parent)
        .arg(dir_name)
        .status()
        .with_context(|| format!("failed to create archive for {}", source_dir.display()))?;
    if !status.success() {
        return Err(anyhow!("tar command failed for {}", source_dir.display()));
    }
    Ok(())
}

fn compute_sha(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn update_ledger(
    ledger_path: &Path,
    bundle_name: &str,
    destination: &Path,
    tar_path: &Path,
    sha: &str,
    quarantine_root: &str,
) -> Result<()> {
    let mut entries: serde_json::Value = if ledger_path.exists() {
        let raw = fs::read_to_string(ledger_path)
            .with_context(|| format!("failed to read {}", ledger_path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("invalid JSON in {}", ledger_path.display()))?
    } else {
        serde_json::json!([])
    };
    let array = entries
        .as_array_mut()
        .ok_or_else(|| anyhow!("ledger must be a JSON array: {}", ledger_path.display()))?;
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    array.push(serde_json::json!({
        "type": "quarantine-rotation",
        "bundle": destination.to_string_lossy(),
        "archive": tar_path.to_string_lossy(),
        "sha256": sha,
        "timestamp": timestamp,
        "source": format!("{}/{}", quarantine_root.trim_end_matches('/'), bundle_name),
    }));
    let mut file = fs::File::create(ledger_path)
        .with_context(|| format!("failed to write {}", ledger_path.display()))?;
    use std::io::Write;
    file.write_all(serde_json::to_string_pretty(&entries)?.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn skips_recent_bundles() {
        let dir = tempdir().unwrap();
        let bundle = dir.path().join("component@commit");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(
            bundle.join("status.yaml"),
            "quarantined_at: 2099-01-01T00:00:00Z\n",
        )
        .unwrap();
        assert!(should_skip_bundle(&bundle).unwrap());
    }

    #[test]
    fn rotates_stale_bundles() {
        let dir = tempdir().unwrap();
        let bundle = dir.path().join("component@commit");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(
            bundle.join("status.yaml"),
            "quarantined_at: 2000-01-01T00:00:00Z\n",
        )
        .unwrap();
        assert!(!should_skip_bundle(&bundle).unwrap());
    }
}
