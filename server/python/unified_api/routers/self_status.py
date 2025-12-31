"""Self status endpoint exposing drift, hot paths, and budget offenders."""

from __future__ import annotations

from fastapi import APIRouter

from ...autonomy.self_status import SelfStatusAggregator

router = APIRouter()
_aggregator = SelfStatusAggregator()


@router.get("/v1/self/status")
async def get_self_status() -> dict:
    status = _aggregator.collect()
    return {
        "generated_at": status.generated_at,
        "drift": [alert.__dict__ for alert in status.drift],
        "hot_paths": [path.__dict__ for path in status.hot_paths],
        "budget_offenders": [offender.__dict__ for offender in status.budget_offenders],
        "telemetry": status.telemetry,
    }
