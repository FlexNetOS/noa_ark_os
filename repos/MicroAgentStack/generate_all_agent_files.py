import os
import json

ROOT = os.path.dirname(os.path.abspath(__file__))
AGENTS_DIR = os.path.join(ROOT, "agents")
MANIFEST_PATH = os.path.join(ROOT, "agent_manifest.json")

# Load manifest
with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

base_reqs = ["fastapi", "uvicorn"]

docker_template = """\
FROM python:3.10-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY main.py .
EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
"""

openapi_skeleton = """openapi: 3.0.0
info:
  title: {name} API
  description: {purpose}
  version: {version}
paths:
  /:
    get:
      summary: Status check for {name}
      responses:
        "200":
          description: OK
    post:
      summary: Run the main function of {name}
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        "200":
          description: Response from {name}
"""

readme_template = """# {name}
**Purpose:** {purpose}  
**Version:** {version}

## Functionality
{functionality}

## Usage
See `openapi.yaml` for endpoints and request examples.
"""

main_py_template = '''"""
Agent Name: {name}
Purpose: {purpose}
Version: {version}
Functionality: {functionality}
"""

from fastapi import FastAPI, Request

AGENT_NAME = "{name}"
AGENT_DESCRIPTION = "{purpose}"
AGENT_VERSION = "{version}"

app = FastAPI(
    title=AGENT_NAME + " API",
    description=AGENT_DESCRIPTION,
    version=AGENT_VERSION,
)

@app.get("/")
def root():
    return {{"agent": AGENT_NAME, "status": "running"}}

@app.post("/run")
async def run_agent(request: Request):
    data = await request.json()
    # Example: handle main agent functionality here
    return {{"agent": AGENT_NAME, "received": data}}
'''

for agent_name, meta in manifest.items():
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    os.makedirs(agent_dir, exist_ok=True)

    # main.py
    main_py = main_py_template.format(
        name=agent_name,
        purpose=meta.get("purpose", ""),
        version=meta.get("version", ""),
        functionality=meta.get("functionality", ""),
    )
    with open(os.path.join(agent_dir, "main.py"), "w", encoding="utf-8") as f:
        f.write(main_py)

    # requirements.txt
    with open(os.path.join(agent_dir, "requirements.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(base_reqs) + "\n")

    # Dockerfile
    with open(os.path.join(agent_dir, "Dockerfile"), "w", encoding="utf-8") as f:
        f.write(docker_template)

    # openapi.yaml
    openapi = openapi_skeleton.format(
        name=agent_name, purpose=meta.get("purpose", ""), version=meta.get("version", "")
    )
    with open(os.path.join(agent_dir, "openapi.yaml"), "w", encoding="utf-8") as f:
        f.write(openapi)

    # README.md (optional, but very useful!)
    readme = readme_template.format(
        name=agent_name,
        purpose=meta.get("purpose", ""),
        version=meta.get("version", ""),
        functionality=meta.get("functionality", ""),
    )
    with open(os.path.join(agent_dir, "README.md"), "w", encoding="utf-8") as f:
        f.write(readme)

    print(f"✅ Generated all files for {agent_name}")

print("🎉 Super generator complete! All agent files are now up-to-date.")


