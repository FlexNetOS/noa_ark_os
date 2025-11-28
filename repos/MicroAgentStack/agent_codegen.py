
import os
import json

def generate_all_agents(manifest_path="updated_agent_manifest.json", agents_dir="agents"):
    with open(manifest_path, "r") as f:
        manifest = json.load(f)
    for agent_name, agent_entry in manifest["agents"].items():
        if agent_entry.get("approval_status") == "approved":
            generate_agent(agent_name, agent_entry, agents_dir=agents_dir)

def generate_agent(agent_name, agent_entry, agents_dir="agents"):
    agent_dir = os.path.join(agents_dir, agent_name)
    os.makedirs(agent_dir, exist_ok=True)
    # Generate main.py
    with open(os.path.join(agent_dir, "main.py"), "w") as f:
        f.write(f"# Auto-generated for {agent_name}\n")
        f.write(f"# Purpose: {agent_entry.get('purpose', '')}\n\n")
        f.write(agent_entry.get("functionality", "# Add implementation here"))
        f.write("\n")
    # Optionally: Generate README.md
    with open(os.path.join(agent_dir, "README.md"), "w") as f:
        f.write(f"# {agent_name}\n\n")
        f.write(f"**Purpose:** {agent_entry.get('purpose', '')}\n")
    # You can add more files (requirements.txt, Dockerfile, etc.) here!
    return agent_dir
