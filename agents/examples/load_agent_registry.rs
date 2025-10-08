//! Agent Registry Demo
//! 
//! Demonstrates loading the 928-agent directory embedded in the crate
//! 
//! Usage:
//!   cargo run --example load_agent_registry
//!   cargo run --example load_agent_registry -- path\to\agent_directory.csv

use noa_agents::{AgentRegistry, AgentLayer};
use std::path::Path;
use std::env;

fn main() -> anyhow::Result<()> {
    println!("🤖 NOA ARK OS - Agent Registry Demo\n");
    
    // Create registry
    let registry = AgentRegistry::new();
    let args: Vec<String> = env::args().collect();
    
    // Optional override: `cargo run --example load_agent_registry -- <csv_path>`
    let load_result = if let Some(path) = args.get(1) {
        let csv_path = Path::new(path);
        println!("📂 Loading agent directory from: {}", csv_path.display());
        
        if !csv_path.exists() {
            eprintln!("❌ Error: Agent directory CSV not found at {}", csv_path.display());
            eprintln!("   Falling back to embedded copy bundled with the crate.\n");
            registry.load_default()
        } else {
            registry.load_from_csv(csv_path)
        }
    } else {
        println!("📦 Loading embedded agent directory bundled with the crate");
        registry.load_default()
    };
    
    // Load agents
    match load_result {
        Ok(count) => {
            println!("✓ Successfully loaded {} agents\n", count);
            
            // Display statistics
            let stats = registry.stats();
            println!("📊 Registry Statistics:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Total agents:    {}", stats.total_agents);
            println!("  Healthy:         {} ({:.1}%)", 
                stats.healthy_agents,
                (stats.healthy_agents as f64 / stats.total_agents as f64) * 100.0
            );
            println!("  Needs repair:    {} ({:.1}%)", 
                stats.needs_repair,
                (stats.needs_repair as f64 / stats.total_agents as f64) * 100.0
            );
            println!("  Unknown status:  {}", stats.unknown_status);
            println!();
            
            // Display agents by layer
            println!("📋 Agents by Layer:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            for (layer, count) in &stats.agents_by_layer {
                println!("  {:20} {}", layer, count);
            }
            println!();
            
            // Show sample agents from each layer
            println!("🔍 Sample Agents by Layer:\n");
            
            for layer in [
                AgentLayer::Board,
                AgentLayer::Executive,
                AgentLayer::StackChief,
                AgentLayer::Specialist,
                AgentLayer::Micro,
            ] {
                let agents = registry.by_layer(&layer);
                if !agents.is_empty() {
                    println!("  {} ({} agents)", layer.name(), agents.len());
                    
                    // Show first 3 agents
                    for agent in agents.iter().take(3) {
                        println!("    • {} - {}", agent.agent_name, agent.role);
                        if !agent.purpose.is_empty() {
                            let purpose = if agent.purpose.len() > 60 {
                                format!("{}...", &agent.purpose[..60])
                            } else {
                                agent.purpose.clone()
                            };
                            println!("      {}", purpose);
                        }
                    }
                    
                    if agents.len() > 3 {
                        println!("    ... and {} more", agents.len() - 3);
                    }
                    println!();
                }
            }
            
            // Show agents needing repair
            let needs_repair = registry.agents_needing_repair();
            if !needs_repair.is_empty() {
                println!("⚠️  Agents Needing Repair ({} total):\n", needs_repair.len());
                
                for agent in needs_repair.iter().take(5) {
                    println!("  {} - {}", agent.agent_name, agent.role);
                    
                    if !agent.issues_identified.is_empty() {
                        println!("    Issues:");
                        for issue in &agent.issues_identified {
                            if !issue.trim().is_empty() {
                                println!("      • {}", issue.trim());
                            }
                        }
                    }
                    
                    if !agent.repair_recommendations.is_empty() {
                        println!("    Recommendations:");
                        for rec in &agent.repair_recommendations {
                            if !rec.trim().is_empty() {
                                println!("      → {}", rec.trim());
                            }
                        }
                    }
                    println!();
                }
                
                if needs_repair.len() > 5 {
                    println!("  ... and {} more agents needing repair", needs_repair.len() - 5);
                }
            }
            
            println!("\n✅ Registry loaded successfully!");
            println!("💡 Use AgentRegistry API to query agents in your code");
        }
        Err(e) => {
            eprintln!("❌ Error loading registry: {}", e);
            eprintln!("\n💡 Troubleshooting:");
            eprintln!("   • Check that the CSV file exists when providing a custom path");
            eprintln!("   • Verify the CSV format (should have headers)");
            eprintln!("   • Check file permissions if loading from disk");
        }
    }
    
    Ok(())
}
