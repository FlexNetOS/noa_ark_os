#!/usr/bin/env python3
"""Offline triage analyzer producing minimal report."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def analyze(log: Path) -> dict[str, int]:
    counts = {"errors": 0, "warnings": 0}
    for line in log.read_text().splitlines():
        if "ERROR" in line.upper():
            counts["errors"] += 1
        if "WARN" in line.upper():
            counts["warnings"] += 1
    return counts


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("log", type=Path)
    parser.add_argument("--output", type=Path, default=Path("out/ci/triage.json"))
    args = parser.parse_args()

    summary = analyze(args.log)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
