# 🤖 Model Selector Agent - Automated Model Selection

**Status**: ✅ Implemented and Ready  
**Purpose**: Automatically select optimal AI models for each task  
**Integration**: Complete with llama.cpp and inference system  

---

## 🎯 **What It Does**

The ModelSelector agent automatically chooses the best AI model for each task based on:

1. **Task Type** - Code generation, reasoning, agent tasks, etc.
2. **Performance Requirements** - Quality, speed, accuracy
3. **Cost Constraints** - Compute resources, latency
4. **Privacy Requirements** - Public, internal, confidential, restricted
5. **Historical Performance** - Learning from past successes

---

## 🏗️ **Architecture**

```
Task Request
     ↓
ModelSelector Agent
     ↓
  ┌──────────────────┐
  │ 1. Filter Models │ ← Privacy tier check
  └──────────────────┘
     ↓
  ┌──────────────────┐
  │ 2. Score Models  │ ← Use case match (40%)
  └──────────────────┘   Performance (30%)
     ↓                   Cost (15%)
  ┌──────────────────┐   History (15%)
  │ 3. Select Best   │
  └──────────────────┘
     ↓
Selected Model + Rationale
```

---

## 📊 **Model Registration**

### **Automatic Registration**

```powershell
# Scan and register all models
.\scripts\dev\register-models.ps1

# Verbose output
.\scripts\dev\register-models.ps1 -Verbose

# Dry run (preview only)
.\scripts\dev\register-models.ps1 -DryRun
```

### **Manual Registration**

```rust
use noa_agents::ModelSelectorAgent;
use noa_agents::model_selector::{ModelInfo, PrivacyTier, UseCase};

let selector = ModelSelectorAgent::new();

// Register a model
let model = ModelInfo {
    name: "deepseek-coder".to_string(),
    file_path: "models/deepseek-coder-v2.gguf".to_string(),
    size_mb: 7000,
    capabilities: vec!["code".to_string()],
    performance_score: 0.9,
    cost_score: 0.6,
    privacy_tier: PrivacyTier::Internal,
    use_cases: vec![UseCase::CodeGeneration, UseCase::CodeAnalysis],
};

selector.register_model(model)?;
```

---

## 🎮 **Usage Examples**

### **1. Code Generation Task**

```rust
use noa_agents::ModelSelectorAgent;
use noa_agents::model_selector::{TaskRequirements, UseCase, PrivacyTier};

let selector = ModelSelectorAgent::new();

// Define task requirements
let requirements = TaskRequirements {
    use_case: UseCase::CodeGeneration,
    privacy_tier: PrivacyTier::Internal,
    max_latency_ms: None,
    max_cost: Some(0.7),
    min_quality: 0.8,
    context_size: Some(8192),
};

// Get optimal model
let selection = selector.select_model(requirements)?;

println!("Selected: {}", selection.model.name);
println!("Confidence: {:.2}", selection.confidence);
println!("Rationale: {}", selection.rationale);
println!("Alternatives: {:?}", selection.alternatives.iter().map(|m| &m.name).collect::<Vec<_>>());
```

**Output**:
```
Selected: deepseek-coder-v2
Confidence: 0.92
Rationale: Specialized for CodeGeneration; Performance score: 0.90; Cost score: 0.60
Alternatives: ["stable-code-3b", "llama-3.1-8b"]
```

### **2. Reasoning Task**

```rust
let requirements = TaskRequirements {
    use_case: UseCase::Reasoning,
    privacy_tier: PrivacyTier::Confidential,  // No external APIs
    max_latency_ms: Some(5000),
    max_cost: None,
    min_quality: 0.85,
    context_size: Some(16384),
};

let selection = selector.select_model(requirements)?;
// Selects: OpenThinker-7B (reasoning specialist)
```

### **3. Agent Task (Function Calling)**

```rust
let requirements = TaskRequirements {
    use_case: UseCase::FunctionCalling,
    privacy_tier: PrivacyTier::Internal,
    max_latency_ms: Some(2000),
    max_cost: Some(0.8),
    min_quality: 0.75,
    context_size: None,
};

let selection = selector.select_model(requirements)?;
// Selects: Octopus-v2 (function calling specialist)
```

