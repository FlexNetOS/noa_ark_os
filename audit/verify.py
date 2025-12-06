#!/usr/bin/env python3
"""Verify an autonomous-release audit bundle."""
import hashlib
import json
import pathlib
import sys


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: verify.py <bundle_dir>")
        return 1

    bundle = pathlib.Path(sys.argv[1])
    try:
        bundle = bundle.resolve(strict=True)
    except FileNotFoundError:
        print(f"Bundle directory does not exist: {bundle}", file=sys.stderr)
        return 2
    if not bundle.is_dir():
        print(f"Bundle path is not a directory: {bundle}", file=sys.stderr)
        return 2
    manifest_path = bundle / "bundle_manifest.json"
    signature_path = bundle / "signature.json"

    if not manifest_path.exists() or not signature_path.exists():
        print("Missing manifest or signature", file=sys.stderr)
        return 2

    manifest_hash = sha256(manifest_path)
    signature = json.loads(signature_path.read_text())
    if signature.get("signature") != manifest_hash:
        print("Manifest signature mismatch", file=sys.stderr)
        return 3

    manifest = json.loads(manifest_path.read_text())
    for entry in manifest.get("files", []):
        artifact_path = bundle / entry["path"]
        # Prevent path traversal: ensure artifact_path is within bundle
        if not artifact_path.resolve().is_relative_to(bundle.resolve()):
            print(f"Invalid artifact path (traversal): {entry['path']}", file=sys.stderr)
            return 6
        if not artifact_path.exists():
            print(f"Missing artifact {entry['path']}", file=sys.stderr)
            return 4
        artifact_hash = sha256(artifact_path)
        if artifact_hash != entry.get("sha256"):
            print(f"Checksum mismatch for {entry['path']}", file=sys.stderr)
            return 5

    print(f"Audit bundle verified: {bundle}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
