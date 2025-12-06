import os
import json
import subprocess
import time
from typing import Dict, Any, List

# Configurable: list of manifest files and agent root dir
MANIFESTS = [
    "global_agent_manifest.json",
    "director_agent_manifest.json",
    "updated_agent_manifest.json"
]
AGENTS_DIR = "agents"

def load_manifest(path: str) -> Dict[str, Any]:
    try:
        with open(path, "r") as f:
            return json.load(f)
    except Exception as e:
        print(f"[orchestrator] Error loading manifest {path}: {e}")
        return {}

def spawn_agent(agent_name: str, args: List[str] = []):
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    script = os.path.join(agent_dir, "main.py")
    if os.path.exists(script):
        print(f"[orchestrator] Launching {agent_name} ...")
        # Optional: pass args for specific agent tasks
        subprocess.Popen(["python3", script] + args)
    else:
        print(f"[orchestrator] Script not found for {agent_name}, skipping.")

def orchestrate():
    agents_started = set()
    for manifest_path in MANIFESTS:
        if not os.path.exists(manifest_path):
            print(f"[orchestrator] Manifest not found: {manifest_path}, skipping.")
            continue
        manifest = load_manifest(manifest_path)
        for agent_name, agent_entry in manifest.get("agents", {}).items():
            if agent_name in agents_started:
                continue
            autostart = agent_entry.get("autostart", True)
            if autostart:
                spawn_agent(agent_name)
                agents_started.add(agent_name)
            else:
                print(f"[orchestrator] autostart is false for {agent_name}, skipping launch.")
    print(f"[orchestrator] All eligible agents spawned.")

def send_task_to_agent(agent_name: str, task: str, params: Dict[str, Any] = {}):
    # For future: extend with IPC, HTTP API, or socket
    print(f"[orchestrator] (Placeholder) Would send task '{task}' to {agent_name} with params {params}")

if __name__ == "__main__":
    print("[orchestrator] Loading manifests and spawning agents ...")
    orchestrate()
    # Main event loop for scheduled or on-demand tasks
    while True:
        # (Placeholder) Periodically send health check or optimize tasks
        # send_task_to_agent('PCOperatorAgent', 'cleanup_temp_files')
        time.sleep(300)  # Every 5 mins, e.g.

