use std::sync::{Arc, Mutex};

use futures::StreamExt;
use noa_inference::{
    CompletionChunk, CompletionRequest, Provider, ProviderRouter, TelemetryEvent, TelemetryHandle,
    TelemetrySink, TelemetryStatus,
};
use tokio::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct RecordingTelemetry {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl RecordingTelemetry {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    fn handle(self: &Arc<Self>) -> TelemetryHandle {
        Arc::clone(self) as TelemetryHandle
    }
}

impl TelemetrySink for RecordingTelemetry {
    fn record(&self, event: TelemetryEvent) {
        self.events.lock().unwrap().push(event);
    }
}

#[tokio::test]
async fn openai_streaming_parses_chunks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let body = [
        "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
        "data: {\"choices\":[{\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":2,\"completion_tokens\":1}}\n\n",
        "data: [DONE]\n\n",
    ]
    .join("");

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(body, "text/event-stream")
                .set_delay(Duration::from_millis(10)),
        )
        .mount(&server)
        .await;

    let provider = noa_inference::providers::openai::OpenAiProvider::new(
        noa_inference::providers::openai::OpenAiConfig {
            base_url: server.uri(),
            model: "gpt-test".into(),
            api_key: "key".into(),
            organization: None,
            request_timeout_secs: 5,
        },
    );

    let stream = provider
        .stream(CompletionRequest {
            prompt: "Hi".into(),
            temperature: None,
            max_tokens: Some(16),
            stop: None,
        })
        .await
        .expect("stream should start");

    let chunks: Vec<CompletionChunk> = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .expect("chunks to parse");

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].content, "Hello");
    assert!(chunks[1].is_final);
    assert_eq!(chunks[1].tokens_evaluated, Some(2));
    assert_eq!(chunks[1].tokens_predicted, Some(1));
}

#[tokio::test]
async fn router_falls_back_and_records_telemetry() {
    let primary = MockServer::start().await;
    let fallback = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&primary)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&primary)
        .await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&fallback)
        .await;

    Mock::given(method("POST"))
        .and(path("/completion"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "Fallback response",
            "model": "llama-local",
            "tokens_evaluated": 3,
            "tokens_predicted": 2,
        })))
        .mount(&fallback)
        .await;

    let openai = noa_inference::providers::openai::OpenAiProvider::new(
        noa_inference::providers::openai::OpenAiConfig {
            base_url: primary.uri(),
            model: "gpt-test".into(),
            api_key: "key".into(),
            organization: None,
            request_timeout_secs: 2,
        },
    );

    let llama = noa_inference::providers::llama_cpp::LlamaCppProvider::new(
        noa_inference::providers::llama_cpp::LlamaCppConfig {
            endpoint: fallback.uri(),
            model: "llama-local".into(),
        },
    );

    let telemetry = Arc::new(RecordingTelemetry::new());
    let router = ProviderRouter::with_providers(vec![Arc::new(openai), Arc::new(llama)])
        .with_telemetry(telemetry.handle());

    let response = router
        .completion(CompletionRequest {
            prompt: "test".into(),
            temperature: None,
            max_tokens: Some(4),
            stop: None,
        })
        .await
        .expect("fallback to succeed");

    assert_eq!(response.content, "Fallback response");
    let events = telemetry.events.lock().unwrap().clone();
    assert_eq!(events.len(), 2);
    assert!(events
        .iter()
        .any(|event| event.status == TelemetryStatus::Failure));
    assert!(events
        .iter()
        .any(|event| event.status == TelemetryStatus::Success));
}

#[tokio::test]
async fn telemetry_records_stream_completion() {
    let fallback = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&fallback)
        .await;

    let stream_body = [
        serde_json::json!({
            "content": "Stream", "done": false
        })
        .to_string(),
        serde_json::json!({
            "content": "", "done": true, "tokens_evaluated": 1, "tokens_predicted": 1
        })
        .to_string(),
    ]
    .join("\n");

    Mock::given(method("POST"))
        .and(path("/completion"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(format!("{}\n", stream_body), "application/json"),
        )
        .mount(&fallback)
        .await;

    let llama = noa_inference::providers::llama_cpp::LlamaCppProvider::new(
        noa_inference::providers::llama_cpp::LlamaCppConfig {
            endpoint: fallback.uri(),
            model: "llama-test".into(),
        },
    );

    let telemetry = Arc::new(RecordingTelemetry::new());
    let router =
        ProviderRouter::with_providers(vec![Arc::new(llama)]).with_telemetry(telemetry.handle());

    let stream = router
        .stream_completion(CompletionRequest {
            prompt: "stream".into(),
            temperature: None,
            max_tokens: Some(4),
            stop: None,
        })
        .await
        .expect("stream should start");

    let collected = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("collect stream");

    assert_eq!(collected.len(), 2);
    let events = telemetry.events.lock().unwrap().clone();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.status, TelemetryStatus::Success);
    assert_eq!(event.tokens_prompt, 1);
    assert_eq!(event.tokens_completion, 1);
}
