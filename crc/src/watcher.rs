// CRC File Watcher - Auto-detect and register code drops
// Monitors incoming folders for new files and triggers processing

use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tracing::{info, error, debug};
use serde::{Deserialize, Serialize};

use crate::{CRCSystem, DropManifest, SourceType, Priority, Error};

/// Source type detection and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub priority: Priority,
    pub auto_approve_threshold: f64,
    pub default_sandbox: String,
}

/// CRC File Watcher for automatic drop detection
pub struct CRCWatcher {
    /// Paths to watch for incoming drops
    watch_paths: Vec<PathBuf>,
    
    /// Source type configurations
    source_configs: HashMap<SourceType, SourceConfig>,
    
    /// Reference to CRC system
    crc_system: CRCSystem,
    
    /// Debounce delay in seconds
    debounce_delay: u64,
}

impl CRCWatcher {
    /// Create new CRC watcher
    pub fn new(crc_system: CRCSystem) -> Self {
        let watch_paths = vec![
            PathBuf::from("crc/drop-in/incoming/repos"),
            PathBuf::from("crc/drop-in/incoming/forks"),
            PathBuf::from("crc/drop-in/incoming/mirrors"),
            PathBuf::from("crc/drop-in/incoming/stale"),
        ];
        
        let mut source_configs = HashMap::new();
        
        source_configs.insert(
            SourceType::ExternalRepo,
            SourceConfig {
                name: "Fresh Repository".to_string(),
                priority: Priority::High,
                auto_approve_threshold: 0.95,
                default_sandbox: "model-a".to_string(),
            },
        );
        
        source_configs.insert(
            SourceType::Fork,
            SourceConfig {
                name: "Forked Project".to_string(),
                priority: Priority::Normal,
                auto_approve_threshold: 0.90,
                default_sandbox: "model-b".to_string(),
            },
        );
        
        source_configs.insert(
            SourceType::Mirror,
            SourceConfig {
                name: "Mirror Repository".to_string(),
                priority: Priority::Normal,
                auto_approve_threshold: 0.85,
                default_sandbox: "model-a".to_string(),
            },
        );
        
        source_configs.insert(
            SourceType::StaleCodebase,
            SourceConfig {
                name: "Stale/Abandoned Code".to_string(),
                priority: Priority::Normal,
                auto_approve_threshold: 0.80,
                default_sandbox: "model-c".to_string(),
            },
        );
        
        Self {
            watch_paths,
            source_configs,
            crc_system,
            debounce_delay: 2,
        }
    }
    
