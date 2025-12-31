# Agent Map → Skeleton Cross-Walk (UPGRADED5)
See `docs/imported/agent_hierarchical_map.md` and `docs/imported/README.md`.

- CECCA / Trifecta / Board Agents → Orchestrator Plane (`orchestrator/`)
- Services / MicroAgentStack → Orchestrator + Sandbox
- Execution Plane (FS guard, RO mounts, verification) → `execution/core` + Make `fs-verity-*`
- Knowledge Capsules → `contracts/`, `sbom/`, `artifacts/` with minisign + fs-verity
- Error/Repair Agents → pre-push hooks + POP/event logs (extend later)
