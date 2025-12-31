from __future__ import annotations

import importlib
import json
from pathlib import Path

import pytest

pytest.importorskip("fastapi")

from fastapi.testclient import TestClient

from server.python.autonomy.self_status import SelfStatusAggregator
from server.python.unified_api.app import create_app


@pytest.fixture()
def telemetry_root(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    (tmp_path / "cicd/ml/reports").mkdir(parents=True)
    (tmp_path / "storage/db/analytics").mkdir(parents=True)
    (tmp_path / "storage/telemetry").mkdir(parents=True)

    drift_entry = {
        "artifact": "model-A",
        "severity": "high",
        "issues": ["accuracy drop"],
        "timestamp": "2024-05-01T00:00:00Z",
    }
    (tmp_path / "cicd/ml/reports/drift_report.jsonl").write_text(
        json.dumps(drift_entry) + "\n",
        encoding="utf-8",
    )

    goal_metrics = [
        {
            "goal_id": "goal-1",
            "workflow_id": "wf-alpha",
            "total_runs": 10,
            "successful_runs": 8,
            "average_lead_time_ms": 45_000,
            "success_rate": 0.8,
        },
        {
            "goal_id": "goal-2",
            "workflow_id": "wf-beta",
            "total_runs": 2,
            "successful_runs": 1,
            "average_lead_time_ms": 150_000,
            "success_rate": 0.5,
        },
    ]
    (tmp_path / "storage/db/analytics/goal_kpis.json").write_text(
        json.dumps(goal_metrics), encoding="utf-8"
    )

    gateway_metrics = {"total_requests": 4, "per_protocol": {"GraphQl": 3}}
    (tmp_path / "storage/telemetry/gateway_metrics.json").write_text(
        json.dumps(gateway_metrics), encoding="utf-8"
    )

    monkeypatch.setenv("NOA_ROOT", str(tmp_path))
    return tmp_path


def test_aggregator_collects_status(telemetry_root: Path) -> None:
    aggregator = SelfStatusAggregator(repo_root=telemetry_root)
    status = aggregator.collect()

    assert len(status.drift) == 1
    assert any(offender.goal_id == "goal-2" for offender in status.budget_offenders)
    assert status.telemetry["gateway"]["total_requests"] == 4


def test_self_status_endpoint_uses_aggregator(telemetry_root: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    module = importlib.import_module("server.python.unified_api.routers.self_status")
    importlib.reload(module)

    app = create_app()
    client = TestClient(app)
    response = client.get("/api/v1/self/status")
    assert response.status_code == 200
    payload = response.json()
    assert payload["drift"][0]["subject"] == "model-A"
    assert payload["hot_paths"]
    assert payload["budget_offenders"]
