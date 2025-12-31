# Workspace Memory Pipeline

The Vibe Kanban workspace now persists planner memory alongside the existing JSON snapshots to accelerate long-running goals.

## Data storage

| Layer | Path | Description |
| ----- | ---- | ----------- |
| SQLite | `.data/ai_assist.sqlite` | Tables `goal_lifecycle_events`, `agent_artifacts`, and `goal_embeddings` capture lifecycle telemetry, artifact metadata, and lightweight embeddings for similarity search. |
| File store | `.data/workspace-memory.json` | Append-only memory log that tracks workspace snapshots and per-goal execution traces with automatic eviction after the newest 200 traces and 32 snapshots. |
| Artifacts | `storage/artifacts/goals/` | Canonical location for planner artifacts referenced by the agent memory index. |

## API surface

- `POST /api/workspaces/{workspaceId}/boards/{boardId}/assist` now records lifecycle events, traces, embeddings, and returns a memory payload with suggestions, similar goals, and stored artifacts.
- `GET /api/goals/{goalId}/memory?workspaceId=` exposes the normalized memory payload so planners and UIs can fetch long-term insights without invoking assist.

## Telemetry

Every write to the memory layer emits structured logs through `@noa-ark/shared-ui/logging`:

- `goal_trace_recorded`, `goal_trace_evicted`
- `workspace_snapshot_recorded`, `workspace_snapshot_evicted`
- `goal_event_recorded`, `goal_embedding_upserted`, `goal_artifact_stored`

Use these events to verify retention behaviour and spot when policies evict older records. The warnings include retained/evicted counts so dashboards can alert before retention becomes tight.

## Eviction policy verification

The memory store trims:

- Goal traces to the newest **200** entries per goal.
- Workspace snapshots to the newest **32** entries per workspace.

Telemetry for `*_evicted` events reports the exact eviction count, giving operators confidence that the store remains within bounds. Tests and manual verification can tail application logs or query the SQLite tables to ensure the counters match expectations.

## UI integration

The Assist panel displays memory-derived insights, while the planner widget renders a high-level summary, similar goal matches, and a manual refresh control. These views consume the same `GoalMemoryInsights` shape returned by the API, ensuring parity between backend storage and user-facing recommendations.
