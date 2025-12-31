# How‑to‑Use v3.1 (Extended Metadata)

This extended guide supplements the original **How‑to‑Use v3** document.  It
introduces version 2 of the CSV schema (`CSV_SCHEMA_v2.md`), a new
normaliser script (`tools/normalize_csv_v2.py`) and an extended
manifest (`stack.manifest.v2.json`).  The goal is to **heal** the
original CSV while preserving the granular metadata present in the
All Inclusive Agent Directory v6+ file.

## 0) Files added

- `schema/CSV_SCHEMA_v2.md` — lists the mandatory v1 columns plus all
  optional v6+ metadata fields.
- `schema/capsule.schema.v2.json` — extended capsule schema supporting
  metadata, dependencies, governance and monitoring objects.
- `tools/normalize_csv_v2.py` — heals and normalises the v6+ CSV and
  retains all metadata columns.
- `All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv` —
  output from the v2 normaliser; includes 81 columns for 862 rows.
- `stack.manifest.v2.json` — generated from the full normalised CSV
  with extended metadata objects.  This manifest maintains the
  original hierarchy (CECCA→Board→Stack Chief→Specialist→Micro) and
  populates metadata objects for each capsule.

## 1) Heal your CSV (extended)

```bash
python3 tools/normalize_csv_v2.py --in All_Inclusive_Agent_Directory_v6_plus.csv \
    --out All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv
```

The v2 normaliser applies the same layer/role heuristics as the
original v1 normaliser and fills defaults for missing values.  It
then retains every other column present in the source CSV (see
`CSV_SCHEMA_v2.md`)【928610657846884†L2-L8】.

## 2) Generate extended manifest

The extended manifest has already been generated as
`stack.manifest.v2.json`.  You can regenerate it with the script
below.  The script reads the v2 normalised CSV, groups agents by
stack and layer, and emits a manifest compatible with the new
capsule schema.

```bash
python3 - <<'PY'
import pandas as pd, json, re
from pathlib import Path

def split_vals(val):
    return [v.strip() for v in re.split(r'[|,;]\s*', str(val)) if v.strip()] if val else []

df = pd.read_csv('All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv')
mandatory = ['agent_name','role','layer','scope','tools','inputs','outputs','guardrails','escalation_to','stack']
manifest = {'meta':{'source_csv':'All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv','generated_by':'NOA_Deployment_Kit_v3.1+ext','version':'3.1-ext'},'cecca':[],'board':[],'executives':[],'stacks':[],'policy':{}}
groups = {}
for _, row in df.iterrows():
    layer=row['layer']
    ident = row.get('agent_id') or row.get('agent_code') or row.get('agent_name') or f"agent_{_+1:04d}"
    name  = row.get('agent_name') or ident
    capsule={'id':str(ident),'name':str(name),'layer':layer,'scope':split_vals(row.get('scope')),'inputs':split_vals(row.get('inputs')),'outputs':split_vals(row.get('outputs')),'tools':split_vals(row.get('tools')) or split_vals(row.get('tools_stack')),'guardrails':split_vals(row.get('guardrails')),'escalation_to':row.get('escalation_to') or None}
    # Additional properties can be added as needed (see csv_to_manifest_v2.py)
    if layer == 'cecca': manifest['cecca'].append(capsule)
    elif layer == 'board': manifest['board'].append(capsule)
    elif layer == 'executive': manifest['executives'].append(capsule)
    else:
        stack=row.get('stack') or 'Subject-001'
        if stack not in groups: groups[stack]={'stack_name':stack,'chief':None,'specialists':[],'micros':[]}
        grp=groups[stack]
        if layer=='stack_chief':
            if grp['chief'] is None: grp['chief']=capsule
            else: grp['specialists'].append(capsule)
        elif layer=='specialist': grp['specialists'].append(capsule)
        else: grp['micros'].append(capsule)
manifest['stacks']=list(groups.values())
Path('stack.manifest.v2.json').write_text(json.dumps(manifest,indent=2))
PY
```

## 3) Dry‑run deployment

For compatibility with existing tooling, you can still run the original
deploy script using the v1 manifest and CSV.  To utilise the extended
metadata, use `stack.manifest.v2.json` when your Agent Factory
supports the v2 capsule schema.

## 4) Task execution

To direct the AI runtime to use this kit for task execution, point
the runtime at the appropriate manifest.  For example:

```bash
export NOA_MANIFEST=stack.manifest.v2.json
python3 run_agent_factory.py --manifest "$NOA_MANIFEST" --task "<your task description>"
```

Replace `run_agent_factory.py` with your own runtime entry point.  The
manifest exposes the complete hierarchy of capsules and the extended
metadata required for rich orchestration.