#!/usr/bin/env python3
"""
AgentAsKit System Integrity Hash Generator
Generates and verifies system-wide integrity hashes for the unified system
"""

import hashlib
import os
import json
from datetime import datetime
from pathlib import Path

def generate_file_hash(filepath):
    """Generate SHA256 hash for a file"""
    hash_sha256 = hashlib.sha256()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_sha256.update(chunk)
    return hash_sha256.hexdigest()

def generate_system_manifest():
    """Generate complete system integrity manifest"""
    manifest = {
        "timestamp": datetime.now().isoformat(),
        "system": "AgentAsKit-Unified",
        "version": "1.0.0-unified",
        "components": {}
    }
    
    base_path = Path(".")
    
    # Hash all unified components
    components = [
        "unified_tools",
        "unified_execution", 
        "unified_orchestration",
        "unified_agents",
        "unified_docs",
        "operational_scripts",
        "operational_logs",
        "operational_audit"
    ]
    
    for component in components:
        comp_path = base_path / component
        if comp_path.exists():
            manifest["components"][component] = {}
            for root, dirs, files in os.walk(comp_path):
                for file in files:
                    file_path = Path(root) / file
                    rel_path = file_path.relative_to(base_path)
                    manifest["components"][component][str(rel_path)] = generate_file_hash(file_path)
    
    return manifest

if __name__ == "__main__":
    manifest = generate_system_manifest()
    
    # Save manifest
    with open("system_integrity.json", "w") as f:
        json.dump(manifest, f, indent=2)
    
    # Generate overall system hash
    manifest_str = json.dumps(manifest, sort_keys=True)
    system_hash = hashlib.sha256(manifest_str.encode()).hexdigest()
    
    print(f"System Integrity Hash: {system_hash}")
    print(f"Manifest saved to: system_integrity.json")
    print(f"Components verified: {len(manifest['components'])}")