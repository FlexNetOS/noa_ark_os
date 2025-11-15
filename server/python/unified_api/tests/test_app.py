from __future__ import annotations

from typing import Dict

import asyncio

import pytest
from httpx import AsyncClient

pytest.importorskip("fastapi")

from server.python.unified_api.app import create_app
from server.python.unified_api.routers.workflows import RUN_HISTORY


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
async def test_workflow_trigger_and_override() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        trigger = await client.post(
            "/api/workflows/build/trigger", json={"payload": {"branch": "main"}}
        )
        assert trigger.status_code == 200
        run_id = trigger.json()["id"]
        override = await client.post(
            f"/api/workflows/build/runs/{run_id}/override",
            json={"stage_id": "lint", "status": "success", "notes": "Manual fix"},
        )
    assert override.status_code == 200
    payload: Dict[str, object] = override.json()
    assert payload["overrides"]
    assert payload["stages"][1]["status"] == "success"


@pytest.mark.asyncio
async def test_agent_filters_and_kill() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        listing = await client.get("/api/agents/?family=builder&status=running")
        assert listing.status_code == 200
        agents = listing.json()
        assert agents
        agent_id = agents[0]["id"]
        kill = await client.post(f"/api/agents/{agent_id}/kill")
        assert kill.status_code == 200
        stats = await client.get("/api/agents/stats")
    assert stats.status_code == 200
    data = stats.json()
    assert data["total"] >= 360


@pytest.mark.asyncio
async def test_ci_merge_and_validation_endpoints() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        merges = await client.get("/api/ci/merges")
        assert merges.status_code == 200
        merge_id = merges.json()[0]["id"]
        promote = await client.post(f"/api/ci/merges/{merge_id}/promote", json={"notes": "Deploy"})
        assert promote.status_code == 200
        validations = await client.get("/api/ci/validations")
        assert validations.status_code == 200
        validation_id = validations.json()[0]["id"]
        rerun = await client.post(f"/api/ci/validations/{validation_id}/run", json={})
    assert rerun.status_code == 200


@pytest.mark.asyncio
async def test_inference_model_and_session_flow() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        models = await client.get("/api/inference/models")
        assert models.status_code == 200
        deploy = await client.post("/api/inference/models/noa-coder/deploy")
        assert deploy.status_code == 200
        session = await client.post(
            "/api/inference/sessions",
            json={"model_id": "noa-coder", "owner": "qa", "gpu_id": "gpu-a40"},
        )
        assert session.status_code == 200
        session_id = session.json()["session"]["id"]
        stop = await client.post(f"/api/inference/sessions/{session_id}/stop")
    assert stop.status_code == 200


@pytest.mark.asyncio
async def test_storage_audit_updates_on_retain() -> None:
    app = create_app()
    async with AsyncClient(app=app, base_url="http://test") as client:
        retain = await client.post("/api/storage/artifacts/artifact-1/retain")
        assert retain.status_code == 200
        audit = await client.get("/api/storage/audit")
    assert audit.status_code == 200
    assert audit.json()[0]["action"] in {"retain", "purge"}


def test_health_endpoint_sync() -> None:
    async def _run() -> None:
        app = create_app()
        async with AsyncClient(app=app, base_url="http://test") as client:
            response = await client.get("/api/health")
        assert response.status_code == 200
        assert response.json() == {"status": "ok"}

    asyncio.run(_run())
