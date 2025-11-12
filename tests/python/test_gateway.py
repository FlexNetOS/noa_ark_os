from __future__ import annotations

import json
from pathlib import Path

from services.gateway.service import Gateway, GatewayRequest, build_default_config


def test_gateway_policy_enforcement(tmp_path: Path) -> None:
    config = build_default_config()
    config.policy_rules["gateway"].rate_limit_per_minute = 2
    gateway = Gateway(config=config, telemetry_dir=tmp_path)

    allowed = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token="token")
    )
    assert allowed.status == 200

    unauthorized = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET")
    )
    assert unauthorized.status == 401

    forbidden = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/unknown", method="GET", token="token")
    )
    assert forbidden.status == 403

    gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token="token")
    )
    rate_limited = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token="token")
    )
    assert rate_limited.status == 429

    export_path = gateway.export_telemetry()
    assert export_path.exists()
    payload = json.loads(export_path.read_text())
    assert payload["requests_total"] >= 1