### **4. Fast General Task**

```rust
let requirements = TaskRequirements {
    use_case: UseCase::General,
    privacy_tier: PrivacyTier::Public,
    max_latency_ms: Some(1000),  // Fast response needed
    max_cost: Some(0.9),          // Low cost
    min_quality: 0.7,             // Acceptable quality
    context_size: Some(4096),
};

let selection = selector.select_model(requirements)?;
// Selects: SmolLM3-3B (fast and efficient)
```

---

## 📈 **Learning & Adaptation**

The ModelSelector learns from usage and improves over time:

```rust
// After using a model, record the outcome
selector.record_usage(
    "deepseek-coder-v2",  // Model name
    true,                  // Success
    2500,                  // Latency in ms
    0.92                   // Quality score
);

// Get statistics
let stats = selector.get_stats("deepseek-coder-v2");
println!("Success rate: {:.1}%", 
    stats.successes as f32 / stats.total_uses as f32 * 100.0);
println!("Avg latency: {:.0}ms", stats.avg_latency_ms);
println!("Avg quality: {:.2}", stats.avg_quality_score);
```

---

## 🎯 **Use Cases & Model Mapping**

| Use Case | Best Models | Alternative |
|----------|-------------|-------------|
| **CodeGeneration** | DeepSeek-Coder-V2, StableCode-3B | Llama-3.1-8B |
| **CodeAnalysis** | DeepSeek-Coder-V2 | Llama-3.2-3B |
| **Reasoning** | OpenThinker-7B, Phi-4-Mini | Llama-3.1-8B |
| **AgentTask** | Octopus-v2, Tiny-Agent-3B | Llama-3.2-3B |
| **FunctionCalling** | Octopus-v2 | Llama-3.1-8B |
| **QuestionAnswering** | Llama-3.1-8B, Mistral-Nemo | Llama-3.2-3B |
| **Documentation** | Llama-3.2-3B, Phi-4-Mini | SmolLM3-3B |
| **General** | Llama-3.2-3B, SmolLM3-3B | Any available |

---

## 🔒 **Privacy Tiers**

### **Tier Hierarchy** (Most to Least Restrictive):

1. **Restricted** - Maximum security, specific models only
2. **Confidential** - Highly sensitive, local only
3. **Internal** - Internal data, no external APIs
4. **Public** - Can use any model including external

### **Privacy Enforcement**:

```rust
// Confidential task - will ONLY use local models
let requirements = TaskRequirements {
    privacy_tier: PrivacyTier::Confidential,
    // ... other requirements
};

// Public task - can use any model
let requirements = TaskRequirements {
    privacy_tier: PrivacyTier::Public,
    // ... other requirements
};
```

---

## 🔧 **Scoring Algorithm**

The ModelSelector scores each model using:

```
Total Score = (Use Case Match × 0.4) +
              (Performance × 0.3) +
              (Cost Efficiency × 0.15) +
              (Historical Success × 0.15)

Penalties:
- Exceeds cost limit: Score × 0.5
- Below quality threshold: Score × 0.7
```

### **Example Scoring**:

```
Model: DeepSeek-Coder-V2
Task: Code Generation

Use Case Match:    0.4 (✅ Specialized)
Performance:       0.27 (0.9 × 0.3)
Cost:              0.09 (0.6 × 0.15)
Historical:        0.135 (0.9 success × 0.15)
─────────────────────────────────────
Total Score:       0.895 (89.5%)

Rationale: "Specialized for CodeGeneration; Performance score: 0.90; Cost score: 0.60; Historical success rate: 90.0%"
```

---

## 🚀 **Integration with Agents**

### **Agent Using ModelSelector**

