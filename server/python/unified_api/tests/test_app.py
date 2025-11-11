"""Integration coverage for the unified FastAPI gateway."""
from __future__ import annotations

from pathlib import Path
from typing import Dict

import pytest
from httpx import AsyncClient

PROJECT_ROOT = Path(__file__).resolve().parents[2]
import sys

if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from unified_api.app import create_app
from unified_api.routers.workflows import RUN_HISTORY


@pytest.fixture(autouse=True)
def _clear_run_history() -> None:
    """Ensure workflow state does not leak between tests."""

    RUN_HISTORY.clear()


@pytest.mark.asyncio
async def test_health_endpoint() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.get("/api/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


@pytest.mark.asyncio
async def test_trigger_workflow_records_run() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.post(
            "/api/workflows/build/trigger", json={"branch": "main"}
        )
    assert response.status_code == 200
    payload: Dict[str, object] = response.json()
    assert payload["workflow_id"] == "build"
    assert RUN_HISTORY


@pytest.mark.asyncio
async def test_analytics_metrics_available() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.get("/api/analytics/metrics")
    assert response.status_code == 200
    metrics = response.json()
    assert any(metric["id"] == "developer_productivity" for metric in metrics)


@pytest.mark.asyncio
async def test_ci_rerun_validates_pipeline() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        missing = await client.post("/api/ci/pipelines/unknown/rerun")
    assert missing.status_code == 404


@pytest.mark.asyncio
async def test_agent_scale_requires_known_agent() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.post(
            "/api/agents/builder-1/scale", json={"replicas": 2}
        )
        missing = await client.post(
            "/api/agents/unknown/scale", json={"replicas": 1}
        )
    assert response.status_code == 200
    assert missing.status_code == 404
