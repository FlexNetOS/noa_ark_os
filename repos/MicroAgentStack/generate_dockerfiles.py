import os
import json

ROOT = os.path.dirname(os.path.abspath(__file__))
AGENTS_DIR = os.path.join(ROOT, "agents")
MANIFEST_PATH = os.path.join(ROOT, "agent_manifest.json")

with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

docker_template = """\
FROM python:3.10-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY main.py .
EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
"""

for agent_name in manifest:
    agent_dir = os.path.join(AGENTS_DIR, agent_name)
    dockerfile_path = os.path.join(agent_dir, "Dockerfile")
    with open(dockerfile_path, "w", encoding="utf-8") as f:
        f.write(docker_template)
    print(f"Wrote Dockerfile for {agent_name}")
print("✅ All agents have Dockerfile")

