#!/usr/bin/env python3
"""
Normalizer/Healer v2

This script extends the original normalizer by retaining the granular
metadata columns found in the ``All_Inclusive_Agent_Directory_v6_plus.csv``
file.  In addition to the mandatory headers specified in the original
``CSV_SCHEMA_v1.md``, the v6+ file defines dozens of extra columns (e.g.,
``agent_id``, ``agent_code``, ``purpose``, ``triggers``, ``data_sources``,
``models``, ``security_level``, etc.).  The v1 normalizer drops these
columns entirely.  This v2 normalizer heals the rows (normalising the
``layer`` field and filling defaults for the core fields) and then
preserves every other column.

The output CSV will therefore contain all columns present in the input
plus the mandatory fields.  Extra columns that are not present in the
input are ignored.  Column names are normalised to lower‑case with
underscores (non‑alphanumeric characters replaced with underscores) to
match the conventions used in the v1 normalizer.

Invocation example:

.. code-block:: bash

    python3 tools/normalize_csv_v2.py --in All_Inclusive_Agent_Directory_v6_plus.csv \
         --out All_Inclusive_Agent_Directory_v6_plus.normalized.full.csv

"""

import csv
import re
import argparse

# Mandatory columns defined in CSV_SCHEMA_v1.md
REQUIRED = [
    "agent_name",
    "role",
    "layer",
    "scope",
    "tools",
    "inputs",
    "outputs",
    "guardrails",
    "escalation_to",
    "stack",
]

# Layer aliases mapping used to derive the canonical layer from free‑form
# values present in the ``layer`` or ``role`` columns.
LAYER_ALIASES = {
    'cecca': {'cecca','chiefexecutivechiefcommanderagent','chief_executive_chief_commander','central','noa'},
    'board': {'board','governance','policy','trustees'},
    'executive': {'executive','exec','chief','vp','director','leadership'},
    'stack_chief': {'stack_chief','stack-chief','chief_commander','stack_commander','stack chief commander','stackchief','orchestrator','subject_orchestrator'},
    'specialist': {'specialist','sme','subject_matter_expert','special','lead'},
    'micro': {'micro','micro_agent','worker','agent','unit'},
}

def norm(name: str) -> str:
    """Normalise a column name to lower‑case snake_case."""
    return re.sub(r'[^a-z0-9]+', '_', (name or '').strip().lower()).strip('_')

def split_values(val: str):
    """Split a semicolon, pipe or comma separated string into a list.

    Many fields in the v6+ CSV (e.g., ``inputs``, ``outputs``, ``tools_stack``) use
    semicolons or pipes to delimit multiple values.  Returning a list makes it
    straightforward to serialise into JSON or further normalise later on.  If
    ``val`` is falsy, an empty list is returned.
    """
    if not val:
        return []
    return [x.strip() for x in re.split(r'[|,;]\s*', val) if x.strip()]

def normalize_layer(layer_val: str, role_val: str) -> str:
    """Derive the canonical layer given the raw ``layer`` and ``role`` values."""
    l = (layer_val or '').strip().lower()
    r = (role_val or '').strip().lower()
    # Direct match via aliases
    for canon, variants in LAYER_ALIASES.items():
        if l in variants:
            return canon
    # Heuristic patterns
    if 'cecca' in l or 'central' in l or 'noa' in l:
        return 'cecca'
    if 'board' in l or 'govern' in l or 'policy' in l:
        return 'board'
    if 'execut' in l or 'vp' in l or 'director' in l or 'cxo' in l:
        return 'executive'
    if ('stack' in l and 'chief' in l) or 'orchestrator' in l:
        return 'stack_chief'
    if 'special' in l or 'sme' in l:
        return 'specialist'
    if 'micro' in l or 'worker' in l or l == 'agent':
        return 'micro'
    # Fallback heuristics based on role
    if 'orchestrator' in r or ('stack' in r and 'chief' in r):
        return 'stack_chief'
    if any(k in r for k in ['vp','director','executive','cxo','officer']):
        return 'executive'
    if any(k in r for k in ['specialist','sme','lead']):
        return 'specialist'
    return 'micro'

def heal_row(row: dict) -> dict:
    """Heal and normalise a single row.

    The returned dictionary contains the same keys as the input (normalised
    names) plus the mandatory columns.  Missing mandatory values are filled
    with sensible defaults (e.g., layer derives from role, empty lists for
    multi‑value fields) and the ``stack`` is inferred from ``scope`` if
    missing.
    """
    # Normalise key names and strip whitespace from values
    out = {norm(k): (v.strip() if isinstance(v, str) else v) for k, v in row.items()}

    # Ensure all mandatory fields exist
    for c in REQUIRED:
        out.setdefault(c, '')

    # Derive canonical layer
    out['layer'] = normalize_layer(out.get('layer'), out.get('role'))

    # Default role based on layer if missing
    if not out.get('role'):
        out['role'] = {
            'cecca': 'Chief Executive Commander',
            'board': 'Board Policy',
            'executive': 'Executive',
            'stack_chief': 'Stack Chief Commander',
            'specialist': 'Specialist',
            'micro': 'Micro Agent',
        }[out['layer']]

    # Default escalation_to for specialists and micros
    if out['layer'] in ('specialist', 'micro') and not out.get('escalation_to'):
        out['escalation_to'] = 'Stack-Chief'

    # Default stack: if scope present, pick first; else fallback to Subject-001
    if not out.get('stack'):
        scopes = split_values(out.get('scope', ''))
        out['stack'] = scopes[0] if scopes else 'Subject-001'

    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", required=True, help="Path to the source CSV")
    ap.add_argument("--out", dest="outp", required=True, help="Path for the normalised output CSV")
    args = ap.parse_args()

    # Read input CSV with original header names
    with open(args.inp, newline='', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        rows = list(reader)

    healed_rows = []
    for row in rows:
        healed_rows.append(heal_row(row))

    # Assign default agent_name values where missing.  Use a zero‑padded index
    # so names are stable and easily cross‑referenced.  Do this after
    # healing so that all rows are present.
    for idx, r in enumerate(healed_rows, start=1):
        if not r.get('agent_name') or r['agent_name'] == '':
            # Use agent_id or agent_code if available
            fallback = r.get('agent_id') or r.get('agent_code')
            if fallback:
                r['agent_name'] = fallback
            else:
                r['agent_name'] = f"agent_{idx:04d}"

    # Determine all column names present across all rows
    all_keys = set()
    for r in healed_rows:
        all_keys.update(r.keys())

    # Sort column names: mandatory fields first, then the rest alphabetically
    other_cols = sorted(k for k in all_keys if k not in REQUIRED)
    fieldnames = REQUIRED + other_cols

    with open(args.outp, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for r in healed_rows:
            # Write the row; missing keys default to ''
            writer.writerow({c: r.get(c, '') for c in fieldnames})

    print(f"Wrote normalized CSV with {len(healed_rows)} rows and {len(fieldnames)} columns to {args.outp}")

if __name__ == "__main__":
    main()