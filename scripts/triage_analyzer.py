#!/usr/bin/env python3
"""Event-driven triage service for offline automation.

This service watches a directory for triage events emitted by CI or the
offline PR queue. Each event contains metadata about a failure, pointers to
logs, and optional remediation hints. When a new event is detected the service
performs the following steps:

1. Classify the failure type (lint, type, flaky test, infrastructure, etc.).
2. Persist an incident workspace under ``tools/offline_pr_queue/triage`` with
   a JSON manifest, copied artefacts, and a policy decision trail.
3. Invoke the remediation CLI so the workflow automation stack can queue the
   appropriate auto-fixer agent.

The design keeps execution offline-first: events are pulled from the local
filesystem, remediation is delegated to the Rust workflow CLI, and each action
is journaled for later audit. The default remediation command targets the
``workflow`` crate binary introduced alongside this service. Supply a
pre-built binary with ``--remediation-cli`` for fully air-gapped execution.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import shutil
import signal
import subprocess
import sys
import textwrap
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, Optional

INCIDENT_ROOT = Path("tools/offline_pr_queue/triage")
SUPPORTED_EXTENSIONS = {".json", ".ndjson"}


@dataclass(slots=True)
class TriageEvent:
    """Representation of an inbound triage event."""

    event_id: str
    source_path: Path
    payload: Dict[str, Any]

    @classmethod
    def from_file(cls, path: Path) -> "TriageEvent":
        text = path.read_text(encoding="utf-8")
        if path.suffix == ".ndjson":
            # Process newline-delimited events individually and wrap the batch.
            lines = [json.loads(line) for line in text.splitlines() if line.strip()]
            payload: Dict[str, Any] = {
                "batch": lines,
                "batched": True,
                "event_id": lines[0].get("event_id")
                if lines and isinstance(lines[0], dict)
                else path.stem,
            }
        else:
            payload = json.loads(text)
        event_id = str(payload.get("event_id") or path.stem)
        return cls(event_id=event_id, source_path=path, payload=payload)

    @property
    def log_path(self) -> Optional[Path]:
        log_value = self.payload.get("log") or self.payload.get("log_path")
        if not log_value:
            return None
        return Path(log_value)

    @property
    def hinted_category(self) -> Optional[str]:
        category = self.payload.get("category") or self.payload.get("hint")
        if isinstance(category, str):
            return category.lower()
        return None


@dataclass(slots=True)
class ClassificationResult:
    category: str
    confidence: float
    signals: list[str] = field(default_factory=list)


class FailureClassifier:
    """Classify failures using heuristics on logs and payload metadata."""

    def classify(self, event: TriageEvent) -> ClassificationResult:
        signals: list[str] = []
        text = ""
        if event.log_path and event.log_path.exists():
            try:
                text = event.log_path.read_text(encoding="utf-8", errors="ignore")
            except OSError as exc:  # pragma: no cover - exercised via tests
                signals.append(f"failed_to_read_log:{exc}")
        elif isinstance(event.payload.get("message"), str):
            text = event.payload["message"]

        text_lower = text.lower()
        hinted = event.hinted_category
        if hinted:
            signals.append(f"hint:{hinted}")

        category = "unknown"
        confidence = 0.45

        patterns = {
            "lint": ["eslint", "flake8", "cargo fmt", "lint"],
            "type": ["mypy", "pyright", "tsc", "typeerror", "typing"],
            "test": ["pytest", "unittest", "assert", "test failed"],
            "flaky_test": ["flake", "flaky", "rerun", "retry"],
            "infrastructure": ["docker", "network", "timeout", "connection"],
        }

        for label, keys in patterns.items():
            for key in keys:
                if key in text_lower:
                    category = label
                    signals.append(f"match:{label}:{key}")
                    confidence = max(confidence, 0.75 if hinted == label else 0.65)
                    break
            if category == label:
                break

        if category == "test" and "timeout" in text_lower:
            category = "flaky_test"
            signals.append("promote:test->flaky")
            confidence = max(confidence, 0.7)

        if hinted and category == "unknown":
            category = hinted
            confidence = max(confidence, 0.6)

        return ClassificationResult(category=category, confidence=confidence, signals=signals)


class ArtifactStore:
    """Persist incident artefacts and manifests for offline review."""

    def __init__(self, root: Path = INCIDENT_ROOT) -> None:
        self.root = root
        self.root.mkdir(parents=True, exist_ok=True)

    def create_incident_dir(self, event_id: str) -> Path:
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        path = self.root / timestamp / event_id
        path.mkdir(parents=True, exist_ok=True)
        return path

    def store(
        self,
        event: TriageEvent,
        classification: ClassificationResult,
        policy: Dict[str, Any],
    ) -> Path:
        incident_dir = self.create_incident_dir(event.event_id)
        manifest = {
            "event_id": event.event_id,
            "source": str(event.source_path),
            "payload": event.payload,
            "classification": {
                "category": classification.category,
                "confidence": classification.confidence,
                "signals": classification.signals,
            },
            "policy_decision": policy,
            "recorded_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        }
        (incident_dir / "manifest.json").write_text(
            json.dumps(manifest, indent=2, sort_keys=True),
            encoding="utf-8",
        )

        if event.log_path and event.log_path.exists():
            target = incident_dir / event.log_path.name
            if event.log_path.is_file():
                shutil.copy2(event.log_path, target)
            else:
                # If log path is a directory copy recursively.
                shutil.copytree(event.log_path, target, dirs_exist_ok=True)

        return incident_dir


class RemediationWorkflowTrigger:
    """Invoke workflow CLI auto-fixers for the classified incident."""

    def __init__(self, cli: Iterable[str], environment: Optional[Dict[str, str]] = None) -> None:
        self.cli = list(cli)
        self.environment = environment or {}

    def _command_for(self, classification: ClassificationResult) -> list[str]:
        mapping = {
            "lint": "lint",
            "type": "type",
            "flaky_test": "flaky-test",
            "test": "flaky-test",
            "infrastructure": "refactor",
        }
        fixer = mapping.get(classification.category, "refactor")
        return [*self.cli, "auto-fix", fixer]

    def trigger(
        self,
        classification: ClassificationResult,
        incident_dir: Path,
        event: TriageEvent,
        dry_run: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        command = self._command_for(classification)
        command.extend(["--incident", str(incident_dir)])
        command.extend(["--reason", classification.category])
        if dry_run:
            command.append("--dry-run")

        env = os.environ.copy()
        env.update(self.environment)
        env.setdefault("TRIAGE_EVENT_ID", event.event_id)
        env.setdefault("TRIAGE_CLASSIFICATION", classification.category)
        env.setdefault("TRIAGE_CONFIDENCE", f"{classification.confidence:.2f}")

        print(f"[TRIAGE] Triggering remediation: {' '.join(command)}")
        return subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )


class DirectoryEventSource:
    """Poll a directory for new triage events."""

    def __init__(self, directory: Path, poll_interval: float = 1.0, run_once: bool = False) -> None:
        self.directory = directory
        self.poll_interval = poll_interval
        self.run_once = run_once
        self._seen: set[Path] = set()
        self.directory.mkdir(parents=True, exist_ok=True)

    async def __aiter__(self) -> Iterable[TriageEvent]:
        while True:
            events = self._collect_events()
            if not events and self.run_once and self._seen:
                break
            for event in events:
                yield event
            await asyncio.sleep(self.poll_interval)
            if self.run_once and self._seen:
                break

    def _collect_events(self) -> list[TriageEvent]:
        collected: list[TriageEvent] = []
        for path in sorted(self.directory.iterdir()):
            if path in self._seen or path.suffix not in SUPPORTED_EXTENSIONS:
                continue
            try:
                event = TriageEvent.from_file(path)
            except Exception as exc:  # pragma: no cover - defensive fallback
                print(f"[TRIAGE] Failed to parse event {path}: {exc}", file=sys.stderr)
                self._seen.add(path)
                continue
            self._seen.add(path)
            collected.append(event)
        return collected


class TriageService:
    """High-level orchestration for the triage loop."""

    def __init__(
        self,
        event_source: DirectoryEventSource,
        classifier: FailureClassifier,
        store: ArtifactStore,
        trigger: RemediationWorkflowTrigger,
        *,
        dry_run: bool = False,
    ) -> None:
        self.event_source = event_source
        self.classifier = classifier
        self.store = store
        self.trigger = trigger
        self.dry_run = dry_run
        self._stopped = asyncio.Event()

    async def run(self) -> None:
        loop = asyncio.get_running_loop()
        loop.add_signal_handler(signal.SIGINT, self._stopped.set)
        loop.add_signal_handler(signal.SIGTERM, self._stopped.set)

        async for event in self.event_source:
            if self._stopped.is_set():
                break
            await self._handle_event(event)
            if self.event_source.run_once:
                break

    async def _handle_event(self, event: TriageEvent) -> None:
        print(f"[TRIAGE] Processing event {event.event_id}")
        classification = self.classifier.classify(event)
        policy = {
            "decision": "allow",
            "reason": f"auto-remediate::{classification.category}",
            "signals": classification.signals,
        }
        incident_dir = self.store.store(event, classification, policy)

        result = self.trigger.trigger(classification, incident_dir, event, dry_run=self.dry_run)
        command_display = result.args if isinstance(result.args, str) else ' '.join(result.args)
        (incident_dir / "remediation.log").write_text(
            textwrap.dedent(
                f"""
                command: {command_display}
                returncode: {result.returncode}
                stdout:\n{result.stdout}
                stderr:\n{result.stderr}
                """
            ).strip()
            + "\n",
            encoding="utf-8",
        )
        if result.returncode != 0:
            print(
                f"[TRIAGE] Remediation command failed for {event.event_id}: {result.returncode}",
                file=sys.stderr,
            )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the NOA ARK OS triage service")
    parser.add_argument(
        "--events",
        type=Path,
        default=Path("out/triage/events"),
        help="Directory containing queued triage events.",
    )
    parser.add_argument(
        "--incident-root",
        type=Path,
        default=INCIDENT_ROOT,
        help="Where to store incident artefacts.",
    )
    parser.add_argument(
        "--remediation-cli",
        nargs=argparse.REMAINDER,
        default=[
            "cargo",
            "run",
            "-p",
            "noa_workflow",
            "--bin",
            "workflow-cli",
            "--",
        ],
        help=(
            "Command used to invoke the workflow CLI (specify everything after the "
            "flag; defaults to running the workspace binary through Cargo)."
        ),
    )
    parser.add_argument(
        "--poll-interval",
        type=float,
        default=1.0,
        help="Polling interval in seconds for the event directory.",
    )
    parser.add_argument(
        "--run-once",
        action="store_true",
        help="Process currently queued events and exit.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Record incidents without executing remediation commands.",
    )
    return parser


def main(argv: Optional[list[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    cli = list(args.remediation_cli)
    if not cli:
        parser.error("--remediation-cli must include at least one argument")

    cleaned_cli: list[str] = []
    mutator = iter(cli)
    for token in mutator:
        if token == "--run-once":
            args.run_once = True
            continue
        if token == "--dry-run":
            args.dry_run = True
            continue
        if token == "--":
            cleaned_cli.append(token)
            cleaned_cli.extend(list(mutator))
            break
        cleaned_cli.append(token)

    if not cleaned_cli:
        parser.error("--remediation-cli must include at least one argument")
    cli = cleaned_cli

    source = DirectoryEventSource(
        directory=args.events,
        poll_interval=args.poll_interval,
        run_once=args.run_once,
    )
    classifier = FailureClassifier()
    store = ArtifactStore(args.incident_root)
    trigger = RemediationWorkflowTrigger(cli)
    service = TriageService(source, classifier, store, trigger, dry_run=args.dry_run)

    print(
        "[TRIAGE] Starting event-driven triage service\n"
        f"        events: {args.events}\n"
        f"        incident_root: {args.incident_root}\n"
        f"        remediation_cli: {' '.join(cli)}\n"
        f"        dry_run: {args.dry_run}\n"
        f"        run_once: {args.run_once}"
    )

    asyncio.run(service.run())
    return 0


if __name__ == "__main__":
    sys.exit(main())
