#!/usr/bin/env bash
set -euo pipefail

# Comprehensive Git Branch Synchronization Script
# Handles pulling, merging, and pushing all branches in noa-ark-os repository

REPO_DIR="/home/deflex/workspace/repos/noa_ark_os"
LOG_FILE="/home/deflex/workspace/repos/noa_ark_os/logs/git_sync_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "$(date +%Y-%m-%dT%H:%M:%S%z) $*" | tee -a "${LOG_FILE}"
}

error() {
    echo -e "${RED}ERROR: $1${NC}" | tee -a "${LOG_FILE}"
}

success() {
    echo -e "${GREEN}SUCCESS: $1${NC}" | tee -a "${LOG_FILE}"
}

info() {
    echo -e "${BLUE}INFO: $1${NC}" | tee -a "${LOG_FILE}"
}

warning() {
    echo -e "${YELLOW}WARNING: $1${NC}" | tee -a "${LOG_FILE}"
}

cd "${REPO_DIR}" || { error "Cannot change to repository directory"; exit 1; }

log "Starting comprehensive git branch synchronization"
log "Repository: ${REPO_DIR}"
log "Log file: ${LOG_FILE}"

# Ensure we're on main and clean
CURRENT_BRANCH=$(git branch --show-current)
if [[ "${CURRENT_BRANCH}" != "main" ]]; then
    warning "Not on main branch (current: ${CURRENT_BRANCH}), switching to main"
    git checkout main
fi

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --staged --quiet; then
    error "Uncommitted changes detected. Please commit or stash them first."
    exit 1
fi

# Fetch all remote branches
info "Fetching all remote branches..."
if git fetch --all --prune; then
    success "Fetched all remote branches"
else
    error "Failed to fetch remote branches"
    exit 1
fi

# Get list of all branches (local and remote)
ALL_BRANCHES=$(git branch -a | sed 's/^[ *]*//' | sed 's/^remotes\/origin\///' | sort | uniq)

# Separate local and remote branches
LOCAL_BRANCHES=$(git branch | sed 's/^[ *]*//')
REMOTE_BRANCHES=$(git branch -r | sed 's/^ *origin\///' | grep -v '^HEAD')

log "Found branches:"
log "Local branches: $(echo "${LOCAL_BRANCHES}" | tr '\n' ' ')"
log "Remote branches: $(echo "${REMOTE_BRANCHES}" | tr '\n' ' ')"

# Function to safely checkout and update a branch
update_branch() {
    local branch="$1"
    local is_local="$2"

    info "Processing branch: ${branch}"

    # Checkout the branch
    if [[ "${is_local}" == "true" ]]; then
        if ! git checkout "${branch}" 2>/dev/null; then
            warning "Could not checkout local branch ${branch}, skipping"
            return 1
        fi
    else
        if ! git checkout -b "${branch}" "origin/${branch}" 2>/dev/null; then
            warning "Could not checkout remote branch ${branch}, skipping"
            return 1
        fi
    fi

    # Pull latest changes if remote exists
    if git ls-remote --exit-code origin "${branch}" >/dev/null 2>&1; then
        if git pull origin "${branch}" 2>/dev/null; then
            success "Updated ${branch} with latest changes"
        else
            warning "Could not pull ${branch}, may have conflicts"
            return 1
        fi
    else
        info "${branch} has no remote counterpart"
    fi

    return 0
}

# Function to push a branch
push_branch() {
    local branch="$1"

    if git ls-remote --exit-code origin "${branch}" >/dev/null 2>&1; then
        if git push origin "${branch}" 2>/dev/null; then
            success "Pushed ${branch} to remote"
        else
            warning "Could not push ${branch} to remote"
        fi
    else
        info "Creating remote branch ${branch}"
        if git push -u origin "${branch}" 2>/dev/null; then
            success "Created and pushed remote branch ${branch}"
        else
            warning "Could not create remote branch ${branch}"
        fi
    fi
}

# Process all branches
PROCESSED_BRANCHES=()

# First, update all remote branches that have local counterparts
for remote_branch in ${REMOTE_BRANCHES}; do
    if echo "${LOCAL_BRANCHES}" | grep -q "^${remote_branch}$"; then
        info "Updating existing local branch: ${remote_branch}"
        if update_branch "${remote_branch}" "true"; then
            PROCESSED_BRANCHES+=("${remote_branch}")
        fi
    fi
done

# Then, checkout and update remote-only branches
for remote_branch in ${REMOTE_BRANCHES}; do
    if ! echo "${LOCAL_BRANCHES}" | grep -q "^${remote_branch}$"; then
        info "Checking out new remote branch: ${remote_branch}"
        if update_branch "${remote_branch}" "false"; then
            PROCESSED_BRANCHES+=("${remote_branch}")
        fi
    fi
done

# Return to main branch
git checkout main

# Push all local branches
info "Pushing all local branches..."
for branch in ${LOCAL_BRANCHES}; do
    info "Pushing branch: ${branch}"
    push_branch "${branch}"
done

# Check for any merge conflicts or issues
info "Checking repository status..."
if git status | grep -q "conflict"; then
    warning "Merge conflicts detected. Please resolve them manually."
else
    success "No merge conflicts detected"
fi

# Final status
success "Branch synchronization completed"
log "Processed branches: ${PROCESSED_BRANCHES[*]}"
log "Log saved to: ${LOG_FILE}"

# Show final status
echo ""
echo "=== FINAL STATUS ==="
git status --short
echo ""
echo "=== BRANCH SUMMARY ==="
git branch -v
echo ""
echo "Log file: ${LOG_FILE}"