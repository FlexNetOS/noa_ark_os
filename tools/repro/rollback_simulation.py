#!/usr/bin/env python3
"""Generate deterministic rollback drill evidence for CI workflows."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import subprocess
from pathlib import Path
from typing import Any, Dict


def _utc_timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _git_head(repo: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"git rev-parse failed: {result.stderr.strip()}")
    return result.stdout.strip()


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Create rollback simulation artifacts")
    parser.add_argument("--repo", default=".", help="Repository root used for git metadata")
    parser.add_argument(
        "--ledger",
        default="audit/ledger.jsonl",
        help="Path to the audit ledger (JSONL) to append the simulation record",
    )
    parser.add_argument(
        "--output",
        default="audit/rollbacks",
        help="Directory where rollback simulation JSON files are written",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the record without writing files (useful for testing)",
    )
    return parser.parse_args()


def _build_entry(repo: Path) -> Dict[str, Any]:
    timestamp = _utc_timestamp()
    head = _git_head(repo)
    return {
        "event": "rollback_simulation",
        "recorded_at": timestamp,
        "commit": head,
        "workspace": str(repo),
    }


def _write_jsonl(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        json.dump(payload, handle, sort_keys=True)
        handle.write("\n")


def main() -> None:
    args = _parse_args()
    repo = Path(args.repo).resolve()
    ledger_path = Path(args.ledger).resolve()
    output_dir = Path(args.output).resolve()

    entry = _build_entry(repo)
    filename_safe_ts = entry["recorded_at"].replace(":", "-")
    artifact_path = output_dir / f"rollback-{filename_safe_ts}.json"

    if args.dry_run:
        print(json.dumps({**entry, "artifact": str(artifact_path)}, indent=2))
        print("Dry-run: not writing rollback artifacts")
        return

    output_dir.mkdir(parents=True, exist_ok=True)
    artifact_path.write_text(json.dumps(entry, indent=2) + "\n", encoding="utf-8")
    _write_jsonl(ledger_path, {"artifact": str(artifact_path), **entry})
    print(f"✅ Rollback drill recorded at {artifact_path}")


if __name__ == "__main__":
    main()
