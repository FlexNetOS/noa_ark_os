use serde_json::Value;
use std::sync::OnceLock;
use tracing::{event, Level};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

static INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn init() {
    INITIALIZED.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .json()
            .init();
    });
}

pub fn new_trace_id() -> String {
    Uuid::new_v4().to_string()
}

fn emit(
    level: Level,
    component: &str,
    event_name: &str,
    message: &str,
    outcome: &str,
    trace_id: Option<&str>,
    details: Option<Value>,
) -> String {
    let trace = trace_id
        .map(|value| value.to_owned())
        .unwrap_or_else(new_trace_id);
    let details = details.unwrap_or(Value::Null);

    match level {
        Level::ERROR => event!(
            target: "noa_crc",
            Level::ERROR,
            trace_id = trace.as_str(),
            component = component,
            event = event_name,
            outcome = outcome,
            message = message,
            details = %details
        ),
        Level::WARN => event!(
            target: "noa_crc",
            Level::WARN,
            trace_id = trace.as_str(),
            component = component,
            event = event_name,
            outcome = outcome,
            message = message,
            details = %details
        ),
        Level::INFO => event!(
            target: "noa_crc",
            Level::INFO,
            trace_id = trace.as_str(),
            component = component,
            event = event_name,
            outcome = outcome,
            message = message,
            details = %details
        ),
        Level::DEBUG => event!(
            target: "noa_crc",
            Level::DEBUG,
            trace_id = trace.as_str(),
            component = component,
            event = event_name,
            outcome = outcome,
            message = message,
            details = %details
        ),
        Level::TRACE => event!(
            target: "noa_crc",
            Level::TRACE,
            trace_id = trace.as_str(),
            component = component,
            event = event_name,
            outcome = outcome,
            message = message,
            details = %details
        ),
    }

    trace
}

pub fn info(
    component: &str,
    event_name: &str,
    message: &str,
    outcome: &str,
    trace_id: Option<&str>,
    details: Option<Value>,
) -> String {
    emit(
        Level::INFO,
        component,
        event_name,
        message,
        outcome,
        trace_id,
        details,
    )
}

pub fn warn(
    component: &str,
    event_name: &str,
    message: &str,
    outcome: &str,
    trace_id: Option<&str>,
    details: Option<Value>,
) -> String {
    emit(
        Level::WARN,
        component,
        event_name,
        message,
        outcome,
        trace_id,
        details,
    )
}

pub fn error(
    component: &str,
    event_name: &str,
    message: &str,
    outcome: &str,
    trace_id: Option<&str>,
    details: Option<Value>,
) -> String {
    emit(
        Level::ERROR,
        component,
        event_name,
        message,
        outcome,
        trace_id,
        details,
    )
}
