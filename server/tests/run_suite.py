#!/usr/bin/env python3
"""End-to-end verification harness for the NOA server stack."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, List, Sequence

import grpc
import requests
from grpc_tools import protoc

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_COMPOSE = Path(__file__).resolve().with_name("docker-compose.test.yml")
DEFAULT_PROJECT = "noa-server-stack"
HTTP_TARGET = "http://localhost:8787"
GRPC_TARGET = "localhost:50051"
TARGET_P95_MS = 100.0
TARGET_ERROR_RATE = 0.001


@dataclass
class MetricSnapshot:
    name: str
    samples: int
    latencies_ms: List[float]
    errors: int
    duration: float

    @property
    def p95_ms(self) -> float:
        if not self.latencies_ms:
            return 0.0
        return percentile(self.latencies_ms, 95.0)

    @property
    def throughput_rps(self) -> float:
        return (self.samples - self.errors) / self.duration if self.duration else 0.0

    @property
    def error_rate(self) -> float:
        if not self.samples:
            return 0.0
        return self.errors / self.samples

    def to_dict(self) -> dict:
        return {
            "name": self.name,
            "samples": self.samples,
            "duration_sec": round(self.duration, 3),
            "p95_ms": round(self.p95_ms, 3),
            "throughput_rps": round(self.throughput_rps, 3),
            "error_rate": round(self.error_rate, 5),
        }


def percentile(values: Sequence[float], pct: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    k = (len(ordered) - 1) * (pct / 100.0)
    f = int(k)
    c = min(f + 1, len(ordered) - 1)
    if f == c:
        return ordered[f]
    d0 = ordered[f] * (c - k)
    d1 = ordered[c] * (k - f)
    return d0 + d1


def detect_compose_command() -> List[str]:
    for candidate in ("docker compose", "docker-compose"):
        parts = candidate.split()
        try:
            subprocess.run(parts + ["version"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return parts
        except (OSError, subprocess.CalledProcessError):
            continue
    raise RuntimeError("docker compose not available")


class ComposeStack:
    def __init__(self, compose_file: Path, project_name: str):
        self.compose_file = compose_file
        self.project_name = project_name
        self._cmd = detect_compose_command()

    @property
    def env(self) -> dict:
        env = os.environ.copy()
        env["COMPOSE_PROJECT_NAME"] = self.project_name
        return env

    @property
    def network_name(self) -> str:
        return f"{self.project_name}_default"

    def up(self) -> None:
        subprocess.run(
            self._cmd + ["-f", str(self.compose_file), "up", "-d", "--build"],
            check=True,
            env=self.env,
            cwd=REPO_ROOT,
        )

    def down(self) -> None:
        subprocess.run(
            self._cmd + ["-f", str(self.compose_file), "down", "-v"],
            check=True,
            env=self.env,
            cwd=REPO_ROOT,
        )


def wait_for_http(url: str, timeout: float = 90.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            response = requests.get(url, timeout=5)
            if response.status_code < 400:
                return
        except requests.RequestException:
            pass
        time.sleep(2)
    raise TimeoutError(f"timed out waiting for {url}")


def wait_for_grpc(target: str, timeout: float = 90.0) -> None:
    channel = grpc.insecure_channel(target)
    grpc.channel_ready_future(channel).result(timeout=timeout)


def compile_proto(tempdir: Path) -> None:
    proto_dir = REPO_ROOT / "server" / "ui_api" / "proto"
    result = protoc.main(
        [
            "protoc",
            f"-I{proto_dir}",
            f"--python_out={tempdir}",
            f"--grpc_python_out={tempdir}",
            str(proto_dir / "ui_schema.proto"),
        ]
    )
    if result != 0:
        raise RuntimeError("failed to compile proto definitions")


def import_stubs(tempdir: Path):
    sys.path.insert(0, str(tempdir))
    import ui_schema_pb2 as schema_pb2  # type: ignore
    import ui_schema_pb2_grpc as schema_pb2_grpc  # type: ignore
    return schema_pb2, schema_pb2_grpc


def collect_metrics(name: str, iterations: int, fn: Callable[[], None]) -> MetricSnapshot:
    latencies: List[float] = []
    errors = 0
    start = time.perf_counter()
    for _ in range(iterations):
        tick = time.perf_counter()
        try:
            fn()
            latencies.append((time.perf_counter() - tick) * 1000)
        except Exception:
            errors += 1
    duration = time.perf_counter() - start
    return MetricSnapshot(name=name, samples=iterations, latencies_ms=latencies, errors=errors, duration=duration)


def run_http_checks(base_url: str, iterations: int) -> MetricSnapshot:
    def call():
        response = requests.get(f"{base_url}/ui/pages/integration-suite", timeout=5)
        response.raise_for_status()

    return collect_metrics("http", iterations, call)


def run_grpc_checks(target: str, schema_pb2, schema_pb2_grpc, iterations: int) -> MetricSnapshot:
    channel = grpc.insecure_channel(target)
    stub = schema_pb2_grpc.UiSchemaServiceStub(channel)

    def call():
        stub.GetPage(schema_pb2.PageRequest(page_id="integration-suite"), timeout=5)

    return collect_metrics("grpc", iterations, call)


def enforce_thresholds(snapshot: MetricSnapshot) -> None:
    if snapshot.p95_ms >= TARGET_P95_MS:
        raise RuntimeError(f"{snapshot.name} p95 latency {snapshot.p95_ms:.2f}ms >= {TARGET_P95_MS}ms")
    if snapshot.error_rate >= TARGET_ERROR_RATE:
        raise RuntimeError(
            f"{snapshot.name} error rate {snapshot.error_rate:.4f} >= {TARGET_ERROR_RATE}"
        )


def run_k6(script: Path, network: str, vus: int, duration: str) -> None:
    cmd = [
        "docker",
        "run",
        "--rm",
        "--network",
        network,
        "-e",
        "BASE_URL=http://ui-api:8787",
        "-e",
        f"K6_VUS={vus}",
        "-e",
        f"K6_DURATION={duration}",
        "-v",
        f"{script.parent}:/scripts:ro",
        "grafana/k6",
        "run",
        f"/scripts/{script.name}",
    ]
    subprocess.run(cmd, check=True, cwd=REPO_ROOT)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compose-file", type=Path, default=DEFAULT_COMPOSE)
    parser.add_argument("--project-name", default=DEFAULT_PROJECT)
    parser.add_argument("--http-url", default=HTTP_TARGET)
    parser.add_argument("--grpc-target", default=GRPC_TARGET)
    parser.add_argument("--samples", type=int, default=20, help="number of HTTP/gRPC samples")
    parser.add_argument("--metrics-output", type=Path, help="write metrics JSON to this path")
    parser.add_argument("--k6-script", type=Path, help="k6 script to execute after functional checks")
    parser.add_argument("--k6-vus", type=int, default=20)
    parser.add_argument("--k6-duration", default="20s")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    stack = ComposeStack(args.compose_file, args.project_name)
    metrics_payload = {"timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        compile_proto(tmp_path)
        schema_pb2, schema_pb2_grpc = import_stubs(tmp_path)

        try:
            stack.up()
            wait_for_http(f"{args.http_url}/healthz")
            wait_for_http(f"{args.http_url}/readyz")
            wait_for_grpc(args.grpc_target)

            http_metrics = run_http_checks(args.http_url, args.samples)
            grpc_metrics = run_grpc_checks(args.grpc_target, schema_pb2, schema_pb2_grpc, args.samples)
            enforce_thresholds(http_metrics)
            enforce_thresholds(grpc_metrics)

            metrics_payload["http"] = http_metrics.to_dict()
            metrics_payload["grpc"] = grpc_metrics.to_dict()

            print(json.dumps(metrics_payload, indent=2))

            if args.k6_script:
                run_k6(args.k6_script, stack.network_name, args.k6_vus, args.k6_duration)
        finally:
            stack.down()

    if args.metrics_output:
        args.metrics_output.write_text(json.dumps(metrics_payload, indent=2))


if __name__ == "__main__":
    main()
