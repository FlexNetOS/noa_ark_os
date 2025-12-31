import os
import json

BASE_DIR = "./agents"
MANIFEST_FILE = "agent_manifest.json"
SCRIPT_NAME = "main.py"  # Or "script.py" if that's your convention

with open(MANIFEST_FILE, "r") as f:
    manifest = json.load(f)

for agent in os.listdir(BASE_DIR):
    agent_dir = os.path.join(BASE_DIR, agent)
    if os.path.isdir(agent_dir) and agent in manifest:
        script_path = os.path.join(agent_dir, SCRIPT_NAME)
        if not os.path.isfile(script_path):
            # Create a new script if missing
            with open(script_path, "w") as s:
                s.write("")

        with open(script_path, "r") as s:
            lines = s.readlines()

        # Remove existing metadata block if present
        lines = [line for line in lines if not line.startswith('"""')]

        metadata = f'''"""
Agent Name: {agent}
Purpose: {manifest[agent]["purpose"]}
Version: {manifest[agent]["version"]}
"""

'''
        # Optional: Insert/replace main function skeleton for this agent
        if "functionality" in manifest[agent]:
            metadata += manifest[agent]["functionality"] + "\n"

        # Write metadata + old script content (skip repeated metadata)
        with open(script_path, "w") as s:
            s.write(metadata)
            for line in lines:
                s.write(line)
print("Metadata injected/updated for all agents!")

