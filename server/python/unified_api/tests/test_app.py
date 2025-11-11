import asyncio
import pathlib
import sys

from httpx import AsyncClient

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[2]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from unified_api.app import create_app


def test_health_endpoint() -> None:
    async def _run() -> None:
        app = create_app()
        async with AsyncClient(app=app, base_url="http://test") as client:
            response = await client.get("/api/health")
        assert response.status_code == 200
        assert response.json() == {"status": "ok"}

    asyncio.run(_run())


def test_trigger_workflow() -> None:
    async def _run() -> None:
        app = create_app()
        async with AsyncClient(app=app, base_url="http://test") as client:
            response = await client.post("/api/workflows/build/trigger", json={"branch": "main"})
        assert response.status_code == 200
        payload = response.json()
        assert payload["workflow_id"] == "build"

    asyncio.run(_run())
