#!/usr/bin/env bash
set -euo pipefail
BASE_REF=${BASE_REF:-origin/main}
CHANGED=$(git diff --name-only "$BASE_REF"... || true)
if [[ -z "$CHANGED" ]]; then
  echo "No changed files; skipping pre-commit archive"
  exit 0
fi
TS=$(date -u +%Y-%m-%dT%H-%M-%S)
YEAR=$(date -u +%Y)
MONTH=$(date -u +%m)
ARCHIVE_DIR=archive/$YEAR/$MONTH
mkdir -p "$ARCHIVE_DIR"
SNAPSHOT=precommit-${TS}-${GITHUB_SHA:-local}.tar.zst
FILE_LIST=$(mktemp)
printf "%s\n" $CHANGED > "$FILE_LIST"
# Filter out existing archive and build artifacts
sed -i '/^archive\//d;/^node_modules\//d;/^target\//d;/^dist\//d;/^build\//d' "$FILE_LIST"
if [[ ! -s "$FILE_LIST" ]]; then
  echo "No eligible files for snapshot after filtering"
  rm -f "$FILE_LIST"
  exit 0
fi
tar --force-local --zstd -cf "$ARCHIVE_DIR/$SNAPSHOT" --no-recursion -T "$FILE_LIST"
SHA=$(sha256sum "$ARCHIVE_DIR/$SNAPSHOT" | awk '{print $1}')
LEDGER="$ARCHIVE_DIR/ledger.json"
if [[ ! -f "$LEDGER" ]]; then echo '[]' > "$LEDGER"; fi
python3 - <<'PY'
import json, os, sys
ledger_path = os.environ.get('LEDGER')
entry = {
  'type': 'precommit_snapshot',
  'path': os.environ.get('SNAPSHOT_PATH'),
  'sha256': os.environ.get('SNAPSHOT_SHA'),
  'commit': os.environ.get('GITHUB_SHA','local'),
  'timestamp': os.environ.get('SNAPSHOT_TS'),
  'file_count': int(os.environ.get('SNAPSHOT_FILE_COUNT','0'))
}
with open(ledger_path,'r+') as f:
  data = json.load(f)
  data.append(entry)
  f.seek(0)
  json.dump(data,f,indent=2)
  f.truncate()
print(json.dumps(entry,indent=2))
PY
rm -f "$FILE_LIST"
echo "✅ Pre-commit archive created: $ARCHIVE_DIR/$SNAPSHOT (sha256=$SHA)"
