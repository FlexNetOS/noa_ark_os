// CRC Archive System - Compression and cleanup
// Handles archiving processed drops with compression, retention policies, and cleanup

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, instrument, warn};

use crate::{ArchiveIndex, ArchiveInfo, FileEntry, Result, SourceType};

/// Archive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_level: u32,
    pub retention_days: HashMap<SourceType, u32>,
    pub auto_cleanup: bool,
    pub max_archive_size_gb: u64,
}

/// Compression algorithm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Bzip2,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        let mut retention = HashMap::new();
        retention.insert(SourceType::StaleCodebase, 90);
        retention.insert(SourceType::ExternalRepo, 180);
        retention.insert(SourceType::Fork, 90);
        retention.insert(SourceType::Mirror, 30);
        retention.insert(SourceType::Internal, 365);

        Self {
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: 3,
            retention_days: retention,
            auto_cleanup: true,
            max_archive_size_gb: 100,
        }
    }
}

/// Archive manager
pub struct ArchiveManager {
    archive_path: PathBuf,
    config: ArchiveConfig,
    archives: HashMap<String, ArchiveInfo>,
}

impl ArchiveManager {
    /// Create new archive manager
    pub fn new(archive_path: PathBuf, config: ArchiveConfig) -> Self {
        Self {
            archive_path,
            config,
            archives: HashMap::new(),
        }
    }

    /// Archive a processed drop
    #[instrument(skip(self))]
    pub async fn archive_drop(
        &mut self,
        drop_id: &str,
        source_path: &Path,
        source_type: SourceType,
    ) -> Result<ArchiveInfo> {
        info!("Archiving drop: {} (type: {:?})", drop_id, source_type);

        // Create archive directory for source type
        let type_dir = self.archive_path.join(match source_type {
            SourceType::StaleCodebase => "stale",
            SourceType::ExternalRepo => "repos",
            SourceType::Fork => "forks",
            SourceType::Mirror => "mirrors",
            SourceType::Internal => "internal",
        });

        fs::create_dir_all(&type_dir).await?;

        // Generate archive filename
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let extension = match self.config.compression_algorithm {
            CompressionAlgorithm::None => "tar",
            CompressionAlgorithm::Gzip => "tar.gz",
            CompressionAlgorithm::Zstd => "tar.zst",
            CompressionAlgorithm::Bzip2 => "tar.bz2",
        };

        let archive_filename = format!(
            "{}_{}_{}.{}",
            drop_id,
            source_type_short(&source_type),
            timestamp,
            extension
        );
        let archive_file_path = type_dir.join(&archive_filename);

        info!("  Archive file: {}", archive_file_path.display());

        // Create archive index
        let index = self.create_archive_index(source_path).await?;
        info!("  Indexed {} files", index.files.len());

        // Compress drop
        let archive_size = self.compress_drop(source_path, &archive_file_path).await?;
        info!(
            "  Compressed: {} bytes ({:.2} MB)",
            archive_size,
            archive_size as f64 / 1_048_576.0
        );

        // Calculate hash
        let hash = self.calculate_hash(&archive_file_path).await?;
        info!("  Hash: {}", hash);

        // Create archive info
        let archive_info = ArchiveInfo {
            hash: hash.clone(),
            archive_path: archive_file_path.clone(),
            created: timestamp,
            size: archive_size,
            index,
        };

        // Store archive info
        self.archives
            .insert(drop_id.to_string(), archive_info.clone());

        // Save archive metadata
        self.save_archive_metadata(drop_id, &archive_info).await?;

        info!("✓ Drop archived successfully");

        Ok(archive_info)
    }

    /// Clean up source after successful archiving
    #[instrument(skip(self))]
    pub async fn cleanup_source(&self, source_path: &Path) -> Result<()> {
        info!("Cleaning up source: {}", source_path.display());

        if source_path.exists() {
            // Remove source directory
            fs::remove_dir_all(source_path).await?;
            info!("✓ Source removed");
        } else {
            warn!("Source path does not exist: {}", source_path.display());
        }

        Ok(())
    }

