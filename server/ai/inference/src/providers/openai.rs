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
pub struct OpenAiConfig {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    config: OpenAiConfig,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(config: OpenAiConfig) -> Self {
        let mut config = config;
        if config.base_url.trim().is_empty() {
            config.base_url = default_base_url();
        }
        let timeout = Duration::from_secs(config.request_timeout_secs);
        let client = Client::builder()
            .user_agent("noa-ark-os-inference/1.0")
            .timeout(timeout)
            .build()
            .expect("failed to construct OpenAI client");

        Self { config, client }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "openai",
            model: self.config.model.clone(),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        let span = tracing::info_span!(
            "openai.complete",
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        self.execute_completion(request, span).await
    }

    async fn stream(&self, request: CompletionRequest) -> anyhow::Result<CompletionStream> {
        let span = tracing::info_span!(
            "openai.stream",
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        self.execute_stream(request, span).await
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/v1/models", self.config.base_url);
        let request = self
            .client
            .get(url)
            .bearer_auth(&self.config.api_key)
            .build()
            .context("failed to build OpenAI health check request")?;
        let response = self
            .client
            .execute(request)
            .await
            .context("failed to execute OpenAI health check")?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "openai health check failed with status {}",
                response.status()
            ))
        }
    }
}

impl OpenAiProvider {
    async fn execute_completion(
        &self,
        request: CompletionRequest,
        span: Span,
    ) -> anyhow::Result<CompletionResponse> {
        let _guard = span.enter();
        let started = std::time::Instant::now();

        let body = ChatRequest::from_completion(&self.config.model, request.clone(), false);
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        let mut builder = self
            .client
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&body);

        if let Some(org) = &self.config.organization {
            builder = builder.header("OpenAI-Organization", org);
        }

        let response = builder
            .send()
            .await
            .context("failed to call OpenAI completions endpoint")?
            .error_for_status()
            .map_err(|err| anyhow!("openai completion failed: {}", err))?;

        let payload: OpenAiChatResponse = response
            .json()
            .await
            .context("failed to parse OpenAI completion response")?;

        let content = payload
            .choices
            .iter()
            .flat_map(|choice| choice.message.content.clone())
            .collect::<Vec<_>>()
            .join("");
        let model = payload.model.unwrap_or_else(|| self.config.model.clone());
        let tokens_evaluated = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.prompt_tokens)
            .unwrap_or_default() as usize;
        let tokens_predicted = payload
            .usage
            .as_ref()
            .and_then(|usage| usage.completion_tokens)
            .unwrap_or_default() as usize;

        Ok(CompletionResponse {
            content,
            model,
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
        let body = ChatRequest::from_completion(&self.config.model, request.clone(), true);
        let url = format!("{}/v1/chat/completions", self.config.base_url);

        let mut builder = self
            .client
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&body);

        if let Some(org) = &self.config.organization {
            builder = builder.header("OpenAI-Organization", org);
        }

        let response = builder
            .send()
            .await
            .context("failed to call OpenAI streaming endpoint")?
            .error_for_status()
            .map_err(|err| anyhow!("openai streaming completion failed: {}", err))?;

        Ok(parse_sse_stream(response, parse_openai_stream_chunk))
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
}

impl ChatRequest {
    fn from_completion(model: &str, request: CompletionRequest, stream: bool) -> Self {
        Self {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: request.prompt,
            }],
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stop: request.stop,
            stream,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamEvent {
    #[serde(default)]
    choices: Vec<OpenAiStreamChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    #[serde(default)]
    delta: Option<OpenAiStreamDelta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    #[serde(default)]
    content: Option<String>,
}

fn parse_openai_stream_chunk(payload: &str) -> Option<CompletionChunk> {
    let event: OpenAiStreamEvent = serde_json::from_str(payload).ok()?;
    let mut content = String::new();
    for choice in &event.choices {
        if let Some(delta) = &choice.delta {
            if let Some(fragment) = &delta.content {
                content.push_str(fragment);
            }
        }
    }
    let is_final = event
        .choices
        .iter()
        .any(|choice| choice.finish_reason.is_some());
    let tokens_evaluated = event
        .usage
        .as_ref()
        .and_then(|usage| usage.prompt_tokens)
        .map(|value| value as usize);
    let tokens_predicted = event
        .usage
        .as_ref()
        .and_then(|usage| usage.completion_tokens)
        .map(|value| value as usize);

    if content.is_empty() && !is_final && tokens_evaluated.is_none() && tokens_predicted.is_none() {
        return None;
    }

    Some(CompletionChunk {
        content,
        is_final,
        tokens_evaluated,
        tokens_predicted,
    })
}

fn default_base_url() -> String {
    "https://api.openai.com".to_string()
}

const fn default_timeout() -> u64 {
    30
}
