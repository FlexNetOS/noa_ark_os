"""Workflow orchestration endpoints."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List, Optional

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class WorkflowStage(BaseModel):
    """Single stage that composes a workflow."""

    id: str
    label: str
    description: str


class WorkflowEdge(BaseModel):
    """An edge inside the workflow DAG."""

    source: str
    target: str
    kind: str = "sequential"


class Workflow(BaseModel):
    """Metadata surfaced to the dashboard."""

    id: str
    label: str
    description: str
    stages: List[WorkflowStage]
    dag: List[WorkflowEdge]
    manual_overrides: bool = True


class WorkflowRunStage(BaseModel):
    """Stage level status for an execution."""

    stage_id: str
    status: str = "pending"
    started_at: Optional[datetime] = None
    finished_at: Optional[datetime] = None
    notes: Optional[str] = None


class WorkflowRun(BaseModel):
    """Runtime information for a workflow execution."""

    id: str
    workflow_id: str
    payload: Dict[str, str] = Field(default_factory=dict)
    triggered_at: datetime
    status: str = "running"
    stages: List[WorkflowRunStage] = Field(default_factory=list)
    overrides: List[Dict[str, str]] = Field(default_factory=list)


class TriggerRequest(BaseModel):
    """Payload accepted when triggering a workflow."""

    payload: Dict[str, str] = Field(default_factory=dict)


class OverrideRequest(BaseModel):
    """Manual override payload for a workflow stage."""

    stage_id: str
    status: str = Field(pattern="^(pending|running|success|failed|skipped)$")
    notes: Optional[str] = None


class WorkflowGraph(BaseModel):
    """Unified graph response consumed by the UI."""

    workflows: List[Workflow]
    runs: List[WorkflowRun]


WORKFLOWS: Dict[str, Workflow] = {
    "build": Workflow(
        id="build",
        label="Build",
        description="Compile, lint, and test",
        stages=[
            WorkflowStage(id="source", label="Checkout", description="Sync repository"),
            WorkflowStage(id="lint", label="Lint", description="Run linting"),
            WorkflowStage(id="test", label="Test", description="Execute unit tests"),
            WorkflowStage(id="package", label="Package", description="Bundle artifacts"),
        ],
        dag=[
            WorkflowEdge(source="source", target="lint"),
            WorkflowEdge(source="lint", target="test"),
            WorkflowEdge(source="test", target="package"),
        ],
    ),
    "deploy": Workflow(
        id="deploy",
        label="Deploy",
        description="Promote artifacts to production",
        stages=[
            WorkflowStage(id="stage", label="Stage", description="Deploy to staging"),
            WorkflowStage(
                id="verify",
                label="Verify",
                description="Run smoke tests and verification",
            ),
            WorkflowStage(
                id="promote",
                label="Promote",
                description="Production rollout",
            ),
        ],
        dag=[
            WorkflowEdge(source="stage", target="verify"),
            WorkflowEdge(source="verify", target="promote"),
        ],
    ),
    "data-refresh": Workflow(
        id="data-refresh",
        label="Data Refresh",
        description="Rebuild analytics caches and features",
        stages=[
            WorkflowStage(id="snapshot", label="Snapshot", description="Snapshot production DB"),
            WorkflowStage(
                id="transform",
                label="Transform",
                description="Execute dbt transforms",
            ),
            WorkflowStage(id="publish", label="Publish", description="Publish marts"),
        ],
        dag=[
            WorkflowEdge(source="snapshot", target="transform"),
            WorkflowEdge(source="transform", target="publish"),
        ],
    ),
}

RUN_HISTORY: Dict[str, WorkflowRun] = {}


def _create_run(workflow_id: str, payload: Dict[str, str]) -> WorkflowRun:
    workflow = WORKFLOWS.get(workflow_id)
    if not workflow:
        raise HTTPException(status_code=404, detail="Workflow not found")
    run_id = f"run-{len(RUN_HISTORY) + 1:05d}"
    run = WorkflowRun(
        id=run_id,
        workflow_id=workflow_id,
        payload=payload,
        triggered_at=datetime.utcnow(),
        status="running",
        stages=[WorkflowRunStage(stage_id=stage.id) for stage in workflow.stages],
    )
    RUN_HISTORY[run.id] = run
    return run


@router.get("/", response_model=List[Workflow])
async def list_workflows() -> List[Workflow]:
    """Return the catalog of available workflows."""

    return list(WORKFLOWS.values())


@router.get("/graph", response_model=WorkflowGraph)
async def workflow_graph() -> WorkflowGraph:
    """Return workflows and runs for unified graph consumption."""

    return WorkflowGraph(
        workflows=list(WORKFLOWS.values()),
        runs=list(RUN_HISTORY.values()),
    )


@router.post("/{workflow_id}/trigger", response_model=WorkflowRun)
async def trigger_workflow(workflow_id: str, request: TriggerRequest) -> WorkflowRun:
    """Kick off a workflow and broadcast the event bus notification."""

    workflow = WORKFLOWS.get(workflow_id)
    if not workflow:
        raise HTTPException(status_code=404, detail="Workflow not found")

    run = WorkflowRun(
        id=f"run-{len(RUN_HISTORY) + 1}",
        workflow_id=workflow_id,
        payload=payload or {},
        triggered_at=datetime.utcnow(),
        status="running",
    )
    RUN_HISTORY[run.id] = run
    run = _create_run(workflow_id, request.payload)
    await GLOBAL_EVENT_BUS.publish(
        "workflows",
        {
            "type": "workflow_triggered",
            "workflow_id": workflow_id,
            "run_id": run.id,
            "stages": [stage.id for stage in workflow.stages],
        },
    )
    return run


@router.get("/runs", response_model=List[WorkflowRun])
async def list_runs() -> List[WorkflowRun]:
    """Return the recorded workflow run history."""

    return sorted(RUN_HISTORY.values(), key=lambda run: run.triggered_at, reverse=True)


@router.post("/{workflow_id}/runs/{run_id}/override", response_model=WorkflowRun)
async def override_stage(
    workflow_id: str, run_id: str, request: OverrideRequest
) -> WorkflowRun:
    """Apply a manual override to a workflow stage."""

    run = RUN_HISTORY.get(run_id)
    if not run or run.workflow_id != workflow_id:
        raise HTTPException(status_code=404, detail="Run not found")
    stage = next((stage for stage in run.stages if stage.stage_id == request.stage_id), None)
    if not stage:
        raise HTTPException(status_code=404, detail="Stage not found")
    now = datetime.utcnow()
    run.stages = [
        stage.model_copy(
            update={
                "status": request.status,
                "finished_at": now if request.status in {"failed", "success", "skipped"} else None,
                "started_at": stage.started_at or now,
                "notes": request.notes,
            }
        )
        if stage.stage_id == request.stage_id
        else stage
        for stage in run.stages
    ]
    run.overrides.append(
        {
            "stage_id": request.stage_id,
            "status": request.status,
            "notes": request.notes or "",
            "timestamp": now.isoformat(),
        }
    )
    if request.status in {"failed", "skipped"}:
        run.status = "attention"
    elif all(stage.status == "success" for stage in run.stages):
        run.status = "success"
    else:
        run.status = "running"
    RUN_HISTORY[run_id] = run
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "workflow_override",
            "workflow_id": workflow_id,
            "run_id": run_id,
            "stage_id": request.stage_id,
            "status": request.status,
        },
    )
    return run
