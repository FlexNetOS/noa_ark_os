from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, Optional

from core.kernel.manifest import KernelManifest, load_manifest


def _load_service_descriptors(root: Path) -> Dict[str, Dict[str, object]]:
    descriptors: Dict[str, Dict[str, object]] = {}
    for path in root.rglob("service.json"):
        data = json.loads(path.read_text())
        service_id = data.get("id")
        if not service_id:
            continue
        descriptors[service_id] = {"data": data, "path": path}
    return descriptors


def check_manifest(root: Path, manifest: KernelManifest) -> None:
    descriptors = _load_service_descriptors(root)

    missing_in_repo = [service_id for service_id in manifest.services if service_id not in descriptors]
    if missing_in_repo:
        raise SystemExit(f"Manifest references services missing descriptors: {sorted(missing_in_repo)}")

    drift: Dict[str, Dict[str, object]] = {}

    for service_id, service in manifest.services.items():
        descriptor = descriptors[service_id]["data"]
        descriptor_version = descriptor.get("version")
        if descriptor_version != service.version:
            drift.setdefault(service_id, {})["version"] = {
                "manifest": service.version,
                "descriptor": descriptor_version,
            }

        descriptor_interfaces = sorted(descriptor.get("interfaces", []))
        manifest_interfaces = sorted(interface["name"] for interface in service.interfaces)
        if descriptor_interfaces != manifest_interfaces:
            drift.setdefault(service_id, {})["interfaces"] = {
                "manifest": manifest_interfaces,
                "descriptor": descriptor_interfaces,
            }

    if drift:
        raise SystemExit(f"Manifest drift detected: {json.dumps(drift, indent=2, sort_keys=True)}")

    # Ensure repository does not contain descriptors missing from manifest
    untracked = [service_id for service_id in descriptors if service_id not in manifest.services]
    if untracked:
        raise SystemExit(f"Service descriptors missing from manifest: {sorted(untracked)}")


def _command(args: argparse.Namespace) -> None:
    repo_root = Path(args.root).resolve()
    manifest_path = Path(args.manifest) if args.manifest else None
    manifest = load_manifest(manifest_path)
    check_manifest(repo_root, manifest)
    print("Kernel manifest compliance check passed")


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Validate kernel manifest against service descriptors")
    parser.add_argument("--root", default=".", help="Repository root")
    parser.add_argument("--manifest", help="Optional manifest path")
    parser.set_defaults(func=_command)
    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = create_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
