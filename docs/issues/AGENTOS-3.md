# AGENTOS-3 — Ship unified multi-surface UI shell

## Description
Create a shared UI shell, interaction model, and component library that can be deployed across desktop, browser, and CLI interfaces.

## Checklist
- [ ] Inventory current UI entry points and map to shared navigation, layout, and identity patterns.
- [ ] Produce component library packages consumable by desktop, web, and terminal surfaces.
- [ ] Implement unified session/state management with pluggable auth providers.
- [ ] Add cross-surface telemetry to capture usage and error flows.
- [ ] Provide migration guides for teams adopting the shared shell.

## Acceptance criteria
- UI shell renders consistently across supported surfaces with parity in core navigation and tasks.
- Component library versioning and distribution documented for all consuming teams.
- Unified state management passes integration tests in single-host profile (AGENTOS-2).
- Usage telemetry spans all surfaces and reports into the observability stack.

## Meta
- Issue: https://github.com/noa-ark/noa_ark_os/issues/103
- Owner: codex
- Priority: P1
- Status: Processing
- Depends on: #102
- Blocked by: #102
- Blocks: #106
