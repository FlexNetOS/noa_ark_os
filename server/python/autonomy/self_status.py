"""Aggregated self-status metrics for NOA ARK OS.

This module centralises access to telemetry emitted by existing
instrumentation components so other subsystems (gateway endpoints,
self-repair routines, scorekeeper) have a single place to query
for drift alerts, workflow hot paths, and budget offenders.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

DRIFT_REPORT_PATH = Path("cicd/ml/reports/drift_report.jsonl")
GOAL_METRICS_PATH = Path("storage/db/analytics/goal_kpis.json")
GATEWAY_METRICS_PATH = Path("storage/telemetry/gateway_metrics.json")


@dataclass
class DriftAlert:
    """A single drift issue captured by offline detectors."""

    subject: str
    severity: str
    issues: List[str] = field(default_factory=list)
    detected_at: Optional[str] = None


@dataclass
class HotPath:
    """A workflow goal that is heavily exercised and impacts velocity."""

    goal_id: str
    workflow_id: str
    total_runs: int
    success_rate: float
    average_lead_time_ms: float


@dataclass
class BudgetOffender:
    """Workflows or agents consuming excessive budget/time."""

    goal_id: str
    workflow_id: str
    average_lead_time_ms: float
    severity: str
    rationale: str


@dataclass
class SelfStatus:
    """Aggregate status surface used by `/v1/self/status`."""

    drift: List[DriftAlert]
    hot_paths: List[HotPath]
    budget_offenders: List[BudgetOffender]
    telemetry: Dict[str, Any]
    generated_at: str


class SelfStatusAggregator:
    """Collects self status signals from existing telemetry artifacts."""

    def __init__(self, repo_root: Optional[Path] = None) -> None:
        self.repo_root = repo_root or Path(
            os.environ.get("NOA_ROOT", Path(__file__).resolve().parents[3])
        )

    def collect(self) -> SelfStatus:
        drift = self._load_drift_alerts()
        goal_metrics = self._load_goal_metrics()
        hot_paths = self._derive_hot_paths(goal_metrics)
        budget_offenders = self._derive_budget_offenders(goal_metrics)
        telemetry = {
            "gateway": self._load_gateway_metrics(),
            "goal_metrics": self._summarise_goal_metrics(goal_metrics),
        }
        return SelfStatus(
            drift=drift,
            hot_paths=hot_paths,
            budget_offenders=budget_offenders,
            telemetry=telemetry,
            generated_at=datetime.now(timezone.utc).isoformat(),
        )

    def _load_drift_alerts(self) -> List[DriftAlert]:
        report_path = self.repo_root / DRIFT_REPORT_PATH
        if not report_path.exists():
            return []
        alerts: List[DriftAlert] = []
        with report_path.open("r", encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                try:
                    payload = json.loads(line)
                except json.JSONDecodeError:
                    continue
                severity = payload.get("severity") or payload.get("status", "info")
                alerts.append(
                    DriftAlert(
                        subject=payload.get("artifact") or payload.get("subject", "unknown"),
                        severity=str(severity).lower(),
                        issues=payload.get("issues", []),
                        detected_at=payload.get("timestamp"),
                    )
                )
        return alerts

    def _load_goal_metrics(self) -> List[Dict[str, Any]]:
        metrics_path = self.repo_root / GOAL_METRICS_PATH
        if not metrics_path.exists():
            return []
        try:
            raw = metrics_path.read_text(encoding="utf-8").strip()
        except OSError:
            return []
        if not raw:
            return []
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            return []
        if isinstance(payload, list):
            return [self._normalise_goal_snapshot(item) for item in payload]
        return []

    def _normalise_goal_snapshot(self, item: Dict[str, Any]) -> Dict[str, Any]:
        snapshot = {
            "goal_id": item.get("goal_id", "unknown"),
            "workflow_id": item.get("workflow_id", "unknown"),
            "total_runs": int(item.get("total_runs", 0)),
            "successful_runs": int(item.get("successful_runs", 0)),
            "average_lead_time_ms": float(item.get("average_lead_time_ms", 0.0)),
            "success_rate": float(item.get("success_rate", 0.0)),
            "context_penalty_score": float(item.get("context_penalty_score", 0.0)),
            "context_p95_bytes": int(item.get("context_p95_bytes", 0)),
            "context_p95_latency_ms": int(item.get("context_p95_latency_ms", 0)),
            "reward_total": float(item.get("reward_total", 0.0)),
            "reward_average": float(item.get("reward_average", 0.0)),
            "reward_recent": float(item.get("reward_recent", 0.0)),
        }
        return snapshot

    def _derive_hot_paths(self, metrics: List[Dict[str, Any]]) -> List[HotPath]:
        ranked = sorted(
            metrics,
            key=lambda item: (item.get("total_runs", 0), item.get("success_rate", 0.0)),
            reverse=True,
        )
        hot_paths = []
        for entry in ranked[:5]:
            hot_paths.append(
                HotPath(
                    goal_id=entry["goal_id"],
                    workflow_id=entry["workflow_id"],
                    total_runs=int(entry.get("total_runs", 0)),
                    success_rate=float(entry.get("success_rate", 0.0)),
                    average_lead_time_ms=float(entry.get("average_lead_time_ms", 0.0)),
                )
            )
        return hot_paths

    def _derive_budget_offenders(
        self, metrics: List[Dict[str, Any]]
    ) -> List[BudgetOffender]:
        offenders: List[BudgetOffender] = []
        for entry in metrics:
            lead_time = float(entry.get("average_lead_time_ms", 0.0))
            success_rate = float(entry.get("success_rate", 0.0))
            severity = "low"
            rationale_parts = []
            if lead_time > 120_000:  # >2 minutes
                severity = "critical"
                rationale_parts.append(
                    f"lead time {lead_time:.0f}ms exceeds 120000ms budget"
                )
            elif lead_time > 60_000:
                severity = "medium"
                rationale_parts.append(
                    f"lead time {lead_time:.0f}ms exceeds 60000ms budget"
                )
            if success_rate < 0.6:
                severity = "critical"
                rationale_parts.append(
                    f"success rate {success_rate:.0%} below 60% threshold"
                )
            elif success_rate < 0.75 and severity != "critical":
                severity = "medium"
                rationale_parts.append(
                    f"success rate {success_rate:.0%} below 75% target"
                )
            if not rationale_parts:
                continue
            offenders.append(
                BudgetOffender(
                    goal_id=entry["goal_id"],
                    workflow_id=entry["workflow_id"],
                    average_lead_time_ms=lead_time,
                    severity=severity,
                    rationale="; ".join(rationale_parts),
                )
            )
        offenders.sort(
            key=lambda offender: (
                {"critical": 2, "medium": 1, "low": 0}.get(offender.severity, 0),
                offender.average_lead_time_ms,
            ),
            reverse=True,
        )
        return offenders[:5]

    def _load_gateway_metrics(self) -> Dict[str, Any]:
        metrics_path = self.repo_root / GATEWAY_METRICS_PATH
        if not metrics_path.exists():
            return {}
        try:
            payload = json.loads(metrics_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return payload

    def _summarise_goal_metrics(
        self, metrics: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        if not metrics:
            return {
                "goal_count": 0,
                "avg_success_rate": 0.0,
                "avg_lead_time_ms": 0.0,
                "avg_context_penalty_score": 0.0,
                "max_context_p95_bytes": 0,
                "max_context_p95_latency_ms": 0,
                "total_reward": 0.0,
                "avg_reward": 0.0,
                "avg_recent_reward": 0.0,
            }
        total_runs = sum(item.get("total_runs", 0) for item in metrics)
        avg_success = sum(item.get("success_rate", 0.0) for item in metrics) / len(metrics)
        avg_lead_time = sum(
            item.get("average_lead_time_ms", 0.0) for item in metrics
        ) / len(metrics)
        avg_penalty = sum(
            item.get("context_penalty_score", 0.0) for item in metrics
        ) / len(metrics)
        max_p95_bytes = max(item.get("context_p95_bytes", 0) for item in metrics)
        max_p95_latency = max(
            item.get("context_p95_latency_ms", 0) for item in metrics
        )
        total_reward = sum(item.get("reward_total", 0.0) for item in metrics)
        avg_reward = total_reward / len(metrics) if metrics else 0.0
        avg_recent_reward = sum(
            item.get("reward_recent", 0.0) for item in metrics
        ) / len(metrics)
        return {
            "goal_count": len(metrics),
            "total_runs": total_runs,
            "avg_success_rate": avg_success,
            "avg_lead_time_ms": avg_lead_time,
            "avg_context_penalty_score": avg_penalty,
            "max_context_p95_bytes": max_p95_bytes,
            "max_context_p95_latency_ms": max_p95_latency,
            "total_reward": total_reward,
            "avg_reward": avg_reward,
            "avg_recent_reward": avg_recent_reward,
        }


__all__ = [
    "BudgetOffender",
    "DriftAlert",
    "HotPath",
    "SelfStatus",
    "SelfStatusAggregator",
]
