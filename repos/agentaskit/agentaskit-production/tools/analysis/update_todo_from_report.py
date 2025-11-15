#!/usr/bin/env python3
import re, json, sys
from pathlib import Path

REPORT = Path("agentaskit-production/docs/reports/cross_reference/artifacts/report.json")
TODO = Path("agentaskit-production/core/src/orchestration/tasks.todo")
BLOCK_BEGIN = "<!-- AUTO:WORKFLOW-009:SUBTASKS:BEGIN -->"
BLOCK_END = "<!-- AUTO:WORKFLOW-009:SUBTASKS:END -->"


def load_report():
    if not REPORT.exists():
        return None
    data = json.loads(REPORT.read_text())
    missing_dirs = data.get("production_missing_dirs", [])
    missing_basenames = data.get("missing_in_production_basenames", [])
    return missing_dirs, missing_basenames


def build_subtasks(missing_dirs, missing_basenames):
    lines = []
    if missing_dirs:
        lines.append("- [ ] Create missing production dirs:")
        for d in missing_dirs:
            lines.append(f"  - [ ] {d}")
    if missing_basenames:
        lines.append("- [ ] Review basenames present outside production (top sample):")
        for n in missing_basenames[:50]:
            lines.append(f"  - [ ] {n}")
    if not lines:
        lines.append("- [x] No gaps detected in current report.json")
    return "\n".join(lines)


def replace_block(text, begin, end, payload):
    pattern = re.compile(re.escape(begin) + r"[\s\S]*?" + re.escape(end))
    return pattern.sub(begin + "\n" + payload + "\n" + end, text)


def main():
    r = load_report()
    if r is None:
        print("report.json not found; nothing to update", file=sys.stderr)
        sys.exit(0)
    missing_dirs, missing_basenames = r
    content = TODO.read_text(encoding="utf-8")
    payload = build_subtasks(missing_dirs, missing_basenames)
    updated = replace_block(content, BLOCK_BEGIN, BLOCK_END, payload)
    if updated != content:
        TODO.write_text(updated, encoding="utf-8")
        print("tasks.todo updated from report.json")
    else:
        print("no changes to tasks.todo")

if __name__ == "__main__":
    main()
