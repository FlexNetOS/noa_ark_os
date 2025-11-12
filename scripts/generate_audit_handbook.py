#!/usr/bin/env python3
"""Generate the auditors' handbook from pipeline telemetry and SOP notes."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parents[1]
LOG_DIR = ROOT / ".workspace" / "indexes"
SOP_FILE = ROOT / ".workspace" / "sop" / "development.md"
HANDBOOK_FILE = ROOT / "docs" / "audits" / "AUDITORS_HANDBOOK.md"

LOG_FILES = {
    "relocation": LOG_DIR / "relocation.log",
    "documentation": LOG_DIR / "documentation.log",
}


def load_entries(path: Path) -> List[Dict[str, Any]]:
    entries: List[Dict[str, Any]] = []
    if not path.exists():
        return entries
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            entries.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return entries


def format_timestamp(value: Any) -> str:
    if value is None:
        return "—"
    if isinstance(value, (int, float)):
        try:
            dt = datetime.fromtimestamp(float(value) / 1000.0, tz=timezone.utc)
            return dt.strftime("%Y-%m-%d %H:%M UTC")
        except (ValueError, OSError):
            return str(value)
    if isinstance(value, str):
        try:
            dt = datetime.fromisoformat(value.replace("Z", "+00:00"))
            return dt.strftime("%Y-%m-%d %H:%M UTC")
        except ValueError:
            return value
    return str(value)


def summarise(entries: List[Dict[str, Any]], ledger: str) -> Dict[str, Any]:
    meaningful = [e for e in entries if not e.get("event", {}).get("event_type", "").endswith("genesis")]
    latest = meaningful[-1] if meaningful else None
    return {
        "total": len(meaningful),
        "latest": latest,
        "ledger": ledger,
    }


def render_entry(entry: Dict[str, Any]) -> str:
    event = entry.get("event", {})
    policy = entry.get("policy", {})
    actor = event.get("actor", "unknown")
    target = event.get("target") or event.get("scope", "n/a")
    signature = policy.get("signature", "")
    timestamp = policy.get("record", {}).get("timestamp") or event.get("timestamp")
    return f"| {format_timestamp(timestamp)} | {actor} | {target} | `{signature[:16]}` |"


def sop_digest() -> str:
    if not SOP_FILE.exists():
        return "Operational SOP file not found."
    content = SOP_FILE.read_text().strip().splitlines()
    head = [line.lstrip("\ufeff") for line in content[:10]]
    return "\n".join(head)


def build_handbook() -> str:
    sections = []
    now = datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")
    sections.append(
        "# Auditors' Handbook\n\n"
        "This handbook is generated automatically from the immutable relocation and "
        "documentation ledgers together with active SOP references. "
        "It reflects the state as of {}.\n".format(now)
    )

    ledger_summaries = {name: summarise(load_entries(path), name) for name, path in LOG_FILES.items()}

    sections.append("## Ledger Overview\n")
    sections.append("| Ledger | Entries | Last Event | Scope |")
    sections.append("| --- | --- | --- | --- |")
    for name, summary in ledger_summaries.items():
        latest = summary["latest"]
        event_scope = latest.get("event", {}).get("scope", "—") if latest else "—"
        timestamp = format_timestamp(
            latest.get("policy", {}).get("record", {}).get("timestamp")
            if latest
            else None
        )
        sections.append(f"| {name.title()} | {summary['total']} | {timestamp} | {event_scope} |")

    for name, summary in ledger_summaries.items():
        sections.append(f"\n## {name.title()} Ledger Detail\n")
        entries = summary["latest"]
        all_entries = load_entries(LOG_FILES[name])
        if not any(e for e in all_entries if not e.get("event", {}).get("event_type", "").endswith("genesis")):
            sections.append("No operational entries recorded yet.\n")
            continue
        sections.append("| Timestamp | Actor | Target | Signature |")
        sections.append("| --- | --- | --- | --- |")
        for entry in all_entries[-10:]:
            if entry.get("event", {}).get("event_type", "").endswith("genesis"):
                continue
            sections.append(render_entry(entry))
        sections.append("")

    sections.append("## SOP Alignment\n")
    sections.append("The first steps from the operational SOP are included for quick reference:")
    sections.append("\n````markdown\n{}\n````\n".format(sop_digest()))

    sections.append("## Verification Checklist\n")
    sections.append("- Confirm ledger signatures chain correctly against the policy secret." )
    sections.append("- Validate that handbook regeneration runs after each SOP execution.")
    sections.append("- Cross-check relocation destinations with signed documentation targets.")

    return "\n".join(sections) + "\n"


def main() -> None:
    handbook = build_handbook()
    HANDBOOK_FILE.parent.mkdir(parents=True, exist_ok=True)
    HANDBOOK_FILE.write_text(handbook)
    print(f"Updated {HANDBOOK_FILE}")


if __name__ == "__main__":
    main()
