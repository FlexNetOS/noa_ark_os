use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::Provider;
use crate::client::{CompletionRequest, CompletionResponse};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlamaCppConfig {
    pub endpoint: String,
}

#[derive(Debug, Clone, Default)]
pub struct LlamaCppProvider {
    config: LlamaCppConfig,
}

impl LlamaCppProvider {
    pub fn new(config: LlamaCppConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Provider for LlamaCppProvider {
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        info!("llama.cpp offline completion prompt={}", request.prompt);
        Ok(CompletionResponse {
            content: format!("[llama.cpp]: {}", request.prompt),
            model: "llama.cpp".into(),
            tokens_evaluated: request.prompt.len(),
            tokens_predicted: request.max_tokens.unwrap_or_default(),
        })
    }
}
