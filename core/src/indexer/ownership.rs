use serde::{Deserialize, Serialize};

use crate::indexer::IndexerError;
use crate::memory::{registry_snapshot, RegistryGraph, RegistryOwner};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraph {
    pub generated_at: u128,
    pub files: Vec<FileOwnership>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOwnership {
    pub file: String,
    pub components: Vec<ComponentOwnership>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOwnership {
    pub id: String,
    pub name: String,
    pub owners: Vec<OwnerInfo>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerInfo {
    pub id: String,
    pub name: String,
    pub contacts: Vec<String>,
}

impl OwnershipGraph {
    pub fn build() -> Result<Self, IndexerError> {
        let registry = registry_snapshot()?;
        Ok(Self::from_registry(&registry))
    }

    pub fn from_registry(registry: &RegistryGraph) -> Self {
        let mut files = Vec::new();
        for (file, components) in registry.file_component_mappings() {
            let entries = components
                .into_iter()
                .map(|component| ComponentOwnership {
                    id: component.id.clone(),
                    name: component.name.clone(),
                    owners: registry
                        .owners_for(&component.owners)
                        .into_iter()
                        .map(OwnerInfo::from)
                        .collect(),
                    tags: component.tags.clone(),
                    dependencies: component.dependencies.clone(),
                })
                .collect();
            files.push(FileOwnership {
                file,
                components: entries,
            });
        }

        files.sort_by(|a, b| a.file.cmp(&b.file));

        Self {
            generated_at: crate::utils::current_timestamp_millis(),
            files,
        }
    }
}

impl From<RegistryOwner> for OwnerInfo {
    fn from(owner: RegistryOwner) -> Self {
        Self {
            id: owner.id,
            name: owner.name,
            contacts: owner.contacts,
        }
    }
}
