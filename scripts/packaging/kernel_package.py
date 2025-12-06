from __future__ import annotations

import argparse
import hashlib
import io
import json
import tarfile
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional

from core.kernel.manifest import KernelManifest, KernelService, load_manifest


@dataclass
class PackagingResult:
    service_id: str
    version: str
    artifacts: List[Path]


def _ensure_directory(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def _write_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")


def _hash_payload(payload: object) -> str:
    digest = hashlib.sha256(json.dumps(payload, sort_keys=True).encode("utf-8")).hexdigest()
    return digest


def _emit_container_layout(service: KernelService, destination: Path) -> Path:
    _ensure_directory(destination)
    config = {
        "service": service.id,
        "version": service.version,
        "interfaces": service.interfaces,
        "dependencies": service.dependencies,
    }
    _write_json(destination / "config.json", config)
    return destination / "config.json"


def _emit_tarball(service: KernelService, destination: Path) -> Path:
    _ensure_directory(destination.parent)
    archive_layout = {
        "service": service.id,
        "version": service.version,
        "boot": service.boot,
        "resources": service.resources,
    }
    payload_path = destination.parent / "payload.json"
    _write_json(payload_path, archive_layout)
    with tarfile.open(destination, "w:gz", compresslevel=9, format=tarfile.GNU_FORMAT) as tar:
        info = tarfile.TarInfo(name="manifest.json")
        data = json.dumps(archive_layout, sort_keys=True).encode("utf-8")
        info.size = len(data)
        info.uid = 0
        info.gid = 0
        info.uname = "root"
        info.gname = "root"
        info.mode = 0o644
        info.mtime = 0
        tar.addfile(info, io.BytesIO(data))
    payload_path.unlink()
    return destination


def _emit_oci_layout(service: KernelService, destination: Path) -> Path:
    _ensure_directory(destination)
    layout = {
        "schemaVersion": 2,
        "service": service.id,
        "version": service.version,
        "interfaces": service.interfaces,
        "artifacts": service.artifacts.tarball,
    }
    _write_json(destination / "index.json", layout)
    return destination / "index.json"


def build_artifacts(
    output_dir: Path,
    manifest: Optional[KernelManifest] = None,
    manifest_path: Optional[Path] = None,
) -> List[PackagingResult]:
    if manifest is None:
        manifest = load_manifest(manifest_path)

    results: List[PackagingResult] = []
    for service in manifest.services.values():
        service_dir = output_dir / service.id / service.version
        _ensure_directory(service_dir)

        container_path = service_dir / "container"
        container_config = _emit_container_layout(service, container_path)

        tarball_path = service_dir / service.artifacts.tarball["file"]
        tarball_file = _emit_tarball(service, tarball_path)

        oci_path = service_dir / "oci"
        oci_index = _emit_oci_layout(service, oci_path)

        manifest_fingerprint = {
            "service": service.id,
            "version": service.version,
            "hash": _hash_payload(
                {
                    "interfaces": service.interfaces,
                    "dependencies": service.dependencies,
                    "optional": service.optionalDependencies,
                }
            ),
        }
        _write_json(service_dir / "fingerprint.json", manifest_fingerprint)

        results.append(
            PackagingResult(
                service_id=service.id,
                version=service.version,
                artifacts=[container_config, tarball_file, oci_index, service_dir / "fingerprint.json"],
            )
        )

    return results


def _assert_exists(path: Path) -> None:
    if not path.exists():
        raise FileNotFoundError(f"Missing artifact: {path}")


def validate_artifacts(output_dir: Path, manifest: Optional[KernelManifest] = None) -> None:
    manifest = manifest or load_manifest()
    for service in manifest.services.values():
        service_dir = output_dir / service.id / service.version
        _assert_exists(service_dir / "container" / "config.json")
        _assert_exists(service_dir / service.artifacts.tarball["file"])
        _assert_exists(service_dir / "oci" / "index.json")
        _assert_exists(service_dir / "fingerprint.json")


def _build_command(args: argparse.Namespace) -> None:
    output = Path(args.output).resolve()
    results = build_artifacts(output)
    for result in results:
        print(f"built {result.service_id}@{result.version} -> {len(result.artifacts)} artifacts")


def _validate_command(args: argparse.Namespace) -> None:
    output = Path(args.output).resolve()
    validate_artifacts(output)
    print(f"validated artifacts under {output}")


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Build AgentOS packaging artifacts")
    subparsers = parser.add_subparsers(dest="command", required=True)

    build_parser = subparsers.add_parser("build", help="Build packaging artifacts")
    build_parser.add_argument("--output", default="build/artifacts", help="Destination directory for artifacts")
    build_parser.set_defaults(func=_build_command)

    validate_parser = subparsers.add_parser("validate", help="Validate previously built artifacts")
    validate_parser.add_argument("--output", default="build/artifacts", help="Directory containing artifacts")
    validate_parser.set_defaults(func=_validate_command)

    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = create_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
