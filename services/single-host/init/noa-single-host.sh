#!/usr/bin/env bash
set -euo pipefail

ACTION=${1:-start}
COMPONENT=${2:-all}
PROFILE_PATH=${NOA_PROFILE:-/etc/noa/server/profiles/single_host/profile.toml}
STATE_DIR=${NOA_STATE_DIR:-/var/lib/noa}
LOG_DIR=${NOA_LOG_DIR:-/var/log/noa}
METRICS_SOCKET=${NOA_METRICS_SOCKET:-/var/run/noa/single-host.sock}
DRY_RUN=${NOA_DRY_RUN:-0}

log() {
    local level=$1
    shift
    printf '[%s] %s\n' "$level" "$*"
}

ensure_paths() {
    mkdir -p "${STATE_DIR}" "${LOG_DIR}" "$(dirname "${METRICS_SOCKET}")"
}

start_component() {
    local name=$1
    log INFO "Starting component: ${name} (profile=${PROFILE_PATH})"

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log INFO "Dry-run enabled — skipping launch for ${name}"
        return
    fi

    case "${name}" in
        api-gateway)
            exec_or_echo noa-gateway --profile "${PROFILE_PATH}" --listen 0.0.0.0:8443 &
            ;;
        mcp)
            exec_or_echo noa-mcp --profile "${PROFILE_PATH}" --control-socket "${METRICS_SOCKET}" &
            ;;
        workflow)
            exec_or_echo noa-workflow --profile "${PROFILE_PATH}" --max-concurrency 8 &
            ;;
        sandbox)
            exec_or_echo noa-sandbox --profile "${PROFILE_PATH}" --kernel-quota &
            ;;
        all)
            start_component api-gateway
            start_component mcp
            start_component workflow
            start_component sandbox
            ;;
        *)
            log ERROR "Unknown component: ${name}"
            exit 2
            ;;
    esac
}

stop_component() {
    local name=$1
    log INFO "Stopping component: ${name}"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log INFO "Dry-run enabled — skipping stop for ${name}"
        return
    fi
    pkill -f "noa-${name}" || true
}

exec_or_echo() {
    if command -v "$1" >/dev/null 2>&1; then
        "$@"
    else
        log WARN "Binary $1 not installed; simulating start"
    fi
}

case "${ACTION}" in
    start)
        ensure_paths
        start_component "${COMPONENT}"
        ;;
    stop)
        if [[ "${COMPONENT}" == "all" ]]; then
            stop_component api-gateway
            stop_component mcp
            stop_component workflow
            stop_component sandbox
        else
            stop_component "${COMPONENT}"
        fi
        ;;
    status)
        pgrep -a noa- || log INFO "No NOA processes running"
        ;;
    *)
        log ERROR "Unsupported action: ${ACTION}"
        exit 2
        ;;
esac
