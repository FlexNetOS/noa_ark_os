#!/usr/bin/env bash
set -euo pipefail
: "${GITHUB_OWNER:?set GITHUB_OWNER}"
: "${GITHUB_REPO:?set GITHUB_REPO}"
DEFAULT_BRANCH=${DEFAULT_BRANCH:-main}
# Requires: gh auth login --with-token < GITHUB_TOKEN
REQUIRED=(
  "CI / build-and-test"
  "Smoke Suite (CAS, Digest, Upload→Digest) / smoke"
)
json=$(printf '%s\n' "${REQUIRED[@]}" | jq -R . | jq -s '{strict: true, contexts: .}')
echo "Setting required status checks for $GITHUB_OWNER/$GITHUB_REPO:$DEFAULT_BRANCH" >&2
gh api \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/$GITHUB_OWNER/$GITHUB_REPO/branches/$DEFAULT_BRANCH/protection/required_status_checks" \
  -f strict=true \
  -f contexts[]="CI / build-and-test" \
  -f contexts[]="Smoke Suite (CAS, Digest, Upload→Digest) / smoke" \
  >/dev/null
echo "Done." >&2
