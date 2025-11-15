use std::time::Duration;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::Span;

use super::{Provider, ProviderMetadata};
use crate::client::{CompletionRequest, CompletionResponse};
use crate::stream::{parse_sse_stream, CompletionChunk, CompletionStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub model: String,
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        let mut config = config;
        if config.base_url.trim().is_empty() {
            config.base_url = default_base_url();
        }
        if config.version.trim().is_empty() {
            config.version = default_version();
        }
        let client = Client::builder()
            .user_agent("noa-ark-os-inference/anthropic")
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to construct Anthropic client");
        Self { config, client }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "anthropic",
            model: self.config.model.clone(),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        let span = tracing::info_span!(
            "anthropic.complete",
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        self.execute_completion(request, span).await
    }

    async fn stream(&self, request: CompletionRequest) -> anyhow::Result<CompletionStream> {
        let span = tracing::info_span!(
            "anthropic.stream",
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        self.execute_stream(request, span).await
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/v1/models", self.config.base_url);
        let response = self
            .client
            .get(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.version)
            .send()
            .await
            .context("failed to perform Anthropic health check")?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "anthropic health check failed with status {}",
                response.status()
            ))
        }
    }
}

impl AnthropicProvider {
    async fn execute_completion(
        &self,
        request: CompletionRequest,
        span: Span,
    ) -> anyhow::Result<CompletionResponse> {
        let _guard = span.enter();
        let started = std::time::Instant::now();
        let body = AnthropicRequest::from_completion(&self.config.model, request.clone(), false);
        let url = format!("{}/v1/messages", self.config.base_url);

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.version)
            .json(&body)
            .send()
            .await
            .context("failed to call Anthropic messages endpoint")?
            .error_for_status()
            .map_err(|err| anyhow!("anthropic completion failed: {}", err))?;

        let payload: AnthropicResponse = response
            .json()
            .await
            .context("failed to parse Anthropic completion response")?;

        let content = payload
            .content
            .iter()
            .map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("");
        let tokens_evaluated = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.input_tokens)
            .unwrap_or_default() as usize;
        let tokens_predicted = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .unwrap_or_default() as usize;

        Ok(CompletionResponse {
            content,
            model: payload.model.unwrap_or_else(|| self.config.model.clone()),
            tokens_evaluated,
            tokens_predicted,
            latency_ms: started.elapsed().as_millis(),
        })
    }

    async fn execute_stream(
        &self,
        request: CompletionRequest,
        span: Span,
    ) -> anyhow::Result<CompletionStream> {
        let _guard = span.enter();
        let body = AnthropicRequest::from_completion(&self.config.model, request.clone(), true);
        let url = format!("{}/v1/messages", self.config.base_url);

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.version)
            .json(&body)
            .send()
            .await
            .context("failed to call Anthropic streaming endpoint")?
            .error_for_status()
            .map_err(|err| anyhow!("anthropic streaming completion failed: {}", err))?;

        Ok(parse_sse_stream(response, parse_anthropic_stream_chunk))
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    #[serde(rename = "max_tokens")]
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

impl AnthropicRequest {
    fn from_completion(model: &str, request: CompletionRequest, stream: bool) -> Self {
        Self {
            model: model.to_string(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: vec![AnthropicContentBlock {
                    r#type: "text".to_string(),
                    text: request.prompt,
                }],
            }],
            temperature: request.temperature,
            stop_sequences: request.stop,
            stream,
        }
    }
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    r#type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    model: Option<String>,
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicContentDelta>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentDelta {
    #[serde(default)]
    text: Option<String>,
}

fn parse_anthropic_stream_chunk(payload: &str) -> Option<CompletionChunk> {
    let event: AnthropicStreamEvent = serde_json::from_str(payload).ok()?;
    match event.event_type.as_str() {
        "content_block_delta" => {
            let content = event.delta.and_then(|delta| delta.text).unwrap_or_default();
            if content.is_empty() {
                None
            } else {
                Some(CompletionChunk {
                    content,
                    is_final: false,
                    tokens_evaluated: None,
                    tokens_predicted: None,
                })
            }
        }
        "message_delta" | "message_stop" => Some(CompletionChunk {
            content: String::new(),
            is_final: true,
            tokens_evaluated: event
                .usage
                .as_ref()
                .and_then(|usage| usage.input_tokens)
                .map(|value| value as usize),
            tokens_predicted: event
                .usage
                .as_ref()
                .and_then(|usage| usage.output_tokens)
                .map(|value| value as usize),
        }),
        _ => None,
    }
}

fn default_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_version() -> String {
    "2023-06-01".to_string()
}

const fn default_timeout() -> u64 {
    30
}
