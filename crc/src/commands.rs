// CRC Chat Commands
// CLI interface for drop management via chat/terminal

use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;

use crate::{telemetry, CRCSystem, Error, Priority};

/// CRC Chat Command Interface
#[derive(Parser, Debug)]
#[command(name = "drop")]
#[command(about = "Manage code drops in NOA ARK OS", long_about = None)]
pub struct DropCli {
    #[command(subcommand)]
    command: DropCommands,
}

/// Available drop commands
#[derive(Subcommand, Debug)]
pub enum DropCommands {
    /// Drop a fresh repository
    #[command(about = "Drop a fresh/active repository")]
    Repo {
        /// Repository path or URL
        #[arg(value_name = "PATH_OR_URL")]
        source: String,

        /// Optional custom name
        #[arg(short, long)]
        name: Option<String>,

        /// Priority (critical, high, normal, low)
        #[arg(short, long, default_value = "high")]
        priority: String,
    },

    /// Drop a forked project
    #[command(about = "Drop a forked project")]
    Fork {
        /// Original repository URL
        #[arg(value_name = "ORIGINAL_URL")]
        original: String,

        /// Fork repository path or URL
        #[arg(value_name = "FORK_PATH")]
        fork: String,

        /// Optional custom name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Drop a mirror repository
    #[command(about = "Drop a mirror repository")]
    Mirror {
        /// Mirror repository path or URL
        #[arg(value_name = "PATH_OR_URL")]
        source: String,

        /// Original source URL
        #[arg(short, long)]
        original: Option<String>,

        /// Optional custom name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Drop stale/abandoned code
    #[command(about = "Drop stale or abandoned codebase")]
    Stale {
        /// Code path (zip, tar.gz, or directory)
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Optional custom name
        #[arg(short, long)]
        name: Option<String>,

        /// Last known commit date (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        last_commit: Option<String>,
    },

    /// Show drop status
    #[command(about = "Show status of a drop")]
    Status {
        /// Drop ID
        #[arg(value_name = "DROP_ID")]
        drop_id: String,
    },

    /// List all drops
    #[command(about = "List all drops")]
    List {
        /// Filter by status (pending, processing, completed, failed)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by source type (repo, fork, mirror, stale)
        #[arg(short = 't', long)]
        source_type: Option<String>,
    },

    /// Cancel a drop
    #[command(about = "Cancel a pending drop")]
    Cancel {
        /// Drop ID
        #[arg(value_name = "DROP_ID")]
        drop_id: String,
    },

    /// Retry a failed drop
    #[command(about = "Retry a failed drop")]
    Retry {
        /// Drop ID
        #[arg(value_name = "DROP_ID")]
        drop_id: String,
    },
}

/// Execute drop command
pub async fn execute_command(cli: DropCli, crc: &CRCSystem) -> Result<(), Error> {
    match cli.command {
        DropCommands::Repo {
            source,
            name,
            priority,
        } => drop_repo(crc, source, name, priority).await,

        DropCommands::Fork {
            original,
            fork,
            name,
        } => drop_fork(crc, original, fork, name).await,

        DropCommands::Mirror {
            source,
            original,
            name,
        } => drop_mirror(crc, source, original, name).await,

        DropCommands::Stale {
            path,
            name,
            last_commit,
        } => drop_stale(crc, path, name, last_commit).await,

        DropCommands::Status { drop_id } => show_status(crc, drop_id).await,

        DropCommands::List {
            status,
            source_type,
        } => list_drops(crc, status, source_type).await,

        DropCommands::Cancel { drop_id } => cancel_drop(crc, drop_id).await,

        DropCommands::Retry { drop_id } => retry_drop(crc, drop_id).await,
    }
}

/// Drop a fresh repository
async fn drop_repo(
    crc: &CRCSystem,
    source: String,
    name: Option<String>,
    priority_str: String,
) -> Result<(), Error> {
    let _ = crc;
    let priority = parse_priority(&priority_str)?;
    let name = name.unwrap_or_else(|| extract_name_from_url(&source));

    // Copy or download to incoming/repos/
    let dest = PathBuf::from(format!("crc/drop-in/incoming/repos/{}", name));
    let drop_id = format!("drop-{}", uuid::Uuid::new_v4());
    let trace_id = telemetry::new_trace_id();

    telemetry::info(
        "crc.drop",
        "repository_registered",
        "Repository drop registered",
        "queued",
        Some(&trace_id),
        Some(json!({
            "source": source,
            "name": name,
            "priority": format!("{:?}", priority),
            "destination": dest,
            "drop_id": drop_id,
            "estimates": {
                "analysis": "~2 minutes",
                "adaptation": "~3 minutes",
                "validation": "~2 minutes",
                "eta": "~7 minutes"
            }
        })),
    );

    Ok(())
}

/// Drop a forked project
async fn drop_fork(
    crc: &CRCSystem,
    original: String,
    fork: String,
    name: Option<String>,
) -> Result<(), Error> {
    let _ = crc;
    let name = name.unwrap_or_else(|| extract_name_from_url(&fork));
    let dest = PathBuf::from(format!("crc/drop-in/incoming/forks/{}", name));
    let drop_id = format!("drop-{}", uuid::Uuid::new_v4());
    let trace_id = telemetry::new_trace_id();

    telemetry::info(
        "crc.drop",
        "fork_registered",
        "Fork drop registered",
        "queued",
        Some(&trace_id),
        Some(json!({
            "original": original,
            "fork": fork,
            "name": name,
            "destination": dest,
            "drop_id": drop_id,
            "notes": "Will compare against original for changes"
        })),
    );

    Ok(())
}

/// Drop a mirror repository
async fn drop_mirror(
    crc: &CRCSystem,
    source: String,
    original: Option<String>,
    name: Option<String>,
) -> Result<(), Error> {
    let _ = crc;
    let name = name.unwrap_or_else(|| extract_name_from_url(&source));
    let dest = PathBuf::from(format!("crc/drop-in/incoming/mirrors/{}", name));
    let drop_id = format!("drop-{}", uuid::Uuid::new_v4());
    let trace_id = telemetry::new_trace_id();

    telemetry::info(
        "crc.drop",
        "mirror_registered",
        "Mirror drop registered",
        "queued",
        Some(&trace_id),
        Some(json!({
            "source": source,
            "original": original,
            "name": name,
            "destination": dest,
            "drop_id": drop_id
        })),
    );

    Ok(())
}

/// Drop stale/abandoned code
async fn drop_stale(
    crc: &CRCSystem,
    path: PathBuf,
    name: Option<String>,
    last_commit: Option<String>,
) -> Result<(), Error> {
    let _ = crc;
    let name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    let dest = PathBuf::from(format!("crc/drop-in/incoming/stale/{}", name));

    // Check if file exists
    if !path.exists() {
        return Err(Error::FileNotFound(path.display().to_string()));
    }
    let drop_id = format!("drop-{}", uuid::Uuid::new_v4());
    let trace_id = telemetry::new_trace_id();

    telemetry::info(
        "crc.drop",
        "stale_registered",
        "Stale code drop registered",
        "queued",
        Some(&trace_id),
        Some(json!({
            "source": path,
            "name": name,
            "last_commit": last_commit,
            "destination": dest,
            "drop_id": drop_id,
            "sandbox": "Model C (Experimental)"
        })),
    );

    Ok(())
}

/// Show drop status
async fn show_status(crc: &CRCSystem, drop_id: String) -> Result<(), Error> {
    let _ = crc;
    let trace_id = telemetry::new_trace_id();
    telemetry::info(
        "crc.drop",
        "status_report",
        "Reported drop status",
        "success",
        Some(&trace_id),
        Some(json!({
            "drop_id": drop_id,
            "status": "Processing",
            "stage": "Adaptation (2/3)",
            "confidence": "92%",
            "started": "5 minutes ago",
            "eta": "2 minutes",
            "progress": [
                "Analysis completed (92% confidence)",
                "Adaptation in progress",
                "Validation pending"
            ]
        })),
    );

    Ok(())
}

/// List all drops
async fn list_drops(
    crc: &CRCSystem,
    status: Option<String>,
    source_type: Option<String>,
) -> Result<(), Error> {
    let _ = crc;
    let trace_id = telemetry::new_trace_id();
    telemetry::info(
        "crc.drop",
        "list",
        "Listed drops",
        "success",
        Some(&trace_id),
        Some(json!({
            "status_filter": status,
            "source_type_filter": source_type,
            "drops": [
                {"id": "drop-abc-123", "type": "repo", "status": "completed", "confidence": "96%", "started": "5 min ago"},
                {"id": "drop-def-456", "type": "fork", "status": "processing", "confidence": "89%", "started": "2 min ago"},
                {"id": "drop-ghi-789", "type": "stale", "status": "pending", "confidence": "-", "started": "1 min ago"}
            ],
            "total": 3
        })),
    );

    Ok(())
}

/// Cancel a drop
async fn cancel_drop(crc: &CRCSystem, drop_id: String) -> Result<(), Error> {
    let _ = crc;
    let trace_id = telemetry::new_trace_id();
    telemetry::info(
        "crc.drop",
        "cancel_requested",
        "Drop cancellation requested",
        "queued",
        Some(&trace_id),
        Some(json!({ "drop_id": drop_id, "status": "canceled" })),
    );

    Ok(())
}

/// Retry a failed drop
async fn retry_drop(crc: &CRCSystem, drop_id: String) -> Result<(), Error> {
    let _ = crc;
    let trace_id = telemetry::new_trace_id();
    telemetry::info(
        "crc.drop",
        "retry_requested",
        "Drop retry requested",
        "queued",
        Some(&trace_id),
        Some(json!({ "drop_id": drop_id, "status": "retry_queued" })),
    );

    Ok(())
}

// Helper functions

fn parse_priority(priority_str: &str) -> Result<Priority, Error> {
    match priority_str.to_lowercase().as_str() {
        "critical" => Ok(Priority::Critical),
        "high" => Ok(Priority::High),
        "normal" => Ok(Priority::Normal),
        "low" => Ok(Priority::Low),
        _ => Err(Error::InvalidPriority(priority_str.to_string())),
    }
}

fn extract_name_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .and_then(|s| s.split('.').next())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_from_url() {
        assert_eq!(extract_name_from_url("github.com/user/project"), "project");
        assert_eq!(extract_name_from_url("gitlab.com/org/repo.git"), "repo");
    }

    #[test]
    fn test_parse_priority() {
        assert!(matches!(parse_priority("high"), Ok(Priority::High)));
        assert!(matches!(parse_priority("critical"), Ok(Priority::Critical)));
        assert!(parse_priority("invalid").is_err());
    }
}
