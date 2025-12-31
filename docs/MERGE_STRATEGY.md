# Merge Strategy: noa_merge_20251007

## Context

- Purpose: Flatten and consolidate noa_ark_os to a single canonical tree while preserving all capabilities (Heal, Don’t Harm)
- Success Criteria: CI baseline remains green, no feature loss, all moved components referenced correctly, evidence with hashes
- Constraints: Non-destructive first; moves applied in small batches with history; legacy breadcrumbs retained until cutover
- Scope: Legacy roots services/, apps/ → Canonical roots mono/services/, unified/ (plus any mono/apps)

## Merge Semantics

```yaml
merge_request:
  id: "noa_merge_20251007"
merge_type: "CONSOLIDATION"
output_structure: "TARGET/ as base with X/Y/Z additions (overlay)"
duplicate_resolution:
  - "LATEST_WINS"
  - "MERGE_CONTENT" # docs only
model: "Model B (Aggressive) for code + Model C (Balanced) for docs"
```

## Sources

- services/ (legacy)
- apps/ (legacy)
- mono/services/ (canonical)
- unified/ (canonical JS/TS)

## Target

- Location: repo root with canonical: mono/ (Rust), unified/ (JS/TS)
- Expected Size: ~ as current canonical + unique legacy content
- Expected Lines: N/A (tracked via hashes)

## Proposed Output Structure (overlay)

```
repo/
├── mono/
│   ├── services/           # canonical Rust services
│   └── apps/               # when Rust apps exist
├── unified/                # canonical JS/TS workspace
├── services/README.md      # breadcrumb to mono/services/
├── apps/README.md          # breadcrumb to unified/
└── docs/
    ├── MERGE_CONSOLIDATION_PLAN.md
    ├── MERGE_STRATEGY.md
    ├── MOVE_PLAN_PROPOSAL_20251007.csv
    └── CONSOLIDATION_SCAN_20251007.txt
```

## Conflict Resolution Rules

1. For Rust crates duplicated in services/_ and mono/services/_ → mono wins (LATEST_WINS), port unique features if found
2. For Node apps duplicated in apps/_ and unified/packages/_ → unified wins; port configs as needed
3. Documentation → MERGE_CONTENT where safe with version markers
4. Backups or alt-manifests (Cargo\__.toml, _\_backup) → archive under docs/archive/ with provenance

## Policies

- ✓ Heal, Don’t Harm
- ✓ Upgrades Only
- ✓ No data loss; no deletions without archived proof

## User Approval

- [ ] Strategy reviewed and approved
- Approver: DEFLEX
- Date: 2025-10-07

## Quantitative Baseline (for verification)

Evidence file: docs/CONSOLIDATION_METRICS_20251007_104106.txt

- services: files=34226, dirs=5331, size_bytes=3506207409, tree_digest=738c561c7cc45b69fc6a57e3d0cf01644dd5f00bc8e7ce411c4d0558724f877c
- apps: files=38399, dirs=4977, size_bytes=465029482, tree_digest=4771fb5c53ea6879c93f5a75c6def37079b89514a6460f1f69d3427bf10fc4ed
- mono/services: files=27, dirs=12, size_bytes=313423, tree_digest=09e559d2902cea5dae56df42d817b09a36935b9b87b45d3ff06777e75eab5636
- mono/apps: files=14, dirs=5, size_bytes=410497, tree_digest=f760b0fbda12d886d8447b3ff7599bc83f4c905d9ca68ac3d4c43fe564d136f8
- unified: files=541, dirs=136, size_bytes=46787851, tree_digest=9088c48afc36d5f204056181a760791fd28d84fb1c11350e498ad30118ad0ca6

SHA-256(doc): 596329f24f791f0db56e3c364c7d5d83ad0f97cd3e2ab886f6e3e70acb35378b
