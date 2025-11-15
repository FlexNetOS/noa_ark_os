"""
Planner: parse sot.md and build a structured plan.

No external deps; uses simple regex/line parsing to extract:
- Queues in section "Execution Queue & Task Hooks"
- Task IDs in backticks like `TASK-001`

Plan schema (v1):
{
  "version": 1,
  "source": "../../sot.md",
  "generated_at": "UTC ISO",
  "queues": [
    {
      "name": "Queue A — Workspace Bootstrapping",
      "tasks": [
        {"id": "TASK-001", "title": "TASK-001", "layer": "execution"},
        ...
      ]
    }
  ]
}
"""
import os
import re
from datetime import datetime
from typing import Any, Dict, List


QUEUE_HEADER_RE = re.compile(r"^\s*\d+\.\s*\*\*Queue\s+([A-Z])\s+—\s+(.+?)\*\*\s*$")
TASK_ID_RE = re.compile(r"`(TASK-[0-9]{3,})`")


def _extract_lines(path: str) -> List[str]:
    with open(path, "r", encoding="utf-8") as f:
        return f.read().splitlines()


def _find_execution_queue_section(lines: List[str]) -> int:
    for i, line in enumerate(lines):
        if line.strip().startswith("### G. Execution Queue & Task Hooks"):
            return i
    return -1


def build_plan_from_sot(sot_path: str) -> Dict[str, Any]:
    lines = _extract_lines(sot_path)
    start = _find_execution_queue_section(lines)
    plan: Dict[str, Any] = {
        "version": 1,
        "source": os.path.relpath(sot_path),
        "generated_at": datetime.utcnow().isoformat() + "Z",
        "queues": [],
    }
    if start < 0:
        return plan

    # Scan for Queue blocks until next top-level section (## or ### after G.)
    i = start
    current_queue = None
    while i < len(lines):
        line = lines[i]
        if i > start and (line.startswith("## ") or (line.startswith("### ") and not line.strip().startswith("### G."))):
            break

        m = QUEUE_HEADER_RE.match(line)
        if m:
            q_letter, q_name = m.group(1), m.group(2).strip()
            current_queue = {
                "name": f"Queue {q_letter} — {q_name}",
                "tasks": [],
            }
            plan["queues"].append(current_queue)
            i += 1
            continue

        if current_queue is not None:
            # Look for a line beginning with "- Tasks:"
            if line.strip().startswith("- Tasks:"):
                # Extract backticked task IDs
                for tid in TASK_ID_RE.findall(line):
                    current_queue["tasks"].append({
                        "id": tid,
                        "title": tid,
                        "layer": _default_layer_for(tid),
                    })
        i += 1

    return plan


def _default_layer_for(task_id: str) -> str:
    # Simple heuristic mapping by ranges
    try:
        n = int(task_id.split("-")[1])
    except Exception:
        return "execution"
    if 1 <= n <= 21:
        return "foundation"
    if 22 <= n <= 29:
        return "orchestration"
    if 30 <= n <= 41:
        return "platform"
    if 42 <= n <= 53:
        return "services"
    if 54 <= n <= 61:
        return "monitoring"
    return "execution"

