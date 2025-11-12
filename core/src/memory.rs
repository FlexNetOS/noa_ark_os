//! Memory management subsystem with registry graph ingestion

use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

const SUPPORTED_REGISTRY_VERSION: &str = "1.0.0";

static ALLOCATED_MEMORY: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref REGISTRY_GRAPH: Arc<RwLock<RegistryGraph>> =
        Arc::new(RwLock::new(RegistryGraph::default()));
}

/// Initialize memory management and load the registry graph.
pub fn init() -> Result<(), &'static str> {
    println!("[MEMORY] Initializing memory manager...");

    if let Err(err) = load_registry(".workspace/registry") {
        eprintln!("[MEMORY] Failed to ingest registry: {err}");
        return Err("memory registry initialization failed");
    }

    Ok(())
}

/// Track memory allocation
pub fn allocate(size: usize) -> Result<(), &'static str> {
    ALLOCATED_MEMORY.fetch_add(size, Ordering::SeqCst);
    Ok(())
}

/// Track memory deallocation
pub fn deallocate(size: usize) -> Result<(), &'static str> {
    ALLOCATED_MEMORY.fetch_sub(size, Ordering::SeqCst);
    Ok(())
}

/// Get total allocated memory
pub fn get_allocated() -> usize {
    ALLOCATED_MEMORY.load(Ordering::SeqCst)
}

/// Load registry data from the provided directory path.
pub fn load_registry<P: AsRef<Path>>(root: P) -> Result<(), RegistryError> {
    let root = root.as_ref();
    if !root.exists() {
        return Err(RegistryError::Validation(format!(
            "registry directory '{:?}' does not exist",
            root
        )));
    }

    let mut graph = RegistryGraph::default();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !is_registry_document(&path) {
            continue;
        }

        let contents = fs::read_to_string(&path)?;
        let document: RegistryDocument =
            serde_json::from_str(&contents).map_err(|source| RegistryError::Parse {
                path: path.clone(),
                source,
            })?;
        graph.ingest_document(document, path)?;
    }

    graph.validate()?;

    let mut global = REGISTRY_GRAPH
        .write()
        .map_err(|_| RegistryError::PoisonedState("registry graph".into()))?;
    *global = graph;

    Ok(())
}

