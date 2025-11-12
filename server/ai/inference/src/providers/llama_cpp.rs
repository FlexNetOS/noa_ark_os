use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
        info!(prompt = %request.prompt, "llama.cpp offline completion");
        tracing::info!(
            prompt = request.prompt.as_str(),
            "llama.cpp offline completion"
        );
        Ok(CompletionResponse {
            content: format!("[llama.cpp]: {}", request.prompt),
            model: "llama.cpp".into(),
            tokens_evaluated: request.prompt.len(),
            tokens_predicted: request.max_tokens.unwrap_or_default(),
        })
    }
}
