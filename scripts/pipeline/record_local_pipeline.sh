#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STATUS_DIR="${ROOT_DIR}/audit"
STATUS_FILE="${STATUS_DIR}/local_pipeline_status.json"
BUILD_LOG="${ROOT_DIR}/build_output.txt"
TEST_LOG="${ROOT_DIR}/test_output.txt"
BUILD_LOG_RELATIVE="build_output.txt"
TEST_LOG_RELATIVE="test_output.txt"
PYTHON_BIN="${PYTHON_BIN:-python3}"

if [[ -z "${NOA_NODE_ENV:-}" || -z "${NOA_CARGO_ENV:-}" ]]; then
  echo "❌ Portable environments not activated. Source server/tools/activate-node.sh and server/tools/activate-cargo.sh first." >&2
  exit 1
fi

if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  echo "❌ ${PYTHON_BIN} is required to record pipeline evidence." >&2
  exit 1
fi

mkdir -p "${STATUS_DIR}"
touch "${BUILD_LOG}" "${TEST_LOG}"

hash_file() {
  local file="$1"
  if [[ ! -s "$file" ]]; then
    echo "missing"
    return 0
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  else
    echo "unavailable"
  fi
}

commands=("$@")
if [[ ${#commands[@]} -eq 0 ]]; then
  commands=("make pipeline.local")
fi

command_json="["
for cmd in "${commands[@]}"; do
  escaped=$(printf '%s' "$cmd" | "${PYTHON_BIN}" -c 'import json,sys; print(json.dumps(sys.stdin.read().rstrip("\n")))')
  command_json+="${escaped},"
done
command_json="${command_json%,}]"

COMMIT="$(git -C "${ROOT_DIR}" rev-parse HEAD)"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
NODE_VERSION="$(node --version | tr -d '\r')"
PNPM_VERSION="$(pnpm --version | tr -d '\r')"
CARGO_VERSION="$(cargo --version | tr -d '\r')"
BUILD_HASH="$(hash_file "${BUILD_LOG}")"
TEST_HASH="$(hash_file "${TEST_LOG}")"

cat >"${STATUS_FILE}" <<JSON
{
  "commit": "${COMMIT}",
  "timestamp": "${TIMESTAMP}",
  "status": "success",
  "node": "${NODE_VERSION}",
  "pnpm": "${PNPM_VERSION}",
  "cargo": "${CARGO_VERSION}",
  "build_log": {
    "path": "${BUILD_LOG_RELATIVE}",
    "sha256": "${BUILD_HASH}"
  },
  "test_log": {
    "path": "${TEST_LOG_RELATIVE}",
    "sha256": "${TEST_HASH}"
  },
  "commands": ${command_json}
}
JSON

echo "✅ Local pipeline evidence written to ${STATUS_FILE}"