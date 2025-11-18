from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Optional

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from core.kernel.security import issue_capability_token
from services.gateway.service import Gateway, GatewayRequest, PolicyRule, build_default_config


def _service_identity(service_id: str) -> str:
    return f"self-heal::{service_id}"


def _issue_service_token(service_id: str, rule: PolicyRule) -> tuple[str, Optional[str]]:
    client_id = _service_identity(service_id)
    if not rule.requires_authentication:
        return client_id, None

    fs_scopes = [rule.fs_scope] if rule.fs_scope else []
    network_scopes = [rule.network_scope] if rule.network_scope else []
    token = issue_capability_token(
        client_id=client_id,
        fs_scopes=fs_scopes,
        network_scopes=network_scopes,
        rate_limit_per_minute=rule.rate_limit_per_minute,
    )
    return client_id, token


def _exercise_service(gateway: Gateway, service_id: str, rule) -> Dict[str, object]:
    if not rule.allowed_methods or not rule.allowed_paths:
        return {
            "service": service_id,
            "status": "skipped",
            "reason": "no policy routes configured",
        }

    client_id, token = _issue_service_token(service_id, rule)
    request = GatewayRequest(
        client_id=client_id,
        service=service_id,
        path=rule.allowed_paths[0],
        method=rule.allowed_methods[0],
        token=token,
    )
    response = gateway.handle(request)
    return {
        "service": service_id,
        "status": response.status,
        "message": response.message,
    }


def run_self_heal(output: Path) -> Dict[str, object]:
    config = build_default_config()
    telemetry_dir = output.parent / "telemetry"
    telemetry_dir.mkdir(parents=True, exist_ok=True)
    gateway = Gateway(config, telemetry_dir=telemetry_dir)
    results: List[Dict[str, object]] = []

    for service_id, rule in config.policy_rules.items():
        results.append(_exercise_service(gateway, service_id, rule))

    telemetry_path = gateway.export_telemetry("self-heal-metrics.json")
    all_services_healthy = all(
        entry.get("status") == 200
        for entry in results
        if isinstance(entry.get("status"), int)
    )
    status = "ok" if all_services_healthy else "attention"
    payload = {
        "status": status,
        "results": results,
        "telemetry": str(telemetry_path),
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2))
    return payload


def main() -> None:
    parser = argparse.ArgumentParser(description="Execute gateway self-healing probes and persist a summary report.")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("build_output/gateway-self-heal.json"),
        help="Where to write the JSON self-heal summary.",
    )
    args = parser.parse_args()

    report = run_self_heal(args.output)
    print(json.dumps(report, indent=2))
    if report.get("status") != "ok":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
