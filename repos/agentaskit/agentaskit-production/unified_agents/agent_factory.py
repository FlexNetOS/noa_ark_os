#!/usr/bin/env python3
"""
NOA Agent Factory - Creates and manages agent instances from manifests
Part of the Multi-Agent AgenticAI Task Deployment Kit
"""

import json
import uuid
import csv
import argparse
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Any
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class AgentLayer:
    """Agent hierarchy layers"""
    CECCA = "CECCA"
    BOARD = "Board"
    EXECUTIVE = "Executive"
    STACK_CHIEF = "StackChief"
    SPECIALIST = "Specialist"
    MICRO = "Micro"

class AgentFactory:
    """Factory for creating and managing agent instances"""
    
    def __init__(self, config_path: Optional[Path] = None):
        self.config_path = config_path or Path("config")
        self.agents_db = {}
        self.load_agent_directory()
    
    def load_agent_directory(self):
        """Load existing agent directory from CSV"""
        csv_path = self.config_path / "agents_directory.csv"
        if csv_path.exists():
            with open(csv_path, 'r', newline='', encoding='utf-8') as f:
                reader = csv.DictReader(f)
                for row in reader:
                    agent_id = row.get('agent_id')
                    if agent_id:
                        self.agents_db[agent_id] = row
            logger.info(f"Loaded {len(self.agents_db)} agents from directory")
    
    def save_agent_directory(self):
        """Save agent directory to CSV"""
        csv_path = self.config_path / "agents_directory.csv"
        csv_path.parent.mkdir(parents=True, exist_ok=True)
        
        fieldnames = [
            'agent_id', 'name', 'layer', 'capabilities', 'status',
            'resource_requirements', 'escalation_path', 'subordinates',
            'created_at', 'last_updated'
        ]
        
        with open(csv_path, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            for agent in self.agents_db.values():
                writer.writerow(agent)
        
        logger.info(f"Saved {len(self.agents_db)} agents to directory")
    
    def create_agent(self, 
                    name: str,
                    layer: str,
                    capabilities: List[str],
                    resource_requirements: Optional[Dict] = None,
                    escalation_path: Optional[str] = None) -> Dict[str, Any]:
        """Create a new agent instance"""
        
        agent_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        
        # Default resource requirements based on layer
        if resource_requirements is None:
            resource_requirements = self._get_default_resources(layer)
        
        agent = {
            'agent_id': agent_id,
            'name': name,
            'layer': layer,
            'capabilities': json.dumps(capabilities),
            'status': 'Initializing',
            'resource_requirements': json.dumps(resource_requirements),
            'escalation_path': escalation_path or '',
            'subordinates': json.dumps([]),
            'created_at': now,
            'last_updated': now
        }
        
        self.agents_db[agent_id] = agent
        logger.info(f"Created agent {name} ({layer}) with ID {agent_id}")
        
        return agent
    
    def _get_default_resources(self, layer: str) -> Dict[str, Any]:
        """Get default resource requirements for agent layer"""
        resource_map = {
            AgentLayer.CECCA: {
                "cpu_cores": 4.0,
                "memory_mb": 8192,
                "storage_mb": 10240,
                "network_bandwidth_mbps": 100
            },
            AgentLayer.BOARD: {
                "cpu_cores": 2.0,
                "memory_mb": 4096,
                "storage_mb": 5120,
                "network_bandwidth_mbps": 50
            },
            AgentLayer.EXECUTIVE: {
                "cpu_cores": 2.0,
                "memory_mb": 4096,
                "storage_mb": 5120,
                "network_bandwidth_mbps": 50
            },
            AgentLayer.STACK_CHIEF: {
                "cpu_cores": 1.5,
                "memory_mb": 2048,
                "storage_mb": 2560,
                "network_bandwidth_mbps": 25
            },
            AgentLayer.SPECIALIST: {
                "cpu_cores": 1.0,
                "memory_mb": 1024,
                "storage_mb": 1280,
                "network_bandwidth_mbps": 10
            },
            AgentLayer.MICRO: {
                "cpu_cores": 0.25,
                "memory_mb": 256,
                "storage_mb": 512,
                "network_bandwidth_mbps": 5
            }
        }
        
        return resource_map.get(layer, resource_map[AgentLayer.MICRO])
    
    def create_hierarchy(self, agent_count: int) -> List[Dict[str, Any]]:
        """Create a complete agent hierarchy with specified total count"""
        
        # Calculate distribution based on design specifications
        distribution = self._calculate_layer_distribution(agent_count)
        created_agents = []
        
        # Create agents for each layer
        for layer, count in distribution.items():
            layer_capabilities = self._get_layer_capabilities(layer)
            
            for i in range(count):
                agent_name = f"{layer}-Agent-{i+1:04d}"
                agent = self.create_agent(
                    name=agent_name,
                    layer=layer,
                    capabilities=layer_capabilities
                )
                created_agents.append(agent)
        
        # Establish hierarchy relationships
        self._establish_hierarchy_relationships(created_agents)
        
        logger.info(f"Created hierarchy with {len(created_agents)} agents")
        return created_agents
    
    def _calculate_layer_distribution(self, total_agents: int) -> Dict[str, int]:
        """Calculate agent distribution across layers"""
        
        cecca_count = min(3, max(1, total_agents // 100))
        board_count = min(15, max(5, total_agents // 20))
        executive_count = min(25, max(10, total_agents // 10))
        stack_chief_count = min(50, max(20, total_agents // 5))
        specialist_count = min(200, max(50, total_agents // 3))
        
        used = cecca_count + board_count + executive_count + stack_chief_count + specialist_count
        micro_count = max(0, total_agents - used) if total_agents > used else total_agents // 2
        
        return {
            AgentLayer.CECCA: cecca_count,
            AgentLayer.BOARD: board_count,
            AgentLayer.EXECUTIVE: executive_count,
            AgentLayer.STACK_CHIEF: stack_chief_count,
            AgentLayer.SPECIALIST: specialist_count,
            AgentLayer.MICRO: micro_count
        }
    
    def _get_layer_capabilities(self, layer: str) -> List[str]:
        """Get capabilities for each agent layer"""
        
        capability_map = {
            AgentLayer.CECCA: [
                "strategic_planning",
                "system_authority",
                "cross_organizational_coordination",
                "emergency_decision_making",
                "resource_allocation"
            ],
            AgentLayer.BOARD: [
                "policy_enforcement",
                "governance_oversight",
                "compliance_monitoring",
                "risk_assessment",
                "ethics_validation"
            ],
            AgentLayer.EXECUTIVE: [
                "operational_coordination",
                "task_orchestration",
                "resource_management",
                "performance_monitoring",
                "emergency_response"
            ],
            AgentLayer.STACK_CHIEF: [
                "domain_leadership",
                "subject_matter_expertise",
                "team_coordination",
                "workflow_orchestration",
                "specialization_management"
            ],
            AgentLayer.SPECIALIST: [
                "deep_domain_expertise",
                "complex_analysis",
                "system_integration",
                "advanced_processing",
                "decision_support"
            ],
            AgentLayer.MICRO: [
                "task_execution",
                "atomic_operations",
                "parallel_processing",
                "rule_based_actions",
                "resource_efficiency"
            ]
        }
        
        return capability_map.get(layer, [])
    
    def _establish_hierarchy_relationships(self, agents: List[Dict[str, Any]]):
        """Establish escalation paths and subordinate relationships"""
        
        # Group agents by layer
        layer_groups = {}
        for agent in agents:
            layer = agent['layer']
            if layer not in layer_groups:
                layer_groups[layer] = []
            layer_groups[layer].append(agent)
        
        # Establish relationships (simplified version)
        hierarchy_order = [
            AgentLayer.CECCA,
            AgentLayer.BOARD,
            AgentLayer.EXECUTIVE,
            AgentLayer.STACK_CHIEF,
            AgentLayer.SPECIALIST,
            AgentLayer.MICRO
        ]
        
        for i in range(1, len(hierarchy_order)):
            current_layer = hierarchy_order[i]
            parent_layer = hierarchy_order[i-1]
            
            if current_layer in layer_groups and parent_layer in layer_groups:
                current_agents = layer_groups[current_layer]
                parent_agents = layer_groups[parent_layer]
                
                # Simple assignment: distribute current layer agents among parent agents
                for j, agent in enumerate(current_agents):
                    parent_idx = j % len(parent_agents)
                    parent_agent = parent_agents[parent_idx]
                    
                    # Set escalation path
                    agent['escalation_path'] = parent_agent['agent_id']
                    
                    # Add to parent's subordinates
                    subordinates = json.loads(parent_agent.get('subordinates', '[]'))
                    subordinates.append(agent['agent_id'])
                    parent_agent['subordinates'] = json.dumps(subordinates)
                    
                    # Update in database
                    self.agents_db[agent['agent_id']] = agent
                    self.agents_db[parent_agent['agent_id']] = parent_agent
    
    def deploy_from_manifest(self, manifest_path: Path) -> List[Dict[str, Any]]:
        """Deploy agents from a manifest file"""
        
        with open(manifest_path, 'r') as f:
            manifest = json.load(f)
        
        deployed_agents = []
        
        for agent_spec in manifest.get('agents', []):
            agent = self.create_agent(
                name=agent_spec['name'],
                layer=agent_spec['layer'],
                capabilities=agent_spec.get('capabilities', []),
                resource_requirements=agent_spec.get('resource_requirements'),
                escalation_path=agent_spec.get('escalation_path')
            )
            deployed_agents.append(agent)
        
        logger.info(f"Deployed {len(deployed_agents)} agents from manifest")
        return deployed_agents
    
    def get_agent(self, agent_id: str) -> Optional[Dict[str, Any]]:
        """Get agent by ID"""
        return self.agents_db.get(agent_id)
    
    def list_agents(self, layer: Optional[str] = None) -> List[Dict[str, Any]]:
        """List all agents, optionally filtered by layer"""
        if layer:
            return [agent for agent in self.agents_db.values() if agent['layer'] == layer]
        return list(self.agents_db.values())
    
    def update_agent_status(self, agent_id: str, status: str):
        """Update agent status"""
        if agent_id in self.agents_db:
            self.agents_db[agent_id]['status'] = status
            self.agents_db[agent_id]['last_updated'] = datetime.now(timezone.utc).isoformat()
            logger.info(f"Updated agent {agent_id} status to {status}")
    
    def generate_deployment_manifest(self, output_path: Path):
        """Generate a deployment manifest from current agents"""
        
        manifest = {
            "version": "1.0",
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "total_agents": len(self.agents_db),
            "agents": []
        }
        
        for agent in self.agents_db.values():
            agent_spec = {
                "name": agent['name'],
                "layer": agent['layer'],
                "capabilities": json.loads(agent['capabilities']),
                "resource_requirements": json.loads(agent['resource_requirements']),
                "escalation_path": agent['escalation_path'] if agent['escalation_path'] else None
            }
            manifest["agents"].append(agent_spec)
        
        with open(output_path, 'w') as f:
            json.dump(manifest, f, indent=2)
        
        logger.info(f"Generated deployment manifest at {output_path}")

def main():
    parser = argparse.ArgumentParser(description="NOA Agent Factory")
    parser.add_argument("--config", type=Path, default=Path("config"), 
                       help="Configuration directory path")
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Create hierarchy command
    create_parser = subparsers.add_parser("create-hierarchy", help="Create agent hierarchy")
    create_parser.add_argument("--count", type=int, required=True, help="Total agent count")
    
    # Deploy from manifest command
    deploy_parser = subparsers.add_parser("deploy", help="Deploy from manifest")
    deploy_parser.add_argument("--manifest", type=Path, required=True, help="Manifest file path")
    
    # List agents command
    list_parser = subparsers.add_parser("list", help="List agents")
    list_parser.add_argument("--layer", help="Filter by layer")
    
    # Generate manifest command
    gen_parser = subparsers.add_parser("generate-manifest", help="Generate deployment manifest")
    gen_parser.add_argument("--output", type=Path, required=True, help="Output manifest path")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    factory = AgentFactory(args.config)
    
    try:
        if args.command == "create-hierarchy":
            agents = factory.create_hierarchy(args.count)
            factory.save_agent_directory()
            print(f"Created hierarchy with {len(agents)} agents")
            
        elif args.command == "deploy":
            agents = factory.deploy_from_manifest(args.manifest)
            factory.save_agent_directory()
            print(f"Deployed {len(agents)} agents from manifest")
            
        elif args.command == "list":
            agents = factory.list_agents(args.layer)
            for agent in agents:
                print(f"{agent['agent_id']}: {agent['name']} ({agent['layer']}) - {agent['status']}")
            
        elif args.command == "generate-manifest":
            factory.generate_deployment_manifest(args.output)
            print(f"Generated manifest at {args.output}")
            
    except Exception as e:
        logger.error(f"Command failed: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())