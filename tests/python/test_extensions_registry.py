from __future__ import annotations

import json
from pathlib import Path

import pytest

from core.kernel.security import CapabilityTokenError, issue_capability_token
from extensions import ExtensionRegistry


def test_extension_registry_loads_adapter_with_valid_token() -> None:
    registry = ExtensionRegistry()
    token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=200,
    )
    adapter = registry.load_adapter("sample-adapter", token)
    result = adapter.invoke({"action": "ping"})
    assert result["status"] == "ok"
    assert adapter.kernel_claims.client_id == "cli-1"


def test_extension_registry_enforces_scope_requirements() -> None:
    registry = ExtensionRegistry()
    token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.other"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=200,
    )
    with pytest.raises(CapabilityTokenError):
        registry.load_adapter("sample-adapter", token)


def test_extension_registry_refresh_supports_hot_swap(tmp_path: Path) -> None:
    registry = ExtensionRegistry(root=tmp_path)
    assert list(registry.registered_extensions()) == []

    extension_dir = tmp_path / "dynamic-adapter"
    extension_dir.mkdir()
    manifest_path = extension_dir / "manifest.json"
    manifest_payload = {
        "name": "dynamic-adapter",
        "version": "1.0.0",
        "module": "extensions.sample_adapter.adapter",
        "entrypoint": "SampleAdapter",
        "scopes": {
            "fs": "fs.policy.read",
            "network": "net.gateway",
            "rate_limit_per_minute": 60,
        },
    }
    manifest_path.write_text(json.dumps(manifest_payload))

    registry.refresh()
    assert "dynamic-adapter" in list(registry.registered_extensions())
    assert registry.manifest_for("dynamic-adapter").version == "1.0.0"

    manifest_payload["version"] = "2.0.0"
    manifest_path.write_text(json.dumps(manifest_payload))
    registry.refresh()
    assert registry.manifest_for("dynamic-adapter").version == "2.0.0"

    token = issue_capability_token(
        client_id="cli-1",
        fs_scopes=["fs.policy.read"],
        network_scopes=["net.gateway"],
        rate_limit_per_minute=120,
    )
    adapter = registry.load_adapter("dynamic-adapter", token)
    assert adapter.kernel_claims.client_id == "cli-1"
