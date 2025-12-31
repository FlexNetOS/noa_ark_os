#!/usr/bin/env python3
"""
NOA Task Execution Kit CLI (self-contained)

Commands:
  - plan: Parse sot.md and emit a structured plan JSON
  - run:  Execute a plan in parallel using registered agents

Zero external dependencies: Python 3 standard library only.
"""
import argparse
import json
import os
import sys
from datetime import datetime
from typing import Any, Dict

from planner import build_plan_from_sot
from executor import Executor


def cmd_plan(args: argparse.Namespace) -> int:
    sot_path = args.sot
    out_path = args.out
    if not os.path.isfile(sot_path):
        print(f"[error] sot file not found: {sot_path}", file=sys.stderr)
        return 1
    plan = build_plan_from_sot(sot_path)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(plan, f, indent=2, ensure_ascii=False)
    print(f"[ok] plan written: {out_path}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    plan_path = args.plan
    if not os.path.isfile(plan_path):
        print(f"[error] plan not found: {plan_path}", file=sys.stderr)
        return 1
    with open(plan_path, "r", encoding="utf-8") as f:
        plan: Dict[str, Any] = json.load(f)

    # Runtime options
    max_workers = int(args.max_workers)
    dry_run = bool(args.dry_run)
    allow_shell = not dry_run
    run_ts = datetime.utcnow().strftime("%Y%m%d_%H%M%SZ")
    run_dir = os.path.join(os.path.dirname(plan_path), "runs", run_ts)
    logs_dir = os.path.join(run_dir, "logs")
    os.makedirs(logs_dir, exist_ok=True)

    # Load config (optional)
    kit_root = os.path.dirname(__file__)
    manifest_path = os.path.join(kit_root, "config", "manifest.json")
    hooks_path = os.path.join(kit_root, "config", "hooks.json")
    manifest = {}
    hooks = {}
    if os.path.isfile(manifest_path):
        with open(manifest_path, "r", encoding="utf-8") as f:
            manifest = json.load(f)
    if os.path.isfile(hooks_path):
        with open(hooks_path, "r", encoding="utf-8") as f:
            hooks = json.load(f)

    ex = Executor(
        manifest=manifest,
        hooks=hooks,
        logs_dir=logs_dir,
        allow_shell=allow_shell,
        max_workers=max_workers,
    )
    result = ex.execute_plan(plan)

    # Persist run summary
    run_json = os.path.join(run_dir, "run.json")
    with open(run_json, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    print(f"[ok] run summary written: {run_json}")
    return 0 if result.get("status") == "ok" else 2


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="NOA self-contained task execution kit")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_plan = sub.add_parser("plan", help="Parse sot.md and emit plan JSON")
    p_plan.add_argument("--sot", required=True, help="Path to sot.md")
    p_plan.add_argument("--out", required=True, help="Path to write plan.json")
    p_plan.set_defaults(func=cmd_plan)

    p_run = sub.add_parser("run", help="Execute a plan.json in parallel")
    p_run.add_argument("--plan", required=True, help="Path to plan.json")
    p_run.add_argument("--max-workers", default="4", help="Parallel workers")
    p_run.add_argument("--dry-run", action="store_true", help="Do not run shell commands, write evidence only")
    p_run.set_defaults(func=cmd_run)

    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except KeyboardInterrupt:
        print("[warn] interrupted", file=sys.stderr)
        return 130


if __name__ == "__main__":
    sys.exit(main())

