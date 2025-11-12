"""CI/CD orchestration endpoints."""
from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Optional

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Pipeline(BaseModel):
    """Representation of a CI pipeline."""

    id: str
    branch: str
    status: str
    last_run: datetime
    coverage: float
    queued_runs: int
    health_score: float


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
    ),
    "pipeline-develop": Pipeline(
        id="pipeline-develop",
        branch="develop",
        status="failing",
        last_run=datetime.fromisoformat("2025-01-14T09:45:00"),
        coverage=87.1,
        queued_runs=3,
        health_score=0.74,
    ),
    "pipeline-experimental": Pipeline(
        id="pipeline-experimental",
        branch="feature/experimental",
        status="running",
        last_run=datetime.fromisoformat("2025-01-14T09:58:00"),
        coverage=81.2,
        queued_runs=0,
        health_score=0.88,
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
    return RerunResponse(status="queued", pipeline_id=pipeline_id)


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
