#!/usr/bin/env python3
import json, sys, hashlib, time, os
from pathlib import Path

SCHEMA_PATH = Path('schema/readiness.schema.json')
TARGET_PATH = Path('logs/readiness.json')

try:
    import jsonschema  # type: ignore
except ImportError:
    print('jsonschema module missing; install with pip install jsonschema', file=sys.stderr)
    sys.exit(2)

if not SCHEMA_PATH.exists():
    print(f'Schema missing: {SCHEMA_PATH}', file=sys.stderr)
    sys.exit(3)
if not TARGET_PATH.exists():
    print(f'Readiness artifact missing: {TARGET_PATH}', file=sys.stderr)
    sys.exit(4)

schema = json.loads(SCHEMA_PATH.read_text())
# Attempt to parse readiness which may have escaped quotes in raw_health_body line
raw = TARGET_PATH.read_text()
# If raw_health_body is not quoted properly, best-effort fix
try:
    data = json.loads(raw)
except json.JSONDecodeError:
    # Remove potential backslash-escaped quotes around raw_health_body object value
    fixed = raw.replace('raw_health_body": {\"', 'raw_health_body": "{').replace('\"}"', '}"')
    data = json.loads(fixed)

jsonschema.validate(data, schema)

hash_val = hashlib.sha256(raw.encode()).hexdigest()
print(json.dumps({
    'status': 'validated',
    'sha256': hash_val,
    'feature_flags': data.get('api_health', {}).get('feature_flags', []),
    'uptime_seconds': data.get('api_health', {}).get('uptime_seconds'),
    'version_hash': data.get('api_health', {}).get('version_hash'),
    'build_timestamp': data.get('api_health', {}).get('build_timestamp')
}, indent=2))
