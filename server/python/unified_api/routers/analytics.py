"""Analytics and ROI endpoints."""
"""Analytics endpoints exposed to the unified dashboard."""

from __future__ import annotations

from typing import Dict, List

from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter()


class Metric(BaseModel):
    """Telemetry metric returned to the UI."""

    id: str
    label: str
    value: float
    unit: str


class InferenceStat(BaseModel):
    """Inference telemetry for model monitoring."""

    model: str
    latency_ms: float
    throughput: float
    last_updated: str


METRICS: Dict[str, Metric] = {
    "developer_productivity": Metric(
        id="developer_productivity",
        label="Developer Productivity",
        value=132.0,
        unit="story points/week",
    ),
    "infrastructure_cost": Metric(
        id="infrastructure_cost",
        label="Infrastructure Cost",
        value=44.0,
        unit="credits/week",
    ),
    "automation_coverage": Metric(
        id="automation_coverage",
        label="Automation Coverage",
        value=92.0,
        unit="percent",
    ),
}

INFERENCE: List[InferenceStat] = [
    InferenceStat(
        model="code-assistant",
        latency_ms=142.0,
        throughput=28.0,
        last_updated="2023-11-14T10:05:00Z",
    ),
    InferenceStat(
        model="deployment-critic",
        latency_ms=88.5,
        throughput=42.0,
        last_updated="2023-11-14T10:02:00Z",
    ),
]


@router.get("/metrics", response_model=List[Metric])
async def list_metrics() -> List[Metric]:
    """List high-level system metrics."""

    return list(METRICS.values())


@router.get("/roi")
async def calculate_roi() -> Dict[str, float | None]:
    """Return a simple ROI calculation for the dashboard spotlight."""

    productivity = METRICS["developer_productivity"].value
    infrastructure = METRICS["infrastructure_cost"].value
    if infrastructure == 0:
        return {"roi": None}
    return {"roi": round(productivity / infrastructure, 2)}


@router.get("/inference", response_model=List[InferenceStat])
async def list_inference_stats() -> List[InferenceStat]:
    """Expose inference telemetry for model management screens."""

    return INFERENCE
