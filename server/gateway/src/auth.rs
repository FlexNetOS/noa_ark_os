use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Credentials supported by the unified authenticator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// Mutually authenticated TLS certificate fingerprint
    pub mtls: Option<String>,
    /// OpenID Connect identity token (already verified upstream)
    pub oidc: Option<String>,
    /// Static API key for automation clients
    pub api_key: Option<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("no credentials supplied")]
    MissingCredentials,
    #[error("mTLS credential rejected")]
    InvalidMtls,
    #[error("oidc token rejected")]
    InvalidOidc,
    #[error("api key rejected")]
    InvalidApiKey,
}

/// Unified authenticator verifying multiple credential types.
#[derive(Debug, Clone)]
pub struct UnifiedAuthenticator {
    trusted_api_keys: Vec<String>,
}

impl UnifiedAuthenticator {
    pub fn new(trusted_api_keys: Vec<String>) -> Self {
        Self { trusted_api_keys }
    }

    pub fn verify(
        &self,
        credentials: &AuthCredentials,
        agent_id: &Option<String>,
    ) -> Result<(), AuthError> {
        if credentials.mtls.is_none() && credentials.oidc.is_none() && credentials.api_key.is_none()
        {
            return Err(AuthError::MissingCredentials);
        }

        if let Some(fingerprint) = &credentials.mtls {
            if fingerprint.is_empty() {
                return Err(AuthError::InvalidMtls);
            }
        }

        if let Some(token) = &credentials.oidc {
            if !token.starts_with("id-") || token.len() < 3 {
                return Err(AuthError::InvalidOidc);
            }
        }

        if let Some(key) = &credentials.api_key {
            if !self.trusted_api_keys.contains(key) {
                if let Some(_agent) = agent_id {
                    // Allow agents with dedicated mTLS but unknown API key to continue when
                    // they present certificate material. This mirrors dual-mode auth.
                    if credentials.mtls.is_none() {
                        return Err(AuthError::InvalidApiKey);
                    }
                } else {
                    return Err(AuthError::InvalidApiKey);
                }
            }
        }

        Ok(())
    }
}

impl Default for UnifiedAuthenticator {
    fn default() -> Self {
        Self::new(vec![
            "key-123".into(),
            "key-ops".into(),
            "key-agents".into(),
        ])
    }
}
