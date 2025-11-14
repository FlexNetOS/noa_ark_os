from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

from core.kernel.manifest import KernelManifest, load_manifest
from core.kernel.security import (
    CapabilityTokenError,
    KernelTokenClaims,
    verify_capability_token,
)


@dataclass(frozen=True)
class GatewayRequest:
    client_id: str
    service: str
    path: str
    method: str
    token: Optional[str] = None


@dataclass(frozen=True)
class GatewayResponse:
    status: int
    upstream: Optional[str]
    message: str


@dataclass
class PolicyRule:
    service_id: str
    allowed_methods: List[str]
    allowed_paths: List[str]
    requires_authentication: bool = True
    rate_limit_per_minute: int = 60
    fs_scope: Optional[str] = None
    network_scope: Optional[str] = None

    def allows(self, method: str, path: str) -> bool:
        method_allowed = method.upper() in {m.upper() for m in self.allowed_methods}
        path_allowed = any(path.startswith(prefix) for prefix in self.allowed_paths)
        return method_allowed and path_allowed


@dataclass
class GatewayConfig:
    manifest: KernelManifest
    policy_rules: Dict[str, PolicyRule]


class RateLimiter:
    def __init__(self) -> None:
        self._buckets: Dict[str, List[float]] = {}

    def allow(self, key: str, limit: int) -> bool:
        now = time.time()
        window_start = now - 60
        history = self._buckets.setdefault(key, [])
        history[:] = [timestamp for timestamp in history if timestamp >= window_start]
        if len(history) >= limit:
            return False
        history.append(now)
        return True


@dataclass
class GatewayTelemetry:
    requests_total: int = 0
    rejected_policy: int = 0
    rejected_auth: int = 0
    rejected_rate: int = 0
    latencies_ms: List[float] = field(default_factory=list)
    events: List[Dict[str, object]] = field(default_factory=list)

    def record(self, event: Dict[str, object]) -> None:
        self.events.append(event)

    def export(self, destination: Path) -> None:
        destination.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "requests_total": self.requests_total,
            "rejected_policy": self.rejected_policy,
            "rejected_auth": self.rejected_auth,
            "rejected_rate": self.rejected_rate,
            "p95_latency_ms": self._percentile(95),
            "events": self.events[-200:],
        }
        destination.write_text(json.dumps(payload, indent=2, sort_keys=True))

    def _percentile(self, percentile: float) -> float:
        if not self.latencies_ms:
            return 0.0
        sorted_latencies = sorted(self.latencies_ms)
        index = min(len(sorted_latencies) - 1, int(len(sorted_latencies) * percentile / 100))
        return sorted_latencies[index]


class Gateway:
    def __init__(self, config: GatewayConfig, telemetry_dir: Optional[Path] = None) -> None:
        self.config = config
        self.telemetry = GatewayTelemetry()
        self.rate_limiter = RateLimiter()
        self.telemetry_dir = telemetry_dir or Path("services/gateway/telemetry")

    def handle(self, request: GatewayRequest) -> GatewayResponse:
        start = time.perf_counter()
        rule = self.config.policy_rules.get(request.service)
        if not self._service_registered(request.service):
            self.telemetry.rejected_policy += 1
            return GatewayResponse(status=404, upstream=None, message="unknown service")

        if not rule:
            self.telemetry.rejected_policy += 1
            return GatewayResponse(status=403, upstream=None, message="no policy rule configured")

        token_claims: Optional[KernelTokenClaims] = None
        effective_rate_limit = rule.rate_limit_per_minute
        if rule.requires_authentication:
            if not request.token:
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=401, upstream=None, message="authentication required")
            try:
                token_claims = verify_capability_token(request.token)
            except CapabilityTokenError as exc:
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=401, upstream=None, message=f"invalid capability token: {exc}")

            if token_claims.client_id != request.client_id:
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=403, upstream=None, message="token client mismatch")

            if not token_claims.allows_scope("fs", rule.fs_scope):
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=403, upstream=None, message="missing fs scope")

            if not token_claims.allows_scope("network", rule.network_scope):
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=403, upstream=None, message="missing network scope")

            if not token_claims.allows_rate(rule.rate_limit_per_minute):
                self.telemetry.rejected_auth += 1
                return GatewayResponse(status=403, upstream=None, message="rate scope below requirement")

            effective_rate_limit = min(rule.rate_limit_per_minute, token_claims.rate_limit_per_minute)

        if not rule.allows(request.method, request.path):
            self.telemetry.rejected_policy += 1
            return GatewayResponse(status=403, upstream=None, message="policy violation")

        if not self.rate_limiter.allow(f"{request.client_id}:{request.service}", effective_rate_limit):
            self.telemetry.rejected_rate += 1
            return GatewayResponse(status=429, upstream=None, message="rate limit exceeded")

        latency = (time.perf_counter() - start) * 1000
        self.telemetry.requests_total += 1
        self.telemetry.latencies_ms.append(latency)
        self.telemetry.record(
            {
                "client_id": request.client_id,
                "service": request.service,
                "path": request.path,
                "method": request.method,
                "latency_ms": latency,
                "timestamp": time.time(),
                "fs_scope": rule.fs_scope,
                "network_scope": rule.network_scope,
                "token_rate_limit": token_claims.rate_limit_per_minute if token_claims else None,
            }
        )
        return GatewayResponse(status=200, upstream=request.service, message="forwarded")

    def export_telemetry(self, filename: str = "gateway-metrics.json") -> Path:
        destination = self.telemetry_dir / filename
        self.telemetry.export(destination)
        return destination

    def _service_registered(self, service_id: str) -> bool:
        return service_id in self.config.manifest.services


def build_default_config(manifest_path: Optional[Path] = None) -> GatewayConfig:
    manifest = load_manifest(manifest_path)
    policy_rules = {
        "gateway": PolicyRule(
            service_id="gateway",
            allowed_methods=["GET", "POST"],
            allowed_paths=["/v1/policy", "/v1/metrics"],
            requires_authentication=True,
            rate_limit_per_minute=120,
            fs_scope="fs.policy.read",
            network_scope="net.gateway",
        ),
        "runtime-manager": PolicyRule(
            service_id="runtime-manager",
            allowed_methods=["POST"],
            allowed_paths=["/v1/schedule"],
            requires_authentication=True,
            rate_limit_per_minute=60,
            fs_scope="fs.runtime.control",
            network_scope="net.runtime",
        ),
    }
    return GatewayConfig(manifest=manifest, policy_rules=policy_rules)
