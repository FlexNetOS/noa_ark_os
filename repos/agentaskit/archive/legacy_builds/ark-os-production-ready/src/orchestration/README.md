# NOA Task Execution Kit (Self-Contained)

A zero-external-dependency task execution kit that parses `sot.md`, builds a multi-layer, multi-agent plan, and executes tasks in parallel. Designed as a clean, self-contained reference build inspired by the Buildkit `noa_deployment` kit, but purpose-built for executing SoT tasks locally within this monorepo.

- Source of truth: `../../sot.md`
- Reference kit: `../../../../projects/work/buildkit-recode/build/kits/noa_deployment/updated_kit/`
- No external libraries — Python standard library only

## Features
- Multi-layer agent architecture (board, orchestration, execution, audit)
- Agent registry with simple plugin mechanism
- Planner parses `sot.md` to extract queues and task identifiers
- Parallel execution of tasks with configurable workers
- Evidence and run logs written under `state/` (JSON + text)
- Optional import of reference agents from Buildkit

## Quick Start

```bash
cd tools/task_exec_kit
python3 cli.py plan --sot ../../sot.md --out state/plan.json
python3 cli.py run --plan state/plan.json --max-workers 4 --dry-run
```

- `--dry-run` creates evidence stubs without executing shell commands.
- Omit `--dry-run` to enable the `shell` capability (if a task maps to a command).

## Concepts
- Layers: `board`, `orchestration`, `execution`, `audit`
- Agents: simple Python classes registered in `agents/registry.py`
- Planner: `planner.py` pulls queues like “Queue A — Workspace Bootstrapping” and task IDs like `TASK-001` from `sot.md`.
- Executor: `executor.py` dispatches tasks to agents concurrently.

## Buildkit Reference Import (Optional)

You can produce a curated agent list from the Buildkit kit to seed the registry:

```bash
python3 tools/task_exec_kit/scripts/import_buildkit_agents.py \
  --src ../../projects/work/buildkit-recode/build/kits/noa_deployment/updated_kit/agents_for_parallel.json \
  --out tools/task_exec_kit/config/agents_parallel.json
```

This import is strictly optional and sanitized (duplicates removed, non-string entries dropped).

## Configuration
- `config/manifest.json`: Layers and default agents
- `config/hooks.json`: Map task IDs or queue labels to actions (e.g., shell commands)

Example hooks (non-binding, commands are optional):
```json
{
  "Queue A — Workspace Bootstrapping": {
    "commands": [
      {"cwd": "../../", "cmd": ["cargo", "metadata", "--no-deps"]},
      {"cwd": "../../", "cmd": ["cargo", "check", "--workspace"]}
    ]
  }
}
```

## Evidence
- Plans: `state/plan.json`
- Runs: `state/runs/<timestamp>/run.json`
- Logs: `state/runs/<timestamp>/logs/`

## Safety & Guardrails
- Default is `--dry-run` to avoid unintended changes
- Shell execution only runs commands explicitly mapped in `config/hooks.json`
- All outputs remain inside the monorepo tree

## Status
Initial clean build scaffolded. Extend by adding agent capabilities and task hooks.

```
Tools Version: task_exec_kit v0.1.0 (self-contained)
```
