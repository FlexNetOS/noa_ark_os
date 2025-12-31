#!/usr/bin/env bash
set -euo pipefail
# Example task runner. Replace with real model invocation.
# Reads $TASK_INPUT and writes transformed output to stdout.
inp="${TASK_INPUT:-}"
if [[ -z "$inp" || ! -f "$inp" ]]; then
  echo "No TASK_INPUT provided or file missing" >&2
  exit 2
fi
# A/B/C differ by a trivial transformation to emulate diversity.
mode="$(basename "$(pwd)")"
case "$mode" in
  A) tr 'a-z' 'A-Z' < "$inp" ;;
  B) rev < "$inp" ;;
  C) sha256sum "$inp" | awk '{print $1}' ;;
  *) cat "$inp" ;;
esac
