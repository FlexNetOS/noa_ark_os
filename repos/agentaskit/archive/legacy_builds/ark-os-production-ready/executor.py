"""
Parallel plan executor with a simple multi-agent architecture.

Agents are discovered via the manifest and registry; each task is dispatched
through a routing step:
  - If a task matches a hook (by queue name or task id) with commands and
    shell is allowed, run those commands.
  - Otherwise, execute a no-op that emits an evidence stub.

Zero external deps: Python standard library only.
"""
import json
import os
import subprocess
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple

from agents.registry import AgentRegistry, NoOpAgent, ShellAgent


class Executor:
    def __init__(
        self,
        manifest: Dict[str, Any],
        hooks: Dict[str, Any],
        logs_dir: str,
        allow_shell: bool,
        max_workers: int = 4,
    ) -> None:
        self.manifest = manifest or {}
        self.hooks = hooks or {}
        self.logs_dir = logs_dir
        self.allow_shell = allow_shell
        self.max_workers = max_workers
        self.registry = AgentRegistry()
        self._register_default_agents()

    def _register_default_agents(self) -> None:
        self.registry.register("noop", NoOpAgent())
        self.registry.register("shell", ShellAgent(allow_shell=self.allow_shell))

    def execute_plan(self, plan: Dict[str, Any]) -> Dict[str, Any]:
        started = datetime.utcnow().isoformat() + "Z"
        tasks: List[Tuple[str, Dict[str, Any], Dict[str, Any]]] = []  # (queue_name, task, hook)

        for queue in plan.get("queues", []):
            qname: str = queue.get("name", "")
            qhook = self.hooks.get(qname, {})
            for task in queue.get("tasks", []):
                thook = self.hooks.get(task.get("id", ""), {})
                # precedence: task-specific hook overrides queue hook
                hook = thook or qhook or {}
                tasks.append((qname, task, hook))

        results: List[Dict[str, Any]] = []
        lock = threading.Lock()

        def run_one(qname: str, task: Dict[str, Any], hook: Dict[str, Any]) -> Dict[str, Any]:
            agent_key = "shell" if (hook.get("commands") and self.allow_shell) else "noop"
            agent = self.registry.get(agent_key)
            try:
                res = agent.execute(qname, task, hook, logs_dir=self.logs_dir)
            except Exception as e:  # pragma: no cover
                res = {
                    "task": task.get("id"),
                    "queue": qname,
                    "agent": agent_key,
                    "status": "error",
                    "error": str(e),
                }
            with lock:
                results.append(res)
            return res

        with ThreadPoolExecutor(max_workers=self.max_workers) as pool:
            futs = [pool.submit(run_one, q, t, h) for (q, t, h) in tasks]
            for _ in as_completed(futs):
                pass

        ended = datetime.utcnow().isoformat() + "Z"
        summary = {
            "status": "ok" if all(r.get("status") in ("ok", "noop") for r in results) else "error",
            "started": started,
            "ended": ended,
            "results": results,
        }
        return summary

