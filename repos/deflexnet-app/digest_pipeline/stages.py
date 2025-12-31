"""Implementation of Digest pipeline stages."""

from __future__ import annotations

import json
import os
import shutil
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List

from .pipeline import PipelineContext


@dataclass
class BaseStage:
    name: str

    def run(self, context: PipelineContext) -> None:  # pragma: no cover - interface only
        raise NotImplementedError

    def _ensure_output_dir(self, context: PipelineContext, *segments: str) -> Path:
        target = context.output_path.joinpath(*segments)
        target.parent.mkdir(parents=True, exist_ok=True)
        return target


class IntakeStage(BaseStage):
    """Collect repository metadata such as file inventory and provenance."""

    def __init__(self) -> None:
        super().__init__(name="intake")

    def run(self, context: PipelineContext) -> None:
        file_inventory: List[Dict[str, str]] = []
        for root, _, files in os.walk(context.repo_path):
            for file_name in files:
                path = Path(root) / file_name
                rel_path = path.relative_to(context.repo_path).as_posix()
                size = path.stat().st_size
                file_inventory.append({"path": rel_path, "bytes": size})
        context.metadata["files"] = file_inventory
        context.log(self.name, f"catalogued {len(file_inventory)} files")


class ClassifierStage(BaseStage):
    """Identify languages, build systems, and licenses heuristically."""

    EXTENSION_LANG = {
        ".py": "python",
        ".js": "javascript",
        ".ts": "typescript",
        ".go": "go",
        ".rs": "rust",
        ".java": "java",
        ".json": "json",
        ".md": "markdown",
        ".yml": "yaml",
        ".yaml": "yaml",
        ".sh": "shell",
    }

    def __init__(self) -> None:
        super().__init__(name="classifier")

    def run(self, context: PipelineContext) -> None:
        counts: Counter[str] = Counter()
        for file_info in context.metadata.get("files", []):
            ext = Path(file_info["path"]).suffix.lower()
            language = self.EXTENSION_LANG.get(ext, "other")
            counts[language] += 1
        context.metadata["languages"] = counts
        context.metadata["build_systems"] = self._detect_build_systems(context.repo_path)
        context.metadata["license_files"] = self._detect_license_files(context.repo_path)
        context.log(self.name, f"detected languages: {dict(counts)}")

    def _detect_build_systems(self, repo_path: Path) -> List[str]:
        candidates = {
            "python": ["pyproject.toml", "requirements.txt"],
            "node": ["package.json"],
            "go": ["go.mod"],
            "rust": ["Cargo.toml"],
        }
        found = []
        for system, markers in candidates.items():
            if any((repo_path / marker).exists() for marker in markers):
                found.append(system)
        return found

    def _detect_license_files(self, repo_path: Path) -> List[str]:
        return [p.name for p in repo_path.glob("LICENSE*")]


class GraphExtractStage(BaseStage):
    """Create a lightweight dependency and artifact graph."""

    def __init__(self) -> None:
        super().__init__(name="graph_extract")

    def run(self, context: PipelineContext) -> None:
        graph = {
            "nodes": [],
            "edges": [],
        }
        for file_info in context.metadata.get("files", []):
            node = {"id": file_info["path"], "type": "file"}
            graph["nodes"].append(node)
        context.artifacts["kg.json"] = graph
        target = self._ensure_output_dir(context, "kg.json")
        target.write_text(json.dumps(graph, indent=2), encoding="utf-8")
        context.log(self.name, f"wrote graph with {len(graph['nodes'])} nodes")


class EmbeddingsStage(BaseStage):
    """Generate deterministic pseudo-embeddings for code and docs."""

    def __init__(self) -> None:
        super().__init__(name="embeddings")

    def run(self, context: PipelineContext) -> None:
        embeddings: Dict[str, List[float]] = {}
        for file_info in context.metadata.get("files", []):
            path = file_info["path"]
            full_path = context.repo_path / path
            text = full_path.read_text(encoding="utf-8", errors="ignore")
            vector = self._hash_to_vector(text)
            embeddings[path] = vector
        context.artifacts["embeddings.json"] = embeddings
        target = self._ensure_output_dir(context, "embeddings.json")
        target.write_text(json.dumps(embeddings, indent=2), encoding="utf-8")
        context.log(self.name, f"generated embeddings for {len(embeddings)} files")

    def _hash_to_vector(self, text: str, dimensions: int = 8) -> List[float]:
        vector = [0.0] * dimensions
        for index, char in enumerate(text.encode("utf-8")):
            vector[index % dimensions] += (char - 128) / 128.0
        length = sum(abs(v) for v in vector) or 1.0
        return [round(v / length, 4) for v in vector]


