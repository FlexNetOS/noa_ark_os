# How‑to‑Use v3 — Heal, Don’t Break, Update

This version adds a **CSV Schema v1.0**, a **Normalizer/Healer**, explicit **Capsule** schema, and refreshed graphs.

## 0) Files
- `schema/CSV_SCHEMA_v1.md` — headers & enums
- `schema/capsule.schema.json`, `schema/manifest.schema.json`
- `tools/normalize_csv.py` — run on any CSV to auto-heal
- `All_Inclusive_Agent_Directory_v6_plus.normalized.csv` — output from your CSV
- `stack.manifest.json` — from normalized CSV
- `graphs/capsule_flow.mmd`, `graphs/orchestration.mmd`, `graphs/stacks.mmd`
- `reports/csv_validation_report_post_heal.md`

## 1) Heal your CSV (offline)
```bash
python3 tools/normalize_csv.py --in "All_Inclusive_Agent_Directory_v6_plus.csv" --out All_Inclusive_Agent_Directory_v6_plus.normalized.csv
```

## 2) Generate & validate manifest
```bash
# Already generated as stack.manifest.json, but you can re-run:
python3 utils/csv_to_manifest.py --csv ./All_Inclusive_Agent_Directory_v6_plus.normalized.csv --out ./stack.manifest.json
python3 utils/validate_manifest.py --in ./stack.manifest.json
```

## 3) Dry-run deployment
```bash
bash scripts/deploy.sh --manifest ./stack.manifest.json --dry-run
# or
powershell -File .\scripts\deploy.ps1 -Manifest .\stack.manifest.json -DryRun
```

## 4) Capsule lifecycle
See `graphs/capsule_flow.mmd` — create → bind → persist → run → validate → upgrade/heal → compress → connect → integrate → loop.

---

## Capsules Are Built-In (Read This First)

- Every **agent** is a **capsule** with explicit fields: `name, layer, scope, inputs, outputs, tools, guardrails, escalation_to`.
- The **capsule lifecycle** is standardized and automated (see `graphs/capsule_flow.mmd`):
  1) **Create** → 2) **Bind tools/guardrails** → 3) **Persist inputs/state** → 4) **Run** →
  5) **Validate/score** → 6) **Upgrade or Heal** → 7) **Compress & deduplicate** →
  8) **Connect to stack** → 9) **Integrate across stacks** → repeat.
- Hierarchy is encoded via the **`layer`** enum and **`escalation_to`**:
  `cecca → board → executive → stack_chief → specialist → micro`.
- The **Agent Factory** reads the **manifest JSON** and spawns capsules by layer. Promotions/demotions are policy‑driven.
- No external SaaS is required. Everything runs offline with additive updates.

### Quick Capsule Example (JSON)
```json
{
  "name": "Spec A1",
  "layer": "specialist",
  "scope": ["subject-alpha"],
  "inputs": ["artifact://subject-alpha"],
  "outputs": ["artifact://subject-alpha/ds"],
  "tools": ["python", "local_tool_A"],
  "guardrails": ["offline-first", "audited"],
  "escalation_to": "Stack-Chief"
}
```
