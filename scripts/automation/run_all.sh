#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

printf "[automation] validating registry documents...\n"
"${SCRIPT_DIR}/validate_registry.py"

printf "[automation] preparing workspace (portable toolchains)...\n"
"${REPO_ROOT}/scripts/full_stack_launch.sh" --prepare-only --skip-tests --skip-notebook
