from __future__ import annotations

import json
from pathlib import Path

import pytest

from server.python.autonomy.scorekeeper_client import ScorekeeperClient
from server.python.autonomy.self_status import SelfStatus


class StubAggregator:
    def collect(self) -> SelfStatus:
        return SelfStatus(
            drift=[],
            hot_paths=[],
            budget_offenders=[],
            telemetry={
                "goal_metrics": {"avg_success_rate": 0.85},
                "budget_guardian": {
                    "latency_ms": {"p90": 320.0, "p99": 640.0, "p50": 150.0},
                    "sample_count": 5,
                },
            },
            generated_at="2024-03-15T12:00:00Z",
        )


def test_scorekeeper_logs_trust_snapshot(tmp_path: Path) -> None:
    log_path = tmp_path / "storage/logs/trust.jsonl"
    client = ScorekeeperClient(aggregator=StubAggregator(), log_path=log_path)
    signals = client.evaluate()

    assert log_path.exists()
    entries = log_path.read_text(encoding="utf-8").strip().splitlines()
    assert len(entries) == 1
    payload = json.loads(entries[0])
    assert pytest.approx(payload["capability"]) == signals.capability
    assert payload["metadata"]["budget_guardian"]["sample_count"] == 5
    assert payload["metadata"]["goal_metrics"]["avg_success_rate"] == 0.85
