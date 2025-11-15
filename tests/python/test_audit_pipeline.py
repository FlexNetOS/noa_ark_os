from __future__ import annotations

import json
import hashlib
import shutil
from pathlib import Path

import pytest

from tools.repro.audit_pipeline import AuditPipeline, _now_utc


def _setup_pipeline(tmp_path: Path) -> AuditPipeline:
    root = tmp_path
    audit = root / "audit"
    dist = root / "dist"
    metrics = root / "metrics"
    key_path = audit / "offline-dev-hmac.key"
    audit.mkdir()
    dist.mkdir()
    metrics.mkdir()
    key_path.write_text("0f" * 32 + "\n", encoding="utf-8")

    kernel_tar = dist / "noa-ark-os-kernel.tar.zst"
    extensions_tar = dist / "noa-ark-os-extensions.tar.zst"
    kernel_tar.write_bytes(b"kernel-binary")
    extensions_tar.write_bytes(b"extensions-binary")

    kernel_sbom = audit / "SBOM.kernel.cdx.json"
    extensions_sbom = audit / "SBOM.extensions.cdx.json"
    kernel_sbom.write_text(json.dumps({"components": []}), encoding="utf-8")
    extensions_sbom.write_text(json.dumps({"components": []}), encoding="utf-8")

    manifest = {
        "generated_at": _now_utc(),
        "source_date_epoch": 0,
        "toolchain": {"package": "test", "version": "test"},
        "artifacts": [
            {
                "name": "kernel",
                "path": str(kernel_tar.relative_to(root)),
                "sha256": hashlib.sha256(kernel_tar.read_bytes()).hexdigest(),
                "size": kernel_tar.stat().st_size,
            },
            {
                "name": "extensions",
                "path": str(extensions_tar.relative_to(root)),
                "sha256": hashlib.sha256(extensions_tar.read_bytes()).hexdigest(),
                "size": extensions_tar.stat().st_size,
            },
        ],
        "sboms": [
            {
                "name": "kernel",
                "path": str(kernel_sbom.relative_to(root)),
                "sha256": hashlib.sha256(kernel_sbom.read_bytes()).hexdigest(),
                "size": kernel_sbom.stat().st_size,
            },
            {
                "name": "extensions",
                "path": str(extensions_sbom.relative_to(root)),
                "sha256": hashlib.sha256(extensions_sbom.read_bytes()).hexdigest(),
                "size": extensions_sbom.stat().st_size,
            },
        ],
    }
    manifest_path = audit / "artifacts.manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    return AuditPipeline(root, audit, dist, metrics, key_path)


def test_sign_and_verify_round_trip(tmp_path: Path) -> None:
    pipeline = _setup_pipeline(tmp_path)
    pipeline.sign_manifests()
    assert pipeline.verify_chain()

    # Tamper with an artifact and ensure verification fails
    kernel_tar = pipeline.dist_dir / "noa-ark-os-kernel.tar.zst"
    kernel_tar.write_bytes(b"tampered")
    with pytest.raises(RuntimeError):
        pipeline.verify_chain()


def test_publish_writes_ledger(tmp_path: Path) -> None:
    if shutil.which("zstd") is None:
        pytest.skip("zstd is required to publish audit bundles")
    pipeline = _setup_pipeline(tmp_path)
    pipeline.sign_manifests()
    assert pipeline.verify_chain()
    pipeline.score()
    pipeline.publish_audit_bundle()

    ledger_path = pipeline.audit_dir / "ledger.jsonl"
    assert ledger_path.exists()
    lines = ledger_path.read_text(encoding="utf-8").strip().splitlines()
    assert len(lines) == 1
    entry = json.loads(lines[0])
    assert entry["bundle"]["path"].startswith("audit/bundles/")
    assert entry["trust_score"] == pytest.approx(1.0)

    bundle_path = pipeline.root / entry["bundle"]["path"]
    assert bundle_path.exists()
    recorded_hash = entry["bundle"]["sha256"]
    actual_hash = hashlib.sha256(bundle_path.read_bytes()).hexdigest()
    assert recorded_hash == actual_hash
