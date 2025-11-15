#!/usr/bin/env python3
import csv, re, argparse

REQUIRED = ["agent_name","role","layer","scope","tools","inputs","outputs","guardrails","escalation_to","stack"]

LAYER_ALIASES = {
    'cecca': {'cecca','chiefexecutivechiefcommanderagent','chief_executive_chief_commander','central','noa'},
    'board': {'board','governance','policy','trustees'},
    'executive': {'executive','exec','chief','vp','director','leadership'},
    'stack_chief': {'stack_chief','stack-chief','chief_commander','stack_commander','stack chief commander','stackchief','orchestrator','subject_orchestrator'},
    'specialist': {'specialist','sme','subject_matter_expert','special','lead'},
    'micro': {'micro','micro_agent','worker','agent','unit'}
}

def norm(s):
    return re.sub(r'[^a-z0-9]+', '_', (s or '').strip().lower()).strip('_')

def pick(row, *cands, default=None):
    for c in cands:
        if c in row and row[c]:
            return row[c]
    return default

def split_pipe(val):
    if not val: return []
    return [x.strip() for x in re.split(r'[|,;]\s*', val) if x.strip()]

def normalize_layer(layer_val, role_val):
    l = (layer_val or '').strip().lower()
    r = (role_val or '').strip().lower()
    for canon, variants in LAYER_ALIASES.items():
        if l in variants:
            return canon
    if 'cecca' in l or 'central' in l or 'noa' in l: return 'cecca'
    if 'board' in l or 'govern' in l or 'policy' in l: return 'board'
    if 'execut' in l or 'vp' in l or 'director' in l or 'cxo' in l: return 'executive'
    if ('stack' in l and 'chief' in l) or 'orchestrator' in l: return 'stack_chief'
    if 'special' in l or 'sme' in l: return 'specialist'
    if 'micro' in l or 'worker' in l or l == 'agent': return 'micro'
    if 'orchestrator' in r or ('stack' in r and 'chief' in r): return 'stack_chief'
    if any(k in r for k in ['vp','director','executive','cxo','officer']): return 'executive'
    if any(k in r for k in ['specialist','sme','lead']): return 'specialist'
    return 'micro'

def heal_row(row):
    out = {k: row.get(k, '') for k in row.keys()}
    for c in REQUIRED:
        out.setdefault(c, '')
    out['layer'] = normalize_layer(out.get('layer'), out.get('role'))
    if not out.get('role'):
        out['role'] = {
            'cecca':'Chief Executive Commander',
            'board':'Board Policy',
            'executive':'Executive',
            'stack_chief':'Stack Chief Commander',
            'specialist':'Specialist',
            'micro':'Micro Agent'
        }[out['layer']]
    if out['layer'] in ('specialist','micro') and not out.get('escalation_to'):
        out['escalation_to'] = 'Stack-Chief'
    if not out.get('stack'):
        sc = split_pipe(out.get('scope',''))
        out['stack'] = sc[0] if sc else 'Subject-001'
    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", required=True)
    ap.add_argument("--out", dest="outp", required=True)
    args = ap.parse_args()

    with open(args.inp, newline='', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        rows = [{norm(k):(v.strip() if isinstance(v,str) else v) for k,v in r.items()} for r in reader]

    healed = [heal_row(r) for r in rows]

    with open(args.outp, 'w', newline='', encoding='utf-8') as f:
        w = csv.DictWriter(f, fieldnames=REQUIRED)
        w.writeheader()
        for r in healed:
            w.writerow({c: r.get(c,'') for c in REQUIRED})

    print(f"Wrote normalized CSV to {args.outp} (rows={len(healed)})")

if __name__ == "__main__":
    main()
