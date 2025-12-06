use noa_core::security::{self, Permission, UserId};
use thiserror::Error;

/// Declarative policy representation tying permissions to intent.
#[derive(Debug, Clone)]
pub struct GatewayPolicy {
    pub permission: Permission,
    pub description: &'static str,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("user {user_id} is missing required permission {permission:?}")]
    MissingPermission {
        user_id: UserId,
        permission: Permission,
    },
}

/// Policy enforcer bridging to the core security module.
#[derive(Debug, Default, Clone)]
pub struct PolicyEnforcer {
    policies: Vec<GatewayPolicy>,
}

impl PolicyEnforcer {
    pub fn new() -> Self {
        Self {
            policies: vec![
                GatewayPolicy {
                    permission: Permission::Read,
                    description: "Read access required for routing metadata",
                },
                GatewayPolicy {
                    permission: Permission::Write,
                    description: "Write access required for mutation routes",
                },
                GatewayPolicy {
                    permission: Permission::Execute,
                    description: "Execute access required for workflow triggers",
                },
            ],
        }
    }

    pub fn enforce(&self, user_id: UserId, permission: Permission) -> Result<(), PolicyError> {
        if security::check_permission(user_id, permission.clone()) {
            Ok(())
        } else {
            Err(PolicyError::MissingPermission {
                user_id,
                permission,
            })
        }
    }

    pub fn policies(&self) -> &[GatewayPolicy] {
        &self.policies
    }
}
