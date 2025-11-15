//! Capability token service responsible for issuing and validating scope-bound tokens.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::config::manifest::TokenPolicyManifestEntry;
use crate::time::current_timestamp_millis;
use crate::utils::simple_hash;

/// Internal counter used to produce deterministic token identifiers.
fn issuance_counter() -> &'static AtomicU64 {
    static COUNTER: OnceLock<AtomicU64> = OnceLock::new();
    COUNTER.get_or_init(|| AtomicU64::new(1))
}

/// Token metadata describing an issued capability ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeToken {
    /// Unique token secret that must accompany capability requests.
    pub token: String,
    /// Actor receiving the token.
    pub issued_to: String,
    /// Scopes authorised by this token.
    pub scopes: Vec<String>,
    /// Timestamp (milliseconds) when the token was issued.
    pub issued_at: u128,
    /// Timestamp (milliseconds) when the token expires.
    pub expires_at: u128,
    /// Optional metadata captured at issuance time.
    pub metadata: HashMap<String, String>,
}

impl ScopeToken {
    /// Whether the token has expired relative to the provided timestamp.
    pub fn is_expired(&self, timestamp_ms: u128) -> bool {
        timestamp_ms >= self.expires_at
    }

    /// Whether the token grants the provided scope.
    pub fn grants_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|candidate| candidate == scope)
    }
}

/// Request payload describing a token to be issued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenIssuanceRequest {
    actor: String,
    scopes: Vec<String>,
    ttl_override: Option<u64>,
    metadata: HashMap<String, String>,
}

impl TokenIssuanceRequest {
    /// Construct a new issuance request for the provided actor and scopes.
    pub fn new(
        actor: impl Into<String>,
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            actor: actor.into(),
            scopes: scopes.into_iter().map(Into::into).collect(),
            ttl_override: None,
            metadata: HashMap::new(),
        }
    }

    /// Override the maximum lifetime permitted by the policy.
    pub fn with_ttl_seconds(mut self, ttl_seconds: u64) -> Self {
        self.ttl_override = Some(ttl_seconds);
        self
    }

    /// Attach metadata to the resulting token.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Access the requested scopes.
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Access the target actor.
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Metadata map to embed in the token.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Policy describing a capability scope exposed via tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPolicy {
    pub scope: String,
    pub description: Option<String>,
    pub ttl_seconds: u64,
    pub capabilities: Vec<String>,
}

impl From<TokenPolicyManifestEntry> for TokenPolicy {
    fn from(entry: TokenPolicyManifestEntry) -> Self {
        Self {
            scope: entry.scope,
            description: entry.description,
            ttl_seconds: entry.ttl_seconds,
            capabilities: entry.capabilities,
        }
    }
}

#[derive(Debug, Default)]
struct TokenStore {
    policies: HashMap<String, TokenPolicy>,
    issued: HashMap<String, ScopeToken>,
    revoked: HashSet<String>,
}

impl TokenStore {
    fn configure(&mut self, policies: Vec<TokenPolicy>) {
        self.policies = policies
            .into_iter()
            .map(|policy| (policy.scope.clone(), policy))
            .collect();
        self.issued.clear();
        self.revoked.clear();
    }
}

/// Errors emitted by the capability token service.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("token service has not been configured")]
    NotConfigured,
    #[error("scope {0} is not registered in the manifest")]
    UnknownScope(String),
    #[error("token {0} is not recognised")]
    UnknownToken(String),
    #[error("token {0} has been revoked")]
    Revoked(String),
    #[error("token expired at {0}")]
    Expired(u128),
    #[error("token missing required scope {0}")]
    ScopeMissing(String),
    #[error("issuance request does not contain any scopes")]
    MissingScopes,
    #[error("requested ttl {requested}s exceeds policy limit {policy}s for scope {scope}")]
    TtlExceedsPolicy {
        scope: String,
        requested: u64,
        policy: u64,
    },
}

/// Global capability token service.
pub struct CapabilityTokenService {
    store: Mutex<TokenStore>,
}

impl CapabilityTokenService {
    fn new() -> Self {
        Self {
            store: Mutex::new(TokenStore::default()),
        }
    }

    /// Configure the service using the manifest token policies.
    pub fn configure(&self, policies: Vec<TokenPolicyManifestEntry>) {
        let policies: Vec<TokenPolicy> = policies.into_iter().map(TokenPolicy::from).collect();
        let mut store = self.store.lock().expect("token store mutex poisoned");
        store.configure(policies);
    }

