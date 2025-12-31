from __future__ import annotations

import json
from pathlib import Path

import pytest

from core.kernel.north_star import NorthStarSchemaError, load_policy, validate


def test_north_star_policy_loads() -> None:
    policy = load_policy()
    assert policy.metrics, "expected at least one metric"
    weights = sum(metric.weight for metric in policy.metrics)
    assert abs(weights - 1.0) < 1e-6


def test_validator_rejects_bad_threshold(tmp_path: Path) -> None:
    broken = tmp_path / "north_star.bad.json"
    baseline = json.loads(Path("core/kernel/north_star.deflex.json").read_text())
    baseline["metrics"][0]["thresholds"]["warning"] = 0.2
    baseline["metrics"][0]["thresholds"]["critical"] = 0.8
    broken.write_text(json.dumps(baseline))

    with pytest.raises(NorthStarSchemaError):
        validate(broken)
