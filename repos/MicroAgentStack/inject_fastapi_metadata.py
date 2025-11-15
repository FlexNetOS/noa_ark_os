import os
import json

ROOT = os.path.dirname(os.path.abspath(__file__))
AGENTS_DIR = os.path.join(ROOT, "agents")
MANIFEST_PATH = os.path.join(ROOT, "agent_manifest.json")

# Load manifest
with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

for agent_name, meta in manifest.items():
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    main_path = os.path.join(agent_dir, "main.py")
    if not os.path.exists(main_path):
        print(f"Skipping {agent_name}: main.py not found.")
        continue

    # Build metadata block with functionality field
    block = f'''"""
Agent Name: {agent_name}
Purpose: {meta.get("purpose", "")}
Version: {meta.get("version", "")}
Functionality:
{meta.get("functionality", "").strip()}
"""

from fastapi import FastAPI, Request

AGENT_NAME = "{agent_name}"
AGENT_DESCRIPTION = "{meta.get("purpose", "")}"
AGENT_VERSION = "{meta.get("version", "")}"

app = FastAPI(
    title=f"{{AGENT_NAME}} API",
    description=AGENT_DESCRIPTION,
    version=AGENT_VERSION,
)
'''

    # Read the rest of the file after first FastAPI instance (if any)
    with open(main_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    start_index = None
    for idx, line in enumerate(lines):
        if "app = FastAPI" in line:
            start_index = idx
            break

    if start_index is not None:
        rest_of_code = "".join(lines[start_index + 1:])
    else:
        rest_of_code = "".join(lines)

    # Write the new main.py with updated metadata and functionality
    with open(main_path, "w", encoding="utf-8") as f:
        f.write(block)
        f.write("\n")
        f.write(rest_of_code.lstrip())

    print(f"Updated {agent_name}/main.py")

print("✅ All agents have up-to-date FastAPI metadata blocks (incl. Functionality field).")



