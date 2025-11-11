# UI Feature Review

This document captures the current capabilities that ship with the NOA ARK OS UI surfaces, the major gaps that prevent them from acting as a single entry point, and the concrete upgrades delivered in this change.

## Current Assets

- **`ui/noa-dashboard`** – static HTML dashboard with hard-coded metrics and a lightweight prompt box that targets legacy `/api/monitoring` and `/api/inference` endpoints.
- **`ui/core`** – Rust workspace offering an embeddable shell, state management, and analytics/workflow primitives but without any direct wiring to the dashboard experience.
- **`ui/vibe-kanban`** – Next.js board focused on task planning; it currently operates independently from the dashboard and shell packages.
- **Server backends** – Disparate FastAPI/Go services without a unified gateway for UI consumption.

## Gaps Identified

1. **No consolidated control plane** – The shipped dashboard only renders static cards. Workflow orchestration, agent control, CI visibility, storage introspection, analytics, and chat each live in separate silos or remain unexposed.
2. **Missing unified API contract** – Front-end calls still reference deprecated monitoring/inference endpoints, so new shell capabilities are unreachable from the web experience.
3. **Lack of AI-assisted development bridge** – The chat box cannot trigger workflows or navigate UI modules, preventing “continue development from chat” scenarios.
4. **Absent real-time signal** – There is no surface that consumes the WebSocket event stream broadcast by the unified gateway, so users lack feedback when workflows, agents, or CI react to commands.
5. **Testing & reliability coverage** – Python gateway tests previously exercised only health checks, leaving core feature routes unverified.

## Upgrades Implemented

- Rebuilt the dashboard into a unified launchpad that lists workflows, agents, CI pipelines, storage artifacts, analytics, chat, and live event telemetry in one page.
- Updated every interaction to call the FastAPI unified gateway (`server/python/unified_api`) so the UI reflects authoritative data sources.
- Added chat-to-action wiring that triggers workflow runs and performs navigation updates through the shared API contract.
- Connected a WebSocket event log to `/ws/events`, surfacing feedback from workflow triggers, agent scale operations, and CI reruns.
- Hardened the unified gateway modules with missing imports, richer schemas, and expanded pytest coverage to validate workflows, analytics, chat, and CI endpoints.

These changes align the UI codebase with the unified shell roadmap and deliver an actionable single surface for operations and AI-assisted development flows.
