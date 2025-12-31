# Project Structure (DEFLEX Convention)

This project follows the DEFLEX per-project layout, mirroring the workspace-level organization:

- dev/     — active development artifacts, scratch code, notebooks, spikes
- docs/    — project documentation, specs, ADRs, governance copies
- logs/    — project-local logs; use consolidate_logs.sh to sync centrally
- tests/   — unit/integration tests, fixtures, test data
- tools/   — project-specific scripts and utilities
- config/  — project configuration (env files, YAML/JSON, schemas)
- data/    — (optional) analysis, exports, snapshots, storage

See workspace-level `documents/references/PROJECT_STRUCTURE.md` for canonical guidance.
