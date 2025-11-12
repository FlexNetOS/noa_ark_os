#!/usr/bin/env bash
# Disable Ubuntu ESM apt sources to avoid firewall blocks.
set -euo pipefail

INFO() { printf '\033[0;34m[INFO]\033[0m %s\n' "$*"; }
WARN() { printf '\033[1;33m[WARN]\033[0m %s\n' "$*"; }
OK()   { printf '\033[0;32m[OK]\033[0m %s\n' "$*"; }
ERR()  { printf '\033[0;31m[ERROR]\033[0m %s\n' "$*"; }

require_root() {
    if [ "${EUID:-$(id -u)}" -ne 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            INFO "Root privileges required. Re-running with sudo..."
            exec sudo -E bash "$0" "$@"
        else
            ERR "Root privileges required and sudo not found."
            ERR "Please run: sudo bash $0"
            exit 1
        fi
    fi
}

backup_file() {
    local file="$1"
    local backup_dir="/etc/apt/noa-backups"
    local timestamp
    timestamp="$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"
    local backup="$backup_dir/$(basename "$file").$timestamp"
    cp "$file" "$backup"
    INFO "Backup saved: $backup"
}

comment_esm_entries() {
    local file="$1"
    if [ ! -f "$file" ]; then
        return
    fi
    if ! grep -qE 'esm\.ubuntu\.com' "$file"; then
        return
    fi

    if ! grep -qE '^[[:space:]]*[^#].*esm\.ubuntu\.com' "$file"; then
        INFO "ESM entries already disabled in $file"
        return
    fi

    backup_file "$file"
    INFO "Disabling ESM entries in $file"
    # Comment any line that points to esm.ubuntu.com and tag it so the script is idempotent.
    sed -i -E 's@^([[:space:]]*[^#].*esm\.ubuntu\.com.*)@# disabled-by-noa \1@' "$file"
    OK "ESM endpoints disabled in $(basename "$file")"
}

neutralize_hook() {
    local hook_file="/etc/apt/apt.conf.d/20apt-esm-hook.conf"
    if [ -f "$hook_file" ]; then
        if grep -q 'Disabled by NOA ARK OS' "$hook_file"; then
            INFO "ESM apt hook already neutralized"
            return
        fi
        backup_file "$hook_file"
        INFO "Neutralizing $hook_file so apt no longer triggers the ESM hook"
        cat <<'HOOK' > "$hook_file"
// Disabled by NOA ARK OS to avoid contacting esm.ubuntu.com in restricted environments.
APT::Update::Post-Invoke-Success { };
HOOK
        OK "ESM apt hook disabled"
    fi
}

main() {
    require_root "$@"

    INFO "Disabling Ubuntu ESM apt sources"
    local changed=0
    for path in /etc/apt/sources.list /etc/apt/sources.list.d/*.list; do
        if [ -e "$path" ]; then
            if grep -qE 'esm\.ubuntu\.com' "$path"; then
                comment_esm_entries "$path"
                changed=1
            fi
        fi
    done

    neutralize_hook

    if [ "$changed" -eq 0 ]; then
        INFO "No ESM sources were found; nothing to change."
    else
        OK "Ubuntu ESM sources disabled. You can now run 'apt-get update' without hitting esm.ubuntu.com."
    fi
}

main "$@"
