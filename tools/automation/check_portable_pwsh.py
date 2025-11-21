#!/usr/bin/env python3
"""Verifies the portable PowerShell manifest and binary integrity."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict


def detect_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    return Path(__file__).resolve().parents[2]


def compute_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def build_result(status: str, message: str, manifest: Path, binary: Path | None, expected_sha: str | None, actual_sha: str | None) -> Dict[str, Any]:
    return {
        "status": status,
        "message": message,
        "manifest": str(manifest) if manifest else None,
        "binary": str(binary) if binary else None,
        "expected_sha256": expected_sha,
        "actual_sha256": actual_sha,
    }


def format_text(result: Dict[str, Any]) -> str:
    return (
        f"CLAIM portable_pwsh_manifest status={result['status']} "
        f"manifest={result['manifest']} binary={result['binary']} "
        f"expected_sha={result['expected_sha256']} actual_sha={result['actual_sha256']} "
        f"message={result['message']}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", help="Workspace root (defaults to repo root)")
    parser.add_argument("--output", choices=["text", "json"], default="text", help="Output format")
    args = parser.parse_args()

    root = detect_root(args.root)
    manifest = root / "server" / "tools" / "pwsh-portable.manifest.json"
    if not manifest.exists():
        result = build_result(
            "missing_manifest",
            "Portable PowerShell manifest not found; run ./server/tools/setup-portable-pwsh.sh",
            manifest,
            None,
            None,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 2

    try:
        data = json.loads(manifest.read_text())
    except json.JSONDecodeError as exc:
        result = build_result(
            "invalid_manifest",
            f"Manifest is not valid JSON: {exc}",
            manifest,
            None,
            None,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 3

    binary_rel = data.get("binary")
    expected_sha = data.get("sha256")
    if not binary_rel:
        result = build_result(
            "missing_field",
            "Manifest missing 'binary' field",
            manifest,
            None,
            expected_sha,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 4

    binary_path = (manifest.parent / binary_rel).resolve()
    if not binary_path.exists():
        result = build_result(
            "missing_binary",
            "Portable PowerShell binary missing; run ./server/tools/setup-portable-pwsh.sh",
            manifest,
            binary_path,
            expected_sha,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 5

    actual_sha = compute_sha256(binary_path)
    status = "ok" if expected_sha in (None, actual_sha) else "sha_mismatch"
    message = "Portable PowerShell bundle verified" if status == "ok" else "SHA mismatch between manifest and binary"
    result = build_result(status, message, manifest, binary_path, expected_sha, actual_sha)

    output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
    print(output)

    return 0 if status == "ok" else 6


if __name__ == "__main__":
    sys.exit(main())
