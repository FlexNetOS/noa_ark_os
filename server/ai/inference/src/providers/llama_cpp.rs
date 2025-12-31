use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Provider, ProviderMetadata};
use crate::client::{CompletionRequest, CompletionResponse, LlamaClient};
use crate::stream::CompletionStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    pub endpoint: String,
    #[serde(default = "default_model")]
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct LlamaCppProvider {
    config: LlamaCppConfig,
    client: LlamaClient,
}

impl LlamaCppProvider {
    pub fn new(config: LlamaCppConfig) -> Self {
        let client = LlamaClient::new(config.endpoint.clone());
        Self { config, client }
    }
}

#[async_trait]
impl Provider for LlamaCppProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "llama.cpp",
            model: self.config.model.clone(),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        let span = tracing::info_span!(
            "llama_cpp.complete",
            endpoint = %self.config.endpoint,
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        let _guard = span.enter();
        let mut response = self
            .client
            .completion(request)
            .await
            .context("failed to complete via llama.cpp")?;
        if response.model.is_empty() {
            response.model = self.config.model.clone();
        }
        Ok(response)
    }

    async fn stream(&self, request: CompletionRequest) -> anyhow::Result<CompletionStream> {
        let span = tracing::info_span!(
            "llama_cpp.stream",
            endpoint = %self.config.endpoint,
            model = %self.config.model,
            prompt_len = request.prompt.len(),
        );
        let _guard = span.enter();
        let stream = self
            .client
            .stream_completion(request)
            .await
            .context("failed to stream completion via llama.cpp")?;
        Ok(stream)
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        if self.client.health_check().await? {
            Ok(())
        } else {
            Err(anyhow!("llama.cpp health check failed"))
        }
    }
}

fn default_model() -> String {
    "llama.cpp".to_string()
}
