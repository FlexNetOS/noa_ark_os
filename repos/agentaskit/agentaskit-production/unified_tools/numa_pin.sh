#!/usr/bin/env bash
set -euo pipefail
usage() { echo "Usage: $0 [--node N] [--cpus LIST] -- CMD [ARGS...]"; }
NODE=0; CPUS=""; while [[ $# -gt 0 ]]; do
  case "$1" in
    --node) NODE="$2"; shift 2;;
    --cpus) CPUS="$2"; shift 2;;
    --help|-h) usage; exit 0;;
    --) shift; break;;
    *) break;;
  esac
done
if ! command -v numactl >/dev/null 2>&1; then
  echo "[numa_pin] numactl not found; run without pinning."; exec "$@"
fi
if [[ -n "${CPUS}" ]]; then
  exec numactl --cpunodebind="${NODE}" --membind="${NODE}" taskset -c "${CPUS}" "$@"
else
  exec numactl --cpunodebind="${NODE}" --membind="${NODE}" "$@"
fi
