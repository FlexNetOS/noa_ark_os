"""Agent registry endpoints."""
from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Agent(BaseModel):
    """Summary for a running agent."""

    id: str
    role: str
    status: str
    load: float


class ScaleRequest(BaseModel):
    """Desired replica count for an agent family."""

    replicas: int = Field(..., ge=0, le=32)


class ScaleResponse(BaseModel):
    """Acknowledgement returned after a scale request."""

    status: str
    agent_id: str
    replicas: int


AGENTS: Dict[str, Agent] = {
    "builder-1": Agent(id="builder-1", role="builder", status="running", load=0.42),
    "qa-1": Agent(id="qa-1", role="qa", status="running", load=0.38),
    "ops-1": Agent(id="ops-1", role="ops", status="idle", load=0.12),
}


@router.get("/", response_model=List[Agent])
async def list_agents() -> List[Agent]:
    """Return the active agent registry."""

    return list(AGENTS.values())


@router.get("/{agent_id}", response_model=Agent)
async def get_agent(agent_id: str) -> Agent:
    """Retrieve a single agent."""

    agent = AGENTS.get(agent_id)
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    return agent


@router.post("/{agent_id}/scale", response_model=ScaleResponse)
async def scale_agent(agent_id: str, request: ScaleRequest) -> ScaleResponse:
    """Adjust the desired replica count and notify listeners."""

    if agent_id not in AGENTS:
        raise HTTPException(status_code=404, detail="Agent not found")

    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "agent_scaled",
            "agent_id": agent_id,
            "replicas": request.replicas,
        },
    )
    return ScaleResponse(status="accepted", agent_id=agent_id, replicas=request.replicas)
