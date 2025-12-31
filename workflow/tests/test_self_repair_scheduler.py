from __future__ import annotations

import json
from pathlib import Path

import asyncio

import pytest

from server.python.autonomy.self_debugging_repair_system import SelfDebuggingRepairSystem
from server.python.autonomy.self_status import SelfStatusAggregator
from server.python.autonomy.scorekeeper_client import ScorekeeperClient
from workflow.cron.self_repair_scheduler import PeriodicPlanner


@pytest.fixture()
def populated_repo(tmp_path: Path) -> Path:
    (tmp_path / "cicd/ml/reports").mkdir(parents=True)
    (tmp_path / "storage/db/analytics").mkdir(parents=True)
    (tmp_path / "storage/telemetry").mkdir(parents=True)
    (tmp_path / "storage/logs").mkdir(parents=True)

    (tmp_path / "cicd/ml/reports/drift_report.jsonl").write_text("", encoding="utf-8")
    goal_metrics = [
        {
            "goal_id": "goal-1",
            "workflow_id": "wf-alpha",
            "total_runs": 5,
            "successful_runs": 5,
            "average_lead_time_ms": 40_000,
            "success_rate": 1.0,
        }
    ]
    (tmp_path / "storage/db/analytics/goal_kpis.json").write_text(
        json.dumps(goal_metrics), encoding="utf-8"
    )
    (tmp_path / "storage/telemetry/gateway_metrics.json").write_text(
        json.dumps({"total_requests": 1}), encoding="utf-8"
    )
    return tmp_path


def test_periodic_planner_generates_plan(populated_repo: Path, tmp_path: Path) -> None:
    aggregator = SelfStatusAggregator(repo_root=populated_repo)
    scorekeeper = ScorekeeperClient(aggregator, capability_threshold=0.5)
    repair_system = SelfDebuggingRepairSystem(
        scorekeeper=scorekeeper,
        status_aggregator=aggregator,
        audit_log_path=tmp_path / "audit.log",
    )
    planner = PeriodicPlanner(
        interval_seconds=1,
        aggregator=aggregator,
        scorekeeper=scorekeeper,
        repair_system=repair_system,
        plan_dir=tmp_path / "plans",
        snapshot_dir=tmp_path / "snapshots",
    )

    result = asyncio.run(planner.run_once())
    assert Path(result["plan_path"]).exists()
    assert Path(result["snapshot"]).exists()
    assert result["repair_result"]["status"] in {"success", "blocked"}
