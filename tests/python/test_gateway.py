from __future__ import annotations

import json
from pathlib import Path

from core.kernel.security import issue_capability_token
from services.gateway.service import Gateway, GatewayRequest, build_default_config


def test_gateway_policy_enforcement(tmp_path: Path) -> None:
    config = build_default_config()
    config.policy_rules["gateway"].rate_limit_per_minute = 2
    gateway = Gateway(config=config, telemetry_dir=tmp_path)

    valid_token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=200,
    )

    allowed = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=valid_token)
    )
    assert allowed.status == 200

    unauthorized = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET")
    )
    assert unauthorized.status == 401

    invalid_token = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token="invalid")
    )
    assert invalid_token.status == 401

    wrong_client_token = issue_capability_token(
        client_id="cli-2",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=200,
    )
    mismatch = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=wrong_client_token)
    )
    assert mismatch.status == 403
    assert mismatch.message == "token client mismatch"

    missing_fs_token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.other"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=200,
    )
    missing_fs = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=missing_fs_token)
    )
    assert missing_fs.status == 403
    assert missing_fs.message == "missing fs scope"

    missing_network_token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.other"],
        rate_limit_per_minute=200,
    )
    missing_network = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=missing_network_token)
    )
    assert missing_network.status == 403
    assert missing_network.message == "missing network scope"

    insufficient_rate_token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=1,
    )
    insufficient_rate = gateway.handle(
        GatewayRequest(
            client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=insufficient_rate_token
        )
    )
    assert insufficient_rate.status == 403
    assert insufficient_rate.message == "rate scope below requirement"

    forbidden = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/unknown", method="GET", token=valid_token)
    )
    assert forbidden.status == 403

    gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=valid_token)
    )
    rate_limited = gateway.handle(
        GatewayRequest(client_id="cli-1", service="gateway", path="/v1/policy", method="GET", token=valid_token)
    )
    assert rate_limited.status == 429

    export_path = gateway.export_telemetry()
    assert export_path.exists()
    payload = json.loads(export_path.read_text())
    assert payload["requests_total"] >= 1


def test_notebook_sync_policy_registration(tmp_path: Path) -> None:
    config = build_default_config()
    gateway = Gateway(config=config, telemetry_dir=tmp_path)
    rule = config.policy_rules["notebook-sync"]

    token = issue_capability_token(
        client_id="notebook-agent",
        fs_scopes=[rule.fs_scope] if rule.fs_scope else [],
        network_scopes=[rule.network_scope] if rule.network_scope else [],
        rate_limit_per_minute=rule.rate_limit_per_minute,
    )

    allowed = gateway.handle(
        GatewayRequest(
            client_id="notebook-agent",
            service="notebook-sync",
            path="/v1/notebooks/sync",
            method="POST",
            token=token,
        )
    )
    assert allowed.status == 200

    missing_scope_token = issue_capability_token(
        client_id="notebook-agent",
        fs_scopes=["fs.other"],
        network_scopes=[rule.network_scope] if rule.network_scope else [],
        rate_limit_per_minute=rule.rate_limit_per_minute,
    )
    denied = gateway.handle(
        GatewayRequest(
            client_id="notebook-agent",
            service="notebook-sync",
            path="/v1/notebooks/sync",
            method="POST",
            token=missing_scope_token,
        )
    )
    assert denied.status == 403
