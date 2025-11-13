# Task Index

The `docs/tasks/index.json` file is the machine-readable source of truth for roadmap-aligned automation. It maps task codes to
summaries, owning files, and dependency edges so planners and agent tooling can reason about execution order without parsing
free-form documentation.

## Usage

1. Read `docs/tasks/index.json` to pull the latest roadmap anchors and metadata.
2. Validate the file locally with `python tools/task_index_validator.py` before generating task plans.
3. Mirror any updates in the owning documentation listed in each entry to keep the index authoritative.

## Scheduled Follow-up

- [ ] Backfill historical tasks documented under `docs/plans/` into the index so legacy anchors remain discoverable.
  - **Owner:** Documentation & Automation guilds
  - **Notes:** Coordinate with roadmap maintainers to migrate remaining anchors referenced outside the PM roadmap build kits.
