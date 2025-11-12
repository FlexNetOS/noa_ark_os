"""Analytics and ROI endpoints."""
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
}


@router.get("/metrics", response_model=List[Metric])
async def list_metrics() -> List[Metric]:
    """List high-level system metrics."""

    return list(METRICS.values())


@router.get("/roi")
async def calculate_roi() -> Dict[str, float]:
    """Return a simple ROI calculation for the dashboard spotlight."""

    productivity = METRICS["developer_productivity"].value
    infrastructure = METRICS["infrastructure_cost"].value
    if infrastructure == 0:
        return {"roi": None}
    return {"roi": round(productivity / infrastructure, 2)}
