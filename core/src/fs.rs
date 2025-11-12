//! File system interface

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: String,
    pub permissions: u32,
    pub size: usize,
}

lazy_static::lazy_static! {
    static ref FILE_TABLE: Arc<Mutex<HashMap<String, FileDescriptor>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize file system
pub fn init() -> Result<(), &'static str> {
    println!("[FS] Initializing file system...");

    // Create root directory
    let root = FileDescriptor {
        path: "/".to_string(),
        permissions: 0o755,
        size: 0,
    };

    let mut table = FILE_TABLE.lock().unwrap();
    table.insert("/".to_string(), root);

    Ok(())
}

fn create_file_inner(path: String, permissions: u32) -> Result<(), &'static str> {
    let descriptor = FileDescriptor {
        path: path.clone(),
        permissions,
        size: 0,
    };

    let mut table = FILE_TABLE.lock().unwrap();
    table.insert(path, descriptor);

    Ok(())
}

fn get_file_inner(path: &str) -> Option<FileDescriptor> {
    let table = FILE_TABLE.lock().unwrap();
    table.get(path).cloned()
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
        let table = FILE_TABLE.lock().unwrap();
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
