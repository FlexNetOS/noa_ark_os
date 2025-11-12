# Build Kits Export Guide

The roadmap build kits provide machine-readable snapshots of the PM roadmap for downstream automations, dashboards, and release tooling.

## Files

- `build_kits/pm_roadmap.csv` — comma-separated export with multi-value fields joined by `|;`.
- `build_kits/pm_roadmap.json` — JSON array mirroring the CSV schema for programmatic ingestion.

Both exports are generated from the `<!-- BEGIN: PM_ROADMAP -->` block in `docs/plans/roadmap.md`. Update the markdown and rerun the exporter to keep the artifacts in sync.

## Schema

| Column / Field | Description |
| --- | --- |
| `code` | Task identifier (`AGENTOS-*`). |
| `title` | Human-readable task title. |
| `theme` | Parent theme label used in the roadmap summary. |
| `description` | Summary suitable for release notes or dashboards. |
| `priority` | Priority indicator (e.g., `P0`, `P1`). |
| `status` | Lifecycle status (Proposed/In Progress/etc.). |
| `depends_on` | Zero or more dependencies joined with `|;`. |
| `acceptance_criteria` | Acceptance statements joined with `|;` in CSV or emitted as an array in JSON. |

## Export workflow

```bash
pnpm install
pnpm export:roadmap
```

The command reads `docs/plans/roadmap.md`, validates that each summary anchor has a matching task detail, and rewrites both build kit files. The script is idempotent and safe to run repeatedly.

Commit the regenerated files to ensure CI/CD and downstream automations stay aligned with the source roadmap.
