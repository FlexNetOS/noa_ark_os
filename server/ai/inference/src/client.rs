use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CompletionRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tokens_evaluated: usize,
    #[serde(default)]
    pub tokens_predicted: usize,
}

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

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send completion request")?;

        let result: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse completion response")?;

        Ok(result)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
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
