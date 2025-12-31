# Agent Map → Skeleton Cross-Walk
- CECCA / Trifecta Court / Board Agents → Orchestrator Plane (`orchestrator/policies/*`, `agent_runtime/*`)
- Orchestration & Services / MicroAgentStack → Orchestrator + Sandbox (`orchestrator/*`, `sandbox/*`)
- Execution Plane (DAG, Sandbox, FS Guard, Temp WS, RLIMIT) → Execution Plane (`execution/*`, Make targets)
- Knowledge Capsules (KIDX/KSCHEMA/KMETRICS/…) → Artifacts + Contracts (`artifacts/`, `contracts/`, `sbom/`)
- Error Agents (detection, rollback, forensics) → POP + event logs (`orchestrator/state/event_log.jsonl`) and strict pre-push
