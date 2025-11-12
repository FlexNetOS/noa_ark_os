use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::task;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{Error, OriginalArtifact, Result};

/// Description of a prepared artifact ready for pipeline processing.
#[derive(Debug, Clone)]
pub struct PreparedArtifact {
    pub processing_path: PathBuf,
    pub original_artifact: Option<OriginalArtifact>,
}

/// Known artifact categories that require special handling prior to processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Zip,
    Tar,
    TarGz,
    SevenZ,
    Iso,
    Wim,
}

impl ArtifactKind {
    fn from_path(path: &Path) -> Option<Self> {
        let file_name = path.file_name()?.to_string_lossy().to_lowercase();

        if file_name.ends_with(".tar.gz") {
            Some(Self::TarGz)
        } else if file_name.ends_with(".tar") {
            Some(Self::Tar)
        } else if file_name.ends_with(".zip") {
            Some(Self::Zip)
        } else if file_name.ends_with(".7z") {
            Some(Self::SevenZ)
        } else if file_name.ends_with(".iso") {
            Some(Self::Iso)
        } else if file_name.ends_with(".wim") {
            Some(Self::Wim)
        } else {
            None
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::SevenZ => "7z",
            Self::Iso => "iso",
            Self::Wim => "wim",
        }
    }

    fn is_supported(&self) -> bool {
        matches!(self, Self::Zip | Self::Tar | Self::TarGz)
    }
}

/// Prepare an artifact for processing by extracting supported archives into the
/// CRC workspace. Non-archive directories are passed through unchanged.
pub async fn prepare_artifact_for_processing(path: PathBuf) -> Result<PreparedArtifact> {
    if path.is_dir() {
        debug!("Bypassing extraction for directory {}", path.display());
        return Ok(PreparedArtifact {
            processing_path: path,
            original_artifact: None,
        });
    }

    let Some(kind) = ArtifactKind::from_path(&path) else {
        debug!("No archive handling required for {}", path.display());
        return Ok(PreparedArtifact {
            processing_path: path,
            original_artifact: None,
        });
    };

    info!(
        "Detected archival artifact ({}) at {}",
        kind.as_str(),
        path.display()
    );

    let extracts_root = Path::new("crc").join("temp").join("extracts");
    fs::create_dir_all(&extracts_root).await?;

    let extraction_dir = extracts_root.join(Uuid::new_v4().to_string());
    fs::create_dir_all(&extraction_dir).await?;

    if !kind.is_supported() {
        warn!(
            "Archive type {} is detected but not supported for automatic extraction; preserving raw artifact",
            kind.as_str()
        );
        let dest = extraction_dir.join(
            path.file_name()
                .map(|name| name.to_owned())
                .unwrap_or_else(|| OsString::from("artifact.bin")),
        );
        fs::copy(&path, &dest).await?;

        info!(
            "Stored unsupported archive {} at {} for manual inspection",
            path.display(),
            dest.display()
        );

        let original_size = std::fs::metadata(&path).map(|m| m.len()).ok();

        return Ok(PreparedArtifact {
            processing_path: extraction_dir.clone(),
            original_artifact: Some(OriginalArtifact {
                path: path.clone(),
                archive_type: Some(kind.as_str().to_string()),
                size: original_size,
                extracted_path: Some(extraction_dir),
                cleanup_after_processing: true,
            }),
        });
    }

    match kind {
        ArtifactKind::Zip => extract_zip(&path, &extraction_dir).await?,
        ArtifactKind::Tar => extract_tar(&path, &extraction_dir, TarCompression::None).await?,
        ArtifactKind::TarGz => extract_tar(&path, &extraction_dir, TarCompression::Gzip).await?,
        ArtifactKind::SevenZ | ArtifactKind::Iso | ArtifactKind::Wim => unreachable!(),
    }

    info!(
        "Extracted {} to {}",
        path.display(),
        extraction_dir.display()
    );

    let original_size = std::fs::metadata(&path).map(|m| m.len()).ok();

    Ok(PreparedArtifact {
        processing_path: extraction_dir.clone(),
        original_artifact: Some(OriginalArtifact {
            path: path.clone(),
            archive_type: Some(kind.as_str().to_string()),
            size: original_size,
            extracted_path: Some(extraction_dir),
            cleanup_after_processing: true,
        }),
    })
}

