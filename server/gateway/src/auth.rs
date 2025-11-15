use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::warn;

pub(crate) const DEV_OIDC_ISSUER: &str = "https://auth.noaark.dev";
pub(crate) const DEV_OIDC_AUDIENCE: &str = "noa-gateway";
pub(crate) const DEV_OIDC_SECRET: &str = "noa-gateway-dev-secret";
pub(crate) const DEV_AGENT_SAN_PREFIX: &str = "agent:";
pub(crate) const DEV_CONTROL_PLANE_SAN: &str = "gateway.noaark";
pub(crate) const DEV_ALLOWED_FINGERPRINT: &str = "agent-cert";
const DEFAULT_VAULT_SECRET_PATH: &str = "secret/data/gateway/auth";
const DEFAULT_CONFIG_PATH: &str = "server/vault/runtime/gateway_auth.json";

/// Credentials supported by the unified authenticator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// Mutually authenticated TLS identity including subject alternative names.
    pub mtls: Option<MtlsCertificate>,
    /// OpenID Connect identity token (already verified upstream)
    pub oidc: Option<String>,
    /// Static API key for automation clients
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MtlsCertificate {
    pub fingerprint: String,
    #[serde(default)]
    pub subject_alt_names: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("no credentials supplied")]
    MissingCredentials,
    #[error("mTLS credential rejected")]
    InvalidMtls,
    #[error("mTLS SANs missing required entry")]
    InvalidMtlsSan,
    #[error("oidc token rejected: {0}")]
    InvalidOidc(String),
    #[error("api key rejected")]
    InvalidApiKey,
}

#[derive(Debug, Error)]
pub enum AuthConfigError {
    #[error("{0}")]
    Io(String),
    #[error("vault request failed: {0}")]
    Vault(String),
    #[error("invalid config: {0}")]
    Invalid(String),
}

/// Unified authenticator verifying multiple credential types.
#[derive(Clone)]
pub struct UnifiedAuthenticator {
    api_keys: ApiKeyStore,
    mtls: MtlsValidator,
    oidc: OidcValidator,
}

impl UnifiedAuthenticator {
    pub fn new(trusted_api_keys: Vec<String>) -> Self {
        Self::from_config(AuthConfig {
            api_keys: trusted_api_keys,
            mtls: MtlsConfig {
                allowed_fingerprints: vec![DEV_ALLOWED_FINGERPRINT.into()],
                agent_san_prefix: DEV_AGENT_SAN_PREFIX.into(),
                control_plane_san: DEV_CONTROL_PLANE_SAN.into(),
            },
            oidc: OidcConfig {
                issuer: DEV_OIDC_ISSUER.into(),
                audience: DEV_OIDC_AUDIENCE.into(),
                hs256_secret: DEV_OIDC_SECRET.into(),
            },
        })
        .expect("dev config always valid")
    }

    pub fn from_config(config: AuthConfig) -> Result<Self, AuthConfigError> {
        Ok(Self {
            api_keys: ApiKeyStore::new(config.api_keys),
            mtls: MtlsValidator::new(config.mtls),
            oidc: OidcValidator::new(config.oidc)?,
        })
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

        if let Some(cert) = &credentials.mtls {
            self.mtls.verify(cert, agent_id)?;
        } else if agent_id.is_some() {
            // Agents must always connect via mTLS.
            return Err(AuthError::InvalidMtls);
        }

        if let Some(token) = &credentials.oidc {
            self.oidc.verify(token)?;
        }

        if let Some(key) = &credentials.api_key {
            self.api_keys.verify(key)?;
        }

        Ok(())
    }
}

