#!/usr/bin/env python3
"""
Agent Auto-Generator
Automatically generates Rust agent implementations from the 928-agent registry.
"""

import csv
import os
import re
from pathlib import Path
from typing import Dict, List, Set

# Existing agents (already implemented)
EXISTING_AGENTS = {
    # L1 Autonomy
    "noa-commander",
    
    # L2 Board
    "digest-agent",
    "finance-agent",
    "legal-agent",
    "operations-agent",
    "strategy-agent",
    
    # L2 Executive
    "emergency-responder",
    "priority-manager",
    "resource-allocator",
    "system-orchestrator",
    
    # L4 Specialist
    "code-generation-agent",
    "data-analytics-agent",
    "deployment-agent",
    "integration-agent",
    "learning-agent",
    "monitoring-agent",
    "security-agent",
    "testing-agent",
    
    # L4 Pre-existing
    "model-selector-agent",
}

# Layer mapping
LAYER_MAP = {
    "L1Autonomy": "autonomy",
    "L2Reasoning": "reasoning",
    "L3Orchestration": "orchestration",
    "L4Operations": "operations",
    "L5Infrastructure": "infrastructure",
}

def sanitize_name(name: str) -> str:
    """Convert agent name to valid Rust identifier"""
    name = name.lower()
    name = re.sub(r'[^a-z0-9_]', '_', name)
    name = re.sub(r'_+', '_', name)
    name = name.strip('_')
    return name

def to_camel_case(name: str) -> str:
    """Convert snake_case to CamelCase"""
    parts = name.split('_')
    return ''.join(p.capitalize() for p in parts if p)

def generate_agent_struct(agent: Dict) -> str:
    """Generate Rust agent struct code"""
    struct_name = to_camel_case(sanitize_name(agent['name']))
    agent_id = sanitize_name(agent['name'])
    
    return f'''//! {agent['name']} - Auto-generated
//! 
//! {agent.get('purpose', 'Agent implementation')}

use crate::unified_types::*;
use crate::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

/// {agent['name']}
pub struct {struct_name} {{
    metadata: AgentMetadata,
    state: RwLock<AgentState>,
}}

impl {struct_name} {{
    pub fn new() -> Self {{
        let metadata = AgentMetadata {{
            id: Uuid::new_v4(),
            agent_id: "{agent_id}".to_string(),
            name: "{agent['name']}".to_string(),
            layer: AgentLayer::{agent['layer']},
            category: AgentCategory::Other,
            agent_type: AgentType::Worker,
            language: AgentLanguage::Rust,
            description: "{agent.get('purpose', 'Auto-generated agent')}".to_string(),
            role: "{agent.get('role', 'Agent')}".to_string(),
            purpose: "{agent.get('purpose', 'Agent functionality')}".to_string(),
            state: AgentState::Created,
            health_status: HealthStatus::Unknown,
            parent_id: None,
            escalation_to: None,
            stack: None,
            capabilities: vec![],
            tools: vec![],
            tags: vec![],
            inputs: vec![],
            outputs: vec![],
            dependencies: vec![],
            cpu_min: "0.5".to_string(),
            ram_min: "256MB".to_string(),
            disk_min: "100MB".to_string(),
            autonomy_level: "autonomous".to_string(),
            disposable: false,
            issues_identified: vec![],
            repair_recommendations: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            version: Some("1.0.0".to_string()),
        }};
        
        Self {{
            metadata,
            state: RwLock::new(AgentState::Created),
        }}
    }}
    
    pub async fn initialize(&mut self) -> Result<()> {{
        *self.state.write().await = AgentState::Ready;
        Ok(())
    }}
    
    pub fn metadata(&self) -> &AgentMetadata {{
        &self.metadata
    }}
    
    pub async fn state(&self) -> AgentState {{
        self.state.read().await.clone()
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[tokio::test]
    async fn test_agent_creation() {{
        let agent = {struct_name}::new();
        assert_eq!(agent.metadata().name, "{agent['name']}");
    }}
    
    #[tokio::test]
    async fn test_agent_initialization() {{
        let mut agent = {struct_name}::new();
        agent.initialize().await.unwrap();
        assert_eq!(agent.state().await, AgentState::Ready);
    }}
}}
'''

