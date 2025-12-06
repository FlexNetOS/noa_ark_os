#![allow(dead_code)]

use async_trait::async_trait;

use crate::client::{CompletionRequest, CompletionResponse};
use crate::stream::CompletionStream;

pub mod anthropic;
pub mod llama_cpp;
pub mod openai;

#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    pub id: &'static str,
    pub model: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> anyhow::Result<CompletionStream>;
    async fn health_check(&self) -> anyhow::Result<()>;
}