/// Obtain an immutable snapshot of the registry graph.
pub fn registry_snapshot() -> RegistryGraph {
    REGISTRY_GRAPH
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Fetch a component by ID.
pub fn registry_component(id: &str) -> Option<RegistryNode> {
    REGISTRY_GRAPH
        .read()
        .ok()
        .and_then(|graph| graph.component(id))
}

/// Fetch the dependencies for a given component ID.
pub fn registry_dependencies(id: &str) -> Vec<RegistryNode> {
    REGISTRY_GRAPH
        .read()
        .map(|graph| graph.dependencies_of(id))
        .unwrap_or_default()
}

/// Fetch the dependents for a given component ID.
pub fn registry_dependents(id: &str) -> Vec<RegistryNode> {
    REGISTRY_GRAPH
        .read()
        .map(|graph| graph.dependents_of(id))
        .unwrap_or_default()
}

/// Fetch component nodes associated with a given file.
pub fn registry_components_for_file(path: &str) -> Vec<RegistryNode> {
    REGISTRY_GRAPH
        .read()
        .map(|graph| graph.components_for_file(path))
        .unwrap_or_default()
}

fn is_registry_document(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => {
            match path.file_name().and_then(|name| name.to_str()) {
                Some(name) if name.eq_ignore_ascii_case("registry.schema.json") => false,
                Some(_) => true,
                None => false,
            }
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RegistryDocument {
    #[serde(rename = "registryVersion")]
    #[serde(default)]
    registry_version: Option<String>,
    #[serde(default)]
    owners: Vec<OwnerEntry>,
    #[serde(default)]
    components: Vec<ComponentEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct OwnerEntry {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    contacts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ComponentEntry {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    files: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    owners: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    links: HashMap<String, String>,
}

/// Representation of the consolidated registry graph.
#[derive(Debug, Clone, Default)]
pub struct RegistryGraph {
    components: HashMap<String, RegistryNode>,
    owners: HashMap<String, RegistryOwner>,
    dependents: HashMap<String, HashSet<String>>,
    file_index: HashMap<String, HashSet<String>>,
}

impl RegistryGraph {
    fn ingest_document(
        &mut self,
        document: RegistryDocument,
        source: PathBuf,
    ) -> Result<(), RegistryError> {
        let RegistryDocument {
            registry_version,
            owners,
            components,
        } = document;

        match registry_version {
            Some(version) if version == SUPPORTED_REGISTRY_VERSION => {}
            Some(version) => {
                return Err(RegistryError::Validation(format!(
                    "document {:?} targets unsupported registryVersion '{version}' (expected {SUPPORTED_REGISTRY_VERSION})",
                    source
                )));
            }
            None => {
                return Err(RegistryError::Validation(format!(
                    "document {:?} missing registryVersion field",
                    source
                )));
            }
        }

        for owner in owners {
            self.insert_owner(owner.into(), source.clone())?;
        }

        for component in components {
            self.insert_component(component.into_registry_node(source.clone()))?;
        }

        Ok(())
    }

    fn insert_owner(&mut self, owner: RegistryOwner, source: PathBuf) -> Result<(), RegistryError> {
        match self.owners.get(&owner.id) {
            Some(existing) if existing != &owner => Err(RegistryError::Conflict {
                kind: "owner",
                id: owner.id.clone(),
                source,
            }),
            _ => {
                self.owners.insert(owner.id.clone(), owner);
                Ok(())
            }
        }
    }

    fn insert_component(&mut self, component: RegistryNode) -> Result<(), RegistryError> {
        if self.components.contains_key(&component.id) {
            return Err(RegistryError::Conflict {
                kind: "component",
                id: component.id.clone(),
                source: component.source.clone(),
            });
        }

        for dependency in &component.dependencies {
            self.dependents
                .entry(dependency.clone())
                .or_default()
                .insert(component.id.clone());
        }

        for file in &component.files {
            let normalized = normalize_path(file);
            self.file_index
                .entry(normalized)
                .or_default()
                .insert(component.id.clone());
        }

        self.components.insert(component.id.clone(), component);

        Ok(())
    }

    fn component(&self, id: &str) -> Option<RegistryNode> {
        self.components.get(id).cloned()
    }

    fn dependencies_of(&self, id: &str) -> Vec<RegistryNode> {
        self.components
            .get(id)
            .map(|node| {
                node.dependencies
                    .iter()
                    .filter_map(|dep| self.components.get(dep))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn dependents_of(&self, id: &str) -> Vec<RegistryNode> {
        self.dependents
            .get(id)
            .map(|dependents| {
                dependents
                    .iter()
                    .filter_map(|node_id| self.components.get(node_id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn components_for_file(&self, path: &str) -> Vec<RegistryNode> {
        let normalized = normalize_path(path);
        self.file_index
            .get(&normalized)
            .map(|component_ids| {
                component_ids
                    .iter()
                    .filter_map(|id| self.components.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn file_component_mappings(&self) -> Vec<(String, Vec<RegistryNode>)> {
        self.file_index
            .iter()
            .map(|(path, ids)| {
                let nodes = ids
                    .iter()
                    .filter_map(|id| self.components.get(id))
                    .cloned()
                    .collect();
                (path.clone(), nodes)
            })
            .collect()
    }

    pub fn owners_for(&self, owner_ids: &[String]) -> Vec<RegistryOwner> {
        owner_ids
            .iter()
            .filter_map(|id| self.owners.get(id))
            .cloned()
            .collect()
    }

    pub fn components(&self) -> Vec<RegistryNode> {
        self.components.values().cloned().collect()
    }

    pub fn owners(&self) -> Vec<RegistryOwner> {
        self.owners.values().cloned().collect()
    }

    fn validate(&self) -> Result<(), RegistryError> {
        let mut missing = Vec::new();

        for (id, component) in &self.components {
            for dependency in &component.dependencies {
                if !self.components.contains_key(dependency) {
                    missing.push(format!(
                        "component '{id}' depends on missing '{dependency}'"
                    ));
                }
            }

            for owner in &component.owners {
                if !self.owners.contains_key(owner) {
                    missing.push(format!(
                        "component '{id}' references unknown owner '{owner}'"
                    ));
                }
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(RegistryError::Validation(missing.join("; ")))
        }
    }
}

fn normalize_path(input: &str) -> String {
    let path = Path::new(input);
    let mut normalized = PathBuf::new();
    for component in path.components() {
        normalized.push(component.as_os_str());
    }
    normalized.to_string_lossy().replace('\\', "/")
}

/// Error type surfaced when working with the registry graph.
#[derive(Debug)]
pub enum RegistryError {
    Io(std::io::Error),
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Conflict {
        kind: &'static str,
        id: String,
        source: PathBuf,
    },
    Validation(String),
    PoisonedState(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Io(err) => write!(f, "I/O error: {err}"),
            RegistryError::Parse { path, source } => {
                write!(f, "failed to parse registry document {:?}: {source}", path)
            }
            RegistryError::Conflict { kind, id, source } => {
                write!(
                    f,
                    "conflicting {kind} definition for '{id}' in document {:?}",
                    source
                )
            }
            RegistryError::Validation(message) => {
                write!(f, "registry validation failed: {message}")
            }
            RegistryError::PoisonedState(resource) => {
                write!(f, "poisoned synchronization primitive for {resource}")
            }
        }
    }
}

impl std::error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RegistryError::Io(err) => Some(err),
            RegistryError::Parse { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        RegistryError::Io(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryOwner {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub contacts: Vec<String>,
}

impl From<OwnerEntry> for RegistryOwner {
    fn from(value: OwnerEntry) -> Self {
        RegistryOwner {
            id: value.id,
            name: value.name,
            description: value.description,
            contacts: value.contacts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryNode {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
    pub owners: Vec<String>,
    pub tags: Vec<String>,
    pub links: HashMap<String, String>,
    pub source: PathBuf,
}

impl ComponentEntry {
    fn into_registry_node(self, source: PathBuf) -> RegistryNode {
        RegistryNode {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            files: self.files,
            dependencies: self.dependencies,
            owners: self.owners,
            tags: self.tags,
            links: self.links,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn registry_ingestion_validates_dependencies() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.json");
        let mut file = fs::File::create(&path).unwrap();
        write!(
            file,
            r#"{{
            "registryVersion": "1.0.0",
            "owners": [{{"id": "owner", "name": "Owner"}}],
            "components": [{{
                "id": "component.a",
                "name": "Component A",
                "version": "0.1.0",
                "files": ["a.rs"],
                "dependencies": [],
                "owners": ["owner"]
            }}]
        }}"#
        )
        .unwrap();

        load_registry(dir.path()).unwrap();
        let snapshot = registry_snapshot();
        assert_eq!(snapshot.components().len(), 1);
        assert_eq!(snapshot.owners().len(), 1);
    }

    #[test]
    fn missing_owner_is_reported() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.json");
        let mut file = fs::File::create(&path).unwrap();
        write!(
            file,
            r#"{{
            "registryVersion": "1.0.0",
            "components": [{{
                "id": "component.a",
                "name": "Component A",
                "version": "0.1.0",
                "files": ["a.rs"],
                "dependencies": [],
                "owners": ["missing"]
            }}]
        }}"#
        )
        .unwrap();

        let error = load_registry(dir.path()).unwrap_err();
        match error {
            RegistryError::Validation(message) => {
                assert!(message.contains("unknown owner 'missing'"));
            }
            other => panic!("unexpected error: {}", other),
        }
    }
}