```rust
use noa_agents::{Agent, AgentFactory, ModelSelectorAgent};
use noa_agents::model_selector::{TaskRequirements, UseCase, PrivacyTier};
use noa_inference::{LlamaClient, CompletionRequest};

pub struct CodeAnalysisAgent {
    model_selector: ModelSelectorAgent,
    inference_client: LlamaClient,
}

impl CodeAnalysisAgent {
    pub async fn analyze_code(&self, code: &str) -> Result<String> {
        // 1. Define task requirements
        let requirements = TaskRequirements {
            use_case: UseCase::CodeAnalysis,
            privacy_tier: PrivacyTier::Internal,
            max_latency_ms: Some(5000),
            max_cost: Some(0.7),
            min_quality: 0.8,
            context_size: Some(8192),
        };
        
        // 2. Let ModelSelector choose optimal model
        let selection = self.model_selector.select_model(requirements)?;
        
        println!("Using model: {} (confidence: {:.2})", 
            selection.model.name, selection.confidence);
        
        // 3. Use selected model for inference
        let prompt = format!("Analyze this code and suggest improvements:\n\n{}", code);
        
        let request = CompletionRequest {
            prompt,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stop: None,
        };
        
        let start = std::time::Instant::now();
        let response = self.inference_client.completion(request).await?;
        let latency = start.elapsed().as_millis() as u64;
        
        // 4. Record usage for learning
        self.model_selector.record_usage(
            &selection.model.name,
            true,  // Assume success if no error
            latency,
            0.85   // Quality score (could be evaluated)
        );
        
        Ok(response.content)
    }
}
```

---

## 📋 **Configuration File**

The registration script creates `models.json`:

```json
{
  "registered_at": "2025-01-08 19:30:00",
  "models": [
    {
      "name": "deepseek-coder-v2",
      "file_path": "models/deepseek-coder-v2.Q4_K_M.gguf",
      "size_mb": 7000,
      "capabilities": [],
      "performance_score": 0.9,
      "cost_score": 0.6,
      "privacy_tier": "Internal",
      "use_cases": ["CodeGeneration", "CodeAnalysis"]
    },
    {
      "name": "openthinker-7b",
      "file_path": "models/OpenThinker3-7B-q8_0.gguf",
      "size_mb": 7500,
      "capabilities": [],
      "performance_score": 0.9,
      "cost_score": 0.5,
      "privacy_tier": "Internal",
      "use_cases": ["Reasoning", "QuestionAnswering"]
    }
  ]
}
```

---

## 🎯 **Quick Start**

### **1. Register Models**

```powershell
cd D:\dev\workspaces\noa_ark_os

# Download some models first
.\scripts\dev\download-models.ps1 -ModelName smollm3-3b
.\scripts\dev\download-models.ps1 -ModelName openthinker-7b

# Register all models
.\scripts\dev\register-models.ps1
```

### **2. Use in Code**

```rust
let selector = ModelSelectorAgent::new();

// Select model for your task
let selection = selector.select_model(requirements)?;

// Use the selected model
// (integrate with your inference client)
```

### **3. Monitor Performance**

```rust
// Check model statistics
for model in selector.list_models() {
    if let Some(stats) = selector.get_stats(&model.name) {
        println!("{}: {:.1}% success, {:.0}ms avg",
            model.name,
            stats.successes as f32 / stats.total_uses as f32 * 100.0,
            stats.avg_latency_ms
        );
    }
}
```

---

## 🔄 **Continuous Improvement**

The ModelSelector automatically:

1. **Tracks Usage** - Records every model use
2. **Measures Performance** - Latency, quality, success rate
3. **Adjusts Scores** - Updates scores based on historical data
4. **Learns Patterns** - Identifies which models work best for which tasks

Over time, the system gets better at selecting the optimal model!

---

## 📊 **Benefits**

✅ **Automated** - No manual model selection needed  
✅ **Optimal** - Always chooses best model for the task  
✅ **Efficient** - Considers cost and performance  
✅ **Secure** - Enforces privacy requirements  
✅ **Learning** - Improves with usage  
✅ **Transparent** - Provides rationale for choices  

---

**Status**: ✅ Fully implemented and ready to use!  
**Next**: Register your models and start using automated selection!
