//! NOA Interactive CLI
//! 
//! Command-line interface to interact with NOA autonomously.
//! Run with: cargo run --example noa_cli

use noa_agents::{
    AgentRegistry, ModelSelectorAgent,
    model_selector::*,
    inference::{LlamaInferenceEngine, InferenceConfig, InferenceEngine},
    communication::*,
};
use anyhow::Result;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🤖 NOA ARK OS - Interactive Agent System");
    println!("=" .repeat(70));
    println!();
    
    // Initialize system
    println!("⏳ Initializing NOA...");
    let registry = AgentRegistry::with_default_data()?;
    let hub = AgentCommunicationHub::new();
    let selector = ModelSelectorAgent::new();
    
    // Register agents
    let _commander = hub.register_agent(
        "noa-commander",
        AgentType::NoaCommander,
        vec!["orchestration".into()]
    ).await;
    
    // Register models
    register_models(&selector)?;
    
    println!("✅ NOA is ready with {} agents and {} models", 
        registry.count(), 
        selector.list_models().len()
    );
    println!();
    
    // Check server
    let engine = LlamaInferenceEngine::new(
        "http://127.0.0.1:8080".to_string(),
        "llama-3.2-3b".to_string()
    );
    
    let server_available = engine.is_available().await;
    if !server_available {
        println!("⚠️  Inference server not running");
        println!("   Start with: .\\scripts\\dev\\start-llama-server.ps1");
        println!();
    }
    
    println!("💡 Example prompts:");
    println!("   - Generate a Python function to sort a list");
    println!("   - Write a Rust HTTP server");
    println!("   - Explain quantum computing");
    println!("   - Create a database schema for a blog");
    println!();
    println!("Type 'exit' to quit");
    println!("=" .repeat(70));
    println!();
    
    // Interactive loop
    loop {
        print!("👤 You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let prompt = input.trim();
        
        if prompt.is_empty() {
            continue;
        }
        
        if prompt.eq_ignore_ascii_case("exit") || prompt.eq_ignore_ascii_case("quit") {
            println!("\n👋 Goodbye!");
            break;
        }
        
        println!();
        
        // Process prompt through NOA
        match process_prompt(&selector, &engine, prompt, server_available).await {
            Ok(_) => {},
            Err(e) => println!("❌ Error: {}", e),
        }
        
        println!();
    }
    
    Ok(())
}

async fn process_prompt(
    selector: &ModelSelectorAgent,
    engine: &LlamaInferenceEngine,
    prompt: &str,
    server_available: bool,
) -> Result<()> {
    println!("🤖 NOA: Processing your request...");
    println!();
    
    // Determine use case from prompt
    let use_case = if prompt.to_lowercase().contains("code") 
        || prompt.to_lowercase().contains("function")
        || prompt.to_lowercase().contains("write")
        || prompt.to_lowercase().contains("generate") {
        UseCase::CodeGeneration
    } else if prompt.to_lowercase().contains("explain")
        || prompt.to_lowercase().contains("what is") {
        UseCase::QuestionAnswering
    } else {
        UseCase::General
    };
    
    // Select model
    let requirements = TaskRequirements {
        use_case,
        privacy_tier: PrivacyTier::Internal,
        max_cost: Some(0.8),
        min_quality: 0.7,
        max_latency_ms: None,
        context_size: Some(8192),
    };
    
    let selection = selector.select_model(requirements)?;
    println!("📊 Model Selected: {} ({:.0}% confidence)", 
        selection.model.name,
        selection.confidence * 100.0
    );
    println!("💡 Rationale: {}", selection.rationale);
    println!();
    
    if !server_available {
        println!("⚠️  Cannot generate - server not running");
        println!("   Start server to see AI response");
        return Ok(());
    }
    
    // Generate response
    println!("⚡ Generating response...");
    let config = InferenceConfig {
        temperature: 0.7,
        max_tokens: 2000,
        top_p: 0.9,
        stop_sequences: vec![],
    };
    
    let start = std::time::Instant::now();
    let response = engine.generate(prompt, config).await?;
    let elapsed = start.elapsed();
    
    println!();
    println!("🤖 NOA Response:");
    println!("{}", "─".repeat(70));
    println!("{}", response);
    println!("{}", "─".repeat(70));
    println!();
    println!("⚡ Generated in {:.2}s ({:.1} tok/s)",
        elapsed.as_secs_f64(),
        2000.0 / elapsed.as_secs_f64()
    );
    
    // Record usage
    selector.record_usage(
        &selection.model.name,
        true,
        elapsed.as_millis() as u64,
        0.9
    );
    
    Ok(())
}

fn register_models(selector: &ModelSelectorAgent) -> Result<()> {
    let models = vec![
        ModelInfo {
            name: "deepseek-coder-v2".to_string(),
            file_path: "models/deepseek-coder-v2-q4.gguf".to_string(),
            size_mb: 9650,
            performance_score: 0.95,
            cost_score: 0.50,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::CodeGeneration, UseCase::CodeAnalysis],
            capabilities: vec![],
        },
        ModelInfo {
            name: "mistral-nemo-12b".to_string(),
            file_path: "models/mistral-nemo-12b-q4.gguf".to_string(),
            size_mb: 6960,
            performance_score: 0.93,
            cost_score: 0.55,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::General, UseCase::Reasoning],
            capabilities: vec![],
        },
        ModelInfo {
            name: "llama-3.1-8b".to_string(),
            file_path: "models/llama-3.1-8b-q4.gguf".to_string(),
            size_mb: 4580,
            performance_score: 0.92,
            cost_score: 0.65,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::General, UseCase::CodeGeneration],
            capabilities: vec![],
        },
        ModelInfo {
            name: "llama-3.2-3b".to_string(),
            file_path: "models/llama-3.2-3b-q4.gguf".to_string(),
            size_mb: 1926,
            performance_score: 0.85,
            cost_score: 0.70,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::General, UseCase::QuestionAnswering],
            capabilities: vec![],
        },
        ModelInfo {
            name: "tinyllama-1.1b".to_string(),
            file_path: "models/tinyllama-1.1b-q4.gguf".to_string(),
            size_mb: 620,
            performance_score: 0.68,
            cost_score: 0.95,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::General],
            capabilities: vec![],
        },
    ];
    
    for model in models {
        selector.register_model(model)?;
    }
    
    Ok(())
}
