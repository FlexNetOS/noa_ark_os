#!/usr/bin/env bash
#
# Comprehensive Workspace Cleanup Script
# Adapted from CRC drop `scripts-1/workspace_optimization.sh`
# Moves bulky artifacts into .workspace/quarantine and normalises docs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
QUARANTINE_ROOT="$PROJECT_ROOT/.workspace/quarantine"
BACKUP_TIMESTAMP="$(date +"%Y%m%d_%H%M%S")"
SESSION_DIR="$QUARANTINE_ROOT/$BACKUP_TIMESTAMP"

mkdir -p "$SESSION_DIR" "$PROJECT_ROOT/docs" "$PROJECT_ROOT/dist/windows"
cd "$PROJECT_ROOT"

info()  { echo -e "\033[0;34m[INFO]\033[0m $1"; }
warn()  { echo -e "\033[1;33m[WARN]\033[0m $1"; }
ok()    { echo -e "\033[0;32m[SUCCESS]\033[0m $1"; }

info "Starting workspace optimisation run ($BACKUP_TIMESTAMP)"

INITIAL_SIZE="$(du -sh . | cut -f1)"
info "Initial workspace size: $INITIAL_SIZE"

# 1. Quarantine build artefacts
mkdir -p "$SESSION_DIR/build"
for dir in target phase1/target phase2/target phase3/target phase6/target; do
    if [ -d "$dir" ]; then
        DEST="$SESSION_DIR/build/$(echo "$dir" | tr '/' '_')"
        mkdir -p "$(dirname "$DEST")"
        info "Quarantining $dir -> $DEST"
        mv "$dir" "$DEST"
    fi
done

# 2. Remove empty directories outside the workspace cache
EMPTY_COUNT="$(find . -path './.workspace' -prune -o -type d -empty -print | wc -l)"
if (( EMPTY_COUNT > 0 )); then
    find . -path './.workspace' -prune -o -type d -empty -delete
    ok "Removed $EMPTY_COUNT empty directories"
fi

# 3. Archive legacy Cargo configurations if present
mkdir -p "$SESSION_DIR/config"
shopt -s nullglob
for file in Cargo_*.toml Cargo.toml.backup; do
    info "Archiving $file"
    mv "$file" "$SESSION_DIR/config/"
done
shopt -u nullglob

# 4. Ensure documentation scaffold exists
mkdir -p docs/{implementation,architecture,phase-reports,user-guides}
ok "Documentation structure ensured under docs/"

# 5. Organise Windows distribution artefacts if any
if ls ark-os-noa-v5-* 1>/dev/null 2>&1; then
    mkdir -p "$SESSION_DIR/dist"
    for artifact in ark-os-noa-v5-windows.exe ark-os-noa-v5-professional-windows-desktop.exe ark-os-noa-v5-windows-testing.zip; do
        if [ -f "$artifact" ]; then
            info "Moving $artifact to dist/windows"
            mv "$artifact" dist/windows/
        fi
    done
    ok "Distribution assets consolidated"
fi

# Unknown binaries quarantine
if [ -f "mc" ]; then
    mkdir -p "$SESSION_DIR/unknown"
    warn "Quarantining unknown binary 'mc'"
    mv mc "$SESSION_DIR/unknown/"
fi

# 6. Ensure .gitignore (or baseline ignore) contains essentials
GITIGNORE="$PROJECT_ROOT/.gitignore"
touch "$GITIGNORE"
ensure_ignore() {
    local pattern="$1"
    if ! grep -Fxq "$pattern" "$GITIGNORE"; then
        echo "$pattern" >> "$GITIGNORE"
        ok "Added '$pattern' to .gitignore"
    fi
}
ensure_ignore "target/"
ensure_ignore ".workspace/"
ensure_ignore "logs/*.log"

# Final report
FINAL_SIZE="$(du -sh . | cut -f1)"
FILE_COUNT="$(find . -path './.workspace' -prune -o -type f -print | wc -l)"

echo "==============================================="
ok "Workspace optimisation complete."
echo "Initial size : $INITIAL_SIZE"
echo "Final size   : $FINAL_SIZE"
echo "Current files: $FILE_COUNT"
echo "Session data stored in: $SESSION_DIR"
