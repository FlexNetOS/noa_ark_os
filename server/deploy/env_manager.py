#!/usr/bin/env python3
"""Helper utilities for Compose and Helm deployments used in CI pipelines."""

from __future__ import annotations

import argparse
import datetime as dt
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional
from urllib import request, error

REPO_ROOT = Path(__file__).resolve().parents[2]
COMPOSE_FILE = REPO_ROOT / "server" / "docker-compose.yml"
HELM_CHART = REPO_ROOT / "server" / "helm"
EVIDENCE_DIR = REPO_ROOT / "var" / "telemetry" / "deploy"
EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)


def _env_with_password(password: Optional[str]) -> Dict[str, str]:
    env = os.environ.copy()
    env.setdefault("POSTGRES_PASSWORD", password or "noa")
    if password:
        env["POSTGRES_PASSWORD"] = password
    return env


def _run(command: List[str], env: Optional[Dict[str, str]] = None) -> None:
    print(f"$ {' '.join(command)}", flush=True)
    subprocess.run(command, check=True, env=env)


def _capture_compose_logs(tag: str, env: Dict[str, str]) -> Path:
    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    destination = EVIDENCE_DIR / f"compose-{timestamp}-{tag}.log"
    result = subprocess.run(
        ["docker", "compose", "-f", str(COMPOSE_FILE), "logs", "--no-color"],
        check=True,
        capture_output=True,
        text=True,
        env=env,
    )
    destination.write_text(result.stdout)
    print(f"Captured compose logs at {destination}")
    return destination


def _wait_for_health(url: str, timeout: int) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with request.urlopen(url, timeout=5) as response:
                if 200 <= response.status < 300:
                    print(f"Health check succeeded for {url}")
                    return
        except error.URLError as exc:  # pragma: no cover - best-effort network probe
            print(f"Waiting for health at {url}: {exc}")
        time.sleep(2)
    raise TimeoutError(f"Timed out waiting for {url}")


def command_compose_up(args: argparse.Namespace) -> None:
    env = _env_with_password(args.password)
    _run(["docker", "compose", "-f", str(COMPOSE_FILE), "up", "-d"], env=env)
    if args.wait:
        _wait_for_health(args.health_url, args.timeout)
    _capture_compose_logs("up", env)


def command_compose_down(args: argparse.Namespace) -> None:
    env = _env_with_password(args.password)
    cmd = ["docker", "compose", "-f", str(COMPOSE_FILE), "down"]
    if args.prune:
        cmd.append("-v")
    _capture_compose_logs("down", env)
    _run(cmd, env=env)


def command_compose_logs(args: argparse.Namespace) -> None:
    env = _env_with_password(args.password)
    _capture_compose_logs(args.tag, env)


def command_compose_health(args: argparse.Namespace) -> None:
    _wait_for_health(args.health_url, args.timeout)


def command_helm_install(args: argparse.Namespace) -> None:
    cmd = [
        "helm",
        "install",
        args.release,
        str(HELM_CHART),
        "-f",
        str(args.values),
    ]
    if args.namespace:
        cmd.extend(["-n", args.namespace])
    _run(cmd)


def command_helm_upgrade(args: argparse.Namespace) -> None:
    cmd = [
        "helm",
        "upgrade",
        args.release,
        str(HELM_CHART),
        "-f",
        str(args.values),
        "--install",
    ]
    if args.namespace:
        cmd.extend(["-n", args.namespace])
    _run(cmd)


def command_helm_uninstall(args: argparse.Namespace) -> None:
    cmd = ["helm", "uninstall", args.release]
    if args.namespace:
        cmd.extend(["-n", args.namespace])
    _run(cmd)


def _helm_subparser(subparsers: argparse._SubParsersAction, name: str, help_text: str, func) -> None:
    parser = subparsers.add_parser(name, help=help_text)
    parser.add_argument("--release", default="noa-server", help="Helm release name")
    parser.add_argument("--namespace", default="default", help="Kubernetes namespace")
    parser.add_argument(
        "--values",
        default=HELM_CHART / "values.yaml",
        type=Path,
        help="values file to apply",
    )
    parser.set_defaults(func=func)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="NOA server deployment helper")
    subparsers = parser.add_subparsers(dest="command", required=True)

    compose_up = subparsers.add_parser("compose-up", help="Start docker compose environment")
    compose_up.add_argument("--password", help="Postgres password override")
    compose_up.add_argument("--wait", action="store_true", help="Wait for HTTP health")
    compose_up.add_argument("--health-url", default="http://localhost:8080/health")
    compose_up.add_argument("--timeout", type=int, default=120)
    compose_up.set_defaults(func=command_compose_up)

    compose_down = subparsers.add_parser("compose-down", help="Stop docker compose environment")
    compose_down.add_argument("--password", help="Postgres password override")
    compose_down.add_argument("--prune", action="store_true", help="Remove volumes")
    compose_down.set_defaults(func=command_compose_down)

    compose_logs = subparsers.add_parser("compose-logs", help="Capture compose logs to telemetry directory")
    compose_logs.add_argument("--password", help="Postgres password override")
    compose_logs.add_argument("--tag", default="manual", help="Tag appended to log filename")
    compose_logs.set_defaults(func=command_compose_logs)

    compose_health = subparsers.add_parser("compose-health", help="Poll health endpoint")
    compose_health.add_argument("--health-url", default="http://localhost:8080/health")
    compose_health.add_argument("--timeout", type=int, default=120)
    compose_health.set_defaults(func=command_compose_health)

    _helm_subparser(subparsers, "helm-install", "helm install release", command_helm_install)
    _helm_subparser(subparsers, "helm-upgrade", "helm upgrade or install release", command_helm_upgrade)
    _helm_subparser(subparsers, "helm-uninstall", "helm uninstall release", command_helm_uninstall)

    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        sys.exit(1)
