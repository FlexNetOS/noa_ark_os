"""Agent orchestration endpoints."""
from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Agent(BaseModel):
    id: str
    role: str
    status: str
    load: float


AGENTS: Dict[str, Agent] = {
    "builder-1": Agent(id="builder-1", role="builder", status="running", load=0.42),
    "qa-1": Agent(id="qa-1", role="qa", status="running", load=0.38),
    "ops-1": Agent(id="ops-1", role="ops", status="idle", load=0.12),
}


@router.get("/", response_model=List[Agent])
async def list_agents() -> List[Agent]:
    return list(AGENTS.values())


@router.post("/{agent_id}/scale")
async def scale_agent(agent_id: str, replicas: int) -> Dict[str, str]:
    agent = AGENTS.get(agent_id)
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")

    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {"type": "agent_scaled", "agent_id": agent_id, "replicas": replicas},
    )
    return {"status": "accepted"}
