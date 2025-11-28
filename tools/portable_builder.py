#!/usr/bin/env python3
"""Portable distribution builder for NOA ARK OS.

This script assembles kernel capabilities, runtimes, AI assets, and application
packages into a reproducible bundle. It is designed to be deterministic so CI
recipes can validate output artefacts easily.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
from typing import Any, Dict, Iterable, List

try:
    import yaml  # type: ignore
except Exception:  # pragma: no cover - optional dependency
    yaml = None


def load_manifest(manifest_path: Path) -> Dict[str, Any]:
    if not manifest_path.exists():
        raise SystemExit(f"manifest file not found: {manifest_path}")

    text = manifest_path.read_text(encoding="utf-8")
    if manifest_path.suffix.lower() == ".json":
        return json.loads(text)

    if yaml is None:
        raise SystemExit(
            "PyYAML is required to parse manifest files. Install with 'pip install pyyaml'."
        )

    return yaml.safe_load(text)


def normalise_runtime(runtime: Dict[str, Any]) -> Dict[str, Any]:
    return {
        "name": runtime["name"],
        "kind": runtime.get("kind", "unknown"),
        "version": runtime.get("version", "unknown"),
        "entrypoint": runtime.get("entrypoint", ""),
        "depends_on": sorted(runtime.get("depends_on", [])),
    }


def reproducible_hash(items: Iterable[str]) -> str:
    digest = hashlib.sha256()
    for item in sorted(items):
        digest.update(item.encode("utf-8"))
    return digest.hexdigest()


def build_bundle(manifest: Dict[str, Any], output: Path, formats: List[str]) -> None:
    if output.exists():
        for path in output.glob("*"):
            if path.is_dir():
                for child in path.rglob("*"):
                    if child.is_file():
                        child.unlink()
                for child in sorted(path.rglob("*"), reverse=True):
                    if child.is_dir():
                        child.rmdir()
                path.rmdir()
            else:
                path.unlink()
    output.mkdir(parents=True, exist_ok=True)

    (output / "kernel").mkdir(exist_ok=True)
    (output / "runtimes").mkdir(exist_ok=True)
    (output / "apps").mkdir(exist_ok=True)
    (output / "assets").mkdir(exist_ok=True)

    runtimes = [normalise_runtime(runtime) for runtime in manifest.get("runtimes", [])]
    runtime_index = {runtime["name"]: runtime for runtime in runtimes}

    bundle_metadata = {
        "version": manifest.get("version", "unknown"),
        "capabilities": sorted(cap["id"] for cap in manifest.get("capabilities", [])),
        "runtimes": runtimes,
        "formats": formats,
    }

    (output / "bundle.json").write_text(
        json.dumps(bundle_metadata, indent=2, sort_keys=True),
        encoding="utf-8",
    )

    for runtime_name, runtime in runtime_index.items():
        runtime_dir = output / "runtimes" / runtime_name
        runtime_dir.mkdir(parents=True, exist_ok=True)
        (runtime_dir / "runtime.json").write_text(
            json.dumps(runtime, indent=2, sort_keys=True),
            encoding="utf-8",
        )

    lock_contents = [
        bundle_metadata["version"],
        *bundle_metadata["capabilities"],
        *(f"{name}:{runtime['version']}" for name, runtime in runtime_index.items()),
        *formats,
    ]
    lock_hash = reproducible_hash(lock_contents)
    (output / "bundle.lock").write_text(lock_hash + "\n", encoding="utf-8")

    for fmt in formats:
        fmt_dir = output / "targets" / fmt
        fmt_dir.mkdir(parents=True, exist_ok=True)
        descriptor = {
            "format": fmt,
            "bundle_hash": lock_hash,
            "runtimes": runtimes,
        }
        (fmt_dir / "descriptor.json").write_text(
            json.dumps(descriptor, indent=2, sort_keys=True),
            encoding="utf-8",
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build portable NOA ARK OS bundles")
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("core/config/default_manifest.yaml"),
        help="Path to kernel manifest (YAML or JSON)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("build/portable"),
        help="Output directory for the assembled bundle",
    )
    parser.add_argument(
        "--format",
        action="append",
        dest="formats",
        help="Target format to generate (e.g. oci, wasi, docker)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest = load_manifest(args.manifest)
    formats = args.formats or ["oci", "wasi", "tar"]
    build_bundle(manifest, args.output, formats)
    print(f"Portable bundle built at {args.output}")


if __name__ == "__main__":
    main()
