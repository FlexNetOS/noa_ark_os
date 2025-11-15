#!/usr/bin/env bash
set -euo pipefail
inp="${TASK_INPUT:-}"
if [[ -z "$inp" || ! -f "$inp" ]]; then
  echo "No TASK_INPUT provided or missing file" >&2
  exit 2
fi
mode="$(basename "$(pwd)")"
case "$mode" in
  A) tr 'a-z' 'A-Z' < "$inp" ;;
  B) rev < "$inp" ;;
  C) sha256sum "$inp" | awk '{print $1}' ;;
  *) cat "$inp" ;;
esac
