"""Pipeline orchestration primitives for the Digest system."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, List, Protocol


@dataclass
class PipelineContext:
    """Shared state that flows through the pipeline."""

    repo_path: Path
    output_path: Path
    metadata: dict = field(default_factory=dict)
    artifacts: dict = field(default_factory=dict)
    logs: List[str] = field(default_factory=list)

    def log(self, scope: str, message: str) -> None:
        entry = f"[{scope}] {message}"
        self.logs.append(entry)


class Stage(Protocol):
    """Protocol describing the callable interface of a stage."""

    name: str

    def run(self, context: PipelineContext) -> None:
        ...


class Pipeline:
    """Simple sequential pipeline runner."""

    def __init__(self, stages: Iterable[Stage]):
        self._stages = list(stages)

    @property
    def stages(self) -> List[Stage]:
        return list(self._stages)

    def run(self, context: PipelineContext) -> PipelineContext:
        for stage in self._stages:
            context.log(stage.name, "starting")
            stage.run(context)
            context.log(stage.name, "completed")
        return context
