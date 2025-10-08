// NOA ARK OS - Agent Registry Demo
// Demonstrates integration of 928-agent ecosystem

use noa_agents::{AgentRegistry, AgentLayer, HealthStatus};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 NOA ARK OS - Agent Registry Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize registry
    println!("\n📦 Initializing agent registry...");
    let registry = AgentRegistry::new();
    
    // Load agent directory from CRC drop
    println!("📥 Loading agent directory from CRC drop...");
    let csv_path = "crc/drop-in/incoming/stale/agents/agent_directory.csv";
    
    match registry.load_from_csv(csv_path) {
        Ok(count) => {
            println!("✓ Loaded {} agents successfully!", count);
            
            // Display statistics
            display_statistics(&registry);
            
            // Show agents by layer
            display_agents_by_layer(&registry);
            
            // Show health summary
            display_health_summary(&registry);
            
            // Show example agents
            display_example_agents(&registry);
        }
        Err(e) => {
            println!("❌ Failed to load registry: {}", e);
            println!("\n💡 Make sure the CSV file exists at:");
            println!("   {}", csv_path);
            println!("\n   Run the simulation first:");
            println!("   .\\simulate-crc-flow.ps1");
        }
    }
    
    Ok(())
}

fn display_statistics(registry: &AgentRegistry) {
    println!("\n📊 Agent Ecosystem Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let stats = registry.stats();
    
    println!("Total Agents: {}", stats.total_agents);
    println!("  ✅ Healthy: {}", stats.healthy_agents);
    println!("  ⚠️  Needs Repair: {}", stats.needs_repair);
    println!("  ❓ Unknown Status: {}", stats.unknown_status);
    
    println!("\nAgents by Layer:");
    for (layer, count) in &stats.agents_by_layer {
        println!("  {}: {}", layer, count);
    }
}

fn display_agents_by_layer(registry: &AgentRegistry) {
    println!("\n👔 Board-Level Agents (Executive Team)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let board_agents = registry.by_layer(&AgentLayer::Board);
    for agent in board_agents.iter().take(10) {
        println!("  • {} - {}", agent.agent_name, agent.role);
        if !agent.purpose.is_empty() {
            println!("    Purpose: {}", agent.purpose);
        }
    }
    
    if board_agents.len() > 10 {
        println!("  ... and {} more", board_agents.len() - 10);
    }
}

fn display_health_summary(registry: &AgentRegistry) {
    println!("\n⚕️  Health Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let needs_repair = registry.agents_needing_repair();
    
    if needs_repair.is_empty() {
        println!("✓ All agents are healthy!");
    } else {
        println!("Found {} agents needing repair:\n", needs_repair.len());
        
        for agent in needs_repair.iter().take(5) {
            println!("  ⚠️  {}", agent.agent_name);
            
            if !agent.issues_identified.is_empty() {
                println!("    Issues:");
                for issue in agent.issues_identified.iter().take(2) {
                    if !issue.is_empty() {
                        println!("      - {}", issue);
                    }
                }
            }
            
            if !agent.repair_recommendations.is_empty() {
                println!("    Recommendations:");
                for rec in agent.repair_recommendations.iter().take(2) {
                    if !rec.is_empty() {
                        println!("      → {}", rec);
                    }
                }
            }
            println!();
        }
        
        if needs_repair.len() > 5 {
            println!("  ... and {} more agents needing repair", needs_repair.len() - 5);
        }
    }
}

fn display_example_agents(registry: &AgentRegistry) {
    println!("\n🔍 Example Agent Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Show a few key agents
    let example_ids = vec![
        "AgentSupervisorHeartbeatAgent",
        "BackupRestoreAgent",
        "CodeGenAgent",
        "ExecutiveCommanderChiefAgent",
    ];
    
    for agent_id in example_ids {
        if let Some(agent) = registry.get(agent_id) {
            println!("\n📋 {}", agent.agent_name);
            println!("   ID: {}", agent.agent_id);
            println!("   Layer: {}", agent.layer_name());
            println!("   Role: {}", agent.role);
            println!("   Health: {:?}", agent.health_status);
            
            if !agent.purpose.is_empty() {
                println!("   Purpose: {}", agent.purpose);
            }
            
            if let Some(stack) = &agent.stack {
                println!("   Stack: {}", stack);
            }
        }
    }
}
