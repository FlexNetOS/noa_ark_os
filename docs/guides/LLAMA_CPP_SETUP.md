# Llama.cpp Server Integration Plan

**Date**: Current  
**Purpose**: Set up llama.cpp for local LLM inference  
**Location**: `server/ai/llama-cpp/`  
**Status**: Ready to Execute  

---

## 🎯 **Objectives**

1. ✅ Install llama.cpp in server
2. ✅ Load and configure models
3. ✅ Integrate with agent flow
4. ✅ Set up inference endpoints

---

## 📋 **Architecture**

### **Directory Structure**

```
server/
├── ai/
│   ├── llama-cpp/              # Llama.cpp installation
│   │   ├── bin/                # Compiled binaries
│   │   │   ├── llama-server    # Main server
│   │   │   ├── llama-cli       # CLI tool
│   │   │   └── quantize        # Model quantization
│   │   ├── models/             # Model files
│   │   │   ├── llama-3-8b/
│   │   │   ├── mistral-7b/
│   │   │   └── codellama-7b/
│   │   ├── configs/            # Server configs
│   │   │   ├── server.yaml
│   │   │   └── models.yaml
│   │   └── logs/               # Server logs
│   ├── inference/              # Inference service (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs       # Llama.cpp client
│   │   │   ├── models.rs       # Model management
│   │   │   └── router.rs       # Request routing
│   │   └── Cargo.toml
│   └── README.md
└── data/
    └── models/                 # Model cache (symlink)
```

---

## 🚀 **Step 1: Install Llama.cpp**

### **Option A: Prebuilt Binaries** (Fastest)

```powershell
cd D:\dev\workspaces\noa_ark_os\server\ai

# Create directory structure
mkdir -p llama-cpp/bin
mkdir -p llama-cpp/models
mkdir -p llama-cpp/configs
mkdir -p llama-cpp/logs

# Download prebuilt binaries
$llamaCppVersion = "b4315"  # Latest stable
$downloadUrl = "https://github.com/ggerganov/llama.cpp/releases/download/$llamaCppVersion/llama-$llamaCppVersion-bin-win-cuda-cu12.2.0-x64.zip"

# Download and extract
Invoke-WebRequest -Uri $downloadUrl -OutFile "llama-cpp.zip"
Expand-Archive -Path "llama-cpp.zip" -DestinationPath "llama-cpp/bin"
Remove-Item "llama-cpp.zip"

Write-Host "✅ Llama.cpp binaries installed"
```

### **Option B: Build from Source** (Customizable)

```powershell
cd D:\dev\workspaces\noa_ark_os\server\ai

# Clone llama.cpp
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp

# Build with CUDA support (if GPU available)
mkdir build
cd build
cmake .. -DGGML_CUDA=ON -DCMAKE_BUILD_TYPE=Release
cmake --build . --config Release

# Or build CPU-only
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build . --config Release

# Copy binaries
Copy-Item build\bin\Release\* ..\llama-cpp\bin\

Write-Host "✅ Llama.cpp built from source"
```

---

## 📥 **Step 2: Download Models**

### **Recommended Models**

```powershell
cd D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp\models

# Option 1: Llama 3.2 3B (Fast, good for agents)
Invoke-WebRequest -Uri "https://huggingface.co/QuantFactory/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct.Q4_K_M.gguf" -OutFile "llama-3.2-3b-q4.gguf"

# Option 2: Mistral 7B (Balanced)
Invoke-WebRequest -Uri "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf" -OutFile "mistral-7b-q4.gguf"

# Option 3: CodeLlama 7B (For code generation)
Invoke-WebRequest -Uri "https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF/resolve/main/codellama-7b-instruct.Q4_K_M.gguf" -OutFile "codellama-7b-q4.gguf"

Write-Host "✅ Models downloaded"
```

### **Model Sizes**

| Model | Quantization | Size | RAM Required | Use Case |
|-------|-------------|------|--------------|----------|
| Llama 3.2 3B | Q4_K_M | ~2GB | 4GB | Fast agent tasks |
| Mistral 7B | Q4_K_M | ~4GB | 8GB | General purpose |
| CodeLlama 7B | Q4_K_M | ~4GB | 8GB | Code generation |
| Llama 3.1 8B | Q4_K_M | ~5GB | 10GB | Advanced reasoning |

---

## ⚙️ **Step 3: Configure Llama.cpp Server**

### **Create Server Config**

```yaml
# server/ai/llama-cpp/configs/server.yaml
server:
  host: 127.0.0.1
  port: 8080
  threads: 8              # CPU threads
  gpu_layers: 35          # GPU acceleration (0 = CPU only)

models:
  - name: llama-3.2-3b
    path: ./models/llama-3.2-3b-q4.gguf
    context_size: 8192
    batch_size: 512
    default: true
    
  - name: mistral-7b
    path: ./models/mistral-7b-q4.gguf
    context_size: 8192
    batch_size: 512
    
  - name: codellama-7b
    path: ./models/codellama-7b-q4.gguf
    context_size: 16384
    batch_size: 512

inference:
  temperature: 0.7
  top_p: 0.9
  top_k: 40
  repeat_penalty: 1.1
  max_tokens: 2048

logging:
  level: info
  file: ./logs/server.log
  rotation: daily
```

### **Start Server Script**

