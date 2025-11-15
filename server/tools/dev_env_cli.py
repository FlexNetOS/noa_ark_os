#!/usr/bin/env python3
"""NOA Ark OS development environment CLI.

Provides a single entry point for reviewing, activating, and diagnosing
workspace-local Rust toolchain configuration across Windows, WSL, and
Linux hosts. The CLI replaces editor-specific `.vscode` guidance so the
same commands work from any terminal.
"""

from __future__ import annotations

import argparse
import base64
import io
import json
import tarfile
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Dict, List

ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = ROOT / "server" / "tools"
CARGO_PORTABLE = TOOLS_DIR / "cargo-portable"
RUSTUP_PORTABLE = TOOLS_DIR / "rustup-portable"
SETTINGS_VERSION = "2025.11"


@dataclass
class ArchiveBundle:
    """Metadata describing a portable archive bundle."""

    name: str
    description: str
    base64_path: Path
    original_path: str
    default_tar_output: Path
    default_extract_dir: Path


ARCHIVE_BUNDLES: Dict[str, ArchiveBundle] = {
    "devcontainer": ArchiveBundle(
        name="devcontainer",
        description="Legacy VS Code devcontainer captured before CLI migration.",
        base64_path=ROOT / "archive" / "2025" / "11" / ".devcontainer.tar.b64",
        original_path=".devcontainer/",
        default_tar_output=ROOT / "out" / ".devcontainer.tar",
        default_extract_dir=ROOT / "out" / ".devcontainer",
    ),
    "vscode": ArchiveBundle(
        name="vscode",
        description="Workspace .vscode settings archived after CLI standardization.",
        base64_path=ROOT / "archive" / "2025" / "11" / ".vscode.tar.b64",
        original_path=".vscode/",
        default_tar_output=ROOT / "out" / ".vscode.tar",
        default_extract_dir=ROOT / "out" / ".vscode",
    ),
}


@dataclass
class ActivationProfile:
    platform: str
    description: str
    commands: List[str]
    notes: List[str]


PROFILES: Dict[str, ActivationProfile] = {
    "windows": ActivationProfile(
        platform="windows",
        description="Windows PowerShell using portable Cargo",
        commands=[
            "cd D:/dev/workspaces/noa_ark_os",
            ".\\server\\tools\\activate-cargo.ps1",
            "cargo --version",
        ],
        notes=[
            "Run inside a PowerShell session.",
            "The activation script configures CARGO_HOME and RUSTUP_HOME",
            "Use `cargo run`, `cargo test`, etc. after activation.",
        ],
    ),
    "wsl": ActivationProfile(
        platform="wsl",
        description="WSL Ubuntu using portable Cargo from Windows share",
        commands=[
            "cd /mnt/d/dev/workspaces/noa_ark_os",
            "source ./server/tools/activate-cargo.sh",
            "cargo --version",
        ],
        notes=[
            "Ensures Windows portable toolchain is available inside WSL.",
            "Select option 2 when prompted to reuse the portable toolchain.",
            "Use `cargo.exe` when cross-compiling for Windows targets.",
        ],
    ),
    "linux": ActivationProfile(
        platform="linux",
        description="Native Linux with system rustup",
        commands=[
            "source $HOME/.cargo/env",
            "cargo --version",
        ],
        notes=[
            "Install rustup once via https://rustup.rs.",
            "Portable tooling is optional on native Linux.",
            "Use the same CLI workflows as other platforms after activation.",
        ],
    ),
}


def _format_commands(commands: List[str]) -> str:
    return "\n".join(f"    {cmd}" for cmd in commands)


def _resolve_output_path(path_str: str | None, default: Path) -> Path:
    if path_str:
        candidate = Path(path_str)
        if not candidate.is_absolute():
            candidate = ROOT / candidate
        return candidate
    return default


def _read_bundle_tar(bundle: ArchiveBundle) -> bytes:
    encoded = bundle.base64_path.read_text().strip().replace("\n", "")
    return base64.b64decode(encoded)


def cmd_summary(args: argparse.Namespace) -> None:
    status = {
        "workspace": str(ROOT),
        "tools_dir": str(TOOLS_DIR),
        "cargo_portable": str(CARGO_PORTABLE),
        "rustup_portable": str(RUSTUP_PORTABLE),
        "settings_version": SETTINGS_VERSION,
        "profiles": [asdict(profile) for profile in PROFILES.values()],
    }
    if args.format == "json":
        print(json.dumps(status, indent=2))
        return

    print("NOA Ark OS Workspace Configuration Summary")
    print(f"Version: {SETTINGS_VERSION}\n")
    print("Portable Toolchain Paths:")
    print(f"  CARGO_HOME : {CARGO_PORTABLE}")
    print(f"  RUSTUP_HOME: {RUSTUP_PORTABLE}\n")
    print("Activation Profiles:")
    for profile in PROFILES.values():
        print(f"- {profile.platform}: {profile.description}")
    print("\nUse `python server/tools/dev_env_cli.py activate --platform <name>` to view detailed steps.")


