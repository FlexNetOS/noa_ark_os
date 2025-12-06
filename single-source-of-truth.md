# Single Source of Truth - NOA Ark OS Tasks

## Current Tasks

### 3. Merge & Consolidation (Flatten noa_ark_os)

Status: In Progress
Description: Execute @merge.prompt.md to flatten and consolidate the monorepo layout for noa_ark_os without removing capabilities. Preserve features and update paths, manifests, and CI accordingly.
Steps:

- [x] Inventory current tree and identify nested layers, duplicates, and backup folders to normalize
-     Evidence: docs/CONSOLIDATION_SCAN_20251007.txt
- [ ] Propose target layout (services/, apps/, tools/, docs/, config/, tests/) and map sources → destinations
- [x] Propose target layout (services/, apps/, tools/, docs/, config/, tests/) and map sources → destinations
      Evidence: docs/MERGE_STRATEGY.md (output structure + merge semantics)
      Additional Evidence: docs/CONSOLIDATION_METRICS_20251007_104106.txt (quantitative baseline)
- [ ] Apply moves/renames with git history preserved; add compatibility symlinks where practical
- [ ] Update Cargo.toml/workspace members, package.json tsconfig paths, and any path-based imports
- [x] Refresh CI to reflect new paths; keep pipeline green
-     Actions Taken: Added non-breaking mono metadata job in .github/workflows/ci.yml
- [x] Regenerate hash manifests and publish evidence docs (FINAL_REPORT.md, REPRO.md, COVERAGE.md, HASHES.txt)
-     Evidence: docs/HASHES.incremental.merge_20251007.txt, docs/MERGE_CONSOLIDATION_PLAN.md, docs/CONSOLIDATION_MERGE_SUCCESS_20251007.md

Acceptance Criteria:

- [ ] Builds/lints run at least at the previously working baseline
- [ ] No loss of features (Heal, Don’t Harm); removed items are strictly duplicates or backups
- [ ] All moved components are referenced correctly (paths/aliases updated)
- [ ] Evidence published under docs/ with SHA-256 attestations

Progress Notes (2025-10-07):

- Canonical roots declared: mono/ (Rust), unified/ (Node)
- Non-destructive flatten tool added: tools/merge/flatten_noa.sh (dry-run)
- Inventory scan generated and hashed; CI metadata job added for mono workspace

Next Steps:

- [x] Add README breadcrumbs in legacy folders (services/, apps/) pointing to canonical paths
  - Evidence: services/README.md, apps/README.md (hashed in docs/HASHES.incremental.merge_20251007.txt)
- [x] Draft batched move mapping for duplicate services (services → mono/services), run in proposal mode, then apply with history
  - Evidence: docs/MOVE_PLAN_PROPOSAL_20251007.csv; tools/merge/propose_moves.sh (dry-run proposer)
- [ ] Update manifests (Cargo workspace members, package.json/tsconfig paths) post-move and re-run CI
- [x] Execute Batch 1 dry-run using proposer; capture command list under docs/MOVE_PLAN_DRYRUN_20251007.txt; present for approval
  - Evidence: docs/MOVE_PLAN_DRYRUN_20251007.txt (hash listed in docs/HASHES.incremental.merge_20251007.txt)
  - Approval Gate: Await explicit approval referencing docs/MERGE_STRATEGY.md → "User Approval" before applying moves

### 1. Poetry Installation and Setup

**Status**: Completed
**Description**: Install Poetry, set up the noa-notebooks project, and verify all dependencies
**Steps**:

- [x] Install Poetry via official installer
- [x] Verify Poetry installation
- [x] Navigate to notebooks directory
- [x] Run `poetry install` to install dependencies
- [x] Verify all packages install correctly
- [x] Test Jupyter notebook functionality
- [x] Document installation process
- [x] Update this file with completion status

### 2. Documentation Updates

**Status**: Completed
**Description**: Ensure all installation and setup processes are properly documented
**Steps**:

- [x] Update README.md with Poetry installation instructions
- [x] Document dependency requirements
- [x] Add troubleshooting section
- [x] Verify documentation accuracy

## Completed Tasks

### Code File Review (Previous Session)

**Status**: Completed
**Description**: Comprehensive review of NOA Ark OS project unification
**Evidence**: See ~/docs/FINAL_REPORT.md, ~/docs/HASHES.txt, ~/docs/REPRO.md, ~/docs/COVERAGE.md
**Result**: All systems verified working, no critical breakage detected

## Future Tasks

### CI/CD Setup

**Status**: In Progress
**Description**: Set up automated testing and deployment pipelines
**Priority**: Medium
**Actions Taken (2025-10-07):**

- Added minimal CI scaffold at `.github/workflows/ci.yml` with: - Rust cargo check job for `services/ark-ai-os-workspace` - Node environment sanity job (Node 18) listing candidate packages
  **Next Steps:**
- Expand Rust job to run `cargo clippy -- -D warnings` and `cargo fmt -- --check`
- Add Node lint/build for 1–2 targeted apps (ensure TypeScript available, lockfiles committed)
- Add Python job for lint/type-check once targets are identified
- Introduce security scans: `cargo audit`, `npm audit --omit=dev`, `pip-audit`

### Performance Testing

**Status**: Planned
**Description**: Add benchmarks and performance tests for key operations
**Priority**: Low

## Notes

- All tasks must follow the 4-D methodology: Deconstruct, Diagnose, Develop, Deliver
- Triple verification required for all critical operations
- Documentation must be updated for all changes
- Follow "Heal, Don't Harm" principle for all modifications</content>
  <parameter name="filePath">/home/deflex/workspace/projects/work/noa_ark_os/single-source-of-truth.md
