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
