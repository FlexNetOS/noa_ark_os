use std::env;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use anyhow::{anyhow, Context as AnyhowContext};
use futures::Stream;
use tracing::{info, warn};

use crate::client::{CompletionRequest, CompletionResponse};
use crate::providers::anthropic::{AnthropicConfig, AnthropicProvider};
use crate::providers::llama_cpp::{LlamaCppConfig, LlamaCppProvider};
use crate::providers::openai::{OpenAiConfig, OpenAiProvider};
use crate::providers::{Provider, ProviderMetadata};
use crate::stream::{CompletionChunk, CompletionStream};
use crate::telemetry::{TelemetryEvent, TelemetryHandle, TelemetryStatus};

struct ProviderEntry {
    metadata: ProviderMetadata,
    provider: Arc<dyn Provider>,
}

impl ProviderEntry {
    fn new(provider: Arc<dyn Provider>) -> Self {
        let metadata = provider.metadata();
        Self { metadata, provider }
    }
}

#[derive(Clone)]
pub struct ProviderRouter {
    providers: Arc<Vec<ProviderEntry>>,
    telemetry: Option<TelemetryHandle>,
}

impl ProviderRouter {
    pub fn from_env() -> anyhow::Result<Self> {
        let provider_ids = provider_sequence_from_env();
        let mut providers = Vec::new();
        for provider_id in provider_ids {
            let provider = build_provider_from_env(&provider_id)?;
            providers.push(ProviderEntry::new(provider));
        }

        if providers.is_empty() {
            return Err(anyhow!("no providers configured"));
        }

        Ok(Self {
            providers: Arc::new(providers),
            telemetry: None,
        })
    }

    pub fn with_providers(providers: Vec<Arc<dyn Provider>>) -> Self {
        let entries = providers.into_iter().map(ProviderEntry::new).collect();
        Self {
            providers: Arc::new(entries),
            telemetry: None,
        }
    }

    pub fn with_telemetry(mut self, telemetry: TelemetryHandle) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    pub async fn completion(
        &self,
        request: CompletionRequest,
    ) -> anyhow::Result<CompletionResponse> {
        let mut errors = Vec::new();
        for entry in self.providers.iter() {
            if let Err(err) = entry.provider.health_check().await {
                warn!(
                    provider = entry.metadata.id,
                    model = entry.metadata.model,
                    error = %err,
                    "provider health check failed"
                );
                self.record_failure(&entry.metadata, 0, 0, 0, Some(err.to_string()));
                errors.push(err);
                continue;
            }

            let started = Instant::now();
            match entry.provider.complete(request.clone()).await {
                Ok(mut response) => {
                    if response.model.is_empty() {
                        response.model = entry.metadata.model.clone();
                    }
                    let latency = started.elapsed().as_millis();
                    self.record_success(
                        &entry.metadata,
                        latency,
                        response.tokens_evaluated,
                        response.tokens_predicted,
                    );
                    info!(
                        provider = entry.metadata.id,
                        model = response.model,
                        latency_ms = latency,
                        "completion routed"
                    );
                    return Ok(response);
                }
                Err(err) => {
                    let latency = started.elapsed().as_millis();
                    warn!(
                        provider = entry.metadata.id,
                        model = entry.metadata.model,
                        error = %err,
                        latency_ms = latency,
                        "provider completion failed"
                    );
                    self.record_failure(&entry.metadata, latency, 0, 0, Some(err.to_string()));
                    errors.push(err);
                }
            }
        }

        let message = errors
            .iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        Err(anyhow!("all providers failed: {message}"))
    }

    pub async fn stream_completion(
        &self,
        request: CompletionRequest,
    ) -> anyhow::Result<CompletionStream> {
        let mut errors = Vec::new();
        for entry in self.providers.iter() {
            if let Err(err) = entry.provider.health_check().await {
                warn!(
                    provider = entry.metadata.id,
                    model = entry.metadata.model,
                    error = %err,
                    "provider health check failed"
                );
                self.record_failure(&entry.metadata, 0, 0, 0, Some(err.to_string()));
                errors.push(err);
                continue;
            }

            let started = Instant::now();
            match entry.provider.stream(request.clone()).await {
                Ok(stream) => {
                    info!(
                        provider = entry.metadata.id,
                        model = entry.metadata.model,
                        "streaming completion routed"
                    );
                    return Ok(self.wrap_stream(stream, entry.metadata.clone(), started));
                }
                Err(err) => {
                    let latency = started.elapsed().as_millis();
                    warn!(
                        provider = entry.metadata.id,
                        model = entry.metadata.model,
                        error = %err,
                        latency_ms = latency,
                        "provider streaming failed"
                    );
                    self.record_failure(&entry.metadata, latency, 0, 0, Some(err.to_string()));
                    errors.push(err);
                }
            }
        }

        let message = errors
            .iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        Err(anyhow!("all providers failed: {message}"))
    }

