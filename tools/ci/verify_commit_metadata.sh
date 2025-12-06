#!/usr/bin/env bash
set -euo pipefail
COMMIT_SHA=$(git rev-parse --short HEAD)
# Conventional commit enforcement is separate; here we cross-check readiness metadata
FILE=logs/readiness.json
if [[ ! -f $FILE ]]; then
  echo "readiness.json missing; skip metadata verification" >&2
  exit 0
fi
HASH_IN_FILE=$(grep -o '"version_hash"[[:space:]]*:[[:space:]]*"[^"]*"' "$FILE" | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')
BUILD_TS=$(grep -o '"build_timestamp"[[:space:]]*:[[:space:]]*"[^"]*"' "$FILE" | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')

status="ok"
error=""
exit_code=0

if [[ -z "$HASH_IN_FILE" || -z "$BUILD_TS" ]]; then
  status="error"; error="missing_version_hash_or_build_timestamp"; exit_code=1
elif [[ "$HASH_IN_FILE" != "$COMMIT_SHA" ]]; then
  status="error"; error="version_hash_mismatch"; exit_code=2
else
  NOW=$(date -u +%s)
  TS_SECS=$(date -u -d "$BUILD_TS" +%s 2>/dev/null || echo 0)
  if [[ "$TS_SECS" -eq 0 ]]; then
    status="error"; error="invalid_build_timestamp_format"; exit_code=3
  else
    AGE=$(( NOW - TS_SECS ))
    if (( AGE > 7200 )); then
      status="error"; error="build_timestamp_too_old"; exit_code=4
    fi
  fi
fi

timestamp_fresh="false"
if [[ "$status" == "ok" && "$AGE" -le 7200 ]]; then timestamp_fresh="true"; fi
json_output=$(jq -n \
  --arg status "$status" \
  --arg commit_sha "$COMMIT_SHA" \
  --arg readiness_version_hash "$HASH_IN_FILE" \
  --arg build_timestamp "$BUILD_TS" \
  --arg age_seconds "${AGE:-0}" \
  --arg timestamp_fresh "$timestamp_fresh" \
  --arg error "$error" '{status:$status,commit_sha:$commit_sha,readiness_version_hash:$readiness_version_hash,build_timestamp:$build_timestamp,age_seconds:(($age_seconds|tonumber)),timestamp_fresh:( $timestamp_fresh == "true" ),error:($error // null)}')
echo "$json_output"
if [[ "$status" == "ok" ]]; then
  echo "✅ Commit metadata verified (hash=$COMMIT_SHA timestamp=$BUILD_TS age=${AGE}s)" >&2
else
  echo "❌ Commit metadata verification failed: $error (hash=$COMMIT_SHA readiness=$HASH_IN_FILE ts=$BUILD_TS)" >&2
fi
exit $exit_code
