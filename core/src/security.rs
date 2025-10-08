//! Security subsystem

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type UserId = u64;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
}

lazy_static::lazy_static! {
    static ref USER_TABLE: Arc<Mutex<HashMap<UserId, User>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize security subsystem
pub fn init() -> Result<(), &'static str> {
    println!("[SECURITY] Initializing security subsystem...");
    
    // Create root user
    let root = User {
        id: 0,
        name: "root".to_string(),
        permissions: vec![Permission::Admin],
    };
    
    let mut table = USER_TABLE.lock().unwrap();
    table.insert(0, root);
    
    Ok(())
}

/// Check if user has permission
pub fn check_permission(user_id: UserId, permission: Permission) -> bool {
    let table = USER_TABLE.lock().unwrap();
    if let Some(user) = table.get(&user_id) {
        user.permissions.contains(&Permission::Admin) || user.permissions.contains(&permission)
    } else {
        false
    }
}
