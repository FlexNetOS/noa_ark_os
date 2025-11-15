#!/usr/bin/env bash
# NOA ARK OS - GitHub CLI authentication helper
# Ensures the portable workflow works whether or not WSL interoperability is enabled.
# Usage: scripts/tools/gh-auth-login.sh [extra gh auth login flags]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PORTABLE_GH_CONFIG_DIR="${GH_PORTABLE_CONFIG_DIR:-$REPO_ROOT/server/tools/gh-portable/config}"
mkdir -p "$PORTABLE_GH_CONFIG_DIR"
export GH_CONFIG_DIR="${GH_CONFIG_DIR:-$PORTABLE_GH_CONFIG_DIR}"

GH_HOST="${GH_HOST:-github.com}"
GH_SCOPES="${GH_SCOPES:-repo,workflow}"
GH_GIT_PROTOCOL="${GH_GIT_PROTOCOL:-ssh}"
PREFERRED_MODE="${GH_LOGIN_MODE:-auto}"

is_wsl_interop_enabled() {
    if [[ ! -f /proc/sys/fs/binfmt_misc/WSLInterop ]]; then
        return 1
    fi

    # When enabled the file contains states such as "enabled" / "status" entries.
    if grep -qi "enabled" /proc/sys/fs/binfmt_misc/WSLInterop 2>/dev/null; then
        return 0
    fi

    return 1
}

if [[ "${1:-}" == "--check" ]]; then
    if is_wsl_interop_enabled; then
        echo "WSL interoperability: ENABLED"
        exit 0
    fi
    echo "WSL interoperability: DISABLED" >&2
    exit 1
fi

resolve_login_mode() {
    case "$PREFERRED_MODE" in
        web|--web)
            echo "--web"
            return
            ;;
        device|--device)
            echo "--device"
            return
            ;;
    esac

    if is_wsl_interop_enabled; then
        echo "--web"
    else
        echo "--device"
    fi
}

LOGIN_MODE="$(resolve_login_mode)"

if [[ "$LOGIN_MODE" == "--device" ]]; then
    cat <<'EOF'
[INFO] WSL interoperability appears to be disabled (or unavailable).
       Falling back to GitHub CLI device login so no Windows browser is required.
       To enable the automatic browser flow again, re-enable WSL interop via /etc/wsl.conf
       and restart WSL (see docs/runbook/WSL_INTEROP.md).
EOF
else
    echo "[INFO] Using GitHub CLI web login (browser will open automatically)."
fi

gh auth login \
    --hostname "$GH_HOST" \
    --scopes "$GH_SCOPES" \
    --git-protocol "$GH_GIT_PROTOCOL" \
    "$LOGIN_MODE" \
    "$@"
