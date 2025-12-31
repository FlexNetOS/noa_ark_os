#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STATUS_FILE="${ROOT_DIR}/audit/local_pipeline_status.json"
CHECK_SCRIPT="${ROOT_DIR}/tools/ci/require_local_pipeline.py"
PYTHON_BIN="${PYTHON_BIN:-python3}"

if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  if command -v python >/dev/null 2>&1; then
    PYTHON_BIN=python
  else
    echo "❌ Python interpreter not found; cannot validate local pipeline evidence." >&2
    exit 1
  fi
fi

if [[ ! -f "${CHECK_SCRIPT}" ]]; then
  echo "❌ Missing ${CHECK_SCRIPT}. Cannot verify local pipeline evidence." >&2
  exit 1
fi

"${PYTHON_BIN}" "${CHECK_SCRIPT}" \
  --status-file "${STATUS_FILE}" \
  --max-age-minutes "${LOCAL_PIPELINE_MAX_AGE_MINUTES:-360}" \
  --require-logs