class EnvSynthesisStage(BaseStage):
    """Collect runtime environment hints and configuration templates."""

    def __init__(self) -> None:
        super().__init__(name="env_synthesis")

    def run(self, context: PipelineContext) -> None:
        env_hints = defaultdict(list)
        for candidate in ("Dockerfile", "docker-compose.yml", "Makefile", ".env", ".env.example"):
            path = context.repo_path / candidate
            if path.exists():
                env_hints[path.suffix or path.name].append(candidate)
        context.metadata["environment"] = dict(env_hints)
        context.log(self.name, f"found env hints: {dict(env_hints)}")


class SafetyStage(BaseStage):
    """Record security scanning placeholders."""

    def __init__(self) -> None:
        super().__init__(name="safety")

    def run(self, context: PipelineContext) -> None:
        report = {
            "sbom": "pending",
            "vulnerabilities": [],
            "secrets": [],
        }
        context.artifacts["safety.json"] = report
        target = self._ensure_output_dir(context, "safety.json")
        target.write_text(json.dumps(report, indent=2), encoding="utf-8")
        context.log(self.name, "stub safety report created")


class RunnerStage(BaseStage):
    """Simulate builds and test execution."""

    def __init__(self) -> None:
        super().__init__(name="runner")

    def run(self, context: PipelineContext) -> None:
        tasks: List[str] = []
        if (context.repo_path / "pyproject.toml").exists():
            tasks.append("poetry build")
        if (context.repo_path / "package.json").exists():
            tasks.append("npm test")
        context.metadata["runbook"] = tasks or ["manual review"]
        context.log(self.name, f"planned tasks: {context.metadata['runbook']}")


class ReverseEngineerStage(BaseStage):
    """Store reverse-engineering placeholder tasks."""

    def __init__(self) -> None:
        super().__init__(name="reverse_engineer")

    def run(self, context: PipelineContext) -> None:
        reverse_report = {
            "binary_analysis": "not_applicable",
            "http_probe": "pending",
            "fuzzing": "pending",
        }
        context.artifacts["reverse.json"] = reverse_report
        target = self._ensure_output_dir(context, "reverse.json")
        target.write_text(json.dumps(reverse_report, indent=2), encoding="utf-8")
        context.log(self.name, "reverse engineering plan stubbed")


class IntegratorStage(BaseStage):
    """Generate integration stubs for adapters and telemetry."""

    def __init__(self) -> None:
        super().__init__(name="integrator")

    def run(self, context: PipelineContext) -> None:
        adapters = {
            "python": "pip install deflexnet-sdk",
            "node": "npm install deflexnet-sdk",
        }
        context.artifacts["integrations.json"] = adapters
        target = self._ensure_output_dir(context, "integrations.json")
        target.write_text(json.dumps(adapters, indent=2), encoding="utf-8")
        context.log(self.name, "integration stubs generated")


class RegistrarStage(BaseStage):
    """Emit registry ready artifacts such as profile.json and system card."""

    def __init__(self) -> None:
        super().__init__(name="registrar")

    def run(self, context: PipelineContext) -> None:
        output_dir = context.output_path
        output_dir.mkdir(parents=True, exist_ok=True)
        profile = {
            "metadata": context.metadata,
            "artifacts": sorted(context.artifacts.keys()),
        }
        (output_dir / "profile.json").write_text(json.dumps(profile, indent=2), encoding="utf-8")
        (output_dir / "system_card.md").write_text(self._system_card(profile), encoding="utf-8")
        context.log(self.name, "profile and system card written")

    def _system_card(self, profile: Dict[str, object]) -> str:
        languages = profile["metadata"].get("languages", {})
        lines = ["# DeflexNet Digest", "", "## Languages"]
        for language, count in languages.items():
            lines.append(f"- {language}: {count}")
        lines.append("\n## Artifacts")
        for artifact in profile["artifacts"]:
            lines.append(f"- {artifact}")
        return "\n".join(lines) + "\n"


class CRMStranglerStage(BaseStage):
    """Plan CRM proxy rollout with feature flags."""

    def __init__(self) -> None:
        super().__init__(name="crm_strangler")

    def run(self, context: PipelineContext) -> None:
        plan = {
            "shadow_mode": True,
            "write_through": False,
            "toggles": {
                "accounts": "shadow",
                "tickets": "observe",
                "billing": "off",
            },
        }
        context.artifacts["crm_plan.json"] = plan
        target = self._ensure_output_dir(context, "crm_plan.json")
        target.write_text(json.dumps(plan, indent=2), encoding="utf-8")
        context.log(self.name, "CRM strangler plan generated")
