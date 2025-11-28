use std::time::Instant;

use crate::stream::{parse_json_lines_stream, parse_sse_stream, CompletionChunk, CompletionStream};
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tokens_evaluated: usize,
    #[serde(default)]
    pub tokens_predicted: usize,
    #[serde(default)]
    pub latency_ms: u128,
}

#[derive(Debug, Clone)]
pub struct LlamaClient {
    client: Client,
    base_url: String,
}

impl LlamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/completion", self.base_url);
        let started = Instant::now();

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send completion request")?
            .error_for_status()
            .map_err(|err| anyhow!("llama.cpp completion failed: {}", err))?;

        let mut result: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse completion response")?;
        result.latency_ms = started.elapsed().as_millis();

        Ok(result)
    }

    pub async fn stream_completion(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let url = format!("{}/completion", self.base_url);
        let mut payload = serde_json::to_value(&request)
            .context("Failed to serialise streaming completion request")?;
        payload["stream"] = json!(true);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send streaming completion request")?
            .error_for_status()
            .map_err(|err| anyhow!("llama.cpp streaming completion failed: {}", err))?;

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/json");

        if content_type.contains("text/event-stream") {
            Ok(parse_sse_stream(response, parse_llama_sse_chunk))
        } else {
            Ok(parse_json_lines_stream(response))
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

fn parse_llama_sse_chunk(payload: &str) -> Option<CompletionChunk> {
    let value: serde_json::Value = serde_json::from_str(payload).ok()?;
    let content = value
        .get("content")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let tokens_evaluated = value
        .get("tokens_evaluated")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize);
    let tokens_predicted = value
        .get("tokens_predicted")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize);
    let is_final = value
        .get("done")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    Some(CompletionChunk {
        content,
        is_final,
        tokens_evaluated,
        tokens_predicted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = LlamaClient::new("http://127.0.0.1:8080".to_string());
        assert_eq!(client.base_url, "http://127.0.0.1:8080");
    }

    // Integration test - requires llama.cpp server running
    #[tokio::test]
    #[ignore]
    async fn test_completion() {
        let client = LlamaClient::new("http://127.0.0.1:8080".to_string());

        let request = CompletionRequest {
            prompt: "Hello, how are you?".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(50),
            stop: None,
        };

        let response = client.completion(request).await.unwrap();
        assert!(!response.content.is_empty());
    }
}
