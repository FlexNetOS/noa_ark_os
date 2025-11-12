use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::Provider;
use crate::client::{CompletionRequest, CompletionResponse};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnthropicConfig {
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct AnthropicProvider {
    config: AnthropicConfig,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        info!(model = %self.config.model, "anthropic offline completion");
        Ok(CompletionResponse {
            content: format!("[anthropic:{}]: {}", self.config.model, request.prompt),
            model: self.config.model.clone(),
            tokens_evaluated: request.prompt.len(),
            tokens_predicted: request.max_tokens.unwrap_or_default(),
        })
    }
}
