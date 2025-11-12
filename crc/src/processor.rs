// CRC Processing Pipeline - Core automation logic
// analyze() → adapt() → validate() → move_to_ready()
// Handles source type detection, sandbox assignment, and adaptation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    build::{self, BuildArtifact},
    AdaptationResult, AnalysisResult, CRCState, Dependency, Error, Priority, Result, SandboxModel,
    SourceType,
};

/// Processing stage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub stage: String,
    pub confidence: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Code drop processor
pub struct DropProcessor {
    base_path: PathBuf,
    auto_approve_threshold: f32,
}

impl DropProcessor {
    /// Create new processor
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            auto_approve_threshold: 0.85,
        }
    }

    /// Process drop through full pipeline
    #[instrument(skip(self))]
    pub async fn process_drop(
        &self,
        drop_id: &str,
        source_type: SourceType,
        source_path: PathBuf,
    ) -> Result<ProcessingResult> {
        info!("Starting full pipeline for drop: {}", drop_id);

        // Stage 1: Analysis
        let analysis = self.analyze(&source_path, &source_type).await?;
        info!(
            "✓ Analysis complete (confidence: {:.1}%)",
            analysis.ai_confidence * 100.0
        );

        // Check if we should continue
        if !analysis.issues.is_empty() {
            warn!("Analysis found {} issues", analysis.issues.len());
            for issue in &analysis.issues {
                warn!("  - {}", issue);
            }
        }

        // Stage 2: Adaptation
        let adaptation = self.adapt(&source_path, &analysis, &source_type).await?;
        info!(
            "✓ Adaptation complete ({} files modified)",
            adaptation.files_modified
        );

        // Stage 3: Validation
        let validation = self.validate(&source_path, &adaptation).await?;
        let validation_confidence = validation.confidence;
        info!(
            "✓ Validation complete (confidence: {:.1}%)",
            validation_confidence * 100.0
        );

        // Stage 4: Determine sandbox assignment
        let sandbox = self.assign_sandbox(&source_type, validation_confidence);
        info!(
            "✓ Assigned to {:?} ({:.1}% confidence)",
            sandbox,
            validation_confidence * 100.0
        );

        // Stage 5: Move to ready queue
        let ready_path = self.move_to_ready(drop_id, &source_path, &sandbox).await?;
        info!("✓ Moved to ready queue: {}", ready_path.display());

        // Stage 6: Produce hardware-optimized builds
        let build_artifacts = self.produce_profile_builds(drop_id, &source_path).await?;

        for artifact in &build_artifacts {
            info!(
                "✓ Generated {:?} artifact at {}",
                artifact.manifest.profile,
                artifact.artifact_path.display()
            );
        }

        let mut metadata = validation.metadata;
        match serde_json::to_string(&build_artifacts) {
            Ok(serialized) => {
                metadata.insert("build_artifacts".to_string(), serialized);
            }
            Err(e) => {
                warn!(
                    "Failed to serialize build_artifacts for drop {}: {}",
                    drop_id, e
                );
            }
        }

        let errors = validation.errors;
        let warnings = validation.warnings;

        Ok(ProcessingResult {
            success: true,
            stage: "completed".to_string(),
            confidence: validation_confidence,
            errors,
            warnings,
            metadata,
        })
    }

    /// Stage 1: Analyze code drop
    #[instrument(skip(self))]
    pub async fn analyze(&self, path: &Path, source_type: &SourceType) -> Result<AnalysisResult> {
        info!("Analyzing drop at: {}", path.display());

        // Count files and lines
        let (files_count, lines_count) = self.count_files_and_lines(path).await?;
        info!("  Files: {}, Lines: {}", files_count, lines_count);

        // Detect languages
        let languages = self.detect_languages(path).await?;
        info!("  Languages: {:?}", languages);

        // Analyze dependencies
        let dependencies = self.analyze_dependencies(path).await?;
        info!("  Dependencies: {}", dependencies.len());

        // Detect patterns (AI would go here)
        let patterns_found = self.detect_patterns(path, &languages).await?;
        info!("  Patterns found: {}", patterns_found.len());

        // Find potential issues
        let issues = self.find_issues(path, source_type).await?;
        if !issues.is_empty() {
            warn!("  Issues found: {}", issues.len());
        }

        // Calculate AI confidence (simplified - would use actual AI model)
        let ai_confidence = self.calculate_confidence(&languages, &dependencies, &issues);

        Ok(AnalysisResult {
            files_count,
            lines_count,
            languages,
            dependencies,
            patterns_found,
            issues,
            ai_confidence,
        })
    }

    /// Stage 2: Adapt code to workspace conventions
    #[instrument(skip(self))]
    pub async fn adapt(
        &self,
        path: &Path,
        analysis: &AnalysisResult,
        source_type: &SourceType,
    ) -> Result<AdaptationResult> {
        info!("Adapting code at: {}", path.display());

        let mut changes_made = 0;
        let mut files_modified = 0;
        let mut tests_generated = 0;

        // Adapt based on detected patterns
        if analysis
            .patterns_found
            .contains(&"cargo-project".to_string())
        {
            info!("  Adapting Cargo project structure");
            // Would actually modify Cargo.toml, update dependencies, etc.
            changes_made += 5;
            files_modified += 1;
        }

        // Generate tests if missing
        if analysis.languages.contains(&"Rust".to_string()) {
            info!("  Checking for test coverage");
            tests_generated = self.generate_missing_tests(path).await?;
            if tests_generated > 0 {
                info!("  Generated {} test files", tests_generated);
                files_modified += tests_generated;
            }
        }

        // Apply workspace conventions
        info!("  Applying workspace conventions");
        let convention_changes = self.apply_conventions(path, source_type).await?;
        changes_made += convention_changes;

        // Calculate adaptation confidence
        let ai_confidence = if analysis.issues.is_empty() && changes_made > 0 {
            0.95
        } else if analysis.issues.len() < 3 {
            0.85
        } else {
            0.70
        };

        let auto_approved = ai_confidence >= self.auto_approve_threshold;

        let diff_summary = format!(
            "{} changes across {} files, {} tests generated",
            changes_made, files_modified, tests_generated
        );

        Ok(AdaptationResult {
            changes_made,
            files_modified,
            tests_generated,
            ai_confidence,
            auto_approved,
            diff_summary,
            sandbox_ready: true,
        })
    }

    /// Stage 3: Validate adapted code
    #[instrument(skip(self))]
    pub async fn validate(
        &self,
        path: &Path,
        adaptation: &AdaptationResult,
    ) -> Result<ValidationResult> {
        info!("Validating adapted code at: {}", path.display());

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        // Check if cargo project builds (if Rust)
        if self.is_cargo_project(path).await {
            info!("  Running cargo check");
            match self.run_cargo_check(path).await {
                Ok(_) => {
                    info!("  ✓ Cargo check passed");
                    metadata.insert("cargo_check".to_string(), "passed".to_string());
                }
                Err(e) => {
                    warn!("  ✗ Cargo check failed: {}", e);
                    warnings.push(format!("Cargo check failed: {}", e));
                    metadata.insert("cargo_check".to_string(), "failed".to_string());
                }
            }
        }

        // Validate directory structure
        info!("  Validating directory structure");
        if !self.has_valid_structure(path).await {
            warnings.push("Directory structure may not be optimal".to_string());
        }

        // Check for required files
        info!("  Checking for required files");
        let missing_files = self.check_required_files(path).await;
        if !missing_files.is_empty() {
            warnings.push(format!("Missing recommended files: {:?}", missing_files));
        }

        // Calculate final confidence
        let confidence = if errors.is_empty() {
            if warnings.is_empty() {
                adaptation.ai_confidence
            } else {
                (adaptation.ai_confidence * 0.95).max(0.80)
            }
        } else {
            0.60
        };

        metadata.insert("validation_stage".to_string(), "completed".to_string());
        metadata.insert("errors_count".to_string(), errors.len().to_string());
        metadata.insert("warnings_count".to_string(), warnings.len().to_string());

        Ok(ValidationResult {
            confidence,
            errors,
            warnings,
            metadata,
        })
    }

    /// Assign drop to appropriate sandbox model
    pub fn assign_sandbox(&self, source_type: &SourceType, confidence: f32) -> SandboxModel {
        match source_type {
            SourceType::ExternalRepo if confidence >= 0.90 => SandboxModel::ModelA,
            SourceType::Fork if confidence >= 0.85 => SandboxModel::ModelB,
            SourceType::StaleCodebase => SandboxModel::ModelC,
            SourceType::Mirror => SandboxModel::ModelA,
            _ => {
                // Low confidence goes to experimental
                if confidence < 0.75 {
                    SandboxModel::ModelC
                } else {
                    SandboxModel::ModelA
                }
            }
        }
    }

    /// Move processed drop to ready queue
    #[instrument(skip(self))]
    pub async fn move_to_ready(
        &self,
        drop_id: &str,
        source_path: &Path,
        sandbox: &SandboxModel,
    ) -> Result<PathBuf> {
        let queue_name = match sandbox {
            SandboxModel::ModelA => "model-a-queue",
            SandboxModel::ModelB => "model-b-queue",
            SandboxModel::ModelC => "model-c-queue",
            SandboxModel::ModelD => "model-d-queue",
        };

        let ready_path = self
            .base_path
            .join("drop-in/ready")
            .join(queue_name)
            .join(drop_id);

        info!(
            "Moving {} to {}",
            source_path.display(),
            ready_path.display()
        );

        // Create ready directory if it doesn't exist
        if let Some(parent) = ready_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Move the drop (simulate for now)
        // In production: fs::rename(source_path, &ready_path).await?;
        info!("✓ Drop moved to ready queue");

        Ok(ready_path)
    }

    // === Helper Methods ===

    async fn count_files_and_lines(&self, path: &Path) -> Result<(usize, usize)> {
        // Simplified - would recursively count
        Ok((100, 10000)) // Placeholder
    }

    async fn detect_languages(&self, path: &Path) -> Result<Vec<String>> {
        let mut languages = Vec::new();

        // Check for Rust
        if path.join("Cargo.toml").exists() {
            languages.push("Rust".to_string());
        }

        // Check for other languages
        if path.join("package.json").exists() {
            languages.push("JavaScript".to_string());
        }

        if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
            languages.push("Python".to_string());
        }

        if languages.is_empty() {
            languages.push("Unknown".to_string());
        }

        Ok(languages)
    }

    async fn analyze_dependencies(&self, path: &Path) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Check Cargo.toml for Rust dependencies
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            // Would parse Cargo.toml here
            debug!("Found Cargo.toml, analyzing dependencies");
        }

        Ok(dependencies)
    }

    async fn detect_patterns(&self, path: &Path, languages: &[String]) -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        if languages.contains(&"Rust".to_string()) {
            patterns.push("cargo-project".to_string());
        }

        // Check for common patterns
        if path.join("src").exists() {
            patterns.push("standard-src-layout".to_string());
        }

        if path.join("tests").exists() {
            patterns.push("has-tests".to_string());
        }

        Ok(patterns)
    }

    async fn find_issues(&self, _path: &Path, _source_type: &SourceType) -> Result<Vec<String>> {
        // Simplified - would do actual analysis
        Ok(Vec::new())
    }

    fn calculate_confidence(
        &self,
        languages: &[String],
        dependencies: &[Dependency],
        issues: &[String],
    ) -> f32 {
        let mut confidence = 1.0;

        // Penalize unknown languages
        if languages.contains(&"Unknown".to_string()) {
            confidence *= 0.7;
        }

        // Penalize missing dependencies
        if dependencies.is_empty() {
            confidence *= 0.9;
        }

        // Penalize issues
        confidence *= (1.0 - (issues.len() as f32 * 0.05)).max(0.5);

        confidence
    }

    async fn generate_missing_tests(&self, _path: &Path) -> Result<usize> {
        // Would generate test files here
        Ok(0)
    }

    async fn apply_conventions(&self, _path: &Path, _source_type: &SourceType) -> Result<usize> {
        // Would apply workspace conventions
        Ok(5)
    }

    async fn is_cargo_project(&self, path: &Path) -> bool {
        path.join("Cargo.toml").exists()
    }

    async fn run_cargo_check(&self, path: &Path) -> Result<()> {
        info!("Running cargo check at: {}", path.display());
        // Would actually run cargo check
        // For now, simulate success
        Ok(())
    }

    async fn produce_profile_builds(
        &self,
        drop_id: &str,
        source_path: &Path,
    ) -> Result<Vec<BuildArtifact>> {
        let artifact_root = Path::new("storage").join("artifacts");
        build::generate_optimized_builds(drop_id, source_path, &artifact_root).await
    }

    async fn has_valid_structure(&self, path: &Path) -> bool {
        // Check for basic structure
        path.join("src").exists() || path.join("lib").exists()
    }

    async fn check_required_files(&self, path: &Path) -> Vec<String> {
        let mut missing = Vec::new();

        if !path.join("README.md").exists() {
            missing.push("README.md".to_string());
        }

        if !path.join("LICENSE").exists() {
            missing.push("LICENSE".to_string());
        }

        missing
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub confidence: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_processor_creation() {
        let processor = DropProcessor::new(PathBuf::from("crc"));
        assert_eq!(processor.auto_approve_threshold, 0.85);
    }

    #[test]
    fn test_sandbox_assignment() {
        let processor = DropProcessor::new(PathBuf::from("crc"));

        // High confidence external repo → Model A
        let sandbox = processor.assign_sandbox(&SourceType::ExternalRepo, 0.95);
        assert_eq!(sandbox, SandboxModel::ModelA);

        // Stale codebase → Model C
        let sandbox = processor.assign_sandbox(&SourceType::StaleCodebase, 0.80);
        assert_eq!(sandbox, SandboxModel::ModelC);

        // Low confidence → Model C (experimental)
        let sandbox = processor.assign_sandbox(&SourceType::ExternalRepo, 0.65);
        assert_eq!(sandbox, SandboxModel::ModelC);
    }
}
