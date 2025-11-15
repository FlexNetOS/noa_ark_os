from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import asyncio

import pytest

from server.python.autonomy.scorekeeper_client import TrustSignals
from server.python.autonomy.self_debugging_repair_system import (
    Selfdebuggingrepairsystem,
)


@dataclass
class DummyScorekeeper:
    capability: float
    threshold: float = 0.7

    def evaluate(self) -> TrustSignals:
        return TrustSignals(
            capability=self.capability,
            integrity=0.9,
            reversibility=0.9,
            capability_threshold=self.threshold,
            metadata={},
        )

    @property
    def capability_threshold(self) -> float:  # pragma: no cover - trivial accessor
        return self.threshold


def test_self_repair_blocked_when_capability_low(tmp_path: Path) -> None:
    scorekeeper = DummyScorekeeper(capability=0.4)
    system = Selfdebuggingrepairsystem(scorekeeper=scorekeeper)
    result = asyncio.run(system.execute_operation({"type": "code_modification"}))
    assert result["status"] == "blocked"


def test_self_repair_executes_when_capability_high(tmp_path: Path) -> None:
    scorekeeper = DummyScorekeeper(capability=0.85)
    system = Selfdebuggingrepairsystem(scorekeeper=scorekeeper)
    result = asyncio.run(
        system.execute_operation({"type": "code_modification", "description": "touch"})
    )
    assert result["status"] == "success"
