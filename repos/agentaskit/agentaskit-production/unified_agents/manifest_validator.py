#!/usr/bin/env python3
"""
Deployment Manifest Validator
Part of the Multi-Agent AgenticAI Task Deployment Kit
"""

import json
import jsonschema
from pathlib import Path
from typing import Dict, List, Tuple, Any, Optional
import argparse
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class ManifestValidator:
    """Validates deployment manifests against schema and business rules"""
    
    def __init__(self, schema_path: Optional[Path] = None):
        self.schema_path = schema_path or Path(__file__).parent / "config" / "schema" / "deployment-manifest.schema.json"
        self.schema = self._load_schema()
    
    def _load_schema(self) -> Dict[str, Any]:
        """Load the JSON schema"""
        try:
            with open(self.schema_path, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            logger.error(f"Schema file not found: {self.schema_path}")
            raise
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON in schema file: {e}")
            raise
    
    def validate_manifest(self, manifest_path: Path) -> Tuple[bool, List[str]]:
        """Validate a deployment manifest"""
        
        try:
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
        except FileNotFoundError:
            return False, [f"Manifest file not found: {manifest_path}"]
        except json.JSONDecodeError as e:
            return False, [f"Invalid JSON in manifest: {e}"]
        
        errors = []
        
        # Schema validation
        try:
            jsonschema.validate(manifest, self.schema)
            logger.info("Schema validation passed")
        except jsonschema.ValidationError as e:
            errors.append(f"Schema validation error: {e.message}")
            # Continue with business rule validation even if schema fails
        
        # Business rule validation
        business_errors = self._validate_business_rules(manifest)
        errors.extend(business_errors)
        
        success = len(errors) == 0
        return success, errors
    
    def _validate_business_rules(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate business rules beyond JSON schema"""
        
        errors = []
        
        # Agent layer distribution validation
        layer_errors = self._validate_layer_distribution(manifest)
        errors.extend(layer_errors)
        
        # Resource allocation validation
        resource_errors = self._validate_resource_allocation(manifest)
        errors.extend(resource_errors)
        
        # Hierarchy validation
        hierarchy_errors = self._validate_hierarchy_structure(manifest)
        errors.extend(hierarchy_errors)
        
        # Naming validation
        naming_errors = self._validate_naming_conventions(manifest)
        errors.extend(naming_errors)
        
        # Configuration validation
        config_errors = self._validate_configuration_consistency(manifest)
        errors.extend(config_errors)
        
        return errors
    
    def _validate_layer_distribution(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate agent layer distribution follows design guidelines"""
        
        errors = []
        
        # Count agents by layer
        layer_counts = {
            "CECCA": 0,
            "Board": 0,
            "Executive": 0,
            "StackChief": 0,
            "Specialist": 0,
            "Micro": 0
        }
        
        # Count individual agents
        for agent in manifest.get('agents', []):
            layer = agent.get('layer')
            if layer in layer_counts:
                layer_counts[layer] += 1
        
        # Count agents from groups
        for group in manifest.get('agent_groups', []):
            template = group.get('template', {})
            layer = template.get('layer')
            count = group.get('count', 0)
            if layer in layer_counts:
                layer_counts[layer] += count
        
        total_agents = sum(layer_counts.values())
        
        # Validate layer distribution
        if layer_counts["CECCA"] == 0:
            errors.append("At least one CECCA agent is required")
        elif layer_counts["CECCA"] > 3:
            errors.append(f"Too many CECCA agents ({layer_counts['CECCA']}), maximum is 3")
        
        if layer_counts["Board"] < 1:
            errors.append("At least one Board agent is required")
        elif layer_counts["Board"] > 15:
            errors.append(f"Too many Board agents ({layer_counts['Board']}), maximum is 15")
        
        if total_agents > 0:
            # Check proportional distribution
            cecca_ratio = layer_counts["CECCA"] / total_agents
            if cecca_ratio > 0.05:  # More than 5%
                errors.append(f"CECCA agents represent {cecca_ratio:.2%} of total, should be < 5%")
            
            micro_ratio = layer_counts["Micro"] / total_agents
            if micro_ratio < 0.3 and total_agents > 20:  # Less than 30% for larger deployments
                errors.append(f"Micro agents represent {micro_ratio:.2%} of total, should be > 30% for efficient operation")
        
        logger.info(f"Layer distribution: {layer_counts} (Total: {total_agents})")
        return errors
    
    def _validate_resource_allocation(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate resource requirements are reasonable"""
        
        errors = []
        
        # Calculate total resource requirements
        total_cpu = 0.0
        total_memory = 0
        total_storage = 0
        total_bandwidth = 0
        
        def add_resources(resource_req):
            nonlocal total_cpu, total_memory, total_storage, total_bandwidth
            total_cpu += resource_req.get('cpu_cores', 0)
            total_memory += resource_req.get('memory_mb', 0)
            total_storage += resource_req.get('storage_mb', 0)
            total_bandwidth += resource_req.get('network_bandwidth_mbps', 0)
        
        # Individual agents
        for agent in manifest.get('agents', []):
            resource_req = agent.get('resource_requirements', {})
            add_resources(resource_req)
        
        # Agent groups
        for group in manifest.get('agent_groups', []):
            template = group.get('template', {})
            resource_req = template.get('resource_requirements', {})
            count = group.get('count', 0)
            
            for _ in range(count):
                add_resources(resource_req)
        
        # Validate total requirements are reasonable
        if total_cpu > 1000:  # More than 1000 CPU cores
            errors.append(f"Total CPU requirement ({total_cpu:.1f} cores) may be excessive")
        
        if total_memory > 1024 * 1024:  # More than 1TB memory
            errors.append(f"Total memory requirement ({total_memory / 1024:.1f} GB) may be excessive")
        
        if total_storage > 10 * 1024 * 1024:  # More than 10TB storage
            errors.append(f"Total storage requirement ({total_storage / 1024:.1f} GB) may be excessive")
        
        logger.info(f"Total resources: CPU={total_cpu:.1f}, Memory={total_memory/1024:.1f}GB, Storage={total_storage/1024:.1f}GB, Bandwidth={total_bandwidth}Mbps")
        return errors
    
    def _validate_hierarchy_structure(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate hierarchy relationships"""
        
        errors = []
        
        # Collect all agent names
        agent_names = set()
        
        for agent in manifest.get('agents', []):
            agent_names.add(agent.get('name'))
        
        for group in manifest.get('agent_groups', []):
            template = group.get('template', {})
            name_pattern = group.get('naming_pattern', '{name}-{index:04d}')
            base_name = template.get('name', 'Agent')
            count = group.get('count', 0)
            
            for i in range(count):
                agent_name = name_pattern.format(name=base_name, index=i+1)
                agent_names.add(agent_name)
        
        # Validate escalation paths exist
        for agent in manifest.get('agents', []):
            escalation_path = agent.get('escalation_path')
            if escalation_path and escalation_path not in agent_names:
                errors.append(f"Agent '{agent.get('name')}' has invalid escalation path: '{escalation_path}'")
        
        # Check for circular escalation paths (simplified check)
        escalation_map = {}
        for agent in manifest.get('agents', []):
            name = agent.get('name')
            escalation_path = agent.get('escalation_path')
            if escalation_path:
                escalation_map[name] = escalation_path
        
        # Simple cycle detection
        for agent_name in escalation_map:
            visited = set()
            current = agent_name
            while current in escalation_map and current not in visited:
                visited.add(current)
                current = escalation_map[current]
                if current == agent_name:
                    errors.append(f"Circular escalation path detected involving agent '{agent_name}'")
                    break
        
        return errors
    
    def _validate_naming_conventions(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate naming conventions"""
        
        errors = []
        
        # Check for duplicate agent names
        all_names = []
        
        for agent in manifest.get('agents', []):
            all_names.append(agent.get('name'))
        
        for group in manifest.get('agent_groups', []):
            template = group.get('template', {})
            name_pattern = group.get('naming_pattern', '{name}-{index:04d}')
            base_name = template.get('name', 'Agent')
            count = group.get('count', 0)
            
            for i in range(count):
                agent_name = name_pattern.format(name=base_name, index=i+1)
                all_names.append(agent_name)
        
        # Check for duplicates
        seen_names = set()
        for name in all_names:
            if name in seen_names:
                errors.append(f"Duplicate agent name: '{name}'")
            seen_names.add(name)
        
        # Validate naming patterns
        for name in all_names:
            if not name:
                errors.append("Empty agent name found")
            elif len(name) > 100:
                errors.append(f"Agent name too long: '{name}' (max 100 characters)")
            elif not name.replace('-', '').replace('_', '').replace('.', '').isalnum():
                errors.append(f"Agent name contains invalid characters: '{name}' (use alphanumeric, hyphen, underscore only)")
        
        return errors
    
    def _validate_configuration_consistency(self, manifest: Dict[str, Any]) -> List[str]:
        """Validate configuration consistency"""
        
        errors = []
        
        config = manifest.get('configuration', {})
        scaling = config.get('scaling', {})
        
        # Validate scaling configuration
        if scaling.get('auto_scale'):
            min_agents = scaling.get('min_agents')
            max_agents = scaling.get('max_agents')
            
            if min_agents and max_agents and min_agents > max_agents:
                errors.append(f"min_agents ({min_agents}) cannot be greater than max_agents ({max_agents})")
            
            scale_up = scaling.get('scale_up_threshold')
            scale_down = scaling.get('scale_down_threshold')
            
            if scale_up and scale_down and scale_down >= scale_up:
                errors.append(f"scale_down_threshold ({scale_down}) should be less than scale_up_threshold ({scale_up})")
        
        # Validate environment-specific requirements
        deployment = manifest.get('deployment', {})
        environment = deployment.get('environment')
        
        if environment == 'production':
            # Production-specific validations
            if config.get('mode') == 'interactive':
                errors.append("Interactive mode is not recommended for production deployment")
            
            security = config.get('security', {})
            if not security.get('compliance_monitoring'):
                errors.append("Compliance monitoring should be enabled for production deployment")
        
        return errors
    
    def validate_and_report(self, manifest_path: Path) -> bool:
        """Validate manifest and print detailed report"""
        
        print(f"🔍 Validating deployment manifest: {manifest_path}")
        print("=" * 80)
        
        success, errors = self.validate_manifest(manifest_path)
        
        if success:
            print("✅ Validation PASSED")
            print("\n📊 Manifest Summary:")
            
            # Load manifest for summary
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            deployment = manifest.get('deployment', {})
            print(f"   Deployment: {deployment.get('name', 'unnamed')}")
            print(f"   Environment: {deployment.get('environment', 'unknown')}")
            print(f"   Mode: {manifest.get('configuration', {}).get('mode', 'unspecified')}")
            
            # Count agents
            individual_agents = len(manifest.get('agents', []))
            group_agents = sum(group.get('count', 0) for group in manifest.get('agent_groups', []))
            total_agents = individual_agents + group_agents
            
            print(f"   Total Agents: {total_agents} ({individual_agents} individual + {group_agents} from groups)")
            
        else:
            print("❌ Validation FAILED")
            print(f"\n🐛 Found {len(errors)} error(s):")
            for i, error in enumerate(errors, 1):
                print(f"   {i}. {error}")
        
        print("\n" + "=" * 80)
        return success

def main():
    parser = argparse.ArgumentParser(description="Validate deployment manifests")
    parser.add_argument("manifest", type=Path, help="Path to deployment manifest")
    parser.add_argument("--schema", type=Path, help="Path to JSON schema file")
    parser.add_argument("--quiet", action="store_true", help="Suppress detailed output")
    
    args = parser.parse_args()
    
    try:
        validator = ManifestValidator(args.schema)
        
        if args.quiet:
            success, errors = validator.validate_manifest(args.manifest)
            if not success:
                for error in errors:
                    print(f"ERROR: {error}")
            return 0 if success else 1
        else:
            success = validator.validate_and_report(args.manifest)
            return 0 if success else 1
            
    except Exception as e:
        logger.error(f"Validation failed: {e}")
        return 1

if __name__ == "__main__":
    exit(main())