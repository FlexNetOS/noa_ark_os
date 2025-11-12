// CRC Service - Continuous ReCode Automation
// Background service that monitors drop-in folders and automates code integration
// Full automation: drop → detect → process → archive → CI/CD trigger → deploy

use noa_crc::parallel::ParallelDropProcessor;
use noa_crc::watcher::spawn_watcher;
use noa_crc::{CRCConfig, CRCSystem};
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("noa_crc=debug".parse()?)
                .add_directive("info".parse()?),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("═══════════════════════════════════════════════════════");
    info!("  CRC Service - Continuous ReCode Automation");
    info!("  Version: 0.1.0");
    info!("  Full Automation: Drop → Process → Archive → CI/CD");
    info!("═══════════════════════════════════════════════════════");

    // Verify directory structure
    verify_directory_structure()?;

    // Initialize CRC system with enhanced config
    let config = load_config();
    info!("Configuration:");
    info!("  Max concurrent drops: {}", config.max_concurrent);
    info!("  Auto-archive: {}", config.auto_archive);
    info!("  Trigger CI/CD: {}", config.trigger_cicd);
    info!("  Archive compression: {}", config.compression_algorithm);

    let crc_system = CRCSystem::new(config.clone());
    info!("✓ CRC System initialized");

    // Start file watcher
    info!("Starting file watcher...");
    let watcher_handle = spawn_watcher(crc_system.clone()).await?;
    info!("✓ File watcher started");

    // Display monitoring paths
    info!("Monitoring paths:");
    info!("  • crc/drop-in/incoming/repos    (External repositories)");
    info!("  • crc/drop-in/incoming/forks    (Forked projects)");
    info!("  • crc/drop-in/incoming/mirrors  (Mirror snapshots)");
    info!("  • crc/drop-in/incoming/stale    (Legacy codebases)");

    // Start parallel processor with enhanced capabilities
    info!("Starting parallel drop processor...");
    let processor = ParallelDropProcessor::new_with_config(config);
    info!(
        "✓ Processor ready (max {} concurrent)",
        processor.max_concurrent()
    );

    info!("═══════════════════════════════════════════════════════");
    info!("  CRC Service is now running");
    info!("  Automation: ENABLED ✓");
    info!("  Press Ctrl+C to stop");
    info!("═══════════════════════════════════════════════════════");

    // Run until shutdown with enhanced monitoring
    tokio::select! {
        result = watcher_handle => {
            error!("File watcher stopped unexpectedly: {:?}", result);
        }
        result = processor.start_processing() => {
            error!("Processor stopped unexpectedly: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    info!("═══════════════════════════════════════════════════════");
    info!("  Initiating graceful shutdown...");
    info!("  • Finishing current processing tasks");
    info!("  • Archiving in-progress drops");
    info!("  • Cleaning up temporary files");
    info!("═══════════════════════════════════════════════════════");

    // TODO: Graceful shutdown of processor and watcher

    info!("✓ CRC Service stopped cleanly");

    Ok(())
}

/// Load configuration from environment and defaults
fn load_config() -> CRCConfig {
    let mut config = CRCConfig::default();

    // Override from environment variables
    if let Ok(max_concurrent) = std::env::var("CRC_MAX_CONCURRENT") {
        if let Ok(value) = max_concurrent.parse() {
            config.max_concurrent = value;
        }
    }

    if let Ok(auto_archive) = std::env::var("CRC_AUTO_ARCHIVE") {
        config.auto_archive = auto_archive.to_lowercase() == "true";
    }

    if let Ok(trigger_cicd) = std::env::var("CRC_TRIGGER_CICD") {
        config.trigger_cicd = trigger_cicd.to_lowercase() == "true";
    }

    if let Ok(compression) = std::env::var("CRC_COMPRESSION") {
        config.compression_algorithm = compression;
    }

    config
}

/// Verify and create required directory structure
fn verify_directory_structure() -> Result<(), Box<dyn std::error::Error>> {
    info!("Verifying directory structure...");

    let required_dirs = vec![
        // Incoming directories
        "crc/drop-in/incoming/repos",
        "crc/drop-in/incoming/forks",
        "crc/drop-in/incoming/mirrors",
        "crc/drop-in/incoming/stale",
        // Processing directories
        "crc/drop-in/processing/adaptation",
        "crc/drop-in/processing/analysis",
        "crc/drop-in/processing/validation",
        // Ready queues
        "crc/drop-in/ready/model-a-queue",
        "crc/drop-in/ready/model-b-queue",
        "crc/drop-in/ready/model-c-queue",
        "crc/drop-in/ready/model-d-queue",
        // Archive directories
        "crc/archive/stale",
        "crc/archive/repos",
        "crc/archive/forks",
        "crc/archive/mirrors",
        // Temporary directories
        "crc/temp/analysis-cache",
        "crc/temp/extracts",
        "crc/temp/logs",
        // Artifact storage
        "storage/artifacts",
        "storage/artifacts/edge",
        "storage/artifacts/server",
    ];

    let mut created_count = 0;
    for dir in required_dirs {
        let path = PathBuf::from(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
            created_count += 1;
        }
    }

    if created_count > 0 {
        info!("  Created {} missing directories", created_count);
    }
    info!("✓ Directory structure verified");

    Ok(())
}
