use crate::rate_limit::RateMetricsSnapshot;
use crate::router::{Protocol, RoutePlan};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub request_id: String,
    pub protocol: Protocol,
    pub route_targets: Vec<String>,
    pub agent_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub otel_span: HashMap<String, String>,
}

impl TelemetryEvent {
    pub fn new(
        request_id: String,
        protocol: Protocol,
        route_plan: RoutePlan,
        agent_id: Option<String>,
    ) -> Self {
        let mut otel_span = HashMap::new();
        otel_span.insert(
            "span.name".into(),
            format!("gateway.{}", span_name(&protocol)),
        );
        otel_span.insert("span.kind".into(), "server".into());
        otel_span.insert("net.protocol".into(), format!("{:?}", protocol));

        Self {
            request_id,
            route_targets: route_plan.targets,
            protocol,
            agent_id,
            recorded_at: Utc::now(),
            otel_span,
        }
    }
}

fn span_name(protocol: &Protocol) -> &'static str {
    match protocol {
        Protocol::GraphQl => "graphql",
        Protocol::Grpc => "grpc",
        Protocol::WebSocket => "websocket",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayMetrics {
    pub total_requests: u64,
    pub per_protocol: HashMap<String, u64>,
    pub last_event: Option<TelemetryEvent>,
    pub rate_limit: RateMetricsSnapshot,
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
}

impl From<std::io::Error> for TelemetryError {
    fn from(value: std::io::Error) -> Self {
        TelemetryError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for TelemetryError {
    fn from(value: serde_json::Error) -> Self {
        TelemetryError::Serde(value.to_string())
    }
}

/// Sink that exports gateway telemetry artefacts into storage/telemetry.
#[derive(Debug)]
pub struct TelemetrySink {
    metrics: Mutex<GatewayMetrics>,
    metrics_path: PathBuf,
    events_path: PathBuf,
}

impl TelemetrySink {
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Result<Self, TelemetryError> {
        create_dir_all(&storage_dir)?;
        let metrics_path = storage_dir.as_ref().join("gateway_metrics.json");
        let events_path = storage_dir.as_ref().join("gateway_events.log");
        Ok(Self {
            metrics: Mutex::new(GatewayMetrics::default()),
            metrics_path,
            events_path,
        })
    }

    pub fn record(&self, event: TelemetryEvent) -> Result<(), TelemetryError> {
        {
            let mut metrics = self.metrics.lock();
            metrics.total_requests += 1;
            *metrics
                .per_protocol
                .entry(format!("{:?}", event.protocol))
                .or_insert(0) += 1;
            metrics.last_event = Some(event.clone());

            self.persist_metrics(&metrics)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        file.write_all(serde_json::to_string(&event)?.as_bytes())?;
        file.write_all(b"\n")?;

        Ok(())
    }

    pub fn record_rate_limits(
        &self,
        snapshot: RateMetricsSnapshot,
    ) -> Result<(), TelemetryError> {
        let mut metrics = self.metrics.lock();
        metrics.rate_limit = snapshot;
        self.persist_metrics(&metrics)
    }

    pub fn snapshot(&self) -> GatewayMetrics {
        self.metrics.lock().clone()
    }

    fn persist_metrics(&self, metrics: &GatewayMetrics) -> Result<(), TelemetryError> {
        let json = serde_json::to_vec_pretty(metrics)?;
        std::fs::write(&self.metrics_path, json)?;
        Ok(())
    }
}

impl Default for TelemetrySink {
    fn default() -> Self {
        Self::new(default_storage_dir()).unwrap_or_else(|e| {
            panic!(
                "Failed to create telemetry directory at {:?}: {}. \
                 Please check filesystem permissions, disk space, and path validity.",
                default_storage_dir(),
                e
            )
        })
    }
}

fn default_storage_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // `server/gateway` -> pop `gateway`, pop `server`
    path.pop();
    path.pop();
    path.push("storage");
    path.push("telemetry");
    path
}
