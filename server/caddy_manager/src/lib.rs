//! Portable client for interacting with the embedded Caddy admin API.
//!
//! The manager wraps a reqwest client with sensible timeouts and exposes
//! helpers to push reverse proxy routes or reload the active configuration.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode, Url};
use serde_json::{json, Value};

/// Configuration for a reverse proxy route that proxies a domain to one or
/// more upstream services.
#[derive(Debug, Clone)]
pub struct ReverseProxyRoute {
    pub domain: String,
    pub upstreams: Vec<String>,
    pub health_probe: Option<HealthProbe>,
    pub rate_limit: Option<RateLimitConfig>,
    pub enable_compression: bool,
    pub inject_security_headers: bool,
}

impl ReverseProxyRoute {
    pub fn validate(&self) -> Result<()> {
        if self.domain.trim().is_empty() {
            return Err(anyhow!("domain cannot be empty"));
        }
        if self.upstreams.is_empty() {
            return Err(anyhow!("at least one upstream must be provided"));
        }
        Ok(())
    }

    fn as_caddy_json(&self) -> Value {
        let upstreams: Vec<Value> = self
            .upstreams
            .iter()
            .map(|target| json!({ "dial": target }))
            .collect();

        let mut reverse_proxy = json!({
            "handler": "reverse_proxy",
            "upstreams": upstreams,
            "load_balancing": {
                "selection_policy": {
                    "policy": "round_robin"
                },
                "try_duration": "5s",
                "try_interval": "250ms"
            },
            "transport": {
                "protocol": "http",
                "dial_timeout": "5s",
                "read_timeout": "30s",
                "write_timeout": "30s"
            },
            "headers": {
                "request": {
                    "set": {
                        "X-Real-IP": ["{remote_host}"],
                        "X-Forwarded-For": ["{remote_host}"],
                        "X-Forwarded-Proto": ["{scheme}"]
                    }
                }
            }
        });

        if let Some(probe) = &self.health_probe {
            reverse_proxy["health_checks"] = json!({
                "active": {
                    "path": probe.uri,
                    "interval": probe.interval,
                    "timeout": probe.timeout
                }
            });
        }

        let mut handles = Vec::new();

        if self.inject_security_headers {
            handles.push(json!({
                "handler": "headers",
                "response": {
                    "set": {
                        "Strict-Transport-Security": [
                            "max-age=31536000; includeSubDomains; preload"
                        ],
                        "X-Content-Type-Options": ["nosniff"],
                        "X-Frame-Options": ["DENY"],
                        "X-XSS-Protection": ["1; mode=block"],
                        "Content-Security-Policy": ["default-src 'self'"],
                        "Referrer-Policy": ["strict-origin-when-cross-origin"]
                    }
                }
            }));
        }

        let proxy_handle = if self.enable_compression {
            json!({
                "handler": "encode",
                "encodings": {
                    "gzip": {},
                    "zstd": {}
                },
                "handle": [reverse_proxy]
            })
        } else {
            reverse_proxy
        };
        handles.push(proxy_handle);

        let mut route = json!({
            "match": [{"host": [self.domain.clone()]}],
            "handle": handles,
            "terminal": true
        });

        if let Some(limit) = &self.rate_limit {
            route["rate_limits"] = json!([{
                "zone": &limit.zone,
                "key": &limit.key,
                "events": limit.events,
                "window": &limit.window
            }]);
        }

        route
    }
}

impl Default for ReverseProxyRoute {
    fn default() -> Self {
        Self {
            domain: "noa-ark-os.com".to_string(),
            upstreams: vec!["localhost:8080".to_string()],
            health_probe: Some(HealthProbe::default()),
            rate_limit: Some(RateLimitConfig::default()),
            enable_compression: true,
            inject_security_headers: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthProbe {
    pub uri: String,
    pub interval: String,
    pub timeout: String,
}

impl Default for HealthProbe {
    fn default() -> Self {
        Self {
            uri: "/health".to_string(),
            interval: "10s".to_string(),
            timeout: "5s".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub zone: String,
    pub key: String,
    pub events: u64,
    pub window: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            zone: "dynamic".to_string(),
            key: "{remote_host}".to_string(),
            events: 100,
            window: "1m".to_string(),
        }
    }
}

/// Client wrapper for the Caddy admin API.
pub struct CaddyManager {
    admin_endpoint: Url,
    client: Client,
}

impl CaddyManager {
    pub fn new(admin_endpoint: impl AsRef<str>) -> Result<Self> {
        let raw = admin_endpoint.as_ref();
        let admin_endpoint = Url::parse(raw).or_else(|_| Url::parse(&format!("http://{}", raw)))?;
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("failed to create reqwest client")?;
        Ok(Self {
            admin_endpoint,
            client,
        })
    }

    pub async fn push_route(&self, route: &ReverseProxyRoute) -> Result<()> {
        route.validate()?;
        let payload = route.as_caddy_json();
        let target = self
            .admin_endpoint
            .join("/config/apps/http/servers/srv0/routes/-")
            .context("invalid admin endpoint URL")?;
        let response = self
            .client
            .patch(target)
            .json(&payload)
            .send()
            .await
            .context("failed to send route configuration to Caddy")?;
        if response.status() != StatusCode::OK && response.status() != StatusCode::CREATED {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "caddy rejected route (status {}): {}",
                status,
                body
            ));
        }
        Ok(())
    }

    pub async fn reload(&self) -> Result<()> {
        let target = self
            .admin_endpoint
            .join("/load")
            .context("invalid admin endpoint URL")?;
        let response = self
            .client
            .post(target)
            .send()
            .await
            .context("failed to request Caddy reload")?;
        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "caddy reload failed with status {}: {}",
                status,
                body
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HealthProbe, RateLimitConfig, ReverseProxyRoute};

    #[test]
    fn route_payload_contains_expected_fields() {
        let mut route = ReverseProxyRoute::default();
        route.domain = "example.test".into();
        route.upstreams = vec!["127.0.0.1:9000".into(), "127.0.0.2:9000".into()];
        route.health_probe = Some(HealthProbe {
            uri: "/ready".into(),
            interval: "5s".into(),
            timeout: "2s".into(),
        });
        route.rate_limit = Some(RateLimitConfig {
            zone: "dynamic".into(),
            key: "{remote_host}".into(),
            events: 50,
            window: "30s".into(),
        });

        let payload = route.as_caddy_json();
        assert_eq!(
            payload
                .get("match")
                .and_then(|v| v.get(0))
                .and_then(|v| v.get("host"))
                .and_then(|v| v.get(0))
                .and_then(|v| v.as_str()),
            Some("example.test")
        );
        assert_eq!(
            payload
                .get("rate_limits")
                .and_then(|v| v.get(0))
                .and_then(|v| v.get("events"))
                .and_then(|v| v.as_u64()),
            Some(50)
        );
    }
}
