#!/usr/bin/env python3
"""Rollout gating based on drift detection report."""

import argparse
import json
import os
import pathlib
import sys
from typing import Any, Dict, List


def load_report(path: pathlib.Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    with path.open("r", encoding="utf-8") as handle:
        return [json.loads(line) for line in handle if line.strip()]


def compute_status(entries: List[Dict[str, Any]], threshold: float) -> str:
    if not entries:
        return "ready"
    drift_count = sum(1 for entry in entries if entry.get("status") == "drift")
    ratio = drift_count / float(len(entries))
    return "blocked" if ratio > threshold else "ready"


def main() -> None:
    parser = argparse.ArgumentParser(description="Determine rollout status")
    parser.add_argument("--report", type=pathlib.Path, required=True)
    parser.add_argument("--threshold", type=float, default=0.05)
    args = parser.parse_args()

    entries = load_report(args.report)
    status = compute_status(entries, args.threshold)
    print(f"Rollout status: {status}")
    env_path = os.environ.get("GITHUB_ENV")
    if env_path:
        with pathlib.Path(env_path).open("a", encoding="utf-8") as handle:
            handle.write(f"ML_ROLLOUT_STATUS={status}\n")
    else:
        print(f"ML_ROLLOUT_STATUS={status}")
    if status == "blocked":
        sys.exit(0)


if __name__ == "__main__":
    main()
