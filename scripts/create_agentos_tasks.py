#!/usr/bin/env python3
"""Create GitHub issues for the AGENTOS roadmap tasks.

The script parses the task definitions inside
``docs/plans/gap_remediation_tasks.md`` and, when requested,
creates (or reuses) GitHub issues for each AGENTOS item.

Usage examples
--------------
Dry-run (no network calls, prints the would-be payloads)::

    scripts/create_agentos_tasks.py --dry-run

Create/refresh issues in the upstream repository and append
issue links next to the in-document anchors::

    scripts/create_agentos_tasks.py --repo FlexNetOS/noa_ark_os --execute --update-doc

Environment requirements
------------------------
* GitHub CLI (``gh``) must be installed and authenticated
  (``gh auth status`` should report "Logged in").
* The ``GH_TOKEN``/``GITHUB_TOKEN`` environment variables are optional but
  recommended for CI usage.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional

REPO_ROOT = Path(__file__).resolve().parents[1]
DOC_PATH = REPO_ROOT / "docs/plans/gap_remediation_tasks.md"
MARKER_START = "<!-- BEGIN: GAP_REMEDIATION_TASKS -->"
MARKER_END = "<!-- END: GAP_REMEDIATION_TASKS -->"


@dataclass
class TaskDefinition:
    anchor_id: str
    code: str
    title: str
    body: str
    issue_url: Optional[str] = None

    @property
    def issue_title(self) -> str:
        return f"{self.code} — {self.title}"

    def issue_body(self) -> str:
        header = (
            "This issue was generated from docs/plans/gap_remediation_tasks.md\n"
            f"(anchor: #{self.anchor_id}).\n\n"
        )
        body = self.body.strip()
        return header + body


class TaskParserError(RuntimeError):
    pass


def load_document() -> str:
    if not DOC_PATH.exists():
        raise TaskParserError(f"Roadmap document not found: {DOC_PATH}")
    return DOC_PATH.read_text(encoding="utf-8")


def extract_task_block(document: str) -> str:
    if MARKER_START not in document or MARKER_END not in document:
        raise TaskParserError(
            "Roadmap markers not found. Ensure the document contains the generated block."
        )
    return document.split(MARKER_START, 1)[1].split(MARKER_END, 1)[0]


def parse_tasks(document: str) -> List[TaskDefinition]:
    block = extract_task_block(document)
    # Matches from the anchor through the subsequent metadata block. The lookahead
    # stops before the next anchor, the roadmap summary, or the end marker.
    pattern = re.compile(
        r"<a id=\"(?P<anchor>[^\"]+)\"></a>\n"  # anchor line
        r"### (?P<code>AGENTOS-\d+) — (?P<title>.+?)\n"  # heading
        r"(?P<body>.*?)(?=\n<a id=\"|\nRoadmap alignment:|<!-- END: GAP_REMEDIATION_TASKS -->)",
        re.DOTALL,
    )
    tasks: List[TaskDefinition] = []
    for match in pattern.finditer(block):
        body = match.group("body").strip()
        anchor = match.group("anchor")
        # Ensure body retains markdown structure and trailing newline for checklists.
        tasks.append(
            TaskDefinition(
                anchor_id=anchor,
                code=match.group("code").strip(),
                title=match.group("title").strip(),
                body=body,
            )
        )
    if not tasks:
        raise TaskParserError("No AGENTOS tasks found inside the roadmap block.")
    return tasks


def ensure_gh_available() -> None:
    if shutil.which("gh") is None:
        raise RuntimeError(
            "GitHub CLI (gh) is required but was not found in PATH. Run scripts/install_gh_cli.sh first."
        )


def run_gh_command(args: List[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, check=check, capture_output=True, text=True)
    return result


def lookup_existing_issue(repo: str, title: str, code: str) -> Optional[Dict[str, str]]:
    search_expr = f'"{code}" in:title'
    try:
        result = run_gh_command(
            [
                "gh",
                "issue",
                "list",
                "--repo",
                repo,
                "--state",
                "all",
                "--search",
                search_expr,
                "--json",
                "number,title,url,state",
            ]
        )
    except subprocess.CalledProcessError as exc:  # pragma: no cover - CLI error path
        raise RuntimeError(exc.stderr.strip() or exc.stdout.strip()) from exc

    try:
        payload = json.loads(result.stdout or "[]")
    except json.JSONDecodeError as exc:  # pragma: no cover - defensive branch
        raise RuntimeError(f"Failed to decode gh response: {exc}\n{result.stdout}") from exc

    for issue in payload:
        issue_title = issue.get("title", "")
        if issue_title.strip().startswith(f"{code} —"):
            return issue
    return None


def create_issue(repo: str, task: TaskDefinition) -> Dict[str, str]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as tmp:
        tmp.write(task.issue_body())
        tmp_path = tmp.name
    try:
        result = run_gh_command(
            [
                "gh",
                "issue",
                "create",
                "--repo",
                repo,
                "--title",
                task.issue_title,
                "--body-file",
                tmp_path,
                "--json",
                "number,url,title,state",
            ]
        )
    finally:
        os.unlink(tmp_path)

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Failed to decode gh response: {exc}\n{result.stdout}") from exc
    return payload


def update_document_links(document: str, tasks: Iterable[TaskDefinition]) -> str:
    tasks_by_anchor = {task.anchor_id: task for task in tasks if task.issue_url}
    if not tasks_by_anchor:
        return document

    def replacement(match: re.Match[str]) -> str:
        line = match.group(0)
        anchor = match.group("anchor")
        task = tasks_by_anchor.get(anchor)
        if not task or not task.issue_url:
            return line
        if "GitHub issue" in line:
            return line
        return f"{line} · [GitHub issue]({task.issue_url})"

    pattern = re.compile(
        r"^- (?P<prefix>.+? — \[View task\]\(#(?P<anchor>[^)]+)\))$",
        re.MULTILINE,
    )
    updated_document = pattern.sub(replacement, document)
    return updated_document


def print_plan(tasks: Iterable[TaskDefinition]) -> None:
    for task in tasks:
        separator = "=" * 80
        print(separator)
        print(task.issue_title)
        print(separator)
        print(task.issue_body())
        print()


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo",
        default="FlexNetOS/noa_ark_os",
        help="Target repository in the form <owner>/<repo> (default: %(default)s)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Do not call the GitHub API; print the planned issue payloads instead",
    )
    parser.add_argument(
        "--execute",
        action="store_true",
        help="Create issues on GitHub (requires gh to be installed and authenticated)",
    )
    parser.add_argument(
        "--update-doc",
        action="store_true",
        help="Append GitHub issue links next to the in-doc anchors once creation succeeds",
    )
    args = parser.parse_args(argv)

    if args.dry_run and args.execute:
        parser.error("--dry-run and --execute are mutually exclusive")
    if not args.dry_run and not args.execute:
        # Default to dry-run for safety.
        args.dry_run = True

    document = load_document()
    tasks = parse_tasks(document)

    if args.dry_run:
        print_plan(tasks)
        return 0

    ensure_gh_available()

    # Verify authentication before performing any writes.
    try:
        run_gh_command(["gh", "auth", "status", "--hostname", "github.com"])
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(
            "GitHub CLI is not authenticated. Run 'gh auth login' first."
        ) from exc

    updated = False
    for task in tasks:
        existing = lookup_existing_issue(args.repo, task.issue_title, task.code)
        if existing:
            task.issue_url = existing.get("url")
            print(f"✔ Reusing existing issue for {task.code}: {task.issue_url}")
            continue
        created = create_issue(args.repo, task)
        task.issue_url = created.get("url")
        print(f"✨ Created issue for {task.code}: {task.issue_url}")
        updated = True

    if args.update_doc and any(task.issue_url for task in tasks):
        new_document = update_document_links(document, tasks)
        if new_document != document:
            DOC_PATH.write_text(new_document, encoding="utf-8")
            updated = True
            print(f"📝 Updated {DOC_PATH.relative_to(REPO_ROOT)} with GitHub issue links")
        else:
            print("ℹ️ Document already contains GitHub issue links")

    if not updated:
        print("No changes made (issues already existed and document was up-to-date).")
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
