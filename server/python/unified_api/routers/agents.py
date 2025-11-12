"""Agent management endpoints."""
from __future__ import annotations

from collections import Counter
from itertools import cycle
from typing import Dict, Iterable, List, Optional

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Agent(BaseModel):
    """Summary for a running agent."""

    id: str
    family: str
    role: str
    status: str
    load: float
    tasks_active: int
    uptime_seconds: int
    region: str
    generation: int
    parent: Optional[str] = None


class AgentStats(BaseModel):
    """High level statistics for the agent hive."""

    total: int
    by_status: Dict[str, int]
    by_family: Dict[str, int]
    average_load: float


class ScaleRequest(BaseModel):
    """Desired replica count for an agent family."""

    replicas: int = Field(..., ge=0, le=128)


class ScaleResponse(BaseModel):
    """Acknowledgement returned after a scale request."""

    status: str
    agent_id: str
    replicas: int


class AgentFactoryRequest(BaseModel):
    """Request payload for spawning a new agent."""

    role: str
    goal: str
    capabilities: List[str] = Field(default_factory=list)


class AgentFactoryResponse(BaseModel):
    """Response returned after spawning an agent."""

    status: str
    agent: Agent


AGENTS: Dict[str, Agent] = {
    "builder-1": Agent(id="builder-1", role="builder", status="running", load=0.42),
    "qa-1": Agent(id="qa-1", role="qa", status="running", load=0.38),
    "ops-1": Agent(id="ops-1", role="ops", status="idle", load=0.12),
}
class KillResponse(BaseModel):
    """Response returned after terminating an agent."""

    status: str
    agent_id: str


_REGIONS = ("us-east", "us-west", "eu-central")
_FAMILIES = (
    ("orchestrator", "orchestration"),
    ("builder", "development"),
    ("qa", "quality"),
    ("ops", "operations"),
    ("analysis", "analytics"),
    ("security", "security"),
)
_STATUSES = ("running", "idle", "degraded")


def _generate_agents() -> Dict[str, Agent]:
    """Create a deterministic 300+ agent hive for the dashboard."""

    agents: Dict[str, Agent] = {}
    status_cycle = cycle(_STATUSES)
    region_cycle = cycle(_REGIONS)
    generation_cycle = cycle(range(1, 5))
    identifier = 1
    for family, role in _FAMILIES:
        for shard in range(1, 61):  # 6 families * 60 = 360 agents
            agent_id = f"{family}-{shard}"
            status = next(status_cycle)
            region = next(region_cycle)
            generation = next(generation_cycle)
            load = round(((shard % 10) + (generation * 3)) / 20.0, 2)
            agents[agent_id] = Agent(
                id=agent_id,
                family=family,
                role=role,
                status=status,
                load=load,
                tasks_active=(shard * generation) % 7,
                uptime_seconds=36_000 + shard * 120,
                region=region,
                generation=generation,
                parent=f"{family}-controller" if shard > 1 else None,
            )
            identifier += 1
    # Add controller parents to complete the hierarchy
    for family, role in _FAMILIES:
        controller_id = f"{family}-controller"
        agents[controller_id] = Agent(
            id=controller_id,
            family=family,
            role=f"{role}-controller",
            status="running",
            load=0.35,
            tasks_active=2,
            uptime_seconds=240_000,
            region="us-east",
            generation=0,
            parent=None,
        )
    return agents


AGENTS: Dict[str, Agent] = _generate_agents()
SCALE_TARGETS: Dict[str, int] = {agent_id: 1 for agent_id in AGENTS}


def _apply_filters(
    agents: Iterable[Agent],
    *,
    family: Optional[str],
    status: Optional[str],
    region: Optional[str],
) -> List[Agent]:
    filtered = []
    for agent in agents:
        if family and agent.family != family:
            continue
        if status and agent.status != status:
            continue
        if region and agent.region != region:
            continue
        filtered.append(agent)
    return filtered


@router.get("/", response_model=List[Agent])
async def list_agents(
    family: Optional[str] = Query(default=None),
    status: Optional[str] = Query(default=None),
    region: Optional[str] = Query(default=None),
) -> List[Agent]:
    """Return the active agent registry."""

    return _apply_filters(AGENTS.values(), family=family, status=status, region=region)


@router.get("/stats", response_model=AgentStats)
async def agent_stats() -> AgentStats:
    """Return roll-up statistics for the hive explorer."""

    snapshot = list(AGENTS.values())
    total = len(snapshot)
    by_status = Counter(agent.status for agent in snapshot)
    by_family = Counter(agent.family for agent in snapshot)
    average_load = round(sum(agent.load for agent in snapshot) / total, 3)
    return AgentStats(
        total=total,
        by_status=dict(by_status),
        by_family=dict(by_family),
        average_load=average_load,
    )


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
    SCALE_TARGETS[agent_id] = request.replicas
    await GLOBAL_EVENT_BUS.publish(
        "agents",
        {
            "type": "agent_scaled",
            "agent_id": agent_id,
            "replicas": request.replicas,
        },
    )
    return ScaleResponse(status="accepted", agent_id=agent_id, replicas=request.replicas)


@router.post("/factory/spawn", response_model=AgentFactoryResponse)
async def spawn_agent(request: AgentFactoryRequest) -> AgentFactoryResponse:
    """Create a new agent instance and broadcast its arrival."""

    agent_id = f"{request.role}-{len(AGENTS) + 1}"
    agent = Agent(id=agent_id, role=request.role, status="starting", load=0.0)
    AGENTS[agent_id] = agent

    await GLOBAL_EVENT_BUS.publish(
        "agents",
        {
            "type": "agent_spawned",
            "agent_id": agent_id,
            "role": request.role,
            "goal": request.goal,
            "capabilities": request.capabilities,
        },
    )

    return AgentFactoryResponse(status="accepted", agent=agent)
@router.post("/{agent_id}/kill", response_model=KillResponse)
async def kill_agent(agent_id: str) -> KillResponse:
    """Terminate an agent and broadcast the event."""

    agent = AGENTS.get(agent_id)
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    AGENTS[agent_id] = agent.model_copy(update={"status": "terminated", "load": 0.0, "tasks_active": 0})
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "agent_killed",
            "agent_id": agent_id,
        },
    )
    return KillResponse(status="terminated", agent_id=agent_id)
