"""Model lifecycle and inference endpoints."""

from __future__ import annotations

from datetime import datetime
from typing import Dict, List

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from ..event_bus import GLOBAL_EVENT_BUS

router = APIRouter()


class ModelDescriptor(BaseModel):
    """Metadata describing a model managed by llama.cpp."""

    id: str
    name: str
    version: str
    format: str
    size_gb: float
    status: str
    gpu_requirement: str
    loaded: bool


class GPUStatus(BaseModel):
    """GPU utilization details."""

    id: str
    name: str
    memory_total_gb: float
    memory_used_gb: float
    utilization: float
    temperature_c: float
    running_sessions: List[str]


class InferenceSession(BaseModel):
    """Active inference session metadata."""

    id: str
    model_id: str
    owner: str
    gpu_id: str
    started_at: datetime
    status: str
    tokens_generated: int

    model_config = {"protected_namespaces": ()}


class SessionCreateRequest(BaseModel):
    model_id: str
    owner: str
    gpu_id: str

    model_config = {"protected_namespaces": ()}


class SessionCreateResponse(BaseModel):
    session: InferenceSession


class SessionStopResponse(BaseModel):
    status: str
    session_id: str


MODELS: Dict[str, ModelDescriptor] = {
    "llama-2-70b": ModelDescriptor(
        id="llama-2-70b",
        name="Llama 2",
        version="70B",
        format="gguf",
        size_gb=140.0,
        status="active",
        gpu_requirement="A100 80GB",
        loaded=True,
    ),
    "noa-coder": ModelDescriptor(
        id="noa-coder",
        name="Noa Coder",
        version="12B",
        format="gguf",
        size_gb=24.0,
        status="standby",
        gpu_requirement="A40 48GB",
        loaded=False,
    ),
}

GPUS: Dict[str, GPUStatus] = {
    "gpu-a100": GPUStatus(
        id="gpu-a100",
        name="NVIDIA A100",
        memory_total_gb=80.0,
        memory_used_gb=48.0,
        utilization=0.76,
        temperature_c=65.0,
        running_sessions=["session-0001"],
    ),
    "gpu-a40": GPUStatus(
        id="gpu-a40",
        name="NVIDIA A40",
        memory_total_gb=48.0,
        memory_used_gb=12.0,
        utilization=0.32,
        temperature_c=54.0,
        running_sessions=[],
    ),
}

SESSIONS: Dict[str, InferenceSession] = {
    "session-0001": InferenceSession(
        id="session-0001",
        model_id="llama-2-70b",
        owner="release-bot",
        gpu_id="gpu-a100",
        started_at=datetime.fromisoformat("2025-01-14T06:45:00"),
        status="running",
        tokens_generated=128_000,
    )
}


@router.get("/models", response_model=List[ModelDescriptor])
async def list_models() -> List[ModelDescriptor]:
    """Return the list of managed models."""

    return list(MODELS.values())


@router.post("/models/{model_id}/deploy", response_model=ModelDescriptor)
async def deploy_model(model_id: str) -> ModelDescriptor:
    """Mark a model as deployed and available for inference."""

    model = MODELS.get(model_id)
    if not model:
        raise HTTPException(status_code=404, detail="Model not found")
    updated = model.model_copy(update={"status": "active", "loaded": True})
    MODELS[model_id] = updated
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "model_deployed",
            "model_id": model_id,
        },
    )
    return updated


@router.post("/models/{model_id}/retire", response_model=ModelDescriptor)
async def retire_model(model_id: str) -> ModelDescriptor:
    """Mark a model as retired."""

    model = MODELS.get(model_id)
    if not model:
        raise HTTPException(status_code=404, detail="Model not found")
    updated = model.model_copy(update={"status": "retired", "loaded": False})
    MODELS[model_id] = updated
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "model_retired",
            "model_id": model_id,
        },
    )
    return updated


@router.get("/gpus", response_model=List[GPUStatus])
async def gpu_status() -> List[GPUStatus]:
    """Return GPU utilization information."""

    return list(GPUS.values())


@router.get("/sessions", response_model=List[InferenceSession])
async def list_sessions() -> List[InferenceSession]:
    """Return active inference sessions."""

    return list(SESSIONS.values())


@router.post("/sessions", response_model=SessionCreateResponse)
async def create_session(request: SessionCreateRequest) -> SessionCreateResponse:
    """Create a new inference session."""

    if request.model_id not in MODELS:
        raise HTTPException(status_code=404, detail="Model not found")
    if request.gpu_id not in GPUS:
        raise HTTPException(status_code=404, detail="GPU not found")
    session_id = f"session-{len(SESSIONS) + 1:04d}"
    session = InferenceSession(
        id=session_id,
        model_id=request.model_id,
        owner=request.owner,
        gpu_id=request.gpu_id,
        started_at=datetime.utcnow(),
        status="starting",
        tokens_generated=0,
    )
    SESSIONS[session_id] = session
    gpu = GPUS[request.gpu_id]
    GPUS[request.gpu_id] = gpu.model_copy(
        update={
            "memory_used_gb": min(gpu.memory_total_gb, gpu.memory_used_gb + 8),
            "utilization": min(1.0, gpu.utilization + 0.1),
            "running_sessions": gpu.running_sessions + [session_id],
        }
    )
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "inference_session_created",
            "session_id": session_id,
            "model_id": request.model_id,
        },
    )
    return SessionCreateResponse(session=session)


@router.post("/sessions/{session_id}/stop", response_model=SessionStopResponse)
async def stop_session(session_id: str) -> SessionStopResponse:
    """Stop an inference session."""

    session = SESSIONS.get(session_id)
    if not session:
        raise HTTPException(status_code=404, detail="Session not found")
    SESSIONS[session_id] = session.model_copy(update={"status": "stopped"})
    gpu = GPUS[session.gpu_id]
    running_sessions = [sid for sid in gpu.running_sessions if sid != session_id]
    GPUS[session.gpu_id] = gpu.model_copy(
        update={
            "running_sessions": running_sessions,
            "memory_used_gb": max(0.0, gpu.memory_used_gb - 8),
            "utilization": max(0.0, gpu.utilization - 0.1),
        }
    )
    await GLOBAL_EVENT_BUS.publish(
        "shell",
        {
            "type": "inference_session_stopped",
            "session_id": session_id,
        },
    )
    return SessionStopResponse(status="stopped", session_id=session_id)
