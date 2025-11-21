#!/usr/bin/env bash
set -euo pipefail
FILE="logs/readiness.json"
MAX_UPTIME=${HEALTH_MAX_UPTIME_SECONDS:-10}
if [[ ! -f "$FILE" ]]; then
  echo "readiness.json not found" >&2
  exit 2
fi
STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FILE" | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')
CODE=$(grep -o '"code"[[:space:]]*:[[:space:]]*"?[0-9]+"?' "$FILE" | head -n1 | sed -E 's/.*:?"?([0-9]+)"?/\1/')
VERSION=$(grep -o '"version"[[:space:]]*:[[:space:]]*"[^"]*"' "$FILE" | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')
HASH=$(grep -o '"version_hash"[[:space:]]*:[[:space:]]*"[^"]*"' "$FILE" | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')
UPTIME=$(grep -o '"uptime_seconds"[[:space:]]*:[[:space:]]*[0-9]+' "$FILE" | head -n1 | sed -E 's/.*:([0-9]+)/\1/')

if [[ "$STATUS" == "healthy" && "$CODE" == "200" ]]; then
  if [[ "$UPTIME" -gt "$MAX_UPTIME" ]]; then
    echo "❌ API healthy but uptime ($UPTIME)s exceeds threshold ($MAX_UPTIME)s (stale process?)" >&2
    exit 3
  fi
  echo "✅ API healthy (code=$CODE version=${VERSION:-unknown} hash=${HASH:-unknown} uptime=${UPTIME:-0}s <= $MAX_UPTIME s)"
  exit 0
else
  echo "❌ API unhealthy (status=$STATUS code=$CODE)" >&2
  exit 1
fi
