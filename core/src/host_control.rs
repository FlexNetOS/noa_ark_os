//! Host control surface enabling environment takeover and resource arbitration.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::config::manifest::{SCOPE_HOST_ENVIRONMENT_TAKEOVER, SCOPE_HOST_RESOURCE_ARBITRATE};
use crate::time::current_timestamp_millis;
use crate::token::{self, TokenError};

/// Lease describing a token-bound environment takeover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentLease {
    pub environment: String,
    pub token: String,
    pub issued_to: String,
    pub granted_at: u128,
    pub expires_at: u128,
}

/// Request payload for resource arbitration decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceArbitrationRequest {
    pub environment: String,
    pub desired_cpu_share: f32,
    pub desired_memory_bytes: u64,
}

/// Arbitration decision containing the granted resource envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceArbitrationDecision {
    pub environment: String,
    pub granted_cpu_share: f32,
    pub granted_memory_bytes: u64,
    pub isolation_enforced: bool,
}

/// Errors produced by the host control surface.
#[derive(Debug, thiserror::Error)]
pub enum HostControlError {
    #[error(transparent)]
    Token(#[from] TokenError),
    #[error("environment {0} already leased by a different token")]
    EnvironmentInUse(String),
    #[error("environment {0} has no active lease")]
    EnvironmentNotLeased(String),
    #[error("environment {0} isolated from token")]
    EnvironmentIsolationViolation(String),
}

#[derive(Debug, Default)]
struct LeaseStore {
    leases: HashMap<String, EnvironmentLease>,
}

/// Host control service responsible for coordinating isolation.
pub struct HostControlService {
    store: Mutex<LeaseStore>,
}

impl HostControlService {
    fn new() -> Self {
        Self {
            store: Mutex::new(LeaseStore::default()),
        }
    }

    /// Request an environment takeover using the provided token.
    pub fn request_environment_takeover(
        &self,
        token: &str,
        environment: impl Into<String>,
    ) -> Result<EnvironmentLease, HostControlError> {
        let environment = environment.into();
        let validated = token::service().validate(token, SCOPE_HOST_ENVIRONMENT_TAKEOVER)?;
        let mut store = self.store.lock().expect("lease store mutex poisoned");
        if let Some(existing) = store.leases.get(&environment) {
            if existing.token != token {
                return Err(HostControlError::EnvironmentInUse(environment));
            }
            return Ok(existing.clone());
        }

        let lease = EnvironmentLease {
            environment: environment.clone(),
            token: validated.token.clone(),
            issued_to: validated.issued_to.clone(),
            granted_at: current_timestamp_millis(),
            expires_at: validated.expires_at,
        };
        store.leases.insert(environment, lease.clone());
        Ok(lease)
    }

    /// Release an active environment lease.
    pub fn release_environment(
        &self,
        token: &str,
        environment: impl Into<String>,
    ) -> Result<(), HostControlError> {
        let environment = environment.into();
        let validated = token::service().validate(token, SCOPE_HOST_ENVIRONMENT_TAKEOVER)?;
        let mut store = self.store.lock().expect("lease store mutex poisoned");
        match store.leases.get(&environment) {
            Some(existing) if existing.token == validated.token => {
                store.leases.remove(&environment);
                Ok(())
            }
            Some(_) => Err(HostControlError::EnvironmentIsolationViolation(environment)),
            None => Err(HostControlError::EnvironmentNotLeased(environment)),
        }
    }

    /// Arbitrate resource usage for an environment.
    pub fn arbitrate_resources(
        &self,
        token: &str,
        request: ResourceArbitrationRequest,
    ) -> Result<ResourceArbitrationDecision, HostControlError> {
        let validated = token::service().validate(token, SCOPE_HOST_RESOURCE_ARBITRATE)?;
        let store = self.store.lock().expect("lease store mutex poisoned");
        let lease = store
            .leases
            .get(&request.environment)
            .ok_or_else(|| HostControlError::EnvironmentNotLeased(request.environment.clone()))?;

        if lease.token != validated.token {
            return Err(HostControlError::EnvironmentIsolationViolation(
                request.environment,
            ));
        }

        // Clamp CPU usage to 75% of requested share and memory to 80%.
        let granted_cpu = request.desired_cpu_share.clamp(0.0, 1.0).min(0.75);
        let granted_memory = ((request.desired_memory_bytes as f64) * 0.8) as u64;

        Ok(ResourceArbitrationDecision {
            environment: request.environment,
            granted_cpu_share: granted_cpu,
            granted_memory_bytes: granted_memory,
            isolation_enforced: true,
        })
    }

    /// Enumerate active leases.
    pub fn active_leases(&self) -> Vec<EnvironmentLease> {
        let store = self.store.lock().expect("lease store mutex poisoned");
        store.leases.values().cloned().collect()
    }

    /// Reset the host control service (test helper).
    pub fn reset(&self) {
        let mut store = self.store.lock().expect("lease store mutex poisoned");
        store.leases.clear();
    }
}

fn global_host_control() -> &'static HostControlService {
    static HOST_CONTROL: OnceLock<HostControlService> = OnceLock::new();
    HOST_CONTROL.get_or_init(HostControlService::new)
}

/// Access the global host control service.
pub fn service() -> &'static HostControlService {
    global_host_control()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::manifest::KernelManifest;
    use crate::token::{service as token_service, TokenIssuanceRequest};

    fn setup() {
        token_service().reset();
        let manifest = KernelManifest::default();
        token::configure_from_manifest(&manifest);
        service().reset();
    }

    #[test]
    fn lease_requires_valid_scope() {
        setup();
        let request = TokenIssuanceRequest::new(
            "tester",
            [
                SCOPE_HOST_ENVIRONMENT_TAKEOVER,
                SCOPE_HOST_RESOURCE_ARBITRATE,
            ],
        );
        let token = token_service()
            .issue_token(request)
            .expect("token should issue");
        let lease = service()
            .request_environment_takeover(&token.token, "lab")
            .expect("lease should be granted");
        assert_eq!(lease.environment, "lab");
    }
}
