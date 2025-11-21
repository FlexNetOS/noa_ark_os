#!/usr/bin/env python3
"""Verifies the portable PowerShell manifest and binary integrity."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Tuple


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


def detect_host_platform() -> Optional[str]:
    system = platform.system().lower()
    machine = platform.machine().lower()
    if system.startswith("linux"):
        if machine in {"x86_64", "amd64"}:
            return "linux-x64"
        if machine in {"aarch64", "arm64"}:
            return "linux-arm64"
    if system == "darwin":
        if machine == "x86_64":
            return "osx-x64"
        if machine == "arm64":
            return "osx-arm64"
    if system.startswith("win"):
        return "win-x64"
    return None


def build_result(
    status: str,
    message: str,
    manifest: Path,
    binary: Optional[Path],
    platform_id: Optional[str],
    expected_sha: Optional[str],
    actual_sha: Optional[str],
    exec_status: Optional[str] = None,
) -> Dict[str, Any]:
    return {
        "status": status,
        "message": message,
        "manifest": str(manifest) if manifest else None,
        "binary": str(binary) if binary else None,
        "platform": platform_id,
        "expected_sha256": expected_sha,
        "actual_sha256": actual_sha,
        "exec_status": exec_status,
    }


def format_text(result: Dict[str, Any]) -> str:
    parts = [
        "CLAIM portable_pwsh_manifest",
        f"status={result['status']}",
        f"platform={result.get('platform')}",
        f"manifest={result['manifest']}",
        f"binary={result['binary']}",
        f"expected_sha={result['expected_sha256']}",
        f"actual_sha={result['actual_sha256']}",
    ]
    if result.get("exec_status"):
        parts.append(f"exec={result['exec_status']}")
    parts.append(f"message={result['message']}")
    return " ".join(parts)


def select_manifest_entry(data: Dict[str, Any], manifest: Path, requested_platform: Optional[str]) -> Tuple[Dict[str, Any], str]:
    platforms = data.get("platforms")
    if platforms and isinstance(platforms, list):
        target = requested_platform or data.get("default_platform") or detect_host_platform()
        if target is None:
            raise ValueError("Unable to determine target platform for manifest validation")
        for entry in platforms:
            if entry.get("platform") == target:
                return entry, target
        raise ValueError(f"Platform '{target}' missing from manifest")
    # Legacy manifest
    platform_id = data.get("platform") or requested_platform or detect_host_platform()
    return data, platform_id or "unknown"


def ensure_exec(binary: Path) -> Tuple[str, int]:
    try:
        completed = subprocess.run(
            [str(binary), "--version"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        return (completed.stdout.strip(), 0)
    except Exception as exc:  # noqa: BLE001
        return (str(exc), 1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", help="Workspace root (defaults to repo root)")
    parser.add_argument("--platform", help="Platform entry from manifest to validate (defaults to host/default)")
    parser.add_argument("--ensure-exec", action="store_true", help="Run pwsh --version to verify executable availability")
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
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 3

    try:
        entry, platform_id = select_manifest_entry(data, manifest, args.platform)
    except ValueError as exc:
        result = build_result(
            "missing_platform",
            str(exc),
            manifest,
            None,
            args.platform,
            None,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 4

    binary_rel = entry.get("binary")
    expected_sha = entry.get("sha256")
    if not binary_rel:
        result = build_result(
            "missing_field",
            "Manifest missing 'binary' field",
            manifest,
            None,
            platform_id,
            expected_sha,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 5

    binary_path = (manifest.parent / binary_rel).resolve()
    if not binary_path.exists():
        result = build_result(
            "missing_binary",
            "Portable PowerShell binary missing; run ./server/tools/setup-portable-pwsh.sh",
            manifest,
            binary_path,
            platform_id,
            expected_sha,
            None,
        )
        output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
        print(output, file=sys.stderr)
        return 6

    actual_sha = compute_sha256(binary_path)
    status = "ok" if expected_sha in (None, actual_sha) else "sha_mismatch"
    message = "Portable PowerShell bundle verified" if status == "ok" else "SHA mismatch between manifest and binary"
    exec_status = None
    exec_code = 0
    if status == "ok" and args.ensure_exec:
        exec_status, exec_code = ensure_exec(binary_path)
        status = "exec_failed" if exec_code else status
        if exec_code:
            message = f"Binary present but execution failed: {exec_status}"
    result = build_result(status, message, manifest, binary_path, platform_id, expected_sha, actual_sha, exec_status)

    output = json.dumps(result, indent=2) if args.output == "json" else format_text(result)
    print(output)

    if status == "ok":
        return 0
    if status == "exec_failed":
        return 7
    return 6


if __name__ == "__main__":
    sys.exit(main())
