#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${DEV_ENV_DEBUG:-}" ]]; then
  set -x
fi

cd /workspace 2>/dev/null || true

if [[ -z "${DEV_ENV_SKIP_POST_CREATE:-}" && -x /opt/devcontainer/post-create.sh ]]; then
  echo "[dev-env] Running post-create script" >&2
  if ! /opt/devcontainer/post-create.sh; then
    exit_code=$?
    echo "[dev-env] Warning: post-create script exited with status $exit_code" >&2
  fi
fi

exec "$@"