    /// Start watching for file changes
    pub async fn start(&self) -> Result<(), Error> {
        info!("Starting CRC file watcher...");
        
        // Create file system watcher
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_secs(self.debounce_delay))
        )
            .map_err(|e| Error::WatcherError(e.to_string()))?;
        
        // Watch all configured paths
        for path in &self.watch_paths {
            info!("Watching path: {}", path.display());
            watcher.watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| Error::WatcherError(e.to_string()))?;
        }
        
        info!("File watcher started successfully");
        info!("Monitoring {} paths", self.watch_paths.len());
        
        // Event processing loop
        loop {
            match rx.recv() {
                Ok(event_result) => {
                    match event_result {
                        Ok(event) => {
                            if let Err(e) = self.handle_event(event).await {
                                error!("Error handling file event: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Watch error: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Channel error: {:?}", e);
                    // Continue watching despite errors
                }
            }
        }
    }
    
    /// Handle file system events
    async fn handle_event(&self, event: Event) -> Result<(), Error> {
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for path in event.paths {
                    info!("File detected: {}", path.display());
                    self.process_new_file(path).await?;
                }
            }
            notify::EventKind::Remove(_) => {
                for path in &event.paths {
                    debug!("File removed: {}", path.display());
                }
            }
            _ => {
                debug!("Unhandled event: {:?}", event);
            }
        }
        
        Ok(())
    }
    
    /// Process newly detected file
    async fn process_new_file(&self, path: PathBuf) -> Result<(), Error> {
        // Ignore temporary files
        if self.is_temp_file(&path) {
            debug!("Ignoring temporary file: {}", path.display());
            return Ok(());
        }
        
        // Detect source type from path
        let source_type = self.detect_source_type(&path)?;
        info!("Detected source type: {:?} for {}", source_type, path.display());
        
        // Get source configuration
        let config = self.source_configs.get(&source_type)
            .ok_or_else(|| Error::ConfigError("Unknown source type".to_string()))?;
        
        // Extract metadata
        let metadata = self.extract_metadata(&path).await?;
        
        // Generate manifest
        let manifest = self.generate_manifest(&path, source_type.clone(), config, metadata).await?;
        
        // Register drop with CRC system
        let drop_id = self.crc_system.register_drop(path.clone(), manifest)
            .map_err(|e| Error::SystemError(e))?;
        
        info!("✓ Drop registered: {} ({})", drop_id, config.name);
        info!("  Source type: {:?}", source_type);
        info!("  Priority: {:?}", config.priority);
        info!("  Default sandbox: {}", config.default_sandbox);
        
        // TODO: Trigger processing via message queue or channel
        // For now, processing will be handled by the parallel processor
        
        Ok(())
    }
    
    /// Detect source type from file path
    fn detect_source_type(&self, path: &Path) -> Result<SourceType, Error> {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("repos") {
            Ok(SourceType::ExternalRepo)
        } else if path_str.contains("forks") {
            Ok(SourceType::Fork)
        } else if path_str.contains("mirrors") {
            Ok(SourceType::Mirror)
        } else if path_str.contains("stale") {
            Ok(SourceType::StaleCodebase)
        } else {
            Err(Error::UnknownSourceType(path.display().to_string()))
        }
    }
    
    /// Check if file is temporary
    fn is_temp_file(&self, path: &Path) -> bool {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Ignore common temporary files
        file_name.starts_with('.') ||
        file_name.ends_with(".tmp") ||
        file_name.ends_with(".partial") ||
        file_name.ends_with(".download") ||
        file_name == ".DS_Store" ||
        file_name == "Thumbs.db"
    }
    
    /// Extract metadata from file
    async fn extract_metadata(&self, path: &Path) -> Result<HashMap<String, String>, Error> {
        let mut metadata = HashMap::new();
        
        // File size
        if let Ok(meta) = std::fs::metadata(path) {
            metadata.insert("file_size".to_string(), meta.len().to_string());
        }
        
        // File extension
        if let Some(ext) = path.extension() {
            metadata.insert("extension".to_string(), ext.to_string_lossy().to_string());
        }
        
        // File name
        if let Some(name) = path.file_name() {
            metadata.insert("filename".to_string(), name.to_string_lossy().to_string());
        }
        
        // TODO: Extract more metadata:
        // - Language detection
        // - Dependency count
        // - Last commit date (for git repos)
        // - File count
        // - Estimated LOC
        
        Ok(metadata)
    }
    
    /// Generate drop manifest
    async fn generate_manifest(
        &self,
        path: &Path,
        source_type: SourceType,
        config: &SourceConfig,
        metadata: HashMap<String, String>,
    ) -> Result<DropManifest, Error> {
        let name = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let manifest = DropManifest {
            name,
            source: path.display().to_string(),
            source_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Error::SystemError(e.to_string()))?
                .as_secs(),
            priority: config.priority.clone(),
            metadata,
        };
        
        Ok(manifest)
    }
}

/// Spawn watcher in background task
pub async fn spawn_watcher(crc_system: CRCSystem) -> Result<tokio::task::JoinHandle<()>, Error> {
    let watcher = CRCWatcher::new(crc_system);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = watcher.start().await {
            error!("Watcher error: {:?}", e);
        }
    });
    
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_source_type() {
        let watcher = CRCWatcher::new(CRCSystem::new_test());
        
        assert!(matches!(
            watcher.detect_source_type(Path::new("crc/drop-in/incoming/repos/test.zip")),
            Ok(SourceType::ExternalRepo)
        ));
        
        assert!(matches!(
            watcher.detect_source_type(Path::new("crc/drop-in/incoming/forks/test.tar.gz")),
            Ok(SourceType::Fork)
        ));
        
        assert!(matches!(
            watcher.detect_source_type(Path::new("crc/drop-in/incoming/stale/old.zip")),
            Ok(SourceType::StaleCodebase)
        ));
    }
    
    #[test]
    fn test_is_temp_file() {
        let watcher = CRCWatcher::new(CRCSystem::new_test());
        
        assert!(watcher.is_temp_file(Path::new(".DS_Store")));
        assert!(watcher.is_temp_file(Path::new("file.tmp")));
        assert!(watcher.is_temp_file(Path::new(".hidden")));
        assert!(!watcher.is_temp_file(Path::new("code.zip")));
    }
}
