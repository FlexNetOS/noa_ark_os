use anyhow::{anyhow, Context, Result};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{self, Resource};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

type Registry = tracing_subscriber::Registry;
type OtlpLayer = OpenTelemetryLayer<Registry, opentelemetry_sdk::trace::Tracer>;

/// Supported log formats for gateway binaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl LogFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogFormat::Pretty => "pretty",
            LogFormat::Json => "json",
        }
    }
}

impl std::str::FromStr for LogFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            "pretty" | "text" => Ok(LogFormat::Pretty),
            other => Err(anyhow!("unsupported log format: {other}")),
        }
    }
}

/// Configuration driving tracing initialisation.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub service_name: String,
    pub log_format: LogFormat,
    pub log_level: String,
    pub otlp_endpoint: Option<String>,
    pub resource_attributes: Vec<(String, String)>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "noa-server".into(),
            log_format: LogFormat::Pretty,
            log_level: "info".into(),
            otlp_endpoint: None,
            resource_attributes: Vec::new(),
        }
    }
}

/// Guard responsible for shutting down tracing exporters on drop.
pub struct TracingGuard {
    otlp_enabled: bool,
}

impl TracingGuard {
    fn new(otlp_enabled: bool) -> Self {
        Self { otlp_enabled }
    }
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        if self.otlp_enabled {
            global::shutdown_tracer_provider();
        }
    }
}

/// Install the tracing subscriber with the configured layers.
pub fn init_tracing(config: &TracingConfig) -> Result<TracingGuard> {
    let env_filter =
        EnvFilter::try_new(config.log_level.clone()).unwrap_or_else(|_| EnvFilter::new("info"));

    let (otel_layer, otlp_enabled) = build_otlp_layer(config)?;
    match config.log_format {
        LogFormat::Pretty => Registry::default()
            .with(otel_layer)
            .with(env_filter)
            .with(fmt::layer().with_target(true))
            .try_init(),
        LogFormat::Json => Registry::default()
            .with(otel_layer)
            .with(env_filter)
            .with(fmt::layer().json().with_target(true))
            .try_init(),
    }
        .map_err(|err| anyhow::anyhow!("failed to install tracing subscriber: {err}"))?;
    Ok(TracingGuard::new(otlp_enabled))
}

fn build_otlp_layer(config: &TracingConfig) -> Result<(Option<OtlpLayer>, bool)> {
    if let Some(endpoint) = &config.otlp_endpoint {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .build()
            .context("failed to build OTLP exporter")?;
        let mut attributes = vec![KeyValue::new("service.name", config.service_name.clone())];
        for (key, value) in &config.resource_attributes {
            attributes.push(KeyValue::new(key.clone(), value.clone()));
        }
        let resource = Resource::new(attributes);
        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .build();
        let tracer = provider.tracer(config.service_name.clone());
        global::set_tracer_provider(provider);
        let layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Ok((Some(layer), true))
    } else {
        Ok((None, false))
    }
}

/// Prometheus exporter wrapper returning rendered metrics.
#[derive(Clone)]
pub struct MetricsExporter {
    handle: PrometheusHandle,
}

impl MetricsExporter {
    pub fn install_with_defaults() -> Result<Self> {
        Self::install(PrometheusBuilder::new())
    }

    pub fn install(builder: PrometheusBuilder) -> Result<Self> {
        let handle = builder
            .install_recorder()
            .context("failed to install Prometheus recorder")?;
        Ok(Self { handle })
    }

    pub fn handle(&self) -> &PrometheusHandle {
        &self.handle
    }

    pub fn render(&self) -> String {
        self.handle.render()
    }
}

/// Convenience helper initialising tracing + metrics with a single call.
pub fn init(
    tracing: &TracingConfig,
    metrics_builder: Option<PrometheusBuilder>,
) -> Result<(TracingGuard, MetricsExporter)> {
    let guard = init_tracing(tracing)?;
    let exporter = if let Some(builder) = metrics_builder {
        MetricsExporter::install(builder)?
    } else {
        MetricsExporter::install_with_defaults()?
    };
    Ok((guard, exporter))
}

#[allow(dead_code)]
fn install_fmt_layer(
    env_filter: EnvFilter,
    otel_layer: Option<OtlpLayer>,
    log_format: LogFormat,
) -> Result<()> {
    match log_format {
        LogFormat::Pretty => Registry::default()
            .with(otel_layer)
            .with(env_filter.clone())
            .with(fmt::layer().with_target(true))
            .try_init(),
        LogFormat::Json => Registry::default()
            .with(otel_layer)
            .with(env_filter)
            .with(fmt::layer().json().with_target(true))
            .try_init(),
    }
    .map_err(|err| anyhow::anyhow!("failed to install tracing subscriber: {err}"))
}
