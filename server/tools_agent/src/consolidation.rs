use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// Capability extracted from source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    Function(String),
    Struct(String),
    Enum(String),
    Trait(String),
    Constant(String),
    Type(String),
    Macro(String),
}

impl Capability {
    pub fn name(&self) -> &str {
        match self {
            Capability::Function(n)
            | Capability::Struct(n)
            | Capability::Enum(n)
            | Capability::Trait(n)
            | Capability::Constant(n)
            | Capability::Type(n)
            | Capability::Macro(n) => n,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Capability::Function(_) => "function",
            Capability::Struct(_) => "struct",
            Capability::Enum(_) => "enum",
            Capability::Trait(_) => "trait",
            Capability::Constant(_) => "constant",
            Capability::Type(_) => "type",
            Capability::Macro(_) => "macro",
        }
    }
}

/// Visitor to extract capabilities from Rust AST
struct CapabilityVisitor {
    capabilities: Vec<Capability>,
}

impl CapabilityVisitor {
    fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for CapabilityVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Function(node.sig.ident.to_string()));
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Struct(node.ident.to_string()));
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Enum(node.ident.to_string()));
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Trait(node.ident.to_string()));
        }
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Constant(node.ident.to_string()));
        }
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        if matches!(node.vis, syn::Visibility::Public(_)) {
            self.capabilities
                .push(Capability::Type(node.ident.to_string()));
        }
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if let Some(ident) = &node.ident {
            self.capabilities.push(Capability::Macro(ident.to_string()));
        }
        syn::visit::visit_item_macro(self, node);
    }
}

/// Extract capabilities from a Rust source file
pub fn extract_capabilities(file_path: &Path) -> Result<Vec<Capability>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {}", file_path.display()))?;

    let mut visitor = CapabilityVisitor::new();
    visitor.visit_file(&syntax);

    Ok(visitor.capabilities)
}

/// Consolidation metadata for a single version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationVersion {
    pub version: String,
    pub source_path: String,
    pub timestamp: String,
    pub sha256: String,
    pub consolidation_reason: String,
    pub preserved_capabilities: Vec<String>,
    pub archived_capabilities: Vec<String>,
    pub merged_by: String,
}

/// Version ledger for tracking consolidations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionLedger {
    pub canonical_file: String,
    pub versions: Vec<ConsolidationVersion>,
}

impl VersionLedger {
    pub fn new(canonical_file: String) -> Self {
        Self {
            canonical_file,
            versions: Vec::new(),
        }
    }

    pub fn add_version(&mut self, version: ConsolidationVersion) {
        self.versions.push(version);
    }

    pub fn next_version_number(&self) -> u32 {
        self.versions.len() as u32 + 1
    }
}

/// Consolidation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub date: String,
    pub canonical_file: String,
    pub sources_merged: usize,
    pub capability_comparison: Vec<CapabilityComparison>,
    pub tests_passed: bool,
    pub total_preserved: usize,
    pub total_archived: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityComparison {
    pub source_file: String,
    pub function_count: usize,
    pub preserved_count: usize,
    pub notes: String,
}

impl ConsolidationReport {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Consolidation Report\n\n");
        md.push_str(&format!("**Date**: {}\n", self.date));
        md.push_str(&format!("**Canonical File**: `{}`\n", self.canonical_file));
        md.push_str(&format!("**Sources Merged**: {}\n\n", self.sources_merged));

        md.push_str("## Capability Comparison\n\n");
        md.push_str("| Source File | Functions | Preserved | Notes |\n");
        md.push_str("|-------------|-----------|-----------|-------|\n");
        for comp in &self.capability_comparison {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                comp.source_file, comp.function_count, comp.preserved_count, comp.notes
            ));
        }

        md.push_str("\n## Tests\n\n");
        md.push_str(&format!(
            "- All existing tests pass: {}\n",
            if self.tests_passed { "✅" } else { "❌" }
        ));
        md.push_str(&format!(
            "- Total capabilities preserved: {}\n",
            self.total_preserved
        ));
        md.push_str(&format!(
            "- Total capabilities archived: {}\n",
            self.total_archived
        ));

        md
    }
}

/// Calculate SHA-256 hash of a file
pub fn calculate_sha256(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path)
        .with_context(|| format!("Failed to read file for hashing: {}", file_path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Archive a file with version number
pub fn archive_file_versioned(file_path: &Path, repo_root: &Path, version: u32) -> Result<PathBuf> {
    use chrono::Datelike;
    use std::io::Write;

    let rel_path = file_path.strip_prefix(repo_root).unwrap_or(file_path);

    let now = chrono::Utc::now();
    let archive_dir = repo_root.join(format!(
        "archive/consolidation/{}/{:02}/{}",
        now.year(),
        now.month(),
        rel_path.parent().unwrap_or(Path::new("")).display()
    ));

    fs::create_dir_all(&archive_dir).context("Failed to create archive directory")?;

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let tar_path = archive_dir.join(format!("{}_v{}.tar", file_stem, version));
    let zst_path = archive_dir.join(format!("{}_v{}.tar.zst", file_stem, version));

    // Create tar archive
    let tar_file = fs::File::create(&tar_path)?;
    let mut builder = tar::Builder::new(tar_file);
    let data = fs::read(file_path)?;
    builder.append_data(&mut tar::Header::new_gnu(), rel_path, data.as_slice())?;
    builder.finish()?;

    // Compress with zstd
    let tar_bytes = fs::read(&tar_path)?;
    let mut encoder = zstd::stream::write::Encoder::new(fs::File::create(&zst_path)?, 3)?;
    encoder.write_all(&tar_bytes)?;
    encoder.finish()?;

    // Clean up uncompressed tar
    let _ = fs::remove_file(&tar_path);

    Ok(zst_path)
}

/// Consolidation index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationIndexEntry {
    pub canonical_file: String,
    pub version_count: usize,
    pub last_consolidation: String,
    pub ledger_path: String,
}

/// Master consolidation index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationIndex {
    pub last_updated: String,
    pub total_consolidations: usize,
    pub entries: Vec<ConsolidationIndexEntry>,
}

impl ConsolidationIndex {
    pub fn new() -> Self {
        Self {
            last_updated: chrono::Utc::now().to_rfc3339(),
            total_consolidations: 0,
            entries: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).context("Failed to parse consolidation index")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: ConsolidationIndexEntry) {
        // Update if exists, otherwise add
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| e.canonical_file == entry.canonical_file)
        {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
        self.total_consolidations += 1;
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

impl Default for ConsolidationIndex {
    fn default() -> Self {
        Self::new()
    }
}
