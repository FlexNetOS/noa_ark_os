"""Workflow orchestration endpoints."""
from __future__ import annotations

from datetime import datetime
from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class WorkflowStage(BaseModel):
    id: str
    label: str
    description: str


class Workflow(BaseModel):
    id: str
    label: str
    description: str
    stages: List[WorkflowStage]


class WorkflowRun(BaseModel):
    id: str
    workflow_id: str
    payload: Dict[str, str] = Field(default_factory=dict)
    triggered_at: datetime
    status: str = "pending"


WORKFLOWS: Dict[str, Workflow] = {
    "build": Workflow(
        id="build",
        label="Build",
        description="Compile, lint, and test",
        stages=[
            WorkflowStage(id="lint", label="Lint", description="Run linting"),
            WorkflowStage(id="test", label="Test", description="Run unit tests"),
        ],
    ),
    "deploy": Workflow(
        id="deploy",
        label="Deploy",
        description="Promote artifacts",
        stages=[
            WorkflowStage(id="stage", label="Stage", description="Stage environment"),
            WorkflowStage(id="promote", label="Promote", description="Production deploy"),
        ],
    ),
}

RUN_HISTORY: Dict[str, WorkflowRun] = {}


@router.get("/", response_model=List[Workflow])
async def list_workflows() -> List[Workflow]:
    return list(WORKFLOWS.values())


@router.post("/{workflow_id}/trigger", response_model=WorkflowRun)
async def trigger_workflow(workflow_id: str, payload: Dict[str, str] | None = None) -> WorkflowRun:
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
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "workflow_triggered",
            "workflow_id": workflow_id,
            "run_id": run.id,
        },
    )
    return run


@router.get("/runs", response_model=List[WorkflowRun])
async def list_runs() -> List[WorkflowRun]:
    return list(RUN_HISTORY.values())
