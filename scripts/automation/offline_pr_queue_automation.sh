#!/usr/bin/env bash
set -euo pipefail

# Offline PR Queue Automation Script
# Automatically runs the offline PR queue tool to manage PRs

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLS_DIR="${REPO_ROOT}/tools"
OFFLINE_PR_DIR="${TOOLS_DIR}/offline_pr_queue"

# Configuration
LOG_DIR="${REPO_ROOT}/logs/automation"
LOG_FILE="${LOG_DIR}/offline_pr_queue_$(date +%Y%m%d_%H%M%S).log"
LOCK_FILE="/tmp/offline_pr_queue_automation.lock"
MAX_RUNTIME=300  # 5 minutes timeout

# Ensure log directory exists
mkdir -p "${LOG_DIR}"

# Logging function
log() {
    echo "[$(date +%Y-%m-%dT%H:%M:%S%z)] $*" | tee -a "${LOG_FILE}"
}

# Error handling
error_exit() {
    log "ERROR: $1"
    rm -f "${LOCK_FILE}"
    exit 1
}

# Check if another instance is running
if [[ -f "${LOCK_FILE}" ]]; then
    log "Another instance is running (lock file exists: ${LOCK_FILE})"
    exit 0
fi

# Create lock file
echo $$ > "${LOCK_FILE}"
trap 'rm -f "${LOCK_FILE}"' EXIT

log "Starting offline PR queue automation"
log "Repository root: ${REPO_ROOT}"
log "Working directory: $(pwd)"

# Change to repository root
cd "${REPO_ROOT}" || error_exit "Failed to change to repository root"

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    error_exit "Not in a git repository"
fi

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "${CURRENT_BRANCH}" != "main" ]]; then
    log "Not on main branch (current: ${CURRENT_BRANCH}), skipping automation"
    exit 0
fi

# Check if there are any uncommitted changes
if ! git diff --quiet || ! git diff --staged --quiet; then
    log "Uncommitted changes detected, skipping automation to avoid conflicts"
    exit 0
fi

# Pull latest changes
log "Pulling latest changes from origin/main"
if git pull origin main 2>/dev/null; then
    log "Successfully pulled latest changes"
else
    log "Failed to pull changes (possibly due to permissions), continuing with local version"
fi

# Check if Node.js and pnpm are available
if ! command -v node >/dev/null 2>&1; then
    error_exit "Node.js not found"
fi

if ! command -v pnpm >/dev/null 2>&1; then
    error_exit "pnpm not found"
fi

# Check if dependencies are installed, install if needed
if [[ ! -d "node_modules" ]]; then
    log "Installing dependencies"
    if ! pnpm install; then
        log "Failed to install dependencies with pnpm, trying npm"
        if ! npm install; then
            error_exit "Failed to install dependencies with both pnpm and npm"
        fi
    fi
else
    log "Dependencies appear to be installed, skipping installation"
fi

# Check if offline PR queue tool exists
if [[ ! -f "${OFFLINE_PR_DIR}/cli.ts" ]]; then
    error_exit "Offline PR queue CLI not found at ${OFFLINE_PR_DIR}/cli.ts"
fi

# Function to run PR queue command with timeout
run_pr_command() {
    local cmd="$1"
    local description="$2"

    log "Running: ${description}"
    log "Command: cd ${REPO_ROOT} && timeout ${MAX_RUNTIME} npx tsx ${OFFLINE_PR_DIR}/cli.ts ${cmd}"

    if cd "${REPO_ROOT}" && timeout "${MAX_RUNTIME}" npx tsx "${OFFLINE_PR_DIR}/cli.ts" "${cmd}" >> "${LOG_FILE}" 2>&1; then
        log "✓ ${description} completed successfully"
        return 0
    else
        local exit_code=$?
        if [[ ${exit_code} -eq 124 ]]; then
            log "✗ ${description} timed out after ${MAX_RUNTIME} seconds"
        else
            log "✗ ${description} failed with exit code ${exit_code}"
        fi
        return 1
    fi
}

# Check for existing PRs
log "Checking for existing offline PRs"
if run_pr_command "show --id 1" "Check existing PR 1"; then
    log "Found existing PR 1, checking its status"
    # If PR exists, we could potentially merge it if checks pass
    # For now, just log that it exists
else
    log "No existing PRs found or PR 1 doesn't exist"
fi

# Run health checks and potentially create/merge PRs
# This is a basic implementation - in production you might want more sophisticated logic

log "Offline PR queue automation completed successfully"
log "Log file: ${LOG_FILE}"
