from agent_codegen import generate_agent
from fastapi import FastAPI, Request
from pydantic import BaseModel
import json
import os

# You can change this to agent_manifest.json if you want to test that one!
MANIFEST_PATH = "updated_agent_manifest.json"

app = FastAPI(title="Agent Orchestrator API")

def load_manifest():
    with open(MANIFEST_PATH, "r") as f:
        return json.load(f)

def save_manifest(data):
    with open(MANIFEST_PATH, "w") as f:
        json.dump(data, f, indent=2)

class AgentProposal(BaseModel):
    agent_name: str
    agent_entry: dict

class ReviewRequest(BaseModel):
    agent_name: str
    approve: bool
    reviewer: str

@app.get("/")
async def root():
    return {"status": "Agent Orchestrator running."}

@app.get("/agents")
async def get_agents():
    manifest = load_manifest()
    return manifest.get("agents", {})

@app.post("/propose_agent")
async def propose_agent(proposal: AgentProposal):
    manifest = load_manifest()
    agent_name = proposal.agent_name
    agent_entry = proposal.agent_entry
    agent_entry['approval_status'] = 'pending'
    manifest['agents'][agent_name] = agent_entry
    save_manifest(manifest)
    return {"message": f"Agent '{agent_name}' proposed and pending review."}

@app.post("/review_agent")
async def review_agent(review: ReviewRequest):
    manifest = load_manifest()
    agent = manifest['agents'].get(review.agent_name)
    if not agent:
        return {"error": f"No agent named '{review.agent_name}'."}
    agent['approval_status'] = "approved" if review.approve else "rejected"
    agent['last_updated_by'] = review.reviewer
    save_manifest(manifest)
    if review.approve:
        # Auto-generate agent code
        generate_agent(review.agent_name, agent)
        return {"message": f"Agent '{review.agent_name}' approved and scaffolded."}
    else:
        return {"message": f"Agent '{review.agent_name}' approval status set to rejected"}


@app.get("/workflows")
async def get_workflows():
    manifest = load_manifest()
    return manifest.get("workflows", {})