impl Default for UnifiedAuthenticator {
    fn default() -> Self {
        let config = AuthConfig::load_or_default();
        Self::from_config(config).expect("default config valid")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_keys: Vec<String>,
    pub mtls: MtlsConfig,
    pub oidc: OidcConfig,
}

impl AuthConfig {
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(config) => config,
            Err(err) => {
                warn!(
                    target = "gateway::auth",
                    "Falling back to development auth config: {}", err
                );
                Self::development_defaults()
            }
        }
    }

    pub fn load() -> Result<Self, AuthConfigError> {
        if let Ok(path) = env::var("GATEWAY_AUTH_CONFIG") {
            return Self::from_file(path);
        }

        if let (Ok(addr), Ok(token)) = (env::var("VAULT_ADDR"), env::var("VAULT_TOKEN")) {
            let secret_path = env::var("GATEWAY_VAULT_SECRET_PATH")
                .unwrap_or_else(|_| DEFAULT_VAULT_SECRET_PATH.to_string());
            return Self::from_vault(&addr, &token, &secret_path);
        }

        if Path::new(DEFAULT_CONFIG_PATH).exists() {
            return Self::from_file(DEFAULT_CONFIG_PATH);
        }

        Err(AuthConfigError::Invalid(
            "No auth config source provided. Set GATEWAY_AUTH_CONFIG or Vault env vars.".into(),
        ))
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AuthConfigError> {
        let data = fs::read_to_string(path).map_err(|err| AuthConfigError::Io(err.to_string()))?;
        serde_json::from_str(&data).map_err(|err| AuthConfigError::Invalid(err.to_string()))
    }

    fn from_vault(addr: &str, token: &str, secret_path: &str) -> Result<Self, AuthConfigError> {
        let client = Client::new();
        let url = format!("{}/v1/{}", addr.trim_end_matches('/'), secret_path);
        let response = client
            .get(url)
            .header("X-Vault-Token", token)
            .send()
            .map_err(|err| AuthConfigError::Vault(err.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthConfigError::Vault(format!(
                "Vault returned status {}",
                response.status()
            )));
        }

        let payload: VaultSecretResponse = response
            .json()
            .map_err(|err| AuthConfigError::Vault(err.to_string()))?;
        Ok(payload.data.data)
    }

    fn development_defaults() -> Self {
        Self {
            api_keys: vec!["key-123".into(), "key-ops".into(), "key-agents".into()],
            mtls: MtlsConfig {
                allowed_fingerprints: vec![DEV_ALLOWED_FINGERPRINT.into()],
                agent_san_prefix: DEV_AGENT_SAN_PREFIX.into(),
                control_plane_san: DEV_CONTROL_PLANE_SAN.into(),
            },
            oidc: OidcConfig {
                issuer: DEV_OIDC_ISSUER.into(),
                audience: DEV_OIDC_AUDIENCE.into(),
                hs256_secret: DEV_OIDC_SECRET.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    pub allowed_fingerprints: Vec<String>,
    #[serde(default = "default_agent_prefix")]
    pub agent_san_prefix: String,
    #[serde(default = "default_control_plane_san")]
    pub control_plane_san: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer: String,
    pub audience: String,
    pub hs256_secret: String,
}

#[derive(Debug, Deserialize)]
struct VaultSecretResponse {
    data: VaultSecretData,
}

#[derive(Debug, Deserialize)]
struct VaultSecretData {
    data: AuthConfig,
}

fn default_agent_prefix() -> String {
    DEV_AGENT_SAN_PREFIX.into()
}

fn default_control_plane_san() -> String {
    DEV_CONTROL_PLANE_SAN.into()
}

#[derive(Debug, Clone)]
struct ApiKeyStore {
    keys: HashSet<String>,
}

impl ApiKeyStore {
    fn new(keys: Vec<String>) -> Self {
        Self {
            keys: keys.into_iter().map(|k| k.to_ascii_lowercase()).collect(),
        }
    }

    fn verify(&self, candidate: &str) -> Result<(), AuthError> {
        if self.keys.contains(&candidate.to_ascii_lowercase()) {
            Ok(())
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }
}

#[derive(Debug, Clone)]
struct MtlsValidator {
    allowed_fingerprints: HashSet<String>,
    agent_san_prefix: String,
    control_plane_san: String,
}

impl MtlsValidator {
    fn new(config: MtlsConfig) -> Self {
        Self {
            allowed_fingerprints: config
                .allowed_fingerprints
                .into_iter()
                .map(|fp| fp.to_ascii_lowercase())
                .collect(),
            agent_san_prefix: config.agent_san_prefix.to_ascii_lowercase(),
            control_plane_san: config.control_plane_san.to_ascii_lowercase(),
        }
    }

    fn verify(&self, cert: &MtlsCertificate, agent_id: &Option<String>) -> Result<(), AuthError> {
        if !self.allowed_fingerprints.is_empty()
            && !self
                .allowed_fingerprints
                .contains(&cert.fingerprint.to_ascii_lowercase())
        {
            return Err(AuthError::InvalidMtls);
        }

        let sans: HashSet<String> = cert
            .subject_alt_names
            .iter()
            .map(|san| san.to_ascii_lowercase())
            .collect();

        if let Some(agent) = agent_id {
            let expected = format!("{}{}", self.agent_san_prefix, agent.to_ascii_lowercase());
            if !sans.contains(&expected) {
                return Err(AuthError::InvalidMtlsSan);
            }
        } else if !sans.contains(&self.control_plane_san) {
            return Err(AuthError::InvalidMtlsSan);
        }

        Ok(())
    }
}

#[derive(Clone)]
struct OidcValidator {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl OidcValidator {
    fn new(config: OidcConfig) -> Result<Self, AuthConfigError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_issuer(&[config.issuer.clone()]);
        validation.set_audience(&[config.audience.clone()]);

        Ok(Self {
            decoding_key: DecodingKey::from_secret(config.hs256_secret.as_bytes()),
            validation,
        })
    }

    fn verify(&self, token: &str) -> Result<(), AuthError> {
        decode::<OidcClaims>(token, &self.decoding_key, &self.validation)
            .map(|_| ())
            .map_err(|err| AuthError::InvalidOidc(err.to_string()))
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OidcClaims {
    #[serde(alias = "iss", alias = "_iss")]
    iss: String,
    #[serde(alias = "aud", alias = "_aud")]
    aud: String,
    #[serde(alias = "exp", alias = "_exp")]
    exp: u64,
    #[serde(default)]
    #[serde(alias = "sub", alias = "_sub")]
    sub: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::NamedTempFile;

    fn test_config() -> AuthConfig {
        AuthConfig::development_defaults()
    }

    fn test_authenticator() -> UnifiedAuthenticator {
        UnifiedAuthenticator::from_config(test_config()).expect("valid config")
    }

    fn signed_token(issuer: &str, audience: &str, secret: &str) -> String {
        let claims = json!({
            "iss": issuer,
            "aud": audience,
            "exp": current_epoch() + 3600,
            "sub": "tester",
        });
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("token")
    }

    fn current_epoch() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn mtls(agent: &str) -> MtlsCertificate {
        MtlsCertificate {
            fingerprint: DEV_ALLOWED_FINGERPRINT.into(),
            subject_alt_names: vec![
                format!("{}{}", DEV_AGENT_SAN_PREFIX, agent),
                DEV_CONTROL_PLANE_SAN.into(),
            ],
        }
    }

    #[test]
    fn accepts_valid_credentials() {
        let auth = test_authenticator();
        let credentials = AuthCredentials {
            mtls: Some(mtls("agent-1")),
            oidc: Some(signed_token(
                DEV_OIDC_ISSUER,
                DEV_OIDC_AUDIENCE,
                DEV_OIDC_SECRET,
            )),
            api_key: Some("key-ops".into()),
        };
        auth.verify(&credentials, &Some("agent-1".into()))
            .expect("valid creds");
    }

    #[test]
    fn rejects_revoked_api_key() {
        let auth = test_authenticator();
        let credentials = AuthCredentials {
            mtls: Some(mtls("agent-2")),
            oidc: Some(signed_token(
                DEV_OIDC_ISSUER,
                DEV_OIDC_AUDIENCE,
                DEV_OIDC_SECRET,
            )),
            api_key: Some("rotated".into()),
        };
        let err = auth
            .verify(&credentials, &Some("agent-2".into()))
            .expect_err("revoked key fails");
        assert!(matches!(err, AuthError::InvalidApiKey));
    }

    #[test]
    fn enforces_agent_san() {
        let auth = test_authenticator();
        let credentials = AuthCredentials {
            mtls: Some(MtlsCertificate {
                fingerprint: DEV_ALLOWED_FINGERPRINT.into(),
                subject_alt_names: vec![DEV_CONTROL_PLANE_SAN.into()],
            }),
            oidc: None,
            api_key: Some("key-123".into()),
        };
        let err = auth
            .verify(&credentials, &Some("agent-99".into()))
            .expect_err("missing san");
        assert!(matches!(err, AuthError::InvalidMtlsSan));
    }

    #[test]
    fn validates_oidc_claims() {
        let auth = test_authenticator();
        let bad_token = signed_token("https://other-issuer", DEV_OIDC_AUDIENCE, DEV_OIDC_SECRET);
        let credentials = AuthCredentials {
            mtls: Some(mtls("agent-3")),
            oidc: Some(bad_token),
            api_key: Some("key-ops".into()),
        };
        let err = auth
            .verify(&credentials, &Some("agent-3".into()))
            .expect_err("bad issuer");
        assert!(matches!(err, AuthError::InvalidOidc(_)));
    }

    #[test]
    fn loads_config_from_file() {
        let mut file = NamedTempFile::new().expect("temp file");
        let config = serde_json::to_string(&test_config()).unwrap();
        use std::io::Write;
        file.write_all(config.as_bytes()).unwrap();
        std::env::set_var("GATEWAY_AUTH_CONFIG", file.path());
        let loaded = AuthConfig::load().expect("config loaded");
        assert_eq!(loaded.api_keys.len(), 3);
        std::env::remove_var("GATEWAY_AUTH_CONFIG");
    }
}
