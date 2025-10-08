#!/usr/bin/env bash
#
# Pre-commit workspace validation helper.
# Adapted from CRC drop `scripts-1/pre-commit-hook.sh`

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

status() { echo -e "${BLUE}[HOOK]${NC} $1"; }
warn()   { echo -e "${YELLOW}[WARN]${NC} $1"; }
ok()     { echo -e "${GREEN}[OK]${NC} $1"; }
err()    { echo -e "${RED}[ERROR]${NC} $1"; }

ensure_ignore() {
    local pattern="$1"
    local file="$2"
    if ! grep -Fxq "$pattern" "$file"; then
        echo "$pattern" >> "$file"
        ok "Added '$pattern' to .gitignore"
    fi
}

status "Starting pre-commit workspace validation..."

has_cargo=false
if command -v cargo >/dev/null 2>&1; then
    has_cargo=true
fi

# 1. Format Rust code
if $has_cargo; then
    status "Formatting Rust code (cargo fmt)"
    cargo fmt --all || warn "cargo fmt failed (check toolchain activation)"
fi

# 2. Clippy linting
if $has_cargo; then
    status "Running Clippy (warnings treated as errors)"
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        err "Clippy reported issues. Fix them before committing."
        exit 1
    fi
    ok "Clippy checks passed"
fi

# 3. Check for build artefacts in root target/
if [ -d "target" ]; then
    err "Build artefacts detected in ./target. Remove or quarantine before commit."
    exit 1
fi
ok "No root-level build artefacts detected"

# 4. Large file scan (>10MB) excluding workspace cache
status "Scanning for large files (>10MB)"
large_files="$(find . -type f -size +10M -not -path './.git/*' -not -path './.workspace/*' | head -5)"
if [ -n "$large_files" ]; then
    warn "Large files detected:"
    while IFS= read -r file; do
        size="$(du -h "$file" | cut -f1)"
        warn "  $file ($size)"
    done <<< "$large_files"
fi

# 5. Secret heuristics on staged files (if git repo)
if command -v git >/dev/null 2>&1 && [ -d ".git" ]; then
    status "Scanning staged files for potential secrets"
    secret_patterns=("password" "api_key" "secret" "token" "private_key")
    secrets_found=false
    while IFS= read -r file; do
        for pattern in "${secret_patterns[@]}"; do
            if grep -qi "$pattern" "$file"; then
                warn "Potential secret pattern '$pattern' in $file"
                secrets_found=true
            fi
        done
    done < <(git diff --cached --name-only)
    $secrets_found && warn "Review the flagged files before committing."
fi

# 6. Validate Cargo.toml if staged
if $has_cargo && command -v git >/dev/null 2>&1 && [ -d ".git" ]; then
    if git diff --cached --name-only | grep -q "Cargo.toml"; then
        status "cargo check (Cargo.toml updated)"
        cargo check --quiet || {
            err "cargo check failed after Cargo.toml changes"
            exit 1
        }
        ok "Cargo manifest validated"
    fi
fi

# 7. Update lightweight metrics snapshot
status "Capturing workspace snapshot"
FILE_COUNT="$(find . -not -path './.workspace/*' -not -path './.git/*' -type f | wc -l)"
WORKSPACE_SIZE="$(du -sh . | cut -f1)"
mkdir -p ".workspace/metrics"
cat > ".workspace/metrics/pre_commit_snapshot.json" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "file_count": $FILE_COUNT,
  "workspace_size": "$WORKSPACE_SIZE"
}
EOF
ok "Snapshot stored in .workspace/metrics/pre_commit_snapshot.json"

# 8. Ensure baseline gitignore entries if repository present
if [ -d ".git" ]; then
    GITIGNORE=".gitignore"
    touch "$GITIGNORE"
    ensure_ignore "target/" "$GITIGNORE"
    ensure_ignore ".workspace/" "$GITIGNORE"
fi

ok "Pre-commit validation completed successfully!"
