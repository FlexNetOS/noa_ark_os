from __future__ import annotations

import argparse
from pathlib import Path
from typing import List, Optional

from core.kernel.manifest import KernelManifest, load_manifest


REQUIRED_TARGETS = ("linux", "macos", "container")


def validate(manifest: KernelManifest) -> None:
    manifest.validate_targets(REQUIRED_TARGETS)

    missing_interfaces = {}
    for service in manifest.services.values():
        if not service.healthChecks:
            missing_interfaces.setdefault(service.id, []).append("healthChecks")
        if not set(service.compatibility).issuperset(REQUIRED_TARGETS):
            missing_interfaces.setdefault(service.id, []).append("compatibility")

    if missing_interfaces:
        raise ValueError(f"Manifest missing required fields: {missing_interfaces}")


def _command(args: argparse.Namespace) -> None:
    manifest_path = Path(args.manifest) if args.manifest else None
    manifest = load_manifest(manifest_path)
    validate(manifest)
    print(f"Manifest {manifest.version} validated for targets: {', '.join(REQUIRED_TARGETS)}")


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Validate manifest compatibility")
    parser.add_argument("--manifest", help="Optional path to kernel manifest")
    parser.set_defaults(func=_command)
    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = create_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
