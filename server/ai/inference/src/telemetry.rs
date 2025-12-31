use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub provider: String,
    pub model: String,
    pub latency_ms: u128,
    pub tokens_prompt: usize,
    pub tokens_completion: usize,
    pub status: TelemetryStatus,
    pub error: Option<String>,
}

pub trait TelemetrySink: Send + Sync {
    fn record(&self, event: TelemetryEvent);
}

#[derive(Clone, Default)]
pub struct NoopTelemetrySink;

impl TelemetrySink for NoopTelemetrySink {
    fn record(&self, _event: TelemetryEvent) {}
}

pub type TelemetryHandle = Arc<dyn TelemetrySink>;