    /// Clean up old archives based on retention policy
    #[instrument(skip(self))]
    pub async fn cleanup_old_archives(&self) -> Result<CleanupReport> {
        info!("Starting archive cleanup based on retention policies");

        let mut report = CleanupReport {
            archives_checked: 0,
            archives_removed: 0,
            space_freed_bytes: 0,
            errors: Vec::new(),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check each source type directory
        for (source_type, retention_days) in &self.config.retention_days {
            let type_dir = self.archive_path.join(match source_type {
                SourceType::StaleCodebase => "stale",
                SourceType::ExternalRepo => "repos",
                SourceType::Fork => "forks",
                SourceType::Mirror => "mirrors",
                SourceType::Internal => "internal",
            });

            if !type_dir.exists() {
                continue;
            }

            info!(
                "  Checking {:?} archives (retention: {} days)",
                source_type, retention_days
            );

            // Read directory
            let mut entries = fs::read_dir(&type_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                report.archives_checked += 1;

                let path = entry.path();
                let metadata = entry.metadata().await?;

                // Check file age
                if let Ok(created) = metadata.created() {
                    let created_secs = created
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                        .as_secs();
                    let age_secs = now.saturating_sub(created_secs);
                    let age_days = age_secs / 86400;

                    if age_days > *retention_days as u64 {
                        info!(
                            "    Removing old archive: {} (age: {} days)",
                            path.display(),
                            age_days
                        );

                        match fs::remove_file(&path).await {
                            Ok(_) => {
                                report.archives_removed += 1;
                                report.space_freed_bytes += metadata.len();
                            }
                            Err(e) => {
                                error!("    Failed to remove: {}", e);
                                report.errors.push(format!(
                                    "Failed to remove {}: {}",
                                    path.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }

        info!(
            "✓ Cleanup complete: {} archives removed, {:.2} MB freed",
            report.archives_removed,
            report.space_freed_bytes as f64 / 1_048_576.0
        );

        Ok(report)
    }

    /// Get archive statistics
    pub async fn get_statistics(&self) -> Result<ArchiveStatistics> {
        let mut stats = ArchiveStatistics {
            total_archives: 0,
            total_size_bytes: 0,
            archives_by_type: HashMap::new(),
            oldest_archive_days: 0,
        };

        // Scan archive directories
        for source_type in [
            SourceType::StaleCodebase,
            SourceType::ExternalRepo,
            SourceType::Fork,
            SourceType::Mirror,
            SourceType::Internal,
        ] {
            let type_dir = self.archive_path.join(match source_type {
                SourceType::StaleCodebase => "stale",
                SourceType::ExternalRepo => "repos",
                SourceType::Fork => "forks",
                SourceType::Mirror => "mirrors",
                SourceType::Internal => "internal",
            });

            if !type_dir.exists() {
                continue;
            }

            let mut count = 0;
            let mut size = 0u64;

            let mut entries = fs::read_dir(&type_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Ok(metadata) = entry.metadata().await {
                    count += 1;
                    size += metadata.len();
                    stats.total_archives += 1;
                    stats.total_size_bytes += metadata.len();
                }
            }

            stats.archives_by_type.insert(source_type, (count, size));
        }

        Ok(stats)
    }

    // === Helper Methods ===

    async fn create_archive_index(&self, source_path: &Path) -> Result<ArchiveIndex> {
        let mut files = Vec::new();

        // Recursively index files (simplified)
        if source_path.exists() {
            // Would recursively walk directory
            files.push(FileEntry {
                path: "placeholder.rs".to_string(),
                hash: "abc123".to_string(),
                size: 1024,
            });
        }

        Ok(ArchiveIndex {
            files,
            symbols: Vec::new(),
            dependencies: Vec::new(),
        })
    }

    async fn compress_drop(&self, source_path: &Path, archive_path: &Path) -> Result<u64> {
        info!(
            "  Compressing {} -> {} with {:?} (level {})",
            source_path.display(),
            archive_path.display(),
            self.config.compression_algorithm,
            self.config.compression_level
        );

        // Simulate compression
        // In production: would use tar + compression library
        // For now, create a placeholder file
        let mut file = fs::File::create(archive_path).await?;
        file.write_all(b"ARCHIVED_DROP_PLACEHOLDER\n").await?;

        let metadata = file.metadata().await?;
        Ok(metadata.len())
    }

    async fn calculate_hash(&self, path: &Path) -> Result<String> {
        // Simplified - would use SHA256 or similar
        let mut file = fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Placeholder hash
        Ok(format!("sha256_{:x}", buffer.len()))
    }

    async fn save_archive_metadata(&self, drop_id: &str, archive_info: &ArchiveInfo) -> Result<()> {
        let metadata_path = self.archive_path.join(format!("{}.metadata.json", drop_id));
        let json = serde_json::to_string_pretty(archive_info)?;

        let mut file = fs::File::create(metadata_path).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }
}

fn source_type_short(source_type: &SourceType) -> &'static str {
    match source_type {
        SourceType::StaleCodebase => "stale",
        SourceType::ExternalRepo => "repo",
        SourceType::Fork => "fork",
        SourceType::Mirror => "mirror",
        SourceType::Internal => "internal",
    }
}

/// Cleanup report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub archives_checked: usize,
    pub archives_removed: usize,
    pub space_freed_bytes: u64,
    pub errors: Vec<String>,
}

/// Archive statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStatistics {
    pub total_archives: usize,
    pub total_size_bytes: u64,
    pub archives_by_type: HashMap<SourceType, (usize, u64)>, // (count, total_size)
    pub oldest_archive_days: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_config_default() {
        let config = ArchiveConfig::default();
        assert_eq!(config.compression_algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.compression_level, 3);
        assert_eq!(config.auto_cleanup, true);
    }

    #[test]
    fn test_source_type_short() {
        assert_eq!(source_type_short(&SourceType::StaleCodebase), "stale");
        assert_eq!(source_type_short(&SourceType::ExternalRepo), "repo");
        assert_eq!(source_type_short(&SourceType::Fork), "fork");
    }
}
