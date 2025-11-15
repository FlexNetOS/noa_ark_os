import os
import json

ROOT = os.path.dirname(os.path.abspath(__file__))
AGENTS_DIR = os.path.join(ROOT, "agents")
MANIFEST_PATH = os.path.join(ROOT, "agent_manifest.json")

base_reqs = ["fastapi", "uvicorn"]

with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

for agent_name in manifest:
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    req_path = os.path.join(agent_dir, "requirements.txt")
    # For now, always write base requirements
    with open(req_path, "w", encoding="utf-8") as f:
        f.write("\n".join(base_reqs) + "\n")
    print(f"Wrote requirements.txt for {agent_name}")
print("✅ All agents have requirements.txt")

