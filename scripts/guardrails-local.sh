#!/usr/bin/env bash
set -euo pipefail
base=${1:-origin/main}
# Deletion Lock + ledger
if git diff --name-status "$base"... | grep -E '^(D|R[0-9]+)\s' >/dev/null; then
  changed=$(git diff --name-only "$base"...)
  echo "$changed" | grep -q '^archive/' || { echo 'Missing archive/ artifacts' >&2; exit 1; }
  echo "$changed" | grep -q '^docs/verification/' || { echo 'Missing docs/verification ledger' >&2; exit 1; }
fi
# Duplicate Fence
python3 - << 'PY'
import hashlib, os, sys, subprocess
ALLOW=("archive/","out/",".git/",".next/","target/","node_modules/","dist/","build/")
base=sys.argv[1] if len(sys.argv)>1 else 'main'
changed=subprocess.run(["bash","-lc",f"git diff --name-only origin/{base}..."], capture_output=True, text=True).stdout.split()
seen={}; bad=[]
for p in changed:
  if any(p.startswith(a) for a in ALLOW): continue
  if not os.path.isfile(p): continue
  h=hashlib.sha256(open(p,'rb').read()).hexdigest()
  if h in seen and seen[h]!=p: bad.append((p,seen[h],h))
  else: seen[h]=p
if bad:
  print('Duplicate content detected:');
  [print(f"  {a} == {b} ({h})") for a,b,h in bad]; sys.exit(1)
PY "$base"
# Report Fence
new_md=$(git diff --name-status "$base"... | awk '$1=="A" && $2 ~ /\.(md|MD)$/ {print $2}')
bad=0; for f in $new_md; do echo "$f"|grep -qE '^docs/'||{ echo "New markdown outside docs/: $f"; bad=1; }; done; exit $bad
