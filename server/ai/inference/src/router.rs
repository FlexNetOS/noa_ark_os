use std::env;
use std::sync::Arc;

use tracing::info;

use crate::client::{CompletionRequest, CompletionResponse};
use crate::providers::anthropic::{AnthropicConfig, AnthropicProvider};
use crate::providers::llama_cpp::{LlamaCppConfig, LlamaCppProvider};
use crate::providers::openai::{OpenAiConfig, OpenAiProvider};
use crate::providers::Provider;

#[derive(Clone)]
pub struct ProviderRouter {
    provider: Arc<dyn Provider>,
}

impl ProviderRouter {
    pub fn from_env() -> Self {
        let selected = env::var("AI_PROVIDER").unwrap_or_else(|_| "llama.cpp".into());
        match selected.as_str() {
            "openai" => {
                let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5".into());
                Self {
                    provider: Arc::new(OpenAiProvider::new(OpenAiConfig { model })),
                }
            }
            "anthropic" => {
                let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3".into());
                Self {
                    provider: Arc::new(AnthropicProvider::new(AnthropicConfig { model })),
                }
            }
            _ => {
                let endpoint = env::var("LLAMA_CPP_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:8080/v1".into());
                Self {
                    provider: Arc::new(LlamaCppProvider::new(LlamaCppConfig { endpoint })),
                }
            }
        }
    }

    pub fn with_provider(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    pub async fn completion(
        &self,
        request: CompletionRequest,
    ) -> anyhow::Result<CompletionResponse> {
        info!("routing completion request");
        self.provider.complete(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn llama_is_default() {
        let router = ProviderRouter::from_env();
        let response = router
            .completion(CompletionRequest {
                prompt: "hello".into(),
                max_tokens: Some(8),
            })
            .await
            .unwrap();
        assert!(response.completion.contains("llama"));
    }
}
