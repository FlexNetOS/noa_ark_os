#!/usr/bin/env bash
# start-all-services.sh — Unified launcher wrapper
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

LEGACY_DOCKER=0
PREPARE_ONLY=0
SKIP_TESTS=0
SKIP_NOTEBOOK=0

usage() {
    cat <<'EOF'
Usage: scripts/start-all-services.sh [options]

Default behavior launches the in-repo full stack (API + UI) using portable toolchains.

Options:
    --legacy-docker     Use legacy docker-compose launch for external repos (deprecated).
    --prepare-only      Prepare toolchains and deps, then exit.
    --skip-tests        Skip tests during prepare.
    --skip-notebook     Do not start the notebook in unified mode.
    -h, --help          Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --legacy-docker) LEGACY_DOCKER=1; shift ;;
        --prepare-only) PREPARE_ONLY=1; shift ;;
        --skip-tests) SKIP_TESTS=1; shift ;;
        --skip-notebook) SKIP_NOTEBOOK=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo -e "${RED}Unknown option: $1${NC}" >&2; usage; exit 2 ;;
    esac
done

launch_unified() {
    echo -e "${YELLOW}Launching unified in-repo stack...${NC}"
    # Prefer WSL/Linux activators when available
    [[ -f "$REPO_ROOT/server/tools/activate-cargo-wsl.sh" ]] && source "$REPO_ROOT/server/tools/activate-cargo-wsl.sh" || true
    [[ -f "$REPO_ROOT/server/tools/activate-cargo.sh" ]] && source "$REPO_ROOT/server/tools/activate-cargo.sh" || true
    [[ -f "$REPO_ROOT/server/tools/activate-node.sh" ]] && source "$REPO_ROOT/server/tools/activate-node.sh" || true

    CMD=("$REPO_ROOT/scripts/full_stack_launch.sh")
    [[ $PREPARE_ONLY -eq 1 ]] && CMD+=("--prepare-only")
    [[ $SKIP_TESTS -eq 1 ]] && CMD+=("--skip-tests")
    [[ $SKIP_NOTEBOOK -eq 1 ]] && CMD+=("--skip-notebook")

    exec "${CMD[@]}"
}

launch_legacy_docker() {
    echo -e "${YELLOW}Legacy docker-compose mode (deprecated)${NC}"
    command -v docker >/dev/null 2>&1 || { echo -e "${RED}docker is not installed${NC}" >&2; exit 1; }
    command -v docker-compose >/dev/null 2>&1 || { echo -e "${RED}docker-compose is not installed${NC}" >&2; exit 1; }

    echo -e "${YELLOW}Creating directories...${NC}"
    mkdir -p "$REPO_ROOT/data" "$REPO_ROOT/logs" "$REPO_ROOT/config"

    start_if_compose() {
        local dir="$1"; local label="$2"
        echo -e "${YELLOW}Starting ${label}...${NC}"
        if [[ -f "$dir/docker-compose.yml" ]]; then
            (cd "$dir" && docker-compose up -d)
            echo -e "${GREEN}✓ ${label} started${NC}"
        else
            echo -e "${YELLOW}⚠ No docker-compose.yml in ${dir}, skipping${NC}"
        fi
    }

    start_if_compose "$REPO_ROOT/repos/MicroAgentStack" "MicroAgentStack (Orchestrator)"
    start_if_compose "$REPO_ROOT/repos/ark-os-noa" "ark-os-noa (Hive Mind)"
    start_if_compose "$REPO_ROOT/repos/deflex-ai-os" "deflex-ai-os (File Operations)"

    echo -e "${YELLOW}Waiting for services to be ready...${NC}"
    sleep 8
    echo -e "${YELLOW}Checking service health...${NC}"
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    echo -e "${GREEN}✓ Legacy services started${NC}"
}

if [[ $LEGACY_DOCKER -eq 1 ]]; then
    launch_legacy_docker
else
    launch_unified
fi

