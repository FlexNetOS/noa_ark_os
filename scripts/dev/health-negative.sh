#!/usr/bin/env bash
set -euo pipefail
# Simulate stale process by exceeding uptime threshold.
HEALTH_MAX_UPTIME_SECONDS=1 export HEALTH_MAX_UPTIME_SECONDS
bash scripts/dev/run-stack.sh & PID=$!
# Sleep beyond threshold
sleep 4
if bash scripts/dev/check-health.sh; then
  echo "Expected stale failure but health-check succeeded" >&2
  kill $PID 2>/dev/null || true
  exit 1
else
  echo "Negative test passed (stale process detected)";
  kill $PID 2>/dev/null || true
  exit 0
fi
