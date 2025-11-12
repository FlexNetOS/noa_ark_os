#![allow(dead_code)]

use async_trait::async_trait;

use crate::client::{CompletionRequest, CompletionResponse};

pub mod anthropic;
pub mod llama_cpp;
pub mod openai;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse>;
}
