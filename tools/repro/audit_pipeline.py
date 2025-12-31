from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import subprocess
import sys
import shutil
from dataclasses import dataclass
from hashlib import sha256
import hmac
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Set


def _now_utc() -> str:
    return _dt.datetime.now(_dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _read_json(path: Path) -> Dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def _write_json(path: Path, payload: Dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def _sha256(path: Path) -> str:
    digest = sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _read_file(path: Path) -> bytes:
    with path.open("rb") as handle:
        return handle.read()


@dataclass
class AuditPipeline:
    """Implements the offline reproducibility chain used by `make pipeline.local`."""

    root: Path
    audit_dir: Path
    dist_dir: Path
    metrics_dir: Path
    signing_key_path: Path

    def __post_init__(self) -> None:
        self.audit_dir.mkdir(parents=True, exist_ok=True)
        self.dist_dir.mkdir(parents=True, exist_ok=True)
        self.metrics_dir.mkdir(parents=True, exist_ok=True)

    # ------------------------------------------------------------------
    # SBOM generation
    def generate_sboms(self) -> None:
        metadata = self._cargo_metadata()
        packages = {pkg["id"]: pkg for pkg in metadata.get("packages", [])}
        kernel_id = self._workspace_package_id(metadata, packages, "noa_core")
        if kernel_id is None:
            raise RuntimeError("Unable to locate noa_core package for kernel SBOM")

        kernel_sbom = self.audit_dir / "SBOM.kernel.cdx.json"
        self._emit_sbom(metadata, packages, {kernel_id}, kernel_sbom)

        workspace_member_ids = set(metadata.get("workspace_members", []))
        extension_roots = {
            pkg_id
            for pkg_id in workspace_member_ids
            if pkg_id != kernel_id
        }
        extensions_sbom = self.audit_dir / "SBOM.extensions.cdx.json"
        if extension_roots:
            self._emit_sbom(metadata, packages, extension_roots, extensions_sbom)
        else:
            self._emit_empty_sbom(extensions_sbom, label="extensions")

        print(f"📋 SBOMs written to {kernel_sbom} and {extensions_sbom}")

    def _cargo_metadata(self) -> Dict:
        cmd = ["cargo", "metadata", "--format-version", "1", "--locked"]
        result = subprocess.run(
            cmd,
            cwd=self.root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            text=True,
        )
        if result.returncode != 0:
            raise RuntimeError(
                "cargo metadata failed: " + result.stderr.strip()
            )
        return json.loads(result.stdout)

    def _workspace_package_id(
        self, metadata: Dict, packages: Dict[str, Dict], package_name: str
    ) -> Optional[str]:
        for pkg_id in metadata.get("workspace_members", []):
            pkg = packages.get(pkg_id)
            if pkg and pkg.get("name") == package_name:
                return pkg_id
        return None

    def _emit_empty_sbom(self, destination: Path, label: str) -> None:
        payload = {
            "bomFormat": "CycloneDX",
            "specVersion": "1.4",
            "version": 1,
            "metadata": {
                "timestamp": _now_utc(),
                "tools": [
                    {
                        "vendor": "NOA",
                        "name": "offline-sbom",
                        "version": "0.1",
                    }
                ],
                "component": {
                    "name": f"noa-ark-os-{label}",
                    "type": "application",
                },
            },
            "components": [],
        }
        _write_json(destination, payload)

    def _emit_sbom(
        self,
        metadata: Dict,
        packages: Dict[str, Dict],
        root_ids: Set[str],
        destination: Path,
    ) -> None:
        component_ids = self._collect_dependencies(metadata, root_ids)
        components = []
        for pkg_id in sorted(component_ids, key=lambda pid: packages[pid]["name"]):
            pkg = packages[pkg_id]
            component_type = "application"
            for target in pkg.get("targets", []):
                kinds = target.get("kind", [])
                if any(kind in {"lib", "rlib", "proc-macro"} for kind in kinds):
                    component_type = "library"
                    break
            components.append(
                {
                    "type": component_type,
                    "name": pkg.get("name"),
                    "version": pkg.get("version"),
                    "purl": f"pkg:cargo/{pkg.get('name')}@{pkg.get('version')}",
                }
            )
        payload = {
            "bomFormat": "CycloneDX",
            "specVersion": "1.4",
            "version": 1,
            "metadata": {
                "timestamp": _now_utc(),
                "tools": [
                    {
                        "vendor": "NOA",
                        "name": "offline-sbom",
                        "version": "0.1",
                    }
                ],
            },
            "components": components,
        }
        _write_json(destination, payload)

    def _collect_dependencies(self, metadata: Dict, root_ids: Set[str]) -> Set[str]:
        nodes = {node["id"]: node for node in metadata.get("resolve", {}).get("nodes", [])}
        discovered: Set[str] = set()
        stack = list(root_ids)
        while stack:
            current = stack.pop()
            if current in discovered:
                continue
            discovered.add(current)
            for dep in nodes.get(current, {}).get("dependencies", []):
                if dep not in discovered:
                    stack.append(dep)
        return discovered

    # ------------------------------------------------------------------
    # Packaging
    def package_artifacts(self) -> None:
        self._cargo_build_release()
        epoch = self._source_date_epoch()

        kernel_tar = self.dist_dir / "noa-ark-os-kernel.tar.zst"
        self._create_tarball(
            output=kernel_tar,
            include=["target/release/noa_core"],
            epoch=epoch,
        )

        extensions_tar = self.dist_dir / "noa-ark-os-extensions.tar.zst"
        self._create_tarball(
            output=extensions_tar,
            include=["target/release"],
            exclude=["target/release/noa_core"],
            epoch=epoch,
        )

        manifest = self._compose_manifest(kernel_tar, extensions_tar, epoch)
        manifest_path = self.audit_dir / "artifacts.manifest.json"
        _write_json(manifest_path, manifest)
        print(f"📦 Wrote artifact manifest to {manifest_path}")

    def _cargo_build_release(self) -> None:
        cmd = [
            "cargo",
            "build",
            "--workspace",
            "--locked",
            "--release",
        ]
        subprocess.run(cmd, cwd=self.root, check=True)

    def _source_date_epoch(self) -> int:
        epoch_env = os.environ.get("SOURCE_DATE_EPOCH")
        if epoch_env:
            try:
                return int(epoch_env)
            except ValueError:
                pass
        return 0

    def _create_tarball(
        self,
        output: Path,
        include: Sequence[str],
        epoch: int,
        exclude: Optional[Sequence[str]] = None,
    ) -> None:
        output.parent.mkdir(parents=True, exist_ok=True)
        if shutil.which("zstd") is None:
            raise RuntimeError("zstd binary not found; install zstd to build reproducible archives")
        cmd: List[str] = [
            "tar",
            "--zstd",
            "--sort=name",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            f"--mtime=@{epoch}",
            "-cf",
            str(output),
        ]
        for pattern in exclude or []:
            cmd.append(f"--exclude={pattern}")
        cmd.extend(["-C", str(self.root)])
        cmd.extend(include)
        subprocess.run(cmd, check=True)

    def _compose_manifest(
        self, kernel_tar: Path, extensions_tar: Path, epoch: int
    ) -> Dict:
        manifest: Dict[str, object] = {
            "generated_at": _now_utc(),
            "source_date_epoch": epoch,
            "toolchain": {
                "package": "python -m tools.repro.audit_pipeline package",
                "version": "0.1",
            },
            "artifacts": [],
            "sboms": [],
        }
        artifacts = []
        for label, tar_path in (
            ("kernel", kernel_tar),
            ("extensions", extensions_tar),
        ):
            if tar_path.exists():
                artifacts.append(
                    {
                        "name": label,
                        "path": str(tar_path.relative_to(self.root)),
                        "sha256": _sha256(tar_path),
                        "size": tar_path.stat().st_size,
                    }
                )
        manifest["artifacts"] = artifacts

        sbom_paths = [
            ("kernel", self.audit_dir / "SBOM.kernel.cdx.json"),
            ("extensions", self.audit_dir / "SBOM.extensions.cdx.json"),
        ]
        sboms = []
        for label, path in sbom_paths:
            if path.exists():
                sboms.append(
                    {
                        "name": label,
                        "path": str(path.relative_to(self.root)),
                        "sha256": _sha256(path),
                        "size": path.stat().st_size,
                    }
                )
        manifest["sboms"] = sboms
        return manifest

    # ------------------------------------------------------------------
    # Signing
    def sign_manifests(self) -> None:
        key = self._load_signing_key()
        targets = [
            self.audit_dir / "artifacts.manifest.json",
            self.audit_dir / "SBOM.kernel.cdx.json",
            self.audit_dir / "SBOM.extensions.cdx.json",
        ]
        signatures: List[Dict[str, str]] = []
        for target in targets:
            if not target.exists():
                continue
            digest = self._hmac_digest(key, target)
            sig_path = self._signature_path(target)
            sig_path.write_text(digest + "\n", encoding="utf-8")
            signatures.append(
                {
                    "source": str(target.relative_to(self.root)),
                    "signature": str(sig_path.relative_to(self.root)),
                    "value": digest,
                }
            )
        summary = {
            "generated_at": _now_utc(),
            "algorithm": "HMAC-SHA256",
            "key_fingerprint": sha256(key).hexdigest(),
            "signatures": signatures,
        }
        signatures_path = self.audit_dir / "signatures.json"
        _write_json(signatures_path, summary)
        summary_sig = self._hmac_digest(key, signatures_path)
        signatures_sig_path = self._signature_path(signatures_path)
        signatures_sig_path.write_text(summary_sig + "\n", encoding="utf-8")
        print(f"✍️  Signatures recorded in {signatures_path}")

    def _load_signing_key(self) -> bytes:
        env_key = os.environ.get("NOA_SIGNING_KEY")
        if env_key:
            key_material = env_key.strip()
        elif self.signing_key_path.exists():
            key_material = self.signing_key_path.read_text(encoding="utf-8").strip()
        else:
            raise RuntimeError(
                f"Signing key missing: {self.signing_key_path}. Set NOA_SIGNING_KEY or provide a key file."
            )
        if key_material.startswith("0x"):
            key_material = key_material[2:]
        if len(key_material) % 2 != 0:
            raise RuntimeError("Signing key must be an even-length hex string")
        try:
            return bytes.fromhex(key_material)
        except ValueError as exc:
            raise RuntimeError("Signing key must be valid hex") from exc

    def _hmac_digest(self, key: bytes, target: Path) -> str:
        digest = hmac.new(key, _read_file(target), sha256).hexdigest()
        return digest

    def _signature_path(self, target: Path) -> Path:
        return target.with_suffix(target.suffix + ".sig")

    # ------------------------------------------------------------------
    # Verification
    def verify_chain(self, raise_on_failure: bool = True) -> bool:
        errors: List[str] = []
        manifest_path = self.audit_dir / "artifacts.manifest.json"
        if not manifest_path.exists():
            errors.append("artifacts.manifest.json missing")
        else:
            manifest = _read_json(manifest_path)
            for entry in manifest.get("artifacts", []):
                path = self.root / entry["path"]
                expected = entry.get("sha256")
                if not path.exists():
                    errors.append(f"Artifact missing: {path}")
                    continue
                actual = _sha256(path)
                if actual != expected:
                    errors.append(
                        f"Digest mismatch for {path}: expected {expected}, got {actual}"
                    )
            for entry in manifest.get("sboms", []):
                path = self.root / entry["path"]
                expected = entry.get("sha256")
                if not path.exists():
                    errors.append(f"SBOM missing: {path}")
                    continue
                actual = _sha256(path)
                if actual != expected:
                    errors.append(
                        f"SBOM digest mismatch for {path}: expected {expected}, got {actual}"
                    )
        key = None
        try:
            key = self._load_signing_key()
        except RuntimeError as exc:
            errors.append(str(exc))
        if key is not None:
            for target in [
                manifest_path,
                self.audit_dir / "SBOM.kernel.cdx.json",
                self.audit_dir / "SBOM.extensions.cdx.json",
                self.audit_dir / "signatures.json",
            ]:
                if not target.exists():
                    errors.append(f"Missing signed file: {target}")
                    continue
                sig_path = self._signature_path(target)
                if not sig_path.exists():
                    errors.append(f"Missing signature: {sig_path}")
                    continue
                expected = sig_path.read_text(encoding="utf-8").strip()
                actual = self._hmac_digest(key, target)
                if actual != expected:
                    errors.append(
                        f"Signature mismatch for {target}: expected {expected}, got {actual}"
                    )
        if errors:
            if raise_on_failure:
                raise RuntimeError("\n".join(errors))
            print("❌ Verification failed:")
            for msg in errors:
                print(f"   - {msg}")
            return False
        print("🔐 Verification successful")
        return True

    # ------------------------------------------------------------------
    # Trust scoring
    def score(self) -> float:
        signatures_valid = self.verify_chain(raise_on_failure=False)
        manifest_path = self.audit_dir / "artifacts.manifest.json"
        sbom_kernel = self.audit_dir / "SBOM.kernel.cdx.json"
        sbom_extensions = self.audit_dir / "SBOM.extensions.cdx.json"
        sboms_present = sbom_kernel.exists() and sbom_extensions.exists()
        score = 1.0 if signatures_valid and sboms_present else 0.0
        payload = {
            "trust_score": score,
            "timestamp": _now_utc(),
            "criteria": {
                "signatures_valid": signatures_valid,
                "sboms_present": sboms_present,
            },
        }
        output = self.metrics_dir / "trust_score.json"
        _write_json(output, payload)
        print(f"🎯 Trust score {score:.2f} written to {output}")
        return score

    def _read_trust_score(self) -> Optional[float]:
        path = self.metrics_dir / "trust_score.json"
        if not path.exists():
            return None
        data = _read_json(path)
        value = data.get("trust_score")
        if value is None:
            return None
        try:
            return float(value)
        except (TypeError, ValueError):
            return None

    # ------------------------------------------------------------------
    # Publishing
    def publish_audit_bundle(self) -> None:
        if not self.verify_chain(raise_on_failure=False):
            raise RuntimeError("Cannot publish audit bundle until verification succeeds")
        bundle_dir = self.audit_dir / "bundles"
        bundle_dir.mkdir(parents=True, exist_ok=True)
        timestamp = _now_utc().replace(":", "-")
        snapshot = self._git_snapshot()
        bundle_name = f"audit-{timestamp}-{snapshot[:8]}.tar.zst"
        bundle_path = bundle_dir / bundle_name
        files = [
            self.audit_dir / "artifacts.manifest.json",
            self.audit_dir / "artifacts.manifest.json.sig",
            self.audit_dir / "SBOM.kernel.cdx.json",
            self.audit_dir / "SBOM.kernel.cdx.json.sig",
            self.audit_dir / "SBOM.extensions.cdx.json",
            self.audit_dir / "SBOM.extensions.cdx.json.sig",
            self.audit_dir / "signatures.json",
            self.audit_dir / "signatures.json.sig",
        ]
        rel_files = [str(path.relative_to(self.root)) for path in files if path.exists()]
        cmd: List[str] = [
            "tar",
            "--zstd",
            "--sort=name",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            f"--mtime=@{self._source_date_epoch()}",
            "-cf",
            str(bundle_path),
            "-C",
            str(self.root),
        ] + rel_files
        subprocess.run(cmd, check=True)
        bundle_hash = _sha256(bundle_path)
        trust_score = self._read_trust_score()
        manifest = _read_json(self.audit_dir / "artifacts.manifest.json")
        ledger_entry = {
            "timestamp": _now_utc(),
            "snapshot_id": snapshot,
            "bundle": {
                "path": str(bundle_path.relative_to(self.root)),
                "sha256": bundle_hash,
                "size": bundle_path.stat().st_size,
            },
            "artifacts": manifest.get("artifacts", []),
            "sboms": manifest.get("sboms", []),
            "trust_score": trust_score,
        }
        ledger_path = self.audit_dir / "ledger.jsonl"
        with ledger_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(ledger_entry, sort_keys=True))
            handle.write("\n")
        print(f"📤 Audit bundle recorded in ledger {ledger_path}")

    def _git_snapshot(self) -> str:
        cmd = ["git", "rev-parse", "HEAD"]
        result = subprocess.run(
            cmd,
            cwd=self.root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            return "unknown"
        return result.stdout.strip()


def _default_pipeline(args: argparse.Namespace) -> AuditPipeline:
    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parents[2]
    audit_dir = Path(args.audit_dir).resolve() if args.audit_dir else root / "audit"
    dist_dir = Path(args.dist_dir).resolve() if args.dist_dir else root / "dist"
    metrics_dir = (
        Path(args.metrics_dir).resolve() if args.metrics_dir else root / "metrics"
    )
    signing_key = (
        Path(args.signing_key).resolve()
        if args.signing_key
        else audit_dir / "offline-dev-hmac.key"
    )
    return AuditPipeline(root, audit_dir, dist_dir, metrics_dir, signing_key)


def main(argv: Optional[Sequence[str]] = None) -> None:
    parser = argparse.ArgumentParser(description="Offline audit pipeline tooling")
    parser.add_argument("--root", help="Workspace root directory")
    parser.add_argument("--audit-dir", help="Directory for audit artifacts")
    parser.add_argument("--dist-dir", help="Directory for packaged artifacts")
    parser.add_argument("--metrics-dir", help="Directory for trust metrics")
    parser.add_argument("--signing-key", help="Hex-encoded signing key file path")

    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("sbom", help="Generate kernel and extension SBOMs")
    subparsers.add_parser("package", help="Create reproducible tarballs and manifest")
    subparsers.add_parser("sign", help="Sign manifests and SBOM files")
    subparsers.add_parser("verify", help="Verify signatures and reproducibility")
    subparsers.add_parser("score", help="Compute trust score from verification results")
    subparsers.add_parser("publish", help="Publish audit ledger entry and bundle")

    args = parser.parse_args(argv)
    pipeline = _default_pipeline(args)

    if args.command == "sbom":
        pipeline.generate_sboms()
    elif args.command == "package":
        pipeline.package_artifacts()
    elif args.command == "sign":
        pipeline.sign_manifests()
    elif args.command == "verify":
        pipeline.verify_chain()
    elif args.command == "score":
        pipeline.score()
    elif args.command == "publish":
        pipeline.publish_audit_bundle()
    else:
        parser.error(f"Unknown command {args.command}")


if __name__ == "__main__":
    main(sys.argv[1:])
