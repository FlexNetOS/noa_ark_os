"""North Star trust policy loader and schema validator."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Optional

POLICY_PATH = Path(__file__).with_name("north_star.deflex.json")


class NorthStarSchemaError(ValueError):
    """Raised when the north_star.deflex.json file fails validation."""


@dataclass(frozen=True)
class Thresholds:
    warning: float
    critical: float

    def validate(self) -> None:
        if not 0.0 <= self.critical <= 1.0:
            raise NorthStarSchemaError(
                f"critical threshold must be within [0, 1], got {self.critical}"
            )
        if not 0.0 <= self.warning <= 1.0:
            raise NorthStarSchemaError(
                f"warning threshold must be within [0, 1], got {self.warning}"
            )
        if self.warning < self.critical:
            raise NorthStarSchemaError(
                "warning threshold must be greater than or equal to critical"
            )


@dataclass(frozen=True)
class ScopeReduction:
    warning_multiplier: float = 1.0
    critical_multiplier: float = 0.5
    minimum_optional: int = 0

    def validate(self) -> None:
        if not 0.0 <= self.warning_multiplier <= 1.0:
            raise NorthStarSchemaError(
                "warning multiplier must be within [0, 1]"
            )
        if not 0.0 <= self.critical_multiplier <= 1.0:
            raise NorthStarSchemaError(
                "critical multiplier must be within [0, 1]"
            )
        if self.minimum_optional < 0:
            raise NorthStarSchemaError("minimum_optional must be >= 0")


@dataclass(frozen=True)
class MetricDefinition:
    id: str
    description: str
    weight: float
    thresholds: Thresholds
    escalation_policy: Optional[str] = None
    signals: List[str] = field(default_factory=list)
    scope_reduction: Optional[ScopeReduction] = None

    def validate(self) -> None:
        if not self.id:
            raise NorthStarSchemaError("metric id is required")
        if not 0.0 < self.weight <= 1.0:
            raise NorthStarSchemaError(
                f"metric '{self.id}' weight must be within (0, 1], got {self.weight}"
            )
        self.thresholds.validate()
        if self.scope_reduction:
            self.scope_reduction.validate()


@dataclass(frozen=True)
class EscalationPolicy:
    id: str
    summary: str
    actions: List[str]
    owners: List[str]
    response_time_minutes: int

    def validate(self) -> None:
        if not self.id:
            raise NorthStarSchemaError("escalation policy id is required")
        if self.response_time_minutes <= 0:
            raise NorthStarSchemaError(
                f"response_time_minutes must be positive for policy '{self.id}'"
            )
        if not self.actions:
            raise NorthStarSchemaError(
                f"policy '{self.id}' must define at least one action"
            )
        if not self.owners:
            raise NorthStarSchemaError(
                f"policy '{self.id}' must define at least one owner"
            )


@dataclass(frozen=True)
class NorthStarPolicy:
    version: str
    metrics: List[MetricDefinition]
    escalation_policies: Dict[str, EscalationPolicy]
    defaults: Dict[str, object] = field(default_factory=dict)

    def metric(self, metric_id: str) -> Optional[MetricDefinition]:
        return next((metric for metric in self.metrics if metric.id == metric_id), None)


def _parse_scope_reduction(raw: Optional[Dict[str, object]]) -> Optional[ScopeReduction]:
    if raw is None:
        return None
    return ScopeReduction(
        warning_multiplier=float(raw.get("warning_multiplier", 1.0)),
        critical_multiplier=float(raw.get("critical_multiplier", 0.5)),
        minimum_optional=int(raw.get("minimum_optional", 0)),
    )


def _parse_metric(item: Dict[str, object]) -> MetricDefinition:
    thresholds = Thresholds(
        warning=float(item["thresholds"]["warning"]),
        critical=float(item["thresholds"]["critical"]),
    )
    scope_reduction = _parse_scope_reduction(item.get("scope_reduction"))
    metric = MetricDefinition(
        id=str(item["id"]),
        description=str(item.get("description", "")),
        weight=float(item.get("weight", 0.0)),
        thresholds=thresholds,
        escalation_policy=item.get("escalation_policy"),
        signals=list(item.get("signals", [])),
        scope_reduction=scope_reduction,
    )
    metric.validate()
    return metric


def _parse_policy(item: Dict[str, object]) -> EscalationPolicy:
    policy = EscalationPolicy(
        id=str(item["id"]),
        summary=str(item.get("summary", "")),
        actions=list(item.get("actions", [])),
        owners=list(item.get("owners", [])),
        response_time_minutes=int(item.get("response_time_minutes", 0)),
    )
    policy.validate()
    return policy


def _validate_weights(metrics: Iterable[MetricDefinition]) -> None:
    total = sum(metric.weight for metric in metrics)
    if abs(total - 1.0) > 1e-6:
        raise NorthStarSchemaError(
            f"metric weights must sum to 1.0 (found {total:.6f})"
        )


def load_policy(path: Optional[Path] = None) -> NorthStarPolicy:
    """Load and validate the north star policy from disk."""

    policy_path = path or POLICY_PATH
    data = json.loads(policy_path.read_text())

    metrics = [_parse_metric(metric) for metric in data.get("metrics", [])]
    if not metrics:
        raise NorthStarSchemaError("at least one metric definition is required")

    policies = {
        policy.id: policy
        for policy in (_parse_policy(policy) for policy in data.get("escalation_policies", []))
    }

    metric_ids = set()
    for metric in metrics:
        if metric.id in metric_ids:
            raise NorthStarSchemaError(f"duplicate metric id '{metric.id}' detected")
        metric_ids.add(metric.id)
        if metric.escalation_policy and metric.escalation_policy not in policies:
            raise NorthStarSchemaError(
                f"metric '{metric.id}' references unknown policy '{metric.escalation_policy}'"
            )

    _validate_weights(metrics)

    policy = NorthStarPolicy(
        version=str(data.get("version", "unknown")),
        metrics=metrics,
        escalation_policies=policies,
        defaults=dict(data.get("defaults", {})),
    )

    return policy


def validate(path: Optional[Path] = None) -> NorthStarPolicy:
    """Validate the north star configuration raising errors when invalid."""

    policy = load_policy(path)
    return policy


if __name__ == "__main__":  # pragma: no cover - CLI entry point
    try:
        policy = validate()
    except NorthStarSchemaError as exc:  # pragma: no cover - CLI entry point
        print(f"north_star.deflex.json failed validation: {exc}")
        raise SystemExit(1)
    else:  # pragma: no cover - CLI entry point
        print(
            f"north_star.deflex.json v{policy.version} loaded with {len(policy.metrics)} metrics"
        )
