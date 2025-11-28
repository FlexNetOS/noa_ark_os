"""Materialize analytics snapshots from raw workflow events.

This module ingests workflow execution and context usage events stored under
``storage/analytics/raw/workflow_events.jsonl`` and emits the canonical
`GoalMetricSnapshot` payload expected by downstream services. The resulting
artifacts are written to both the durable analytics database and a pipeline
mirror for ad-hoc tooling.
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterator, List, Optional, Sequence

RAW_EVENTS_PATH = Path("storage/analytics/raw/workflow_events.jsonl")
GOAL_SNAPSHOT_PATH = Path("storage/db/analytics/goal_kpis.json")
PIPELINE_MIRROR_PATH = Path("storage/analytics/pipelines/goal_metrics.jsonl")

RECENT_REWARD_WINDOW = 5


def _parse_timestamp(value: object) -> Optional[datetime]:
    if value is None:
        return None
    if isinstance(value, (int, float)):
        # Interpret numeric timestamps as milliseconds since epoch
        seconds = float(value) / 1000.0
        return datetime.fromtimestamp(seconds, tz=timezone.utc)
    if isinstance(value, str):
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return None
    return None


def _safe_float(value: object) -> float:
    try:
        return float(value) if value is not None else 0.0
    except (TypeError, ValueError):
        return 0.0


def _safe_int(value: object) -> int:
    try:
        return int(value) if value is not None else 0
    except (TypeError, ValueError):
        return 0


def _percentile(values: Sequence[float], percentile: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return float(values[0])
    clamped = max(0.0, min(1.0, percentile))
    rank = clamped * (len(values) - 1)
    lower_index = math.floor(rank)
    upper_index = math.ceil(rank)
    lower_value = float(values[lower_index])
    upper_value = float(values[upper_index])
    if lower_index == upper_index:
        return lower_value
    weight = rank - lower_index
    return (upper_value * weight) + (lower_value * (1.0 - weight))


@dataclass
class AgentAccumulator:
    total_runs: int = 0
    successful_runs: int = 0

    def record(self, success: bool) -> None:
        self.total_runs += 1
        if success:
            self.successful_runs += 1

    def snapshot(self, name: str) -> Dict[str, object]:
        runs = max(1, self.total_runs)
        return {
            "agent": name,
            "total_runs": self.total_runs,
            "successful_runs": self.successful_runs,
            "success_rate": self.successful_runs / runs,
        }


@dataclass
class ContextAccumulator:
    penalties: List[float] = field(default_factory=list)
    bytes: List[int] = field(default_factory=list)
    latency: List[int] = field(default_factory=list)

    def record(self, event: Dict[str, object]) -> None:
        penalty = _safe_float(event.get("penalty") or event.get("penalty_score"))
        context_bytes = _safe_int(event.get("context_bytes"))
        latency_ms = _safe_int(
            event.get("latency_ms") or event.get("retrieval_latency_ms")
        )
        if penalty:
            self.penalties.append(penalty)
        if context_bytes:
            self.bytes.append(context_bytes)
        if latency_ms:
            self.latency.append(latency_ms)

    def summary(self) -> Dict[str, object]:
        penalties = self.penalties
        bytes_samples = sorted(float(value) for value in self.bytes)
        latency_samples = sorted(float(value) for value in self.latency)
        avg_penalty = sum(penalties) / len(penalties) if penalties else 0.0
        return {
            "context_penalty_score": avg_penalty,
            "context_p95_bytes": int(_percentile(bytes_samples, 0.95)),
            "context_p95_latency_ms": int(_percentile(latency_samples, 0.95)),
        }


@dataclass
class GoalAccumulator:
    goal_id: str
    workflow_id: str
    total_runs: int = 0
    successful_runs: int = 0
    total_duration_ms: float = 0.0
    agents: Dict[str, AgentAccumulator] = field(default_factory=dict)
    reward_deltas: List[float] = field(default_factory=list)
    last_timestamp: Optional[datetime] = None

    def record_goal(self, event: Dict[str, object]) -> None:
        self.total_runs += 1
        if bool(event.get("success")):
            self.successful_runs += 1
        self.total_duration_ms += _safe_float(event.get("duration_ms"))

        for agent_event in event.get("agents", []) or []:
            if not isinstance(agent_event, dict):
                continue
            agent_name = agent_event.get("agent")
            if not isinstance(agent_name, str) or not agent_name:
                continue
            accumulator = self.agents.setdefault(agent_name, AgentAccumulator())
            accumulator.record(bool(agent_event.get("success", False)))

        reward_delta = event.get("reward_delta")
        if reward_delta is None:
            reward_delta = event.get("reward") or event.get("reward_total")
        reward_value = _safe_float(reward_delta)
        if reward_value:
            self.reward_deltas.append(reward_value)

        timestamp = _parse_timestamp(event.get("timestamp"))
        if timestamp and (self.last_timestamp is None or timestamp > self.last_timestamp):
            self.last_timestamp = timestamp

    def snapshot(
        self,
        context: Optional[ContextAccumulator] = None,
        default_timestamp: Optional[datetime] = None,
    ) -> Dict[str, object]:
        runs = max(1, self.total_runs)
        average_lead_time_ms = self.total_duration_ms / runs
        success_rate = self.successful_runs / runs
        agents = [acc.snapshot(name) for name, acc in self.agents.items()]
        agents.sort(key=lambda entry: entry["success_rate"], reverse=True)

        reward_total = sum(self.reward_deltas) if self.reward_deltas else 0.0
        reward_average = (
            reward_total / len(self.reward_deltas) if self.reward_deltas else 0.0
        )
        recent_window = self.reward_deltas[-RECENT_REWARD_WINDOW :]
        reward_recent = (
            sum(recent_window) / len(recent_window) if recent_window else 0.0
        )

        timestamp = self.last_timestamp or default_timestamp or datetime.now(timezone.utc)
        if timestamp.tzinfo is None:
            timestamp = timestamp.replace(tzinfo=timezone.utc)

        snapshot = {
            "goal_id": self.goal_id,
            "workflow_id": self.workflow_id,
            "total_runs": self.total_runs,
            "successful_runs": self.successful_runs,
            "average_lead_time_ms": average_lead_time_ms,
            "success_rate": success_rate,
            "agents": agents,
            "updated_at": timestamp.astimezone(timezone.utc).isoformat(),
            "context_penalty_score": 0.0,
            "context_p95_bytes": 0,
            "context_p95_latency_ms": 0,
            "reward_total": reward_total,
            "reward_average": reward_average,
            "reward_recent": reward_recent,
        }

        if context:
            summary = context.summary()
            snapshot.update(summary)

        return snapshot


def _load_events(path: Path) -> Iterator[Dict[str, object]]:
    if not path.exists():
        return iter(())
    try:
        contents = path.read_text(encoding="utf-8")
    except OSError:
        return iter(())
    lines = [line for line in contents.splitlines() if line.strip()]

    def _iterator() -> Iterator[Dict[str, object]]:
        for line in lines:
            try:
                payload = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(payload, dict):
                yield payload

    return _iterator()


def materialise_goal_metrics(
    raw_events_path: Path = RAW_EVENTS_PATH,
    output_path: Path = GOAL_SNAPSHOT_PATH,
    pipeline_path: Path = PIPELINE_MIRROR_PATH,
    now: Optional[datetime] = None,
) -> List[Dict[str, object]]:
    accumulators: Dict[str, GoalAccumulator] = {}
    context_accumulators: Dict[str, ContextAccumulator] = {}

    timestamp = now or datetime.now(timezone.utc)

    for event in _load_events(raw_events_path):
        event_type = event.get("type") or event.get("event_type")
        if event_type == "goal.outcome":
            goal_id = event.get("goal_id")
            workflow_id = event.get("workflow_id")
            if not isinstance(goal_id, str) or not isinstance(workflow_id, str):
                continue
            accumulator = accumulators.setdefault(
                goal_id,
                GoalAccumulator(goal_id=goal_id, workflow_id=workflow_id),
            )
            accumulator.record_goal(event)
        elif event_type == "context.usage":
            workflow_id = event.get("workflow_id")
            if not isinstance(workflow_id, str):
                continue
            accumulator = context_accumulators.setdefault(
                workflow_id, ContextAccumulator()
            )
            accumulator.record(event)

    snapshots: List[Dict[str, object]] = []
    for goal_id, accumulator in accumulators.items():
        context = context_accumulators.get(accumulator.workflow_id)
        snapshots.append(accumulator.snapshot(context=context, default_timestamp=timestamp))

    snapshots.sort(key=lambda entry: entry["updated_at"], reverse=True)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    pipeline_path.parent.mkdir(parents=True, exist_ok=True)

    with output_path.open("w", encoding="utf-8") as handle:
        json.dump(snapshots, handle, indent=2, sort_keys=True)
        handle.write("\n")

    with pipeline_path.open("w", encoding="utf-8") as handle:
        for snapshot in snapshots:
            handle.write(json.dumps(snapshot, sort_keys=True))
            handle.write("\n")

    return snapshots


def main() -> None:
    materialise_goal_metrics()


if __name__ == "__main__":
    main()


__all__ = ["materialise_goal_metrics", "main"]
