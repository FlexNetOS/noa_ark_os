from __future__ import annotations

import json
from pathlib import Path

from services.marketplace.ecosystem_bundle import create_default_bundle


def test_ecosystem_bundle_install(tmp_path: Path) -> None:
    bundle = create_default_bundle()
    assert bundle.validate_dependencies() == []

    licenses = {
        "workflow-library": "LIC-001",
        "partner-integrations": "LIC-002",
    }
    bundle.install(licenses)

    assert bundle.toggle_manager.is_enabled("workflow-library")
    assert bundle.toggle_manager.is_enabled("partner-integrations")

    state_file = tmp_path / "ecosystem_state.json"
    bundle.export_state(state_file)
    payload = json.loads(state_file.read_text())
    assert "dashboard" in payload["entry_points"]
    assert payload["features"]["workflow-library"]["enabled"] is True
