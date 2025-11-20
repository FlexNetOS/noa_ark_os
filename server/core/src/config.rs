use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use config::{Config, ConfigError as BuilderError, Environment, File, FileFormat};
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_CONFIG_DIR: &str = "server/config";

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSection,
    pub database: DatabaseSection,
    pub cache: CacheSection,
    pub qdrant: QdrantSection,
    pub inference: InferenceSection,
    pub observability: ObservabilitySection,
}

impl ServerConfig {
    pub fn metrics_addr(&self) -> std::result::Result<SocketAddr, std::net::AddrParseError> {
        self.observability.metrics_socket_addr()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    #[serde(default)]
    pub tls: Option<ServerTlsConfig>,
}

impl ServerSection {
    pub fn bind_address(&self) -> std::result::Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerTlsConfig {
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub client_ca_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheSection {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QdrantSection {
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InferenceSection {
    pub device: String,
    pub model_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilitySection {
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_metrics_bind")]
    pub metrics_bind: String,
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
}

fn default_log_format() -> String {
    "pretty".into()
}

fn default_metrics_bind() -> String {
    "127.0.0.1".into()
}

fn default_metrics_port() -> u16 {
    9100
}

impl ObservabilitySection {
    pub fn metrics_socket_addr(&self) -> std::result::Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.metrics_bind, self.metrics_port).parse()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub config_path: Option<PathBuf>,
    pub profile: Option<String>,
    pub server_host: Option<String>,
    pub server_port: Option<u16>,
    pub server_workers: Option<usize>,
    pub metrics_bind: Option<String>,
    pub metrics_port: Option<u16>,
    pub log_level: Option<String>,
    pub log_format: Option<String>,
    pub otlp_endpoint: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Builder(#[from] BuilderError),
}

impl From<std::net::AddrParseError> for ConfigError {
    fn from(err: std::net::AddrParseError) -> Self {
        ConfigError::Message(err.to_string())
    }
}

pub fn load(overrides: ConfigOverrides) -> std::result::Result<ServerConfig, ConfigError> {
    let mut builder = Config::builder();
    builder = builder.add_source(required_file(DEFAULT_CONFIG_DIR, "default.toml"));

    if let Some(profile) = overrides.profile.as_deref() {
        let profile_file = format!("{profile}.toml");
        builder = builder.add_source(optional_file(DEFAULT_CONFIG_DIR, &profile_file));
    }

    if let Some(path) = overrides.config_path {
        builder = builder.add_source(File::from(path));
    }

    builder = builder.add_source(Environment::with_prefix("NOA").separator("__"));

    if let Some(host) = overrides.server_host {
        builder = builder.set_override("server.host", host)?;
    }
    if let Some(port) = overrides.server_port {
        builder = builder.set_override("server.port", port as i64)?;
    }
    if let Some(workers) = overrides.server_workers {
        builder = builder.set_override("server.workers", workers as i64)?;
    }
    if let Some(metrics_bind) = overrides.metrics_bind {
        builder = builder.set_override("observability.metrics_bind", metrics_bind)?;
    }
    if let Some(metrics_port) = overrides.metrics_port {
        builder = builder.set_override("observability.metrics_port", metrics_port as i64)?;
    }
    if let Some(log_level) = overrides.log_level {
        builder = builder.set_override("observability.log_level", log_level)?;
    }
    if let Some(log_format) = overrides.log_format {
        builder = builder.set_override("observability.log_format", log_format)?;
    }
    if let Some(otlp_endpoint) = overrides.otlp_endpoint {
        builder = builder.set_override("observability.otlp_endpoint", otlp_endpoint)?;
    }

    let config = builder.build()?;
    Ok(config.try_deserialize()?)
}

fn required_file(dir: &str, name: &str) -> File<config::FileSourceFile, FileFormat> {
    File::from(Path::new(dir).join(name)).format(FileFormat::Toml)
}

fn optional_file(dir: &str, name: &str) -> File<config::FileSourceFile, FileFormat> {
    File::from(Path::new(dir).join(name))
        .required(false)
        .format(FileFormat::Toml)
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< Updated upstream

    #[test]
    fn loads_default_configuration() {
=======
    use std::path::Path;

    fn chdir_workspace_root() {
        // server/core/Cargo.toml -> workspace root is two levels up
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        if let (Some(parent), Some(root)) = (manifest_dir.parent(), manifest_dir.parent().and_then(|p| p.parent())) {
            // Prefer the actual workspace root if detected; otherwise best-effort
            let target = root;
            std::env::set_current_dir(target).expect("set CWD to workspace root");
        }
    }

    #[test]
    fn loads_default_configuration() {
        chdir_workspace_root();
>>>>>>> Stashed changes
        let config = load(ConfigOverrides::default()).expect("config loads");
        assert_eq!(config.server.port, 8080);
        assert!(config.observability.log_level.len() > 0);
    }

    #[test]
    fn applies_cli_overrides() {
<<<<<<< Updated upstream
=======
        chdir_workspace_root();
>>>>>>> Stashed changes
        let mut overrides = ConfigOverrides::default();
        overrides.server_port = Some(9090);
        overrides.log_level = Some("debug".into());
        let config = load(overrides).expect("config loads with overrides");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.observability.log_level, "debug");
    }
}