    fn wrap_stream(
        &self,
        stream: CompletionStream,
        metadata: ProviderMetadata,
        started: Instant,
    ) -> CompletionStream {
        if let Some(telemetry) = &self.telemetry {
            Box::pin(InstrumentedStream::new(
                stream,
                Arc::clone(telemetry),
                metadata,
                started,
            ))
        } else {
            stream
        }
    }

    fn record_success(
        &self,
        metadata: &ProviderMetadata,
        latency_ms: u128,
        tokens_prompt: usize,
        tokens_completion: usize,
    ) {
        if let Some(telemetry) = &self.telemetry {
            telemetry.record(TelemetryEvent {
                provider: metadata.id.to_string(),
                model: metadata.model.clone(),
                latency_ms,
                tokens_prompt,
                tokens_completion,
                status: TelemetryStatus::Success,
                error: None,
            });
        }
    }

    fn record_failure(
        &self,
        metadata: &ProviderMetadata,
        latency_ms: u128,
        tokens_prompt: usize,
        tokens_completion: usize,
        error: Option<String>,
    ) {
        if let Some(telemetry) = &self.telemetry {
            telemetry.record(TelemetryEvent {
                provider: metadata.id.to_string(),
                model: metadata.model.clone(),
                latency_ms,
                tokens_prompt,
                tokens_completion,
                status: TelemetryStatus::Failure,
                error,
            });
        }
    }
}

struct InstrumentedStream {
    inner: CompletionStream,
    telemetry: TelemetryHandle,
    metadata: ProviderMetadata,
    started: Instant,
    tokens_prompt: usize,
    tokens_completion: usize,
    finished: bool,
}

impl InstrumentedStream {
    fn new(
        inner: CompletionStream,
        telemetry: TelemetryHandle,
        metadata: ProviderMetadata,
        started: Instant,
    ) -> Self {
        Self {
            inner,
            telemetry,
            metadata,
            started,
            tokens_prompt: 0,
            tokens_completion: 0,
            finished: false,
        }
    }

    fn finish(&mut self, status: TelemetryStatus, error: Option<String>) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.telemetry.record(TelemetryEvent {
            provider: self.metadata.id.to_string(),
            model: self.metadata.model.clone(),
            latency_ms: self.started.elapsed().as_millis(),
            tokens_prompt: self.tokens_prompt,
            tokens_completion: self.tokens_completion,
            status,
            error,
        });
    }
}

