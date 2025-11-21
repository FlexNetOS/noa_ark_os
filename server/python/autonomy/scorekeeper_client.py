"""Scorekeeper integration providing trust signals for self-repair."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Optional

from .self_status import SelfStatusAggregator


def _clamp(value: float, minimum: float = 0.0, maximum: float = 1.0) -> float:
    return max(minimum, min(value, maximum))


@dataclass
class TrustSignals:
    capability: float
    integrity: float
    reversibility: float
    capability_threshold: float
    metadata: Dict[str, Any]

    def meets_capability(self) -> bool:
        return self.capability >= self.capability_threshold


class ScorekeeperClient:
    """Computes trust signals derived from collected telemetry."""

    def __init__(
        self,
        aggregator: Optional[SelfStatusAggregator] = None,
        capability_threshold: float = 0.7,
    ) -> None:
        self.aggregator = aggregator or SelfStatusAggregator()
        self.capability_threshold = capability_threshold

    def evaluate(self) -> TrustSignals:
        status = self.aggregator.collect()
        goal_metrics = status.telemetry.get("goal_metrics", {})
        avg_success = float(goal_metrics.get("avg_success_rate", 0.0))
        drift_count = len(status.drift)
        offender_penalty = sum(
            0.1 if offender.severity == "medium" else 0.2
            for offender in status.budget_offenders
        )
        capability = _clamp(avg_success)
        integrity = _clamp(1.0 - min(1.0, drift_count * 0.1))
        reversibility = _clamp(1.0 - offender_penalty)
        metadata = {
            "drift_count": drift_count,
            "budget_offenders": [
                {
                    "goal_id": offender.goal_id,
                    "severity": offender.severity,
                    "rationale": offender.rationale,
                }
                for offender in status.budget_offenders
            ],
            "goal_metrics": goal_metrics,
            "generated_at": status.generated_at,
        }
        return TrustSignals(
            capability=capability,
            integrity=integrity,
            reversibility=reversibility,
            capability_threshold=self.capability_threshold,
            metadata=metadata,
        )


__all__ = ["ScorekeeperClient", "TrustSignals"]
