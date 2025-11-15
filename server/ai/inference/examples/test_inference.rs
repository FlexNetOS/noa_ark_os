use noa_inference::{CompletionRequest, LlamaClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 Testing Llama.cpp Inference\n");

    let client = LlamaClient::new("http://127.0.0.1:8080".to_string());

    // Test connection
    print!("Checking server health... ");
    match client.health_check().await {
        Ok(true) => println!("✅ Healthy"),
        Ok(false) => {
            println!("❌ Unhealthy");
            println!("\n💡 Make sure to start the server first:");
            println!("   .\\scripts\\dev\\start-llama-server.ps1");
            return Ok(());
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("\n💡 Make sure to start the server first:");
            println!("   .\\scripts\\dev\\start-llama-server.ps1");
            return Ok(());
        }
    }

    // Test 1: Simple completion
    println!("\n📝 Test 1: Simple completion");
    let request = CompletionRequest {
        prompt: "What is Rust programming language?".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(100),
        stop: None,
    };

    let response = client.completion(request).await?;
    println!("Response: {}", response.content);
    println!("Tokens: {}", response.tokens_predicted);

    // Test 2: Code generation
    println!("\n💻 Test 2: Code generation");
    let request = CompletionRequest {
        prompt: "Write a Rust function to calculate fibonacci numbers".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(200),
        stop: None,
    };

    let response = client.completion(request).await?;
    println!("Response:\n{}", response.content);

    // Test 3: Code analysis
    println!("\n🔍 Test 3: Code analysis");
    let code = r#"
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
"#;

    let request = CompletionRequest {
        prompt: format!("Analyze this Rust code and explain what it does:\n{}", code),
        temperature: Some(0.7),
        max_tokens: Some(150),
        stop: None,
    };

    let response = client.completion(request).await?;
    println!("Analysis:\n{}", response.content);

    println!("\n✅ All tests completed successfully!");
    println!("\n💡 The inference client is ready to use in your code!");

    Ok(())
}
