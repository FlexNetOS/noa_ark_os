# MERGE_CONSOLIDATION_PLAN

Scope: Flatten and consolidate noa_ark_os while preserving capabilities (Heal, Don’t Harm). Canonicalize layout and reduce duplication without deleting functionality.

Canonical layout targets

- Rust: mono/ as canonical workspace; services/ (legacy) retained but de-emphasized
- Node: unified/ (pnpm workspace) as canonical; apps/ legacy retained with pointers
- Shared: tools/, docs/, config/, tests/ at repo root as the stable anchors

High-level mapping

- services/_ (legacy crates) → mono/services/_ (canonical modules) where equivalents exist
- apps/_ (legacy) → unified/packages/_ or unified/apps/\* as appropriate
- archives, backup Cargo\__.toml and _\_backup files → docs/archive/ with provenance notes

Execution strategy (non-destructive)

1. Declare canonical roots in docs and SoT; update CI to exercise canonical roots
2. Provide flatten script (dry-run default) to detect and optionally move duplicates
3. Add compatibility breadcrumbs (README pointers) in legacy folders
4. After verification, phase moves per service family and update manifests/paths

Risks & mitigations

- Build breakage → Phase CI tightening only after green baseline; perform moves in small batches
- Hidden references → Use grep to surface path strings and update imports
- Data loss → No deletions; move-only with logs and hash attestations

Acceptance criteria

- CI continues green baseline on canonical roots
- Evidence: FINAL_REPORT, REPRO, COVERAGE, HASHES with scan logs
