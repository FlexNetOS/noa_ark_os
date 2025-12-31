"""
Agent registry and built-in agents.

No external deps. Agents expose .execute(queue_name, task, hook, logs_dir) -> dict
"""
import json
import os
import subprocess
from datetime import datetime
from typing import Any, Dict


class Agent:
    def name(self) -> str:
        return self.__class__.__name__

    def execute(self, queue_name: str, task: Dict[str, Any], hook: Dict[str, Any], *, logs_dir: str) -> Dict[str, Any]:
        raise NotImplementedError


class NoOpAgent(Agent):
    def execute(self, queue_name: str, task: Dict[str, Any], hook: Dict[str, Any], *, logs_dir: str) -> Dict[str, Any]:
        # Emit a small evidence file indicating a stub execution
        tid = task.get("id", "unknown")
        ts = datetime.utcnow().strftime("%Y%m%d_%H%M%SZ")
        path = os.path.join(logs_dir, f"{tid}_noop_{ts}.txt")
        os.makedirs(logs_dir, exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(f"NOOP: {tid} in {queue_name} at {ts}\n")
        return {"task": tid, "queue": queue_name, "agent": self.name(), "status": "noop", "evidence": path}


class ShellAgent(Agent):
    def __init__(self, allow_shell: bool) -> None:
        self.allow_shell = allow_shell

    def execute(self, queue_name: str, task: Dict[str, Any], hook: Dict[str, Any], *, logs_dir: str) -> Dict[str, Any]:
        tid = task.get("id", "unknown")
        cmds = hook.get("commands", []) if isinstance(hook, dict) else []
        if not self.allow_shell or not cmds:
            return {"task": tid, "queue": queue_name, "agent": self.name(), "status": "noop"}
        os.makedirs(logs_dir, exist_ok=True)
        all_ok = True
        cmd_logs = []
        for i, spec in enumerate(cmds, start=1):
            cwd = spec.get("cwd") if isinstance(spec, dict) else None
            cmd = spec.get("cmd") if isinstance(spec, dict) else None
            if not cmd:
                continue
            log_path = os.path.join(logs_dir, f"{tid}_cmd{i}.log")
            try:
                res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
                with open(log_path, "w", encoding="utf-8") as f:
                    f.write("$ "+" ".join(cmd)+"\n")
                    f.write(res.stdout)
                    if res.stderr:
                        f.write("\n[stderr]\n"+res.stderr)
                cmd_logs.append({"cmd": cmd, "cwd": cwd, "rc": res.returncode, "log": log_path})
                if res.returncode != 0:
                    all_ok = False
            except Exception as e:  # pragma: no cover
                all_ok = False
                cmd_logs.append({"cmd": cmd, "cwd": cwd, "error": str(e)})
        status = "ok" if all_ok else "error"
        return {"task": tid, "queue": queue_name, "agent": self.name(), "status": status, "commands": cmd_logs}


class AgentRegistry:
    def __init__(self) -> None:
        self._agents: Dict[str, Agent] = {}

    def register(self, key: str, agent: Agent) -> None:
        self._agents[key] = agent

    def get(self, key: str) -> Agent:
        if key not in self._agents:
            raise KeyError(f"agent not registered: {key}")
        return self._agents[key]