    /// Reset the token store. Intended for tests.
    pub fn reset(&self) {
        let mut store = self.store.lock().expect("token store mutex poisoned");
        *store = TokenStore::default();
    }

    /// Issue a new token based on the provided request.
    pub fn issue_token(&self, request: TokenIssuanceRequest) -> Result<ScopeToken, TokenError> {
        let mut store = self.store.lock().expect("token store mutex poisoned");
        if store.policies.is_empty() {
            return Err(TokenError::NotConfigured);
        }
        if request.scopes.is_empty() {
            return Err(TokenError::MissingScopes);
        }

        let mut unique_scopes: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for scope in request.scopes.iter() {
            if seen.insert(scope.clone()) {
                if !store.policies.contains_key(scope) {
                    return Err(TokenError::UnknownScope(scope.clone()));
                }
                unique_scopes.push(scope.clone());
            }
        }

        let now = current_timestamp_millis();
        // Find the scope with the minimum TTL (ttl_ceiling)
        let (ttl_ceiling, ttl_scope) = unique_scopes
            .iter()
            .filter_map(|scope| {
                store.policies.get(scope).map(|policy| (policy.ttl_seconds, scope))
            })
            .min_by_key(|(ttl, _scope)| *ttl)
            .map(|(ttl, scope)| (ttl, scope.clone()))
            .expect("at least one scope should be present");

        if let Some(requested_ttl) = request.ttl_override {
            if requested_ttl > ttl_ceiling {
                return Err(TokenError::TtlExceedsPolicy {
                    scope: ttl_scope,
                    requested: requested_ttl,
                    policy: ttl_ceiling,
                });
            }
        }

        let ttl_seconds = request.ttl_override.unwrap_or(ttl_ceiling);
        let expires_at = now + (ttl_seconds as u128 * 1000);
        let counter = issuance_counter().fetch_add(1, Ordering::SeqCst);
        let token_secret = simple_hash(&format!("token::{}::{}::{}", request.actor, now, counter));

        let token = ScopeToken {
            token: token_secret.clone(),
            issued_to: request.actor.clone(),
            scopes: unique_scopes,
            issued_at: now,
            expires_at,
            metadata: request.metadata.clone(),
        };

        store.issued.insert(token_secret, token.clone());
        Ok(token)
    }

    /// Validate a token against the provided scope.
    pub fn validate(&self, token: &str, scope: &str) -> Result<ScopeToken, TokenError> {
        let now = current_timestamp_millis();
        let store = self.store.lock().expect("token store mutex poisoned");
        if store.policies.is_empty() {
            return Err(TokenError::NotConfigured);
        }
        let issued = store
            .issued
            .get(token)
            .cloned()
            .ok_or_else(|| TokenError::UnknownToken(token.to_string()))?;

        if store.revoked.contains(token) {
            return Err(TokenError::Revoked(token.to_string()));
        }
        if issued.is_expired(now) {
            return Err(TokenError::Expired(issued.expires_at));
        }
        if !issued.grants_scope(scope) {
            return Err(TokenError::ScopeMissing(scope.to_string()));
        }
        Ok(issued)
    }

    /// Revoke a previously issued token.
    pub fn revoke(&self, token: &str) -> Result<(), TokenError> {
        let mut store = self.store.lock().expect("token store mutex poisoned");
        if store.policies.is_empty() {
            return Err(TokenError::NotConfigured);
        }
        if !store.issued.contains_key(token) {
            return Err(TokenError::UnknownToken(token.to_string()));
        }
        store.revoked.insert(token.to_string());
        Ok(())
    }

    /// List all configured scopes for diagnostics.
    pub fn configured_scopes(&self) -> Vec<String> {
        let store = self.store.lock().expect("token store mutex poisoned");
        store.policies.keys().cloned().collect()
    }
}

fn global_service() -> &'static CapabilityTokenService {
    static SERVICE: OnceLock<CapabilityTokenService> = OnceLock::new();
    SERVICE.get_or_init(CapabilityTokenService::new)
}

/// Access the global capability token service.
pub fn service() -> &'static CapabilityTokenService {
    global_service()
}

/// Configure the global token service using the manifest policies.
pub fn configure_from_manifest(manifest: &crate::config::manifest::KernelManifest) {
    service().configure(manifest.token_policies().to_vec());
}
