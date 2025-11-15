//! Request and Response types for llama.cpp API

use serde::{Deserialize, Serialize};

/// Completion request to llama.cpp server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The prompt text
    pub prompt: String,
    
    /// Temperature for sampling (0.0-2.0, default: 0.7)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    
    /// Top-p sampling (0.0-1.0, default: 0.9)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    
    /// Repeat penalty (default: 1.1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// Stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            temperature: Some(0.7),
            max_tokens: Some(2048),
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: Some(1.1),
            stop: None,
            stream: Some(false),
        }
    }
}

/// Completion response from llama.cpp server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated text
    pub content: String,
    
    /// Model used
    #[serde(default)]
    pub model: String,
    
    /// Number of tokens generated
    #[serde(default)]
    pub tokens_predicted: usize,
    
    /// Generation stopped reason
    #[serde(default)]
    pub stop: bool,
    
    /// Tokens per second
    #[serde(default)]
    pub tokens_per_second: f32,
    
    /// Generation time in ms
    #[serde(default)]
    pub generation_time_ms: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Server status
    pub status: String,
    
    /// Model loaded
    #[serde(default)]
    pub model_loaded: bool,
    
    /// Model name
    #[serde(default)]
    pub model: String,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/path
    pub name: String,
    
    /// Model type
    #[serde(default)]
    pub model_type: String,
    
    /// Context size
    #[serde(default)]
    pub n_ctx: usize,
    
    /// Embedding dimension
    #[serde(default)]
    pub n_embd: usize,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: usize,
    
    /// Completion tokens
    pub completion_tokens: usize,
    
    /// Total tokens
    pub total_tokens: usize,
}
