"""CI/CD orchestration endpoints."""
from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Pipeline(BaseModel):
    """Representation of a CI pipeline."""

    id: str
    branch: str
    status: str
    last_run: str


PIPELINES: Dict[str, Pipeline] = {
    "pipeline-main": Pipeline(
        id="pipeline-main",
        branch="main",
        status="passing",
        last_run="2023-11-14T10:00:00Z",
    ),
    "pipeline-develop": Pipeline(
        id="pipeline-develop",
        branch="develop",
        status="failing",
        last_run="2023-11-14T09:45:00Z",
    ),
}


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


@router.post("/pipelines/{pipeline_id}/rerun")
async def rerun_pipeline(pipeline_id: str) -> Dict[str, str]:
    """Request a pipeline rerun."""

    if pipeline_id not in PIPELINES:
        raise HTTPException(status_code=404, detail="Pipeline not found")

    await GLOBAL_EVENT_BUS.publish(
        "ci",
        {"type": "ci_rerun", "pipeline_id": pipeline_id},
    )
    return {"status": "queued", "pipeline_id": pipeline_id}
