#!/usr/bin/env bash
set -euo pipefail

log() { echo "[TRACE] $*"; }

log "Run stack script starting"
mkdir -p logs

ports=(8787 50051 3000 3001 3002)
log "Cleanup phase (ports)"
for port in "${ports[@]}"; do
  if command -v lsof >/dev/null 2>&1; then
    if pids=$(lsof -ti tcp:${port} 2>/dev/null); then
      if [[ -n "$pids" ]]; then log "Killing PIDs on :$port → $pids"; kill $pids 2>/dev/null || true; fi
    fi
  fi
done

if ui_api_pids=$(pgrep -x noa_ui_api 2>/dev/null); then
  if [[ -n "$ui_api_pids" ]]; then log "Killing noa_ui_api exact PIDs=$ui_api_pids"; kill $ui_api_pids 2>/dev/null || true; fi
fi
log "Skipping broad pkill patterns"

log "Post-cleanup port scan"
ss -ltnp 2>/dev/null | awk 'NR==1 || /:(8787|50051|3000|3001|3002)\s/' || true

trap 'log Teardown; [[ -n "${UI_PID:-}" ]] && kill ${UI_PID} 2>/dev/null || true; [[ -n "${API_PID:-}" ]] && kill ${API_PID} 2>/dev/null || true' EXIT INT TERM

log "Starting UI dev server"
pnpm --filter vibe-kanban dev > logs/ui_dev.log 2>&1 &
UI_PID=$!
log "UI_PID=$UI_PID"

log "Starting API server"
cargo run -p noa_ui_api > logs/ui_api.log 2>&1 &
API_PID=$!
log "API_PID=$API_PID"

sleep 1
log "Initial log heads"
head -n 20 logs/ui_dev.log || true
head -n 20 logs/ui_api.log || true

READY_LOG=logs/launch_readiness.log
: > "$READY_LOG"
READY_TIMEOUT=60

wait_port() {
  local port="$1" name="$2" start_ts=$(date +%s)
  echo "[TRACE] Waiting for $name on :$port (timeout=${READY_TIMEOUT}s)" | tee -a "$READY_LOG"
  for i in $(seq 1 $READY_TIMEOUT); do
    if ss -ltn 2>/dev/null | grep -q ":$port\s"; then
      local dur=$(( $(date +%s) - start_ts ))
      echo "[TRACE] $name READY on :$port (t=${dur}s)" | tee -a "$READY_LOG"
      return 0
    fi
    sleep 1
    if (( i % 10 == 0 )); then echo "[TRACE] $name still waiting (t=${i}s)" | tee -a "$READY_LOG"; fi
  done
  echo "[WARN] $name NOT READY after ${READY_TIMEOUT}s" | tee -a "$READY_LOG"
  return 1
}

wait_port 3000 UI_HTTP || true
wait_port 3001 UI_HTTP_ALT || true
wait_port 3002 UI_HTTP_ALT2 || true
wait_port 8787 API_HTTP || true
wait_port 50051 API_GRPC || true

# Determine UI port from Next.js log (most recent Local line)
UI_PORT=$(grep -E 'Local:\s+http://localhost:' logs/ui_dev.log | tail -n1 | sed -E 's/.*localhost:([0-9]+).*/\1/') || UI_PORT=""
if [[ -z "$UI_PORT" ]]; then
  # Fallback preference order
  for p in 3000 3001 3002; do if ss -ltn 2>/dev/null | grep -q ":$p\s"; then UI_PORT=$p; break; fi; done
fi
log "Resolved UI_PORT=$UI_PORT"

API_HEALTH_BODY=$(curl -s --max-time 3 http://localhost:8787/health || echo '{}')
API_HEALTH_CODE=$(echo "$API_HEALTH_BODY" | grep -q '"status"\s*:\s*"ok"' && echo 200 || echo 000)
if [[ "$API_HEALTH_CODE" == "200" ]]; then API_HEALTH_STATUS=healthy; else API_HEALTH_STATUS=unverified; fi
log "API health status=$API_HEALTH_STATUS code=$API_HEALTH_CODE body=$API_HEALTH_BODY"

JSON_TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
API_VERSION=$(echo "$API_HEALTH_BODY" | sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)
API_VERSION_HASH=$(echo "$API_HEALTH_BODY" | sed -n 's/.*"version_hash"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)
API_BUILD_TS=$(echo "$API_HEALTH_BODY" | sed -n 's/.*"build_timestamp"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)
API_FEATURE_FLAGS=$(echo "$API_HEALTH_BODY" | sed -n 's/.*"feature_flags"[[:space:]]*:[[:space:]]*\(\[[^]]*\]\).*/\1/p' | head -n1)
API_UPTIME=$(echo "$API_HEALTH_BODY" | sed -n 's/.*"uptime_seconds"[[:space:]]*:[[:space:]]*\([0-9]\+\).*/\1/p' | head -n1)
printf '{\n  "timestamp": "%s",\n  "ui_port": "%s",\n  "api_health": {"status": "%s", "code": "%s", "version": "%s", "version_hash": "%s", "build_timestamp": "%s", "uptime_seconds": %s, "feature_flags": %s},\n  "raw_health_body": %s\n}\n' \
  "$JSON_TS" "$UI_PORT" "$API_HEALTH_STATUS" "$API_HEALTH_CODE" "${API_VERSION:-unknown}" "${API_VERSION_HASH:-unknown}" "${API_BUILD_TS:-unknown}" "${API_UPTIME:-0}" "${API_FEATURE_FLAGS:-[]}" "$(printf '%s' "$API_HEALTH_BODY" | sed 's/"/\\"/g')" > logs/readiness.json
log "readiness.json written size=$(wc -c < logs/readiness.json)"

log "Log tails (UI/API)"
tail -n 40 logs/ui_dev.log || true
tail -n 40 logs/ui_api.log || true

log "Readiness probe summary"
tail -n 120 "$READY_LOG" || true

log "Entering wait (Ctrl-C to terminate)"
wait $UI_PID $API_PID
