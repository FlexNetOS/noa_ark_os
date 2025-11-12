"""Storage and artifact management endpoints."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

router = APIRouter()


class Artifact(BaseModel):
    """Build artifact metadata."""

    id: str
    name: str
    size_bytes: int
    checksum: str
    stored_at: datetime
    retention_policy: str
    retained: bool = False
    download_url: str


class VfsNode(BaseModel):
    """Virtual file system node."""

    name: str
    kind: str
    children: List["VfsNode"] = Field(default_factory=list)


class AuditLog(BaseModel):
    """Audit log entry for storage operations."""

    id: str
    action: str
    actor: str
    timestamp: datetime
    artifact_id: str
    notes: str


VfsNode.model_rebuild()


ARTIFACTS: Dict[str, Artifact] = {
    "artifact-1": Artifact(
        id="artifact-1",
        name="build.tar.gz",
        size_bytes=245_760,
        checksum="abc123",
        stored_at=datetime.fromisoformat("2025-01-14T07:30:00"),
        retention_policy="30d",
        retained=False,
        download_url="https://storage.local/artifacts/build.tar.gz",
    ),
    "artifact-2": Artifact(
        id="artifact-2",
        name="reports.zip",
        size_bytes=102_400,
        checksum="def456",
        stored_at=datetime.fromisoformat("2025-01-13T19:15:00"),
        retention_policy="90d",
        retained=True,
        download_url="https://storage.local/artifacts/reports.zip",
    ),
}

VFS_PLAN: List[VfsNode] = [
    VfsNode(
        name="artifacts",
        kind="directory",
        children=[
            VfsNode(name="builds", kind="directory", children=[]),
            VfsNode(name="reports", kind="directory", children=[]),
        ],
    ),
    VfsNode(
        name="databases",
        kind="directory",
        children=[
            VfsNode(name="analytics.sqlite", kind="file"),
            VfsNode(name="vector_store.parquet", kind="file"),
        ],
    ),
]

AUDIT_LOGS: List[AuditLog] = [
    AuditLog(
        id="audit-1",
        action="download",
        actor="ci-bot",
        timestamp=datetime.fromisoformat("2025-01-14T08:05:00"),
        artifact_id="artifact-2",
        notes="metrics snapshot for release",
    ),
    AuditLog(
        id="audit-2",
        action="retain",
        actor="release-manager",
        timestamp=datetime.fromisoformat("2025-01-13T17:20:00"),
        artifact_id="artifact-2",
        notes="long-term compliance hold",
    ),
]


@router.get("/plan", response_model=List[VfsNode])
async def storage_plan() -> List[VfsNode]:
    """Return the virtual file system plan."""

    return VFS_PLAN


@router.get("/artifacts", response_model=List[Artifact])
async def list_artifacts() -> List[Artifact]:
    """Return all stored artifacts."""

    return list(ARTIFACTS.values())


@router.get("/artifacts/{artifact_id}", response_model=Artifact)
async def get_artifact(artifact_id: str) -> Artifact:
    """Retrieve a single artifact."""

    artifact = ARTIFACTS.get(artifact_id)
    if not artifact:
        raise HTTPException(status_code=404, detail="artifact not found")
    return artifact


@router.post("/artifacts/{artifact_id}/retain", response_model=Artifact)
async def retain_artifact(artifact_id: str) -> Artifact:
    """Mark an artifact for long term retention."""

    artifact = ARTIFACTS.get(artifact_id)
    if not artifact:
        raise HTTPException(status_code=404, detail="artifact not found")
    updated = artifact.model_copy(update={"retained": True})
    ARTIFACTS[artifact_id] = updated
    AUDIT_LOGS.append(
        AuditLog(
            id=f"audit-{len(AUDIT_LOGS) + 1}",
            action="retain",
            actor="dashboard",
            timestamp=datetime.utcnow(),
            artifact_id=artifact_id,
            notes="Retention requested via control plane",
        )
    )
    return updated


@router.post("/artifacts/{artifact_id}/purge", response_model=Artifact)
async def purge_artifact(artifact_id: str) -> Artifact:
    """Release the retention hold on an artifact."""

    artifact = ARTIFACTS.get(artifact_id)
    if not artifact:
        raise HTTPException(status_code=404, detail="artifact not found")
    updated = artifact.model_copy(update={"retained": False})
    ARTIFACTS[artifact_id] = updated
    AUDIT_LOGS.append(
        AuditLog(
            id=f"audit-{len(AUDIT_LOGS) + 1}",
            action="purge",
            actor="dashboard",
            timestamp=datetime.utcnow(),
            artifact_id=artifact_id,
            notes="Retention released via control plane",
        )
    )
    return updated


@router.get("/audit", response_model=List[AuditLog])
async def audit_logs() -> List[AuditLog]:
    """Return storage audit logs."""

    return sorted(AUDIT_LOGS, key=lambda entry: entry.timestamp, reverse=True)
