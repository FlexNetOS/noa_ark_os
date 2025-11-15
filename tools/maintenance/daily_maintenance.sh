#!/usr/bin/env bash
#
# Daily Workspace Maintenance Script
# Adapted from CRC drop `scripts-1/daily_maintenance.sh`
# Performs lightweight hygiene tasks and captures workspace metrics.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_DIR="$PROJECT_ROOT/.workspace/logs"
METRICS_DIR="$PROJECT_ROOT/.workspace/metrics"
QUARANTINE_DIR="$PROJECT_ROOT/.workspace/quarantine"
LOG_FILE="$LOG_DIR/maintenance_$(date +%Y%m%d).log"
RETENTION_DAYS=30

mkdir -p "$LOG_DIR" "$METRICS_DIR" "$QUARANTINE_DIR"
cd "$PROJECT_ROOT"

log() {
    local message="$1"
    local timestamp
    timestamp="$(date '+%Y-%m-%d %H:%M:%S')"
    echo "[$timestamp] $message" | tee -a "$LOG_FILE"
}

log "Starting daily workspace maintenance..."

# 1. Quarantine accidental build outputs
if [ -d "target" ]; then
    DEST="$QUARANTINE_DIR/target_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$DEST"
    log "Build artifacts detected; moving ./target to $DEST"
    mv target "$DEST/"
fi

# 2. Rotate log files older than 7 days within .workspace/logs
find "$LOG_DIR" -name "*.log" -type f -mtime +7 -print -delete 2>/dev/null || true

# 3. Cleanup old quarantine snapshots
find "$QUARANTINE_DIR" -mindepth 1 -maxdepth 1 -type d -mtime +"$RETENTION_DAYS" -print -exec rm -rf {} + 2>/dev/null || true

# 4. Collect metrics
CURRENT_SIZE="$(du -sh . | cut -f1)"
FILE_COUNT="$(find . -path './.workspace/quarantine' -prune -o -path './.workspace/logs' -prune -o -path './.workspace/metrics' -prune -o -type f -print | wc -l)"
RUST_FILES="$(find . -name '*.rs' -not -path './.workspace/*' | wc -l)"
DOC_FILES="$(find . -name '*.md' -not -path './.workspace/*' | wc -l)"

phase_size() {
    local dir="$1"
    if [ -d "$dir" ]; then
        du -sh "$dir" | cut -f1
    else
        echo "0"
    fi
}

PHASE1_SIZE="$(phase_size 'phase1')"
PHASE2_SIZE="$(phase_size 'phase2')"
PHASE3_SIZE="$(phase_size 'phase3')"
PHASE6_SIZE="$(phase_size 'phase6')"

# 5. Basic issue detection
EMPTY_DIRS="$(find . -path './.git' -prune -o -path './.workspace/quarantine' -prune -o -type d -empty -print | wc -l)"
LARGE_FILES="$(find . -type f -size +50M -not -path './.git/*' -not -path './.workspace/quarantine/*' | wc -l)"
DISK_USAGE_PERCENT="$(df "$PROJECT_ROOT" | awk 'NR==2 {sub(/%/,"",$5); print $5}')"

ISSUES_FOUND=0
(( EMPTY_DIRS > 10 )) && ISSUES_FOUND=$((ISSUES_FOUND + 1))
(( LARGE_FILES > 0 )) && ISSUES_FOUND=$((ISSUES_FOUND + 1))
(( DISK_USAGE_PERCENT > 80 )) && ISSUES_FOUND=$((ISSUES_FOUND + 1))

# 6. Health score heuristic
HEALTH_SCORE=100
(( EMPTY_DIRS > 20 )) && HEALTH_SCORE=$((HEALTH_SCORE - 15))
(( LARGE_FILES > 2 )) && HEALTH_SCORE=$((HEALTH_SCORE - 10))
(( DISK_USAGE_PERCENT > 85 )) && HEALTH_SCORE=$((HEALTH_SCORE - 20))

# Write metrics snapshot
cat > "$METRICS_DIR/daily_snapshot.json" <<EOF
{
  "date": "$(date -u +%Y-%m-%d)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "workspace_size": "$CURRENT_SIZE",
  "total_files": $FILE_COUNT,
  "rust_files": $RUST_FILES,
  "documentation_files": $DOC_FILES,
  "phases": {
    "phase1": "$PHASE1_SIZE",
    "phase2": "$PHASE2_SIZE",
    "phase3": "$PHASE3_SIZE",
    "phase6": "$PHASE6_SIZE"
  },
  "health_score": $HEALTH_SCORE,
  "issues_found": $ISSUES_FOUND
}
EOF

log "Metrics updated (size: $CURRENT_SIZE, files: $FILE_COUNT, health: $HEALTH_SCORE)"

# 7. Emit alerts if needed
if (( HEALTH_SCORE < 60 )); then
    ALERT_FILE="$LOG_DIR/alert_$(date +%Y%m%d).txt"
    {
        echo "Workspace health alert"
        echo "Health Score: $HEALTH_SCORE/100"
        echo "Issues Detected: $ISSUES_FOUND"
    } > "$ALERT_FILE"
    log "Alert generated at $ALERT_FILE"
fi

log "Daily maintenance complete."

echo "ARK-OS NOA Daily Maintenance Summary:"
echo "  Size: $CURRENT_SIZE"
echo "  Files: $FILE_COUNT"
echo "  Health: $HEALTH_SCORE/100"
echo "  Issues: $ISSUES_FOUND"
