use anyhow::Result;
use async_trait::async_trait;

/// Configuration for inference requests
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            stop_sequences: vec![],
        }
    }
}

/// Trait for inference engines
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// Generate text from a prompt
    async fn generate(&self, prompt: &str, config: InferenceConfig) -> Result<String>;

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Check if the engine is available
    async fn is_available(&self) -> bool;
}

/// Llama.cpp inference engine implementation
pub struct LlamaInferenceEngine {
    client: noa_inference::LlamaClient,
    model_name: String,
}

impl LlamaInferenceEngine {
    pub fn new(base_url: String, model_name: String) -> Self {
        Self {
            client: noa_inference::LlamaClient::new(base_url),
            model_name,
        }
    }
}

#[async_trait]
impl InferenceEngine for LlamaInferenceEngine {
    async fn generate(&self, prompt: &str, config: InferenceConfig) -> Result<String> {
        let request = noa_inference::CompletionRequest {
            prompt: prompt.to_string(),
            temperature: Some(config.temperature),
            max_tokens: Some(config.max_tokens),
            stop: if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences)
            },
        };

        let response = self.client.completion(request).await?;
        Ok(response.content)
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn is_available(&self) -> bool {
        self.client.health_check().await.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = InferenceConfig::default();
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 2048);
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = LlamaInferenceEngine::new(
            "http://127.0.0.1:8080".to_string(),
            "llama-3.2-3b".to_string(),
        );
        assert_eq!(engine.model_name(), "llama-3.2-3b");
    }
}
