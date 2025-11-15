"""Periodic planner that generates improvement plans and invokes self-repair."""

from __future__ import annotations

import asyncio
import json
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

from server.python.autonomy.scorekeeper_client import ScorekeeperClient
from server.python.autonomy.self_debugging_repair_system import (
    SelfDebuggingRepairSystem,
)
from server.python.autonomy.self_status import SelfStatusAggregator

SNAPSHOT_DIR = Path("storage/snapshots/self_repair")
PLAN_DIR = Path("storage/self_repair/plans")


@dataclass
class ImprovementPlan:
    plan_id: str
    generated_at: str
    recommendations: List[Dict[str, str]]
    capability: float

    def to_dict(self) -> Dict[str, object]:
        return {
            "plan_id": self.plan_id,
            "generated_at": self.generated_at,
            "recommendations": self.recommendations,
            "capability": self.capability,
        }


class PeriodicPlanner:
    def __init__(
        self,
        interval_seconds: int = 900,
        aggregator: Optional[SelfStatusAggregator] = None,
        scorekeeper: Optional[ScorekeeperClient] = None,
        repair_system: Optional[SelfDebuggingRepairSystem] = None,
        plan_dir: Optional[Path] = None,
        snapshot_dir: Optional[Path] = None,
    ) -> None:
        self.interval_seconds = interval_seconds
        self.aggregator = aggregator or SelfStatusAggregator()
        self.scorekeeper = scorekeeper or ScorekeeperClient(self.aggregator)
        self.repair_system = repair_system or SelfDebuggingRepairSystem(
            scorekeeper=self.scorekeeper,
            status_aggregator=self.aggregator,
        )
        self.plan_dir = plan_dir or Path(
            os.environ.get("NOA_SELF_REPAIR_PLAN_DIR", PLAN_DIR.as_posix())
        )
        self.snapshot_dir = snapshot_dir or Path(
            os.environ.get("NOA_SELF_REPAIR_SNAPSHOT_DIR", SNAPSHOT_DIR.as_posix())
        )
        self.plan_dir.mkdir(parents=True, exist_ok=True)
        self.snapshot_dir.mkdir(parents=True, exist_ok=True)

    async def run_once(self) -> Dict[str, object]:
        status = self.aggregator.collect()
        trust = self.scorekeeper.evaluate()
        plan = self._generate_plan(status.telemetry.get("goal_metrics", {}), trust.capability)
        plan_path = self._persist_plan(plan)
        snapshot_path = self._create_snapshot(plan.plan_id)
        operation = {
            "type": "improvement_plan",
            "plan_path": plan_path.as_posix(),
            "snapshot": snapshot_path.as_posix(),
            "affected_files": [],
            "description": "auto-generated improvement plan",
        }
        result = await self.repair_system.execute_operation(operation)
        return {
            "plan": plan.to_dict(),
            "plan_path": plan_path.as_posix(),
            "snapshot": snapshot_path.as_posix(),
            "repair_result": result,
        }

    async def serve(self) -> None:
        while True:
            await self.run_once()
            await asyncio.sleep(self.interval_seconds)

    def _generate_plan(
        self, metrics: Dict[str, object], capability: float
    ) -> ImprovementPlan:
        plan_id = f"plan-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
        recommendations = []
        goal_count = int(metrics.get("goal_count", 0)) if isinstance(metrics, dict) else 0
        if goal_count:
            recommendations.append(
                {
                    "action": "optimize_hot_paths",
                    "detail": f"prioritize top {min(goal_count, 3)} workflows by run volume",
                }
            )
        else:
            recommendations.append(
                {
                    "action": "seed_metrics",
                    "detail": "populate goal metrics to enable hot path analysis",
                }
            )
        recommendations.append(
            {
                "action": "evaluate_budget",
                "detail": "inspect budget offenders surfaced via /v1/self/status",
            }
        )
        recommendations.append(
            {
                "action": "trust_review",
                "detail": f"capability score={capability:.2f}; ensure threshold maintained",
            }
        )
        return ImprovementPlan(
            plan_id=plan_id,
            generated_at=datetime.now(timezone.utc).isoformat(),
            recommendations=recommendations,
            capability=capability,
        )

    def _persist_plan(self, plan: ImprovementPlan) -> Path:
        path = self.plan_dir / f"{plan.plan_id}.json"
        with path.open("w", encoding="utf-8") as handle:
            json.dump(plan.to_dict(), handle, indent=2, sort_keys=True)
        return path

    def _create_snapshot(self, plan_id: str) -> Path:
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        snapshot_path = self.snapshot_dir / f"{plan_id}-{timestamp}"
        snapshot_path.mkdir(parents=True, exist_ok=True)
        marker = snapshot_path / "snapshot.json"
        metadata = {
            "plan_id": plan_id,
            "created_at": datetime.now(timezone.utc).isoformat(),
            "workspace": os.environ.get("NOA_WORKSPACE_PATH", "workspace"),
        }
        marker.write_text(json.dumps(metadata, indent=2), encoding="utf-8")
        return snapshot_path


__all__ = ["PeriodicPlanner", "ImprovementPlan"]
