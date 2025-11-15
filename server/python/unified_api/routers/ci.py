"""CI/CD orchestration endpoints."""
from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Optional

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class AgentApprovalRequirement(BaseModel):
    """Agent role and trust requirement."""

    role: str
    minimum_trust_score: float
    evidence_tags: List[str] = Field(default_factory=list)


class AgentApproval(BaseModel):
    """Recorded approval from an automation agent."""

    role: str
    agent_id: str
    trust_score: float
    evidence_tags: List[str] = Field(default_factory=list)
    evidence_references: List[str] = Field(default_factory=list)
    recorded_at: datetime


class AgentApprovalRequest(BaseModel):
    """Request payload for approving a pipeline."""

    role: str
    agent_id: str
    trust_score: float
    evidence_tags: List[str] = Field(default_factory=list)
    evidence_references: List[str] = Field(default_factory=list)


class Pipeline(BaseModel):
    """Representation of a CI pipeline."""

    id: str
    branch: str
    status: str
    last_run: datetime
    coverage: float
    queued_runs: int
    health_score: float
    agent_requirements: List[AgentApprovalRequirement] = Field(default_factory=list)
    agent_approvals: List[AgentApproval] = Field(default_factory=list)


class MergePlan(BaseModel):
    """A/B/C -> D sandbox merge plan."""

    id: str
    sources: List[str]
    target: str
    status: str
    approvals: int
    required_approvals: int
    latest_validation: str


class Validation(BaseModel):
    """Validation job for a merge plan."""

    id: str
    merge_id: str
    description: str
    status: str
    report_url: str


class RolloutMetric(BaseModel):
    """Rollout telemetry for blue/green and canary deployments."""

    strategy: str
    environment: str
    success_rate: float
    error_rate: float
    active: bool


PIPELINES: Dict[str, Pipeline] = {
    "pipeline-main": Pipeline(
        id="pipeline-main",
        branch="main",
        status="passing",
        last_run=datetime.fromisoformat("2025-01-14T10:00:00"),
        coverage=92.4,
        queued_runs=1,
        health_score=0.98,
        agent_requirements=[],
        agent_approvals=[],
    ),
    "pipeline-develop": Pipeline(
        id="pipeline-develop",
        branch="develop",
        status="agent-review",
        last_run=datetime.fromisoformat("2025-01-14T09:45:00"),
        coverage=87.1,
        queued_runs=3,
        health_score=0.74,
        agent_requirements=[
            AgentApprovalRequirement(
                role="release-agent",
                minimum_trust_score=0.8,
                evidence_tags=["ledger:deploy"],
            )
        ],
        agent_approvals=[],
    ),
    "pipeline-experimental": Pipeline(
        id="pipeline-experimental",
        branch="feature/experimental",
        status="running",
        last_run=datetime.fromisoformat("2025-01-14T09:58:00"),
        coverage=81.2,
        queued_runs=0,
        health_score=0.88,
        agent_requirements=[],
        agent_approvals=[],
    ),
}

MERGES: Dict[str, MergePlan] = {
    "merge-abd": MergePlan(
        id="merge-abd",
        sources=["A", "B"],
        target="D",
        status="pending",
        approvals=1,
        required_approvals=2,
        latest_validation="validation-abd-1",
    ),
    "merge-cd": MergePlan(
        id="merge-cd",
        sources=["C"],
        target="D",
        status="ready",
        approvals=2,
        required_approvals=2,
        latest_validation="validation-cd-1",
    ),
}

VALIDATIONS: Dict[str, Validation] = {
    "validation-abd-1": Validation(
        id="validation-abd-1",
        merge_id="merge-abd",
        description="Integration tests against sandbox AB",
        status="running",
        report_url="https://ci.local/reports/abd",
    ),
    "validation-cd-1": Validation(
        id="validation-cd-1",
        merge_id="merge-cd",
        description="Canary verification on staging",
        status="passed",
        report_url="https://ci.local/reports/cd",
    ),
}

ROLLOUTS: List[RolloutMetric] = [
    RolloutMetric(
        strategy="blue-green",
        environment="production",
        success_rate=0.997,
        error_rate=0.001,
        active=True,
    ),
    RolloutMetric(
        strategy="canary",
        environment="staging",
        success_rate=0.982,
        error_rate=0.004,
        active=False,
    ),
]


class RerunResponse(BaseModel):
    status: str
    pipeline_id: str


class PromoteRequest(BaseModel):
    notes: str = Field(default="")


class PromoteResponse(BaseModel):
    status: str
    merge_id: str


class ValidationRunRequest(BaseModel):
    description: Optional[str] = None


class ValidationRunResponse(BaseModel):
    status: str
    validation_id: str


@router.get("/pipelines", response_model=List[Pipeline])
async def list_pipelines() -> List[Pipeline]:
    """Return all tracked pipelines."""

    return list(PIPELINES.values())


