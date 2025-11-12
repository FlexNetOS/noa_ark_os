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

fn check_permission_inner(user_id: UserId, permission: Permission) -> bool {
    let table = USER_TABLE.lock().unwrap();
    if let Some(user) = table.get(&user_id) {
        user.permissions.contains(&Permission::Admin) || user.permissions.contains(&permission)
    } else {
        false
    }
}

fn register_user_inner(user: User) {
    let mut table = USER_TABLE.lock().unwrap();
    table.insert(user.id, user);
}

/// Kernel-managed security capability.
#[derive(Clone, Default)]
pub struct SecurityService;

impl SecurityService {
    /// Register or update a user.
    pub fn register_user(&self, user: User) {
        register_user_inner(user);
    }

    /// Validate a permission check.
    pub fn check_permission(&self, user_id: UserId, permission: Permission) -> bool {
        check_permission_inner(user_id, permission)
    }
}

/// Check if user has permission.
pub fn check_permission(user_id: UserId, permission: Permission) -> bool {
    SecurityService::default().check_permission(user_id, permission)
}
