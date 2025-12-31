#!/usr/bin/env bash
set -euo pipefail

TMP_ROOT=$(mktemp -d)
trap 'rm -rf "${TMP_ROOT}"' EXIT

cp Makefile "${TMP_ROOT}/Makefile"
mkdir -p "${TMP_ROOT}/tools"
cp tools/snapshot_ledger.py "${TMP_ROOT}/tools/snapshot_ledger.py"
cat <<'DOC' > "${TMP_ROOT}/sample.txt"
initial snapshot content
DOC

pushd "${TMP_ROOT}" >/dev/null

git init -q
git config user.email test@example.com
git config user.name "Snapshot Tester"
git add Makefile sample.txt
git commit -q -m "initial"

SNAPSHOT_ARCHIVE_ROOT="${TMP_ROOT}/archive" SNAPSHOT_BUNDLE_PREFIX=test-suite SNAPSHOT_TAR_COMPRESS="" SNAPSHOT_TAR_DECOMPRESS="" SNAPSHOT_BUNDLE_EXT="tar" make snapshot >/dev/null

LEDGER=$(find "${TMP_ROOT}/archive" -name ledger.json -print -quit)
BUNDLE=$(python3 - <<'PY' "${LEDGER}"
import json, sys
with open(sys.argv[1], 'r', encoding='utf-8') as fh:
    data = json.load(fh)
print(data[-1]['bundle'])
PY
)

echo "mutated" > sample.txt
SNAPSHOT_ARCHIVE_ROOT="${TMP_ROOT}/archive" SNAPSHOT_TAR_COMPRESS="" SNAPSHOT_TAR_DECOMPRESS="" SNAPSHOT_BUNDLE_EXT="tar" make rollback BUNDLE="${BUNDLE}" >/dev/null

grep -q "initial snapshot content" sample.txt

popd >/dev/null