def cmd_activate(args: argparse.Namespace) -> None:
    profile = PROFILES[args.platform]
    print(f"Activation Profile: {profile.description}\n")
    if args.format == "json":
        payload = asdict(profile)
        payload["commands"] = profile.commands
        payload["notes"] = profile.notes
        print(json.dumps(payload, indent=2))
        return

    print("Commands:")
    print(_format_commands(profile.commands))
    if profile.notes:
        print("\nNotes:")
        for note in profile.notes:
            print(f"  - {note}")


def cmd_doctor(args: argparse.Namespace) -> None:
    checks = {
        "cargo_portable_exists": CARGO_PORTABLE.exists(),
        "rustup_portable_exists": RUSTUP_PORTABLE.exists(),
        "activate_ps1_exists": (TOOLS_DIR / "activate-cargo.ps1").exists(),
        "activate_sh_exists": (TOOLS_DIR / "activate-cargo.sh").exists(),
    }
    checks["timestamp"] = datetime.now(UTC).isoformat()
    if args.format == "json":
        print(json.dumps(checks, indent=2))
        return

    print("Workspace Doctor")
    for key, value in checks.items():
        if key == "timestamp":
            continue
        status = "OK" if value else "MISSING"
        print(f"- {key.replace('_', ' ')}: {status}")
    print(f"\nChecked at: {checks['timestamp']}")


def cmd_diagnostics(_: argparse.Namespace) -> None:
    print("Rust-Analyzer Diagnostics Guidance")
    print(
        "Use rust-analyzer's CLI settings instead of editing VS Code files.\n"
        "Run the following command to inspect current diagnostics mapping:"
    )
    print("    rust-analyzer diagnostics")
    print(
        "If warnings are misclassified, add overrides to\n"
        "server/tools/rustup-portable/settings.toml or configure rust-analyzer via\n"
        "`rust-analyzer diagnostics edit` (see official docs)."
    )


def cmd_archives_list(_: argparse.Namespace) -> None:
    print("Archived IDE Bundles")
    for bundle in ARCHIVE_BUNDLES.values():
        print(f"- {bundle.name}: {bundle.description}")
        print(f"    original path : {bundle.original_path}")
        print(f"    storage asset : {bundle.base64_path}")


def cmd_archives_restore(args: argparse.Namespace) -> None:
    bundle = ARCHIVE_BUNDLES[args.bundle]
    tar_bytes = _read_bundle_tar(bundle)

    if args.mode == "tar":
        target = _resolve_output_path(args.output, bundle.default_tar_output)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(tar_bytes)
        print(f"Wrote tarball to {target}")
        return

    destination = _resolve_output_path(args.output, bundle.default_extract_dir)
    destination.mkdir(parents=True, exist_ok=True)
    if any(destination.iterdir()):
        raise SystemExit(
            f"Destination {destination} is not empty; specify an empty directory or remove files first."
        )

    with tarfile.open(fileobj=io.BytesIO(tar_bytes)) as tar:
        tar.extractall(destination)

    print(f"Extracted archive into {destination}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="NOA Ark OS workspace environment CLI"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    summary = subparsers.add_parser("summary", help="Show workspace summary")
    summary.add_argument("--format", choices=["human", "json"], default="human")
    summary.set_defaults(func=cmd_summary)

    activate = subparsers.add_parser("activate", help="Show activation steps")
    activate.add_argument("--platform", choices=list(PROFILES.keys()), required=True)
    activate.add_argument("--format", choices=["human", "json"], default="human")
    activate.set_defaults(func=cmd_activate)

    doctor = subparsers.add_parser("doctor", help="Verify required files exist")
    doctor.add_argument("--format", choices=["human", "json"], default="human")
    doctor.set_defaults(func=cmd_doctor)

    diagnostics = subparsers.add_parser(
        "diagnostics", help="Rust-analyzer troubleshooting tips"
    )
    diagnostics.set_defaults(func=cmd_diagnostics)

    archives = subparsers.add_parser(
        "archives", help="Inspect or restore archived IDE bundles"
    )
    archives_sub = archives.add_subparsers(dest="archives_command", required=True)

    archives_list = archives_sub.add_parser("list", help="List available bundles")
    archives_list.set_defaults(func=cmd_archives_list)

    archives_restore = archives_sub.add_parser(
        "restore", help="Restore a bundle to a tarball or directory"
    )
    archives_restore.add_argument(
        "--bundle", choices=list(ARCHIVE_BUNDLES.keys()), required=True
    )
    archives_restore.add_argument(
        "--mode", choices=["tar", "extract"], default="extract"
    )
    archives_restore.add_argument(
        "--output",
        help="Destination path for the tarball or extracted directory (defaults to out/)",
    )
    archives_restore.set_defaults(func=cmd_archives_restore)

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
