#!/usr/bin/env bash
set -euo pipefail

# Offline PR Queue Status Checker
# Quick script to check the status of automated PR queue operations

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_DIR="${REPO_ROOT}/logs/automation"

echo "=== Offline PR Queue Automation Status ==="
echo "Repository: ${REPO_ROOT}"
echo "Log Directory: ${LOG_DIR}"
echo ""

# Check if log directory exists
if [[ ! -d "${LOG_DIR}" ]]; then
    echo "❌ Log directory does not exist: ${LOG_DIR}"
    exit 1
fi

# Find the latest log file
LATEST_LOG=$(find "${LOG_DIR}" -name "offline_pr_queue_*.log" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)

if [[ -z "${LATEST_LOG}" ]]; then
    echo "❌ No automation logs found"
    exit 1
fi

echo "📄 Latest Log: ${LATEST_LOG}"
echo "📅 Last Run: $(stat -c '%y' "${LATEST_LOG}" | cut -d'.' -f1)"
echo ""

echo "🔍 Last Run Summary:"
echo "----------------------------------------"
tail -20 "${LATEST_LOG}" | grep -E "(Starting|Successfully|ERROR|completed|failed|timed out)" || echo "No summary lines found"

echo ""
echo "📊 Recent Automation Runs:"
echo "----------------------------------------"
find "${LOG_DIR}" -name "offline_pr_queue_*.log" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -5 | while read -r line; do
    timestamp=$(echo "$line" | cut -d' ' -f1)
    filepath=$(echo "$line" | cut -d' ' -f2-)
    filename=$(basename "$filepath")
    date_str=$(date -d "@${timestamp%.*}" '+%Y-%m-%d %H:%M:%S')
    status=$(grep -q "completed successfully" "$filepath" && echo "✅" || echo "❌")
    echo "${status} ${date_str} - ${filename}"
done

echo ""
echo "⏰ Next Scheduled Run:"
echo "----------------------------------------"
crontab -l | grep offline_pr_queue_automation.sh | head -1 || echo "No cron job found"

echo ""
echo "🔧 Manual Commands:"
echo "----------------------------------------"
echo "Run now:    ./scripts/automation/offline_pr_queue_automation.sh"
echo "View logs:  tail -f ${LATEST_LOG}"
echo "Check cron: crontab -l"
