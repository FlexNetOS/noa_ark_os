from __future__ import annotations

import argparse
import json
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional

from core.kernel.manifest import KernelManifest, KernelService, load_manifest
from scripts.packaging.kernel_package import build_artifacts, validate_artifacts

DEFAULT_PROFILE_PATH = Path(__file__).with_name("single_host_profile.json")
STATE_ROOT = Path("var/state/single-host")
TELEMETRY_ROOT = Path("var/telemetry")


@dataclass
class ServiceStatus:
    service: str
    ready: bool
    readiness_file: Path
    last_transition: Optional[float]


class SingleHostOrchestrator:
    def __init__(self, root: Path, profile_path: Optional[Path] = None):
        self.root = root
        self.profile_path = profile_path or DEFAULT_PROFILE_PATH
        self.profile = self._load_profile(self.profile_path)
        manifest_path = (self.profile_path.parent / self.profile["manifest"]).resolve()
        self.manifest: KernelManifest = load_manifest(manifest_path)
        self.state_dir = (root / STATE_ROOT).resolve()
        self.telemetry_dir = (root / TELEMETRY_ROOT).resolve()
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self.telemetry_dir.mkdir(parents=True, exist_ok=True)

    @staticmethod
    def _load_profile(path: Path) -> Dict[str, object]:
        data = json.loads(path.read_text())
        required_keys = {"profile", "manifest", "boot", "health", "observability", "resources"}
        missing = required_keys.difference(data)
        if missing:
            raise ValueError(f"Profile missing keys: {sorted(missing)}")
        return data

    def inventory(self) -> List[KernelService]:
        return self.manifest.services_in_boot_order(self.profile["profile"])

    def readiness_file(self, service: KernelService) -> Path:
        template = self.profile["health"]["readinessFile"]
        path = template.format(service=service.id)
        return self.state_dir / path

    def _write_state_file(self, service: KernelService) -> None:
        path = self.readiness_file(service)
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "service": service.id,
            "version": service.version,
            "timestamp": time.time(),
            "dependencies": service.dependencies,
        }
        path.write_text(json.dumps(payload, indent=2, sort_keys=True))

    def _generate_metrics(self, service: KernelService) -> None:
        metrics_map = self.profile["observability"].get("metrics", {})
        metrics_path = metrics_map.get(service.id)
        if metrics_path:
            destination = self.telemetry_dir / metrics_path
            destination.parent.mkdir(parents=True, exist_ok=True)
            metric_payload = {
                "service": service.id,
                "version": service.version,
                "health": "ready",
                "timestamp": time.time(),
            }
            destination.write_text(json.dumps(metric_payload, indent=2, sort_keys=True))

    def start(self) -> List[ServiceStatus]:
        artifact_dir = self.root / "build/artifacts"
        artifact_dir.mkdir(parents=True, exist_ok=True)
        build_artifacts(artifact_dir, manifest=self.manifest)
        validate_artifacts(artifact_dir, manifest=self.manifest)

        statuses = []
        for service in self.inventory():
            self._write_state_file(service)
            self._generate_metrics(service)
            statuses.append(
                ServiceStatus(
                    service=service.id,
                    ready=True,
                    readiness_file=self.readiness_file(service),
                    last_transition=time.time(),
                )
            )
        return statuses

    def stop(self) -> None:
        if self.state_dir.exists():
            shutil.rmtree(self.state_dir)
            self.state_dir.mkdir(parents=True, exist_ok=True)
        if self.telemetry_dir.exists():
            shutil.rmtree(self.telemetry_dir)
            self.telemetry_dir.mkdir(parents=True, exist_ok=True)

    def status(self) -> List[ServiceStatus]:
        statuses: List[ServiceStatus] = []
        for service in self.inventory():
            readiness_file = self.readiness_file(service)
            ready = readiness_file.exists()
            timestamp = None
            if ready:
                data = json.loads(readiness_file.read_text())
                timestamp = data.get("timestamp")
            statuses.append(
                ServiceStatus(
                    service=service.id,
                    ready=ready,
                    readiness_file=readiness_file,
                    last_transition=timestamp,
                )
            )
        return statuses

    def health_check(self) -> Dict[str, bool]:
        results: Dict[str, bool] = {}
        for status in self.status():
            results[status.service] = status.ready
        return results

    def snapshot(self) -> Path:
        bundle_path = self.profile["observability"]["bundlePath"]
        destination = (self.root / bundle_path).resolve()
        destination.parent.mkdir(parents=True, exist_ok=True)
        archive_path = shutil.make_archive(str(destination), "gztar", root_dir=self.telemetry_dir)
        return Path(archive_path)

    def resource_envelopes(self) -> Dict[str, Dict[str, int]]:
        return self.profile["resources"]


def _print_inventory(services: Iterable[KernelService]) -> None:
    print("SERVICE\tVERSION\tROLE")
    for service in services:
        print(f"{service.id}\t{service.version}\t{service.boot.get('role', 'n/a')}")


def _print_status(statuses: Iterable[ServiceStatus]) -> None:
    print("SERVICE\tREADY\tREADINESS_FILE")
    for status in statuses:
        print(f"{status.service}\t{status.ready}\t{status.readiness_file}")


def _command_inventory(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    _print_inventory(orchestrator.inventory())


def _command_start(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    statuses = orchestrator.start()
    _print_status(statuses)


def _command_stop(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    orchestrator.stop()
    print("Stopped single-host services")


def _command_status(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    _print_status(orchestrator.status())


def _command_health(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    health = orchestrator.health_check()
    print(json.dumps(health, indent=2, sort_keys=True))


def _command_snapshot(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    archive = orchestrator.snapshot()
    print(f"Created snapshot at {archive}")


def _command_resources(args: argparse.Namespace) -> None:
    orchestrator = SingleHostOrchestrator(Path(args.root))
    print(json.dumps(orchestrator.resource_envelopes(), indent=2, sort_keys=True))


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Single-host AgentOS orchestrator")
    parser.add_argument("--root", default=".", help="Repository root containing state directories")

    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("inventory", help="List services in boot order").set_defaults(func=_command_inventory)
    subparsers.add_parser("start", help="Start services deterministically").set_defaults(func=_command_start)
    subparsers.add_parser("stop", help="Stop services and reset state").set_defaults(func=_command_stop)
    subparsers.add_parser("status", help="Print service readiness state").set_defaults(func=_command_status)
    subparsers.add_parser("health", help="Run health checks").set_defaults(func=_command_health)
    subparsers.add_parser("snapshot", help="Produce observability snapshot").set_defaults(func=_command_snapshot)
    subparsers.add_parser("resources", help="Print resource envelope guidance").set_defaults(func=_command_resources)

    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = create_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
