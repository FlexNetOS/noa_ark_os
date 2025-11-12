"""Artifact catalog routes."""
"""Storage and artifact management endpoints."""
from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

router = APIRouter()


class Artifact(BaseModel):
    """Build artifact metadata."""

    id: str
    name: str
    size_bytes: int
    checksum: str


ARTIFACTS: List[Artifact] = [
    Artifact(id="artifact-1", name="build.tar.gz", size_bytes=245_760, checksum="abc123"),
    Artifact(id="artifact-2", name="reports.zip", size_bytes=102_400, checksum="def456"),
]


@router.get("/artifacts", response_model=List[Artifact])
async def list_artifacts() -> List[Artifact]:
    """Return all stored artifacts."""

    return ARTIFACTS


@router.get("/artifacts/{artifact_id}", response_model=Artifact)
async def get_artifact(artifact_id: str) -> Artifact:
    """Retrieve a single artifact."""

    for artifact in ARTIFACTS:
        if artifact.id == artifact_id:
            return artifact
    raise HTTPException(status_code=404, detail="artifact not found")


@router.post("/artifacts/{artifact_id}/retain")
async def retain_artifact(artifact_id: str) -> Dict[str, str]:
    """Mark an artifact for long term retention."""

    if not any(artifact.id == artifact_id for artifact in ARTIFACTS):
        raise HTTPException(status_code=404, detail="artifact not found")
    return {"status": "retained", "artifact_id": artifact_id}
