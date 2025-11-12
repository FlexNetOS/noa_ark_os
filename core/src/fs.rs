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

/// Create a file
pub fn create_file(path: String, permissions: u32) -> Result<(), &'static str> {
    let descriptor = FileDescriptor {
        path: path.clone(),
        permissions,
        size: 0,
    };

    let mut table = FILE_TABLE.lock().unwrap();
    table.insert(path, descriptor);

    Ok(())
}

/// Get file descriptor
pub fn get_file(path: &str) -> Option<FileDescriptor> {
    let table = FILE_TABLE.lock().unwrap();
    table.get(path).cloned()
}

/// Move a file to a new destination path.
pub fn move_file(source: &str, destination: String) -> Result<FileDescriptor, &'static str> {
    if source == "/" {
        return Err("cannot_move_root");
    }

    let mut table = FILE_TABLE.lock().unwrap();

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

    let mut table = FILE_TABLE.lock().unwrap();

    table.remove(path).ok_or("file_not_found")?;

    Ok(())
}

/// List all tracked files.
pub fn list_files() -> Vec<FileDescriptor> {
    let table = FILE_TABLE.lock().unwrap();
    table.values().cloned().collect()
}
