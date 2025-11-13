#!/usr/bin/env python3
"""Validate docs/tasks/index.json schema and references."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
INDEX_PATH = REPO_ROOT / "docs/tasks/index.json"
REQUIRED_FIELDS = ("task_code", "summary", "owning_files", "dependencies")


def _fail(message: str) -> None:
    print(f"Task index validation failed: {message}", file=sys.stderr)
    sys.exit(1)


def _validate_task(task: Any, codes: set[str], line: int) -> str:
    if not isinstance(task, dict):
        _fail(f"Task entry at index {line} must be an object.")

    for field in REQUIRED_FIELDS:
        if field not in task:
            _fail(f"Task entry at index {line} is missing required field '{field}'.")

    code = task["task_code"]
    if not isinstance(code, str) or not code.strip():
        _fail(f"Task entry at index {line} must provide a non-empty string task_code.")
    if code in codes:
        _fail(f"Duplicate task_code detected: {code}")

    summary = task["summary"]
    if not isinstance(summary, str) or not summary.strip():
        _fail(f"Task {code} must provide a non-empty string summary.")

    owning_files = task["owning_files"]
    if not isinstance(owning_files, list) or not owning_files:
        _fail(f"Task {code} must provide a non-empty list of owning_files.")
    for path in owning_files:
        if not isinstance(path, str) or not path.strip():
            _fail(f"Task {code} has an invalid owning file entry: {path!r}")
        resolved = REPO_ROOT / path
        if not resolved.exists():
            _fail(f"Task {code} references missing file: {path}")

    dependencies = task["dependencies"]
    if not isinstance(dependencies, list):
        _fail(f"Task {code} dependencies must be a list.")
    for dep in dependencies:
        if not isinstance(dep, str) or not dep.strip():
            _fail(f"Task {code} has an invalid dependency entry: {dep!r}")

    codes.add(code)
    return code


def main() -> None:
    if not INDEX_PATH.exists():
        _fail("docs/tasks/index.json is missing.")

    try:
        data = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _fail(f"Invalid JSON: {exc}")

    if not isinstance(data, dict):
        _fail("Top-level JSON must be an object.")

    metadata = data.get("metadata")
    if not isinstance(metadata, dict):
        _fail("metadata block is required and must be an object.")

    schema = metadata.get("schema")
    if not isinstance(schema, dict):
        _fail("metadata.schema must be an object describing required fields.")
    for field in REQUIRED_FIELDS:
        if field not in schema:
            _fail(f"metadata.schema must describe '{field}'.")

    tasks = data.get("tasks")
    if not isinstance(tasks, list) or not tasks:
        _fail("tasks must be a non-empty array.")

    codes: set[str] = set()
    for index, task in enumerate(tasks):
        _validate_task(task, codes, index)

    for task in tasks:
        code = task["task_code"]
        seen_deps: set[str] = set()
        for dep in task["dependencies"]:
            if dep == code:
                _fail(f"Task {code} cannot depend on itself.")
            if dep not in codes:
                _fail(f"Task {code} depends on unknown task_code '{dep}'.")
            if dep in seen_deps:
                _fail(f"Task {code} lists dependency '{dep}' more than once.")
            seen_deps.add(dep)

    print("Task index validation passed.")


if __name__ == "__main__":
    main()