#[derive(Debug, Clone, Copy)]
enum TarCompression {
    None,
    Gzip,
}

async fn extract_zip(source: &Path, destination: &Path) -> Result<()> {
    let source = source.to_path_buf();
    let destination = destination.to_path_buf();

    task::spawn_blocking(move || -> Result<()> {
        let file = std::fs::File::open(&source)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| Error::ArchiveError(format!("Failed to read zip: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| Error::ArchiveError(format!("Zip entry error: {}", e)))?;
            let outpath = destination.join(file.mangled_name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                    }
                }
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| Error::ArchiveError(format!("Zip extraction task error: {}", e)))??;

    Ok(())
}

async fn extract_tar(source: &Path, destination: &Path, compression: TarCompression) -> Result<()> {
    let source = source.to_path_buf();
    let destination = destination.to_path_buf();

    task::spawn_blocking(move || -> Result<()> {
        let file = std::fs::File::open(&source)?;

        match compression {
            TarCompression::None => {
                let mut archive = tar::Archive::new(file);
                extract_tar_entries(&mut archive, &destination)?;
            }
            TarCompression::Gzip => {
                let decoder = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                extract_tar_entries(&mut archive, &destination)?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| Error::ArchiveError(format!("Tar extraction task error: {}", e)))??;

    Ok(())
}

/// Safely extract tar entries by validating paths to prevent path traversal attacks.
fn extract_tar_entries<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    destination: &Path,
) -> Result<()> {
    let canonical_dest = destination
        .canonicalize()
        .map_err(|e| Error::ArchiveError(format!("Failed to canonicalize destination: {}", e)))?;

    for entry_result in archive.entries()? {
        let mut entry = entry_result
            .map_err(|e| Error::ArchiveError(format!("Failed to read tar entry: {}", e)))?;

        let entry_path = entry
            .path()
            .map_err(|e| Error::ArchiveError(format!("Failed to read entry path: {}", e)))?;

        // Validate the entry path to prevent path traversal attacks
        let target_path = destination.join(&entry_path);
        
        // Canonicalize the target path to resolve any ".." or "." components
        // If the path doesn't exist yet, we need to check its parent directories
        let canonical_target = if target_path.exists() {
            target_path
                .canonicalize()
                .map_err(|e| Error::ArchiveError(format!("Failed to canonicalize target path: {}", e)))?
        } else {
            // For non-existent paths, construct the canonical path by joining with the destination
            let mut path_buf = canonical_dest.clone();
            for component in entry_path.components() {
                match component {
                    std::path::Component::Normal(name) => path_buf.push(name),
                    std::path::Component::RootDir => {
                        return Err(Error::ArchiveError(format!(
                            "Absolute paths are not allowed in tar archives: {}",
                            entry_path.display()
                        )));
                    }
                    std::path::Component::ParentDir | std::path::Component::CurDir => {
                        return Err(Error::ArchiveError(format!(
                            "Path traversal detected in tar entry: {}",
                            entry_path.display()
                        )));
                    }
                    std::path::Component::Prefix(_) => {
                        return Err(Error::ArchiveError(format!(
                            "Windows path prefixes are not allowed in tar archives: {}",
                            entry_path.display()
                        )));
                    }
                }
            }
            path_buf
        };

        // Ensure the canonical target is within the destination directory
        if !canonical_target.starts_with(&canonical_dest) {
            return Err(Error::ArchiveError(format!(
                "Path traversal attack detected: entry '{}' would extract to '{}'",
                entry_path.display(),
                canonical_target.display()
            )));
        }

        // Extract the entry
        // Ensure parent directory exists before unpacking
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        entry
            .unpack(&target_path)
            .map_err(|e| Error::ArchiveError(format!("Failed to unpack entry: {}", e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_kind_detection() {
        assert_eq!(
            ArtifactKind::from_path(Path::new("test.zip")),
            Some(ArtifactKind::Zip)
        );
        assert_eq!(
            ArtifactKind::from_path(Path::new("test.tar")),
            Some(ArtifactKind::Tar)
        );
        assert_eq!(
            ArtifactKind::from_path(Path::new("test.tar.gz")),
            Some(ArtifactKind::TarGz)
        );
        assert_eq!(
            ArtifactKind::from_path(Path::new("test.7z")),
            Some(ArtifactKind::SevenZ)
        );
        assert_eq!(ArtifactKind::from_path(Path::new("test.txt")), None);
    }

    #[test]
    fn test_path_traversal_detection() {
        use std::path::Component;

        // Test that we properly validate path components
        let safe_path = Path::new("safe/file.txt");
        let mut has_traversal = false;
        for component in safe_path.components() {
            if matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            ) {
                has_traversal = true;
            }
        }
        assert!(!has_traversal, "Safe path should not trigger traversal detection");

        // Test parent directory detection
        let malicious_path = Path::new("../etc/passwd");
        let mut has_parent = false;
        for component in malicious_path.components() {
            if matches!(component, Component::ParentDir) {
                has_parent = true;
            }
        }
        assert!(has_parent, "Path with .. should be detected");

        // Test absolute path detection
        #[cfg(unix)]
        {
            let absolute_path = Path::new("/etc/passwd");
            let mut has_root = false;
            for component in absolute_path.components() {
                if matches!(component, Component::RootDir) {
                    has_root = true;
                }
            }
            assert!(has_root, "Absolute path should be detected");
        }
    }

    #[test]
    fn test_extract_tar_entries_validates_paths() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir()?;
        let dest_path = temp_dir.path().join("extract");
        std::fs::create_dir_all(&dest_path)?;

        // Create a simple tar archive with a safe file
        let tar_path = temp_dir.path().join("test.tar");
        let tar_file = std::fs::File::create(&tar_path)?;
        let mut builder = tar::Builder::new(tar_file);

        let data = b"safe content";
        let mut header = tar::Header::new_gnu();
        header.set_path("safe.txt")?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &data[..])?;
        builder.finish()?;

        // Extract the archive
        let file = std::fs::File::open(&tar_path)?;
        let mut archive = tar::Archive::new(file);
        extract_tar_entries(&mut archive, &dest_path)?;

        // Verify the file was extracted
        assert!(dest_path.join("safe.txt").exists());
        let content = std::fs::read_to_string(dest_path.join("safe.txt"))?;
        assert_eq!(content, "safe content");

        Ok(())
    }

    #[test]
    fn test_nested_directories_extraction() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let dest_path = temp_dir.path().join("extract");
        std::fs::create_dir_all(&dest_path)?;

        // Create a tar with nested directories
        let tar_path = temp_dir.path().join("nested.tar");
        let tar_file = std::fs::File::create(&tar_path)?;
        let mut builder = tar::Builder::new(tar_file);

        // Add directories first
        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_path("dir1/")?;
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_cksum();
        builder.append(&dir_header, std::io::empty())?;

        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_path("dir1/dir2/")?;
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_cksum();
        builder.append(&dir_header, std::io::empty())?;

        // Add a file in the nested directory
        let data = b"nested content";
        let mut header = tar::Header::new_gnu();
        header.set_path("dir1/dir2/file.txt")?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &data[..])?;
        builder.finish()?;

        // Extract the archive
        let file = std::fs::File::open(&tar_path)?;
        let mut archive = tar::Archive::new(file);
        extract_tar_entries(&mut archive, &dest_path)?;

        // Verify the nested file was extracted
        assert!(dest_path.join("dir1/dir2/file.txt").exists());

        Ok(())
    }
}
