#!/usr/bin/env python3
"""Detect duplicate TODO tasks under .workspace/todo.

This helper scans Markdown TODO trackers for checkbox bullet entries and
reports duplicates based on a normalized task label (case-insensitive,
collapsed whitespace). It exits non-zero when duplicates are found so it
can gate CI/CD or autonomous policy agents.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Dict, List

TASK_PATTERN = re.compile(r"^\s*-\s*\[(?: |x|X)\]\s+(?P<label>.+)$")


def discover_markdown_files(todo_root: Path) -> List[Path]:
    """Yield Markdown trackers directly under the todo workspace."""
    markdown_files: List[Path] = []
    for path in sorted(todo_root.rglob("*.md")):
        # Skip Truth Gate artifacts tracked per task workspace.
        if path.name in {"claims.table.md", "truth_gate.checklist.md"}:
            continue
        markdown_files.append(path)
    return markdown_files


def extract_tasks(file_path: Path) -> List[Dict[str, object]]:
    tasks: List[Dict[str, object]] = []
    try:
        lines = file_path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError as err:
        raise SystemExit(f"Failed to read {file_path}: {err}") from err

    for idx, line in enumerate(lines, start=1):
        match = TASK_PATTERN.match(line)
        if not match:
            continue
        label = match.group("label").strip()
        normalized = " ".join(label.lower().split())
        tasks.append(
            {
                "path": str(file_path),
                "line": idx,
                "label": label,
                "normalized": normalized,
            }
        )
    return tasks


def detect_duplicates(todo_root: Path) -> Dict[str, List[Dict[str, object]]]:
    duplicates: Dict[str, List[Dict[str, object]]] = {}
    for tracker in discover_markdown_files(todo_root):
        for task in extract_tasks(tracker):
            normalized = task["normalized"]  # type: ignore[index]
            duplicates.setdefault(normalized, []).append(task)
    # Keep only entries that appear more than once.
    return {
        normalized: entries
        for normalized, entries in duplicates.items()
        if len(entries) > 1
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--todo-root",
        default=".workspace/todo",
        type=Path,
        help="Workspace TODO root to scan",
    )
    parser.add_argument(
        "--output",
        choices=["json", "text"],
        default="json",
        help="Output format",
    )
    args = parser.parse_args()

    todo_root = args.todo_root.resolve()
    if not todo_root.exists():
        print(f"todo root {todo_root} does not exist", file=sys.stderr)
        return 2

    duplicates = detect_duplicates(todo_root)
    if args.output == "json":
        print(
            json.dumps(
                {
                    "todo_root": str(todo_root),
                    "duplicate_task_count": len(duplicates),
                    "duplicates": duplicates,
                },
                indent=2,
                sort_keys=True,
            )
        )
    else:
        if duplicates:
            print("Duplicate tasks detected:")
            for normalized, entries in duplicates.items():
                print(f"- {normalized}")
                for entry in entries:
                    print(
                        f"  -> {entry['path']}:{entry['line']} — {entry['label']}"
                    )
        else:
            print("No duplicate tasks found.")

    return 1 if duplicates else 0


if __name__ == "__main__":
    sys.exit(main())
