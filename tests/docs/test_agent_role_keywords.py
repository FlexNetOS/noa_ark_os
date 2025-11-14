from __future__ import annotations

from pathlib import Path

import pytest

HUMAN_ROLE_KEYWORDS = {
    "operator",
    "operators",
    "owner",
    "owners",
    "approver",
    "approvers",
    "maintainer",
    "maintainers",
    "user",
    "users",
}

CANONICAL_AGENT_TOKENS = {
    "agent",
    "orchestrator",
    "planner",
    "worker",
    "verifier",
}

TARGET_DIRECTORIES = [
    Path("docs/operations"),
    Path("workflow"),
    Path("policies"),
]

VALID_EXTENSIONS = {".md", ".markdown", ".mdx", ".yaml", ".yml", ".txt"}


def _iter_target_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for directory in TARGET_DIRECTORIES:
        target = root / directory
        if not target.exists():
            continue
        for file_path in target.rglob("*"):
            if file_path.is_file() and file_path.suffix.lower() in VALID_EXTENSIONS:
                files.append(file_path)
    return files


@pytest.mark.parametrize("doc_path", _iter_target_files(Path(__file__).resolve().parents[2]))
def test_human_role_keywords_always_map_to_agents(doc_path: Path) -> None:
    text = doc_path.read_text(encoding="utf-8")
    failures: list[str] = []
    for idx, line in enumerate(text.splitlines(), start=1):
        lowered = line.lower()
        for keyword in HUMAN_ROLE_KEYWORDS:
            if keyword not in lowered:
                continue
            if f"--{keyword}" in lowered:
                continue
            if not any(token in lowered for token in CANONICAL_AGENT_TOKENS):
                failures.append(f"{doc_path}:{idx}: '{keyword}' -> {line.strip()}")
    if failures:
        pytest.fail(
            "Human-role keywords must be paired with agent terminology:\n" + "\n".join(failures)
        )
