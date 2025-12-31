#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_PATH="${REPO_ROOT}/scripts/dev-env.sh"

if [[ ! -f "${SCRIPT_PATH}" ]]; then
  echo "Error: dev environment script not found at ${SCRIPT_PATH}" >&2
  exit 1
fi

if ! command -v podman >/dev/null 2>&1 && ! command -v docker >/dev/null 2>&1; then
  echo "Warning: skipping dev-env smoke test because no container runtime is available." >&2
  exit 0
fi

"${SCRIPT_PATH}" smoke
