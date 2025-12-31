import os
import json

ROOT = os.path.dirname(os.path.abspath(__file__))
AGENTS_DIR = os.path.join(ROOT, "agents")
MANIFEST_PATH = os.path.join(ROOT, "agent_manifest.json")

with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

for agent_name, meta in manifest.items():
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    openapi_path = os.path.join(agent_dir, "openapi.yaml")
    # Basic OpenAPI skeleton
    content = f"""openapi: 3.0.0
info:
  title: {agent_name} API
  description: {meta.get('purpose','')}
  version: {meta.get('version','')}
paths:
  /:
    get:
      summary: Status check for {agent_name}
      responses:
        "200":
          description: OK
    post:
      summary: Run the main function of {agent_name}
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        "200":
          description: Response from {agent_name}
"""
    with open(openapi_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Wrote openapi.yaml for {agent_name}")

print("✅ All agents have openapi.yaml (Swagger spec)")

