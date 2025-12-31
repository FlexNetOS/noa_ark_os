from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from services.gateway.service import Gateway, GatewayRequest, build_default_config


def _exercise_service(gateway: Gateway, service_id: str, rule) -> Dict[str, object]:
    if not rule.allowed_methods or not rule.allowed_paths:
        return {
            "service": service_id,
            "status": "skipped",
            "reason": "no policy routes configured",
        }

    request = GatewayRequest(
        client_id="self-heal",
        service=service_id,
        path=rule.allowed_paths[0],
        method=rule.allowed_methods[0],
        token="self-heal-token",
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