@router.get("/pipelines/{pipeline_id}", response_model=Pipeline)
async def get_pipeline(pipeline_id: str) -> Pipeline:
    """Return a single pipeline summary."""

    pipeline = PIPELINES.get(pipeline_id)
    if not pipeline:
        raise HTTPException(status_code=404, detail="Pipeline not found")
    return pipeline


@router.post("/pipelines/{pipeline_id}/rerun", response_model=RerunResponse)
async def rerun_pipeline(pipeline_id: str) -> RerunResponse:
    """Request a pipeline rerun."""

    if pipeline_id not in PIPELINES:
        raise HTTPException(status_code=404, detail="Pipeline not found")
    PIPELINES[pipeline_id] = PIPELINES[pipeline_id].model_copy(
        update={"status": "queued", "queued_runs": PIPELINES[pipeline_id].queued_runs + 1}
    )
    await GLOBAL_EVENT_BUS.publish(
        "ci",
        {"type": "ci_rerun", "pipeline_id": pipeline_id},
    )
    return {"status": "queued", "pipeline_id": pipeline_id}


@router.post("/pipelines/{pipeline_id}/approve", response_model=Pipeline)
async def approve_pipeline_agent(
    pipeline_id: str, request: AgentApprovalRequest
) -> Pipeline:
    """Approve a pipeline via agent trust policies."""

    pipeline = PIPELINES.get(pipeline_id)
    if pipeline is None:
        raise HTTPException(status_code=404, detail="Pipeline not found")

    requirement = next(
        (req for req in pipeline.agent_requirements if req.role == request.role),
        None,
    )
    if requirement is None:
        raise HTTPException(status_code=400, detail="Agent role not required")

    missing_tags = [
        tag for tag in requirement.evidence_tags if tag not in request.evidence_tags
    ]
    if request.trust_score < requirement.minimum_trust_score or missing_tags:
        PIPELINES[pipeline_id] = pipeline.model_copy(update={"status": "agent-escalated"})
        reason = (
            "trust score below requirement"
            if request.trust_score < requirement.minimum_trust_score
            else f"missing evidence tags: {', '.join(missing_tags)}"
        )
        raise HTTPException(status_code=409, detail=reason)

    approvals = [
        approval
        for approval in pipeline.agent_approvals
        if not (approval.role == request.role and approval.agent_id == request.agent_id)
    ]
    approvals.append(
        AgentApproval(
            role=request.role,
            agent_id=request.agent_id,
            trust_score=request.trust_score,
            evidence_tags=request.evidence_tags,
            evidence_references=request.evidence_references,
            recorded_at=datetime.utcnow(),
        )
    )
    outstanding = [
        req.role
        for req in pipeline.agent_requirements
        if not any(
            approval.role == req.role
            and approval.trust_score >= req.minimum_trust_score
            and all(tag in approval.evidence_tags for tag in req.evidence_tags)
            for approval in approvals
        )
    ]
    new_status = "agent-approved" if not outstanding else "agent-review"
    updated = pipeline.model_copy(
        update={"agent_approvals": approvals, "status": new_status}
    )
    PIPELINES[pipeline_id] = updated
    return updated


@router.get("/merges", response_model=List[MergePlan])
async def list_merges() -> List[MergePlan]:
    """Return active merge plans."""

    return list(MERGES.values())


@router.post("/merges/{merge_id}/promote", response_model=PromoteResponse)
async def promote_merge(merge_id: str, request: PromoteRequest) -> PromoteResponse:
    """Promote a merge plan into the target environment."""

    merge = MERGES.get(merge_id)
    if not merge:
        raise HTTPException(status_code=404, detail="Merge plan not found")
    MERGES[merge_id] = merge.model_copy(update={"status": "promoting"})
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "merge_promoted",
            "merge_id": merge_id,
            "notes": request.notes,
        },
    )
    return PromoteResponse(status="promoting", merge_id=merge_id)


@router.get("/validations", response_model=List[Validation])
async def list_validations() -> List[Validation]:
    """Return validations that belong to merge plans."""

    return list(VALIDATIONS.values())


@router.post("/validations/{validation_id}/run", response_model=ValidationRunResponse)
async def run_validation(
    validation_id: str, request: ValidationRunRequest
) -> ValidationRunResponse:
    """Re-run a specific validation."""

    validation = VALIDATIONS.get(validation_id)
    if not validation:
        raise HTTPException(status_code=404, detail="Validation not found")
    VALIDATIONS[validation_id] = validation.model_copy(update={"status": "running"})
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "validation_requested",
            "validation_id": validation_id,
            "description": request.description or validation.description,
        },
    )
    return ValidationRunResponse(status="running", validation_id=validation_id)


@router.get("/rollouts", response_model=List[RolloutMetric])
async def rollout_metrics() -> List[RolloutMetric]:
    """Return rollout telemetry."""

    return ROLLOUTS
