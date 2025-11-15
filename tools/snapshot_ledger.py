#!/usr/bin/env python3
"""Utility helpers for managing archive ledgers from Makefile recipes."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import List


def load_ledger(path: Path) -> List[dict]:
    if not path.exists():
        return []
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:  # pragma: no cover - Makefile guard
        raise SystemExit(f"Invalid JSON in {path}: {exc}")


def write_ledger(path: Path, data: List[dict]) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def ensure_snapshot(ledger: List[dict], bundle: str, sha: str) -> None:
    for entry in ledger:
        if entry.get("type") == "snapshot" and entry.get("bundle") == bundle:
            if entry.get("sha256") != sha:
                raise SystemExit("SHA-256 mismatch for bundle; aborting rollback")
            return
    raise SystemExit(f"No snapshot entry found for {bundle}")


def append_snapshot(path: Path, bundle: str, sha: str, commit: str, timestamp: str, prefix: str) -> None:
    ledger = load_ledger(path)
    ledger.append(
        {
            "type": "snapshot",
            "bundle": bundle,
            "sha256": sha,
            "timestamp": timestamp,
            "commit": commit,
            "prefix": prefix,
        }
    )
    write_ledger(path, ledger)


def append_rollback(path: Path, bundle: str, sha: str, timestamp: str) -> None:
    ledger = load_ledger(path)
    ensure_snapshot(ledger, bundle, sha)
    ledger.append(
        {
            "type": "rollback",
            "bundle": bundle,
            "sha256": sha,
            "timestamp": timestamp,
        }
    )
    write_ledger(path, ledger)


def main(argv: List[str]) -> None:
    if len(argv) < 2:
        raise SystemExit("Usage: snapshot_ledger.py <snapshot|rollback> ...")
    command = argv[1]
    if command == "snapshot":
        if len(argv) != 8:
            raise SystemExit("Usage: snapshot_ledger.py snapshot <ledger> <bundle> <sha> <commit> <timestamp> <prefix>")
        _, _, ledger, bundle, sha, commit, timestamp, prefix = argv
        append_snapshot(Path(ledger), bundle, sha, commit, timestamp, prefix)
    elif command == "rollback":
        if len(argv) != 6:
            raise SystemExit("Usage: snapshot_ledger.py rollback <ledger> <bundle> <sha> <timestamp>")
        _, _, ledger, bundle, sha, timestamp = argv
        append_rollback(Path(ledger), bundle, sha, timestamp)
    else:
        raise SystemExit(f"Unknown command: {command}")


if __name__ == "__main__":
    main(sys.argv)
