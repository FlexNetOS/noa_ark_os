#!/usr/bin/env python3
import json, sys, hashlib
from pathlib import Path

SCHEMA_PATH = Path('schema/archive.ledger.schema.json')
LEDGER_PATH = Path('audit/ledger.jsonl')

try:
    import jsonschema  # type: ignore
except ImportError:
    print('jsonschema module missing; install with pip install jsonschema', file=sys.stderr)
    sys.exit(2)

if not SCHEMA_PATH.exists():
    print(json.dumps({'status':'error','error':'schema_missing','path':str(SCHEMA_PATH)}))
    sys.exit(3)
if not LEDGER_PATH.exists():
    print(json.dumps({'status':'error','error':'ledger_missing','path':str(LEDGER_PATH)}))
    sys.exit(4)

schema = json.loads(SCHEMA_PATH.read_text())
validator = jsonschema.Draft202012Validator(schema)

errors = []
valid_count = 0
line_count = 0
hashes = []

with LEDGER_PATH.open() as f:
    for idx, line in enumerate(f, start=1):
        line_count += 1
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError as e:
            errors.append({'line': idx, 'error': 'json_decode', 'message': str(e)})
            continue
        for err in validator.iter_errors(entry):
            errors.append({'line': idx, 'error': 'schema', 'message': err.message, 'path': list(err.path)})
        if not any(e['line'] == idx for e in errors):
            valid_count += 1
            hashes.append(hashlib.sha256(line.encode()).hexdigest())

status = 'validated' if not errors else 'invalid'
summary = {
    'status': status,
    'lines': line_count,
    'valid_entries': valid_count,
    'invalid_entries': len(errors),
    'errors': errors[:25],  # cap to prevent overload
    'sha256_aggregate': hashlib.sha256((''.join(hashes)).encode()).hexdigest() if hashes else None
}
print(json.dumps(summary, indent=2))
sys.exit(0 if status == 'validated' else 5)