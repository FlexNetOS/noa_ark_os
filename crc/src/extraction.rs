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
                archive.unpack(&destination)?;
            }
            TarCompression::Gzip => {
                let decoder = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(&destination)?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| Error::ArchiveError(format!("Tar extraction task error: {}", e)))??;

    Ok(())
}
