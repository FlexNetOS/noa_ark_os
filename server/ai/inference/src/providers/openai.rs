use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::Provider;
use crate::client::{CompletionRequest, CompletionResponse};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct OpenAiProvider {
    config: OpenAiConfig,
}

impl OpenAiProvider {
    pub fn new(config: OpenAiConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        info!("openai offline completion", model = %self.config.model);
        Ok(CompletionResponse {
            content: format!("[openai:{}]: {}", self.config.model, request.prompt),
            model: self.config.model.clone(),
            tokens_evaluated: request.prompt.len(),
            tokens_predicted: request.max_tokens.unwrap_or_default(),
        })
    }
}
