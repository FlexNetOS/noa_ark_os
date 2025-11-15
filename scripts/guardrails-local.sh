#!/usr/bin/env bash
set -euo pipefail

base=${1:-origin/main}

export COMMAND_POLICY_PATH="${COMMAND_POLICY_PATH:-tools/automation/policy.yaml}"
export COMMAND_POLICY_VALIDATOR="${COMMAND_POLICY_VALIDATOR:-pnpm exec tsx tools/automation/validate-command.ts}"

IFS=' ' read -r -a COMMAND_POLICY_VALIDATOR_ARR <<<"${COMMAND_POLICY_VALIDATOR}"

validate_command() {
  "${COMMAND_POLICY_VALIDATOR_ARR[@]}" "$@"
}

# Deletion Lock + ledger
validate_command git diff --name-status "$base"...
if git diff --name-status "$base"... | grep -E '^(D|R[0-9]+)\s' >/dev/null; then
  validate_command git diff --name-only "$base"...
  changed=$(git diff --name-only "$base"...)
  echo "$changed" | grep -q '^archive/' || { echo 'Missing archive/ artifacts' >&2; exit 1; }
  echo "$changed" | grep -q '^docs/verification/' || { echo 'Missing docs/verification ledger' >&2; exit 1; }
fi

# Duplicate Fence
python3 - <<'PY' "$base"
import hashlib
import os
import shlex
import subprocess
import sys

ALLOW = ("archive/", "out/", ".git/", ".next/", "target/", "node_modules/", "dist/", "build/")


def validator_command() -> list[str]:
    raw = os.environ.get("COMMAND_POLICY_VALIDATOR", "pnpm exec tsx tools/automation/validate-command.ts")
    return shlex.split(raw)


def ensure_allowed(cmd: list[str]) -> None:
    validator = validator_command()
    if not validator:
        return
    subprocess.run(validator + cmd, check=True, stdout=subprocess.DEVNULL)


def capture(cmd: list[str]) -> str:
    ensure_allowed(cmd)
    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    return result.stdout


base_arg = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
diff_range = f"{base_arg}..."

changed = capture(["git", "diff", "--name-only", diff_range]).split()

seen: dict[str, str] = {}
duplicates: list[tuple[str, str, str]] = []

for path in changed:
    if any(path.startswith(prefix) for prefix in ALLOW):
        continue
    if not os.path.isfile(path):
        continue
    with open(path, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    other = seen.get(digest)
    if other and other != path:
        duplicates.append((path, other, digest))
    else:
        seen[digest] = path

if duplicates:
    print("Duplicate content detected:")
    for current, previous, digest in duplicates:
        print(f"  {current} == {previous} ({digest})")
    sys.exit(1)
PY

# Report Fence
validate_command git diff --name-status "$base"...
new_md=$(git diff --name-status "$base"... | awk '$1=="A" && $2 ~ /\.(md|MD)$/ {print $2}')
bad=0
for f in $new_md; do
  echo "$f" | grep -qE '^docs/' || { echo "New markdown outside docs/: $f"; bad=1; }
done
exit $bad