def load_registry(csv_path: str) -> List[Dict]:
    """Load agents from CSV registry"""
    agents = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            agent_name = row.get('agent_name', '').strip()
            if not agent_name:
                continue
            
            # Determine layer
            layer_str = row.get('layer', 'L4Operations')
            if 'board' in layer_str.lower() or 'l2' in layer_str.lower():
                layer = 'L2Reasoning'
            elif 'executive' in layer_str.lower() or 'l1' in layer_str.lower():
                layer = 'L1Autonomy'
            elif 'stack' in layer_str.lower() or 'l3' in layer_str.lower():
                layer = 'L3Orchestration'
            elif 'micro' in layer_str.lower() or 'l5' in layer_str.lower():
                layer = 'L5Infrastructure'
            else:
                layer = 'L4Operations'
            
            agents.append({
                'name': agent_name,
                'role': row.get('role', 'Agent'),
                'layer': layer,
                'purpose': row.get('purpose', 'Agent functionality'),
                'agent_id': row.get('agent_id', sanitize_name(agent_name)),
            })
    
    return agents

def generate_agents(registry_path: str, output_dir: str, batch_size: int = 50):
    """Generate agent implementations in batches"""
    
    print(f"🔍 Loading registry from: {registry_path}")
    agents = load_registry(registry_path)
    print(f"✓ Loaded {len(agents)} agents from registry")
    
    # Filter out existing agents
    new_agents = [a for a in agents if sanitize_name(a['name']) not in EXISTING_AGENTS]
    print(f"✓ Found {len(new_agents)} new agents to generate")
    print(f"✓ Skipping {len(agents) - len(new_agents)} existing agents")
    
    # Group by layer
    by_layer = {}
    for agent in new_agents:
        layer = agent['layer']
        if layer not in by_layer:
            by_layer[layer] = []
        by_layer[layer].append(agent)
    
    print(f"\n📊 Agent distribution:")
    for layer, agent_list in by_layer.items():
        print(f"   {layer}: {len(agent_list)} agents")
    
    # Process in batches
    total_generated = 0
    
    for layer, agent_list in by_layer.items():
        layer_dir = LAYER_MAP.get(layer, layer.lower())
        layer_path = Path(output_dir) / layer_dir
        layer_path.mkdir(parents=True, exist_ok=True)
        
        print(f"\n🔧 Generating {layer} agents...")
        
        # Process in batches
        for i in range(0, len(agent_list), batch_size):
            batch = agent_list[i:i+batch_size]
            batch_num = (i // batch_size) + 1
            
            print(f"   Batch {batch_num}: {len(batch)} agents", end='')
            
            for agent in batch:
                file_name = f"{sanitize_name(agent['name'])}.rs"
                file_path = layer_path / file_name
                
                # Generate code
                code = generate_agent_struct(agent)
                
                # Write file
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(code)
                
                total_generated += 1
            
            print(f" ✓")
    
    # Generate mod.rs files
    print(f"\n📝 Generating module files...")
    for layer, agent_list in by_layer.items():
        layer_dir = LAYER_MAP.get(layer, layer.lower())
        layer_path = Path(output_dir) / layer_dir
        
        mod_content = f"//! {layer} Agents - Auto-generated\n\n"
        
        for agent in agent_list:
            module_name = sanitize_name(agent['name'])
            struct_name = to_camel_case(module_name)
            mod_content += f"pub mod {module_name};\n"
        
        mod_content += "\n// Re-exports\n"
        for agent in agent_list:
            module_name = sanitize_name(agent['name'])
            struct_name = to_camel_case(module_name)
            mod_content += f"pub use {module_name}::{struct_name};\n"
        
        with open(layer_path / "mod.rs", 'w', encoding='utf-8') as f:
            f.write(mod_content)
    
    print(f"\n✅ Generation complete!")
    print(f"   Total agents generated: {total_generated}")
    print(f"   Output directory: {output_dir}")
    
    return total_generated

if __name__ == "__main__":
    import sys
    
    # Paths
    workspace_root = Path(__file__).parent.parent
    registry_path = workspace_root / "agents" / "data" / "agent_directory.csv"
    output_dir = workspace_root / "agents" / "src" / "implementations" / "generated"
    
    if not registry_path.exists():
        print(f"❌ Registry not found: {registry_path}")
        sys.exit(1)
    
    print("🚀 NOA ARK OS - Agent Auto-Generator")
    print("=" * 50)
    print()
    
    # Generate agents
    count = generate_agents(str(registry_path), str(output_dir), batch_size=50)
    
    print(f"\n🎉 Successfully generated {count} agents!")
    print(f"\n📋 Next steps:")
    print(f"   1. Review generated code in: {output_dir}")
    print(f"   2. Run: cargo build -p noa_agents")
    print(f"   3. Run: cargo test -p noa_agents")
    print(f"   4. Integrate into main module system")