impl Stream for InstrumentedStream {
    type Item = anyhow::Result<CompletionChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = self.inner.as_mut().poll_next(cx);
        match result {
            Poll::Ready(Some(Ok(chunk))) => {
                if let Some(tokens) = chunk.tokens_evaluated {
                    self.tokens_prompt = tokens;
                }
                if let Some(tokens) = chunk.tokens_predicted {
                    self.tokens_completion = tokens;
                }
                if chunk.is_final {
                    self.finish(TelemetryStatus::Success, None);
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(err))) => {
                let message = err.to_string();
                self.finish(TelemetryStatus::Failure, Some(message.clone()));
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(None) => {
                self.finish(TelemetryStatus::Success, None);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for InstrumentedStream {
    fn drop(&mut self) {
        if !self.finished {
            self.finish(
                TelemetryStatus::Failure,
                Some("stream dropped before completion".to_string()),
            );
        }
    }
}

fn provider_sequence_from_env() -> Vec<String> {
    if let Ok(chain) = env::var("AI_PROVIDER_CHAIN") {
        chain
            .split(',')
            .filter_map(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_lowercase())
                }
            })
            .collect()
    } else if let Ok(single) = env::var("AI_PROVIDER") {
        vec![single.trim().to_lowercase()]
    } else {
        vec!["llama.cpp".to_string()]
    }
}

fn build_provider_from_env(id: &str) -> anyhow::Result<Arc<dyn Provider>> {
    match id {
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY environment variable is required")?;
            let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".into());
            let base_url = env::var("OPENAI_BASE_URL").unwrap_or_default();
            let organization = env::var("OPENAI_ORG").ok();
            let timeout = env::var("OPENAI_TIMEOUT_SECS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(30);
            let config = OpenAiConfig {
                base_url,
                model,
                api_key,
                organization,
                request_timeout_secs: timeout,
            };
            Ok(Arc::new(OpenAiProvider::new(config)))
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY environment variable is required")?;
            let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-sonnet".into());
            let base_url = env::var("ANTHROPIC_BASE_URL").unwrap_or_default();
            let version = env::var("ANTHROPIC_VERSION").unwrap_or_default();
            let timeout = env::var("ANTHROPIC_TIMEOUT_SECS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(30);
            let config = AnthropicConfig {
                model,
                api_key,
                base_url,
                version,
                request_timeout_secs: timeout,
            };
            Ok(Arc::new(AnthropicProvider::new(config)))
        }
        "llama" | "llama.cpp" => {
            let endpoint = env::var("LLAMA_CPP_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:8080/v1".into());
            let model = env::var("LLAMA_CPP_MODEL").unwrap_or_else(|_| "llama.cpp".into());
            let config = LlamaCppConfig { endpoint, model };
            Ok(Arc::new(LlamaCppProvider::new(config)))
        }
        other => Err(anyhow!("unsupported provider: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct TestProvider {
        metadata: ProviderMetadata,
        healthy: bool,
        fail: bool,
    }

    impl TestProvider {
        fn success(id: &'static str, model: &'static str, _content: &'static str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id,
                    model: model.to_string(),
                },
                healthy: true,
                fail: false,
            }
        }

        fn failing(id: &'static str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id,
                    model: "test".to_string(),
                },
                healthy: true,
                fail: true,
            }
        }

        fn unhealthy(id: &'static str) -> Self {
            Self {
                metadata: ProviderMetadata {
                    id,
                    model: "test".to_string(),
                },
                healthy: false,
                fail: false,
            }
        }
    }

    #[async_trait]
    impl Provider for TestProvider {
        fn metadata(&self) -> ProviderMetadata {
            self.metadata.clone()
        }

        async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
            if self.fail {
                anyhow::bail!("forced failure")
            }

            Ok(CompletionResponse {
                content: format!("{}:{}", self.metadata.id, request.prompt),
                model: self.metadata.model.clone(),
                tokens_evaluated: request.prompt.len(),
                tokens_predicted: 4,
                latency_ms: 0,
            })
        }

        async fn stream(&self, _request: CompletionRequest) -> anyhow::Result<CompletionStream> {
            let metadata = self.metadata.clone();
            Ok(Box::pin(async_stream::try_stream! {
                yield CompletionChunk {
                    content: metadata.model.clone(),
                    is_final: true,
                    tokens_evaluated: Some(1),
                    tokens_predicted: Some(1),
                };
            }))
        }

        async fn health_check(&self) -> anyhow::Result<()> {
            if self.healthy {
                Ok(())
            } else {
                anyhow::bail!("unhealthy")
            }
        }
    }

    #[tokio::test]
    async fn routes_to_first_successful_provider() {
        let router = ProviderRouter::with_providers(vec![Arc::new(TestProvider::success(
            "primary", "model-a", "ok",
        ))]);

        let response = router
            .completion(CompletionRequest {
                prompt: "hello".into(),
                temperature: None,
                max_tokens: Some(8),
                stop: None,
            })
            .await
            .unwrap();
        assert_eq!(response.model, "model-a");
        assert!(response.content.contains("primary"));
    }

    #[tokio::test]
    async fn falls_back_when_primary_fails() {
        let router = ProviderRouter::with_providers(vec![
            Arc::new(TestProvider::failing("primary")),
            Arc::new(TestProvider::success("secondary", "model-b", "fallback")),
        ]);

        let response = router
            .completion(CompletionRequest {
                prompt: "hello".into(),
                temperature: None,
                max_tokens: Some(8),
                stop: None,
            })
            .await
            .unwrap();
        assert!(response.content.contains("secondary"));
    }

    #[tokio::test]
    async fn skips_unhealthy_providers() {
        let router = ProviderRouter::with_providers(vec![
            Arc::new(TestProvider::unhealthy("bad")),
            Arc::new(TestProvider::success("good", "model-c", "pass")),
        ]);

        let response = router
            .completion(CompletionRequest {
                prompt: "ping".into(),
                temperature: None,
                max_tokens: Some(8),
                stop: None,
            })
            .await
            .unwrap();
        assert!(response.content.contains("good"));
    }
}
