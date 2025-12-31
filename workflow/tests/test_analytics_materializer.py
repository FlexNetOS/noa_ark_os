import json
from datetime import datetime, timezone
from pathlib import Path

from workflow.cron.analytics_materializer import materialise_goal_metrics


def test_materialiser_generates_snapshots(tmp_path: Path) -> None:
    raw_path = tmp_path / "workflow_events.jsonl"
    events = [
        {
            "type": "goal.outcome",
            "goal_id": "goal-1",
            "workflow_id": "wf-alpha",
            "duration_ms": 40_000,
            "success": True,
            "timestamp": "2024-06-01T00:00:00Z",
            "agents": [{"agent": "agent-a", "success": True}],
            "reward_delta": 1.2,
        },
        {
            "type": "goal.outcome",
            "goal_id": "goal-1",
            "workflow_id": "wf-alpha",
            "duration_ms": 80_000,
            "success": False,
            "timestamp": "2024-06-01T01:00:00Z",
            "agents": [{"agent": "agent-a", "success": False}],
            "reward_delta": -0.2,
        },
        {
            "type": "goal.outcome",
            "goal_id": "goal-2",
            "workflow_id": "wf-beta",
            "duration_ms": 55_000,
            "success": True,
            "timestamp": "2024-06-01T02:00:00Z",
            "agents": [{"agent": "agent-b", "success": True}],
            "reward_delta": 0.5,
        },
        {
            "type": "context.usage",
            "workflow_id": "wf-alpha",
            "penalty": 0.3,
            "context_bytes": 16_000,
            "latency_ms": 120,
        },
        {
            "type": "context.usage",
            "workflow_id": "wf-alpha",
            "penalty_score": 0.6,
            "context_bytes": 32_000,
            "retrieval_latency_ms": 200,
        },
    ]
    raw_path.write_text("\n".join(json.dumps(event) for event in events), encoding="utf-8")

    output_path = tmp_path / "goal_kpis.json"
    pipeline_path = tmp_path / "goal_metrics.jsonl"
    now = datetime(2024, 6, 2, 0, 0, 0, tzinfo=timezone.utc)

    snapshots = materialise_goal_metrics(
        raw_events_path=raw_path,
        output_path=output_path,
        pipeline_path=pipeline_path,
        now=now,
    )

    assert output_path.exists()
    assert pipeline_path.exists()

    with output_path.open("r", encoding="utf-8") as handle:
        stored = json.load(handle)

    assert len(snapshots) == 2
    assert stored == snapshots

    alpha = next(item for item in snapshots if item["goal_id"] == "goal-1")
    assert alpha["total_runs"] == 2
    assert alpha["successful_runs"] == 1
    assert abs(alpha["average_lead_time_ms"] - 60_000) < 1e-6
    assert alpha["success_rate"] == 0.5
    assert alpha["reward_total"] == 1.0
    assert alpha["reward_average"] == 0.5
    assert alpha["reward_recent"] == 0.5
    assert abs(alpha["context_penalty_score"] - 0.45) < 1e-6
    assert alpha["context_p95_bytes"] == 31_200
    assert alpha["context_p95_latency_ms"] == 196

    beta = next(item for item in snapshots if item["goal_id"] == "goal-2")
    assert beta["total_runs"] == 1
    assert beta["success_rate"] == 1.0
    assert beta["reward_total"] == 0.5
    assert beta["context_penalty_score"] == 0.0

    with pipeline_path.open("r", encoding="utf-8") as handle:
        lines = [line for line in handle.read().splitlines() if line.strip()]
    assert len(lines) == len(snapshots)
