#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/../.. && pwd)"
merge-polish simulate --model C --sources "$@" --target "$ROOT_DIR/merged" || "$ROOT_DIR/workspace/tools/bin/merge-polish" simulate --model C --sources "$@" --target "$ROOT_DIR/merged"
merge-polish merge --model C --sources "$@" --target "$ROOT_DIR/merged" --confirm yes || "$ROOT_DIR/workspace/tools/bin/merge-polish" merge --model C --sources "$@" --target "$ROOT_DIR/merged" --confirm yes
merge-polish verify --target "$ROOT_DIR/merged" || "$ROOT_DIR/workspace/tools/bin/merge-polish" verify --target "$ROOT_DIR/merged"
