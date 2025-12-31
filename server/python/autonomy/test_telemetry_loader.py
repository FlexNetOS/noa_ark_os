from __future__ import annotations

import json
from pathlib import Path

from server.python.autonomy.telemetry_loader import (
    iter_gateway_events,
    load_gateway_events,
    load_gateway_metrics,
)


def _write_sample(repo_root: Path) -> None:
    telemetry_dir = repo_root / "storage/telemetry"
    telemetry_dir.mkdir(parents=True, exist_ok=True)
    metrics = {
        "total_requests": 5,
        "per_protocol": {"http": 3, "ws": 2},
    }
    (telemetry_dir / "gateway_metrics.json").write_text(
        json.dumps(metrics), encoding="utf-8"
    )
    events = [
        {"otel_span": {"tokens_total": 10, "latency_ms": 20}},
        {"otel_span": {"tokens_total": 30, "latency_ms": 40}},
        {"otel_span": {"tokens_total": 50, "latency_ms": 60}},
    ]
    with (telemetry_dir / "gateway_events.log").open("w", encoding="utf-8") as handle:
        for entry in events:
            handle.write(json.dumps(entry))
            handle.write("\n")


def test_load_gateway_metrics_uses_repo_root(tmp_path: Path) -> None:
    _write_sample(tmp_path)
    metrics = load_gateway_metrics(repo_root=tmp_path)
    assert metrics["total_requests"] == 5
    assert metrics["per_protocol"]["http"] == 3


def test_iter_gateway_events_honours_limit(tmp_path: Path) -> None:
    _write_sample(tmp_path)
    events = list(iter_gateway_events(repo_root=tmp_path, limit=2))
    assert len(events) == 2
    assert events[0]["otel_span"]["latency_ms"] == 60
    assert events[1]["otel_span"]["latency_ms"] == 40


def test_load_gateway_events_returns_all(tmp_path: Path) -> None:
    _write_sample(tmp_path)
    events = load_gateway_events(repo_root=tmp_path)
    assert len(events) == 3
    assert events[-1]["otel_span"]["tokens_total"] == 10
