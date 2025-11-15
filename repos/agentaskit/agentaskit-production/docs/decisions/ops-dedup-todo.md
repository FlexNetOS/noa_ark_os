# Decision: Single-source TODO

- Source of truth: `agentaskit-production/.todo`
- Subject inbox: `agentaskit-production/agentask.subject.todo` (ingested by CI), may be archived when empty
- Program backlog: `core/src/orchestration/tasks.todo` mirrors and expands
- Rationale: deduplication reduces drift; CI lints and syncs