```powershell
# scripts/dev/start-llama-server.ps1
$ServerPath = "D:\dev\workspaces\noa_ark_os\server\ai\llama-cpp"
$BinPath = Join-Path $ServerPath "bin\llama-server.exe"
$ModelPath = Join-Path $ServerPath "models\llama-3.2-3b-q4.gguf"

# Start llama.cpp server
& $BinPath `
    --model $ModelPath `
    --host 127.0.0.1 `
    --port 8080 `
    --ctx-size 8192 `
    --batch-size 512 `
    --threads 8 `
    --n-gpu-layers 35 `
    --log-format text `
    --log-file "$ServerPath\logs\server.log"

Write-Host "✅ Llama.cpp server started on http://127.0.0.1:8080"
```

---

## 🔧 **Step 4: Create Rust Inference Client**

### **Add Dependencies**

```toml
# server/ai/inference/Cargo.toml
[package]
name = "noa_inference"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

### **Create Client**

```rust
// server/ai/inference/src/client.rs
use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CompletionRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub stop_reason: Option<String>,
    pub tokens_evaluated: usize,
    pub tokens_predicted: usize,
}

pub struct LlamaClient {
    client: Client,
    base_url: String,
}

impl LlamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    pub async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/completion", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send completion request")?;
        
        let result: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse completion response")?;
        
        Ok(result)
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_completion() {
        let client = LlamaClient::new("http://127.0.0.1:8080".to_string());
        
        let request = CompletionRequest {
            prompt: "What is Rust programming language?".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(100),
            stop: None,
        };
        
        let response = client.completion(request).await.unwrap();
        assert!(!response.content.is_empty());
    }
}
```

---

## 🤖 **Step 5: Integrate with Agent Flow**

### **Agent Inference Trait**

```rust
// agents/src/inference.rs
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(&self, prompt: &str, config: InferenceConfig) -> Result<String>;
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
    fn model_name(&self) -> &str;
}

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

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

// Llama.cpp implementation
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
    
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        // Format messages into prompt
        let prompt = format_chat_prompt(&messages);
        
        let config = InferenceConfig::default();
        self.generate(&prompt, config).await
    }
    
    fn model_name(&self) -> &str {
        &self.model_name
    }
}

fn format_chat_prompt(messages: &[ChatMessage]) -> String {
    let mut prompt = String::new();
    
    for msg in messages {
        match msg.role {
            MessageRole::System => {
                prompt.push_str(&format!("<|system|>\n{}\n", msg.content));
            }
            MessageRole::User => {
                prompt.push_str(&format!("<|user|>\n{}\n", msg.content));
            }
            MessageRole::Assistant => {
                prompt.push_str(&format!("<|assistant|>\n{}\n", msg.content));
            }
        }
    }
    
    prompt.push_str("<|assistant|>\n");
    prompt
}
```

### **Update Agent Implementation**

```rust
// agents/src/implementations/digest_agent.rs (example)
use crate::inference::{InferenceEngine, InferenceConfig, ChatMessage, MessageRole};

pub struct DigestAgent {
    inference: Box<dyn InferenceEngine>,
    // ...other fields
}

impl DigestAgent {
    pub async fn analyze_repository(&self, repo_content: &str) -> Result<RepositoryDigest> {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "You are a code analysis assistant. Analyze the repository and provide a concise digest.".to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: format!("Analyze this repository:\n\n{}", repo_content),
            },
        ];
        
        let response = self.inference.chat(messages).await?;
        
        // Parse response into RepositoryDigest
        Ok(parse_digest(&response)?)
    }
}
```

---

## 📊 **Step 6: Testing & Validation**

### **Health Check Script**

```powershell
# scripts/testing/test-llama-server.ps1
$baseUrl = "http://127.0.0.1:8080"

Write-Host "Testing Llama.cpp server..."

# Health check
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get
    Write-Host "✅ Server is healthy"
} catch {
    Write-Host "❌ Server is not responding"
    exit 1
}

# Test completion
$body = @{
    prompt = "Write a hello world program in Rust."
    temperature = 0.7
    max_tokens = 200
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/completion" -Method Post -Body $body -ContentType "application/json"
    Write-Host "✅ Completion test passed"
    Write-Host "Response: $($response.content)"
} catch {
    Write-Host "❌ Completion test failed"
    exit 1
}

Write-Host "✅ All tests passed"
```

---

## 🎯 **Quick Start Commands**

```powershell
# 1. Install llama.cpp
cd D:\dev\workspaces\noa_ark_os
.\scripts\dev\setup-llama-cpp.ps1

# 2. Download models
.\scripts\dev\download-models.ps1

# 3. Start server
.\scripts\dev\start-llama-server.ps1

# 4. Test server
.\scripts\testing\test-llama-server.ps1

# 5. Integrate with agents
cargo build -p noa_agents
cargo test -p noa_agents --test inference_tests
```

---

## 📋 **Next Steps**

1. ✅ Install llama.cpp binaries
2. ✅ Download at least one model (recommend Llama 3.2 3B)
3. ✅ Start llama.cpp server
4. ✅ Create Rust inference client
5. ✅ Integrate with agent trait
6. ✅ Test with DigestAgent
7. ✅ Expand to other agents

---

**Status**: Ready to implement  
**Time Estimate**: 2-3 hours  
**Dependencies**: ~4GB disk space for model  
