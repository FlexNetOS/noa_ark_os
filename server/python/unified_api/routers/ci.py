"""CI/CD control surface."""
from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter
from pydantic import BaseModel

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class Pipeline(BaseModel):
    id: str
    branch: str
    status: str
    last_run: str


PIPELINES: List[Pipeline] = [
    Pipeline(id="pipeline-main", branch="main", status="passing", last_run="2023-11-14T10:00:00Z"),
    Pipeline(id="pipeline-develop", branch="develop", status="failing", last_run="2023-11-14T09:45:00Z"),
]


@router.get("/pipelines", response_model=List[Pipeline])
async def list_pipelines() -> List[Pipeline]:
    return PIPELINES


@router.post("/pipelines/{pipeline_id}/rerun")
async def rerun_pipeline(pipeline_id: str) -> Dict[str, str]:
    await GLOBAL_EVENT_BUS.publish(
        "shell", {"type": "ci_rerun", "pipeline_id": pipeline_id}
    )
    return {"status": "queued"}
