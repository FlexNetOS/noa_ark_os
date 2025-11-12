//! File system interface with registry metadata syncing

use crate::memory;
use crate::memory::{RegistryGraph, RegistryNode};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::{Mutex, OnceLock};

const DEFAULT_FILE_MODE: u32 = 0o644;

#[derive(Debug, Clone, Default)]
pub struct FileMetadata {
    pub components: Vec<ComponentMetadata>,
}

impl FileMetadata {
    pub fn add_component(&mut self, component: ComponentMetadata) {
        if let Some(existing) = self
            .components
            .iter_mut()
            .find(|metadata| metadata.id == component.id)
        {
            *existing = component;
        } else {
            self.components.push(component);
        }
    }

    pub fn clear(&mut self) {
        self.components.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub owners: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: String,
    pub permissions: u32,
    pub size: usize,
    pub metadata: FileMetadata,
}

fn file_table() -> &'static Arc<Mutex<HashMap<String, FileDescriptor>>> {
    static FILE_TABLE: OnceLock<Arc<Mutex<HashMap<String, FileDescriptor>>>> = OnceLock::new();
    FILE_TABLE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
static FILE_TABLE: OnceLock<Mutex<HashMap<String, FileDescriptor>>> = OnceLock::new();

fn file_table() -> &'static Mutex<HashMap<String, FileDescriptor>> {
    FILE_TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Errors surfaced by the virtual file system module.
#[derive(Debug)]
pub enum FsError {
    StatePoisoned,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::StatePoisoned => write!(f, "file table mutex poisoned"),
        }
    }
}

impl std::error::Error for FsError {}

/// Initialize file system
pub fn init() -> Result<(), &'static str> {
    println!("[FS] Initializing file system...");

    // Create root directory
    let root = FileDescriptor {
        path: "/".to_string(),
        permissions: 0o755,
        size: 0,
        metadata: FileMetadata::default(),
    };

    let mut table = file_table().lock().unwrap();
    table.insert("/".to_string(), root);
    drop(table);

    if let Err(err) = sync_registry_metadata() {
        eprintln!("[FS] Failed to sync registry metadata: {err}");
        return Err("filesystem initialization failed");
    }

    Ok(())
}

fn create_file_inner(path: String, permissions: u32) -> Result<(), &'static str> {
    let descriptor = FileDescriptor {
        path: path.clone(),
        permissions,
        size: 0,
        metadata: FileMetadata::default(),
    };

    let mut table = file_table().lock().unwrap();
    table.insert(path, descriptor);

    Ok(())
}

fn get_file_inner(path: &str) -> Option<FileDescriptor> {
    let table = file_table().lock().unwrap();
    table.get(path).cloned()
}

/// Synchronise file descriptors with registry metadata.
pub fn sync_registry_metadata() -> Result<(), FsError> {
    let snapshot = memory::registry_snapshot().map_err(|_| FsError::StatePoisoned)?;
    let mut table = file_table().lock().map_err(|_| FsError::StatePoisoned)?;

    for descriptor in table.values_mut() {
        descriptor.metadata.clear();
    }

    for (path, nodes) in snapshot.file_component_mappings() {
        let entry = table.entry(path.clone()).or_insert_with(|| FileDescriptor {
            path: path.clone(),
            permissions: DEFAULT_FILE_MODE,
            size: 0,
            metadata: FileMetadata::default(),
        });

        for node in nodes {
            entry
                .metadata
                .add_component(to_component_metadata(&snapshot, &node));
        }
    }

    Ok(())
}

fn to_component_metadata(graph: &RegistryGraph, node: &RegistryNode) -> ComponentMetadata {
    let owners = graph
        .owners_for(&node.owners)
        .into_iter()
        .map(|owner| owner.name)
        .collect();

    ComponentMetadata {
        id: node.id.clone(),
        name: node.name.clone(),
        version: node.version.clone(),
        owners,
    }
}

/// Kernel-managed file-system capability.
#[derive(Clone, Default)]
pub struct FileSystemService;

impl FileSystemService {
    /// Create a file entry tracked by the kernel registry.
    pub fn create_file(&self, path: String, permissions: u32) -> Result<(), &'static str> {
        create_file_inner(path, permissions)
    }

    /// Lookup a file descriptor.
    pub fn get_file(&self, path: &str) -> Option<FileDescriptor> {
        get_file_inner(path)
    }

    /// List all tracked descriptors.
    pub fn list(&self) -> Vec<FileDescriptor> {
        let table = file_table().lock().unwrap();
        table.values().cloned().collect()
    }
}

/// Create a file.
pub fn create_file(path: String, permissions: u32) -> Result<(), &'static str> {
    FileSystemService::default().create_file(path, permissions)
}

/// Get file descriptor.
pub fn get_file(path: &str) -> Option<FileDescriptor> {
    FileSystemService::default().get_file(path)
}

/// Move a file to a new destination path.
pub fn move_file(source: &str, destination: String) -> Result<FileDescriptor, &'static str> {
    if source == "/" {
        return Err("cannot_move_root");
    }

    let mut table = file_table().lock().unwrap();

    if !table.contains_key(source) {
        return Err("source_not_found");
    }

    if table.contains_key(&destination) && source != destination {
        return Err("destination_exists");
    }

    let mut descriptor = table.remove(source).ok_or("source_not_found")?;
    descriptor.path = destination.clone();
    table.insert(destination.clone(), descriptor.clone());

    Ok(descriptor)
}

/// Delete a file from the table.
pub fn delete_file(path: &str) -> Result<(), &'static str> {
    if path == "/" {
        return Err("cannot_delete_root");
    }

    let mut table = file_table().lock().unwrap();

    table.remove(path).ok_or("file_not_found")?;

    Ok(())
}

/// List all tracked files.
pub fn list_files() -> Vec<FileDescriptor> {
    let table = file_table().lock().unwrap();
    table.values().cloned().collect()
}
