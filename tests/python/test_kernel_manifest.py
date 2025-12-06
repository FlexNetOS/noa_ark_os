from __future__ import annotations

import json
from pathlib import Path

from core.kernel.manifest import load_manifest
from scripts.ci.check_kernel_manifest import check_manifest
from scripts.packaging.kernel_package import build_artifacts, validate_artifacts
from scripts.packaging.validate_compatibility import validate


def test_manifest_loads_and_artifacts_build(tmp_path: Path) -> None:
    manifest = load_manifest()
    artifacts_dir = tmp_path / "artifacts"
    build_artifacts(artifacts_dir, manifest=manifest)
    validate_artifacts(artifacts_dir, manifest=manifest)

    fingerprints = list(artifacts_dir.glob("*/**/fingerprint.json"))
    assert fingerprints, "expected fingerprint artifacts"
    for fingerprint in fingerprints:
        data = json.loads(fingerprint.read_text())
        assert data["service"] in manifest.services

    validate(manifest)
    check_manifest(Path("."), manifest)
