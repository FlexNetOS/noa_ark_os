#!/usr/bin/env bash
set -euo pipefail
if [[ ${1:-} == "--show" || -z ${1:-} ]]; then
  grep -i Huge /proc/meminfo || true
  echo "To set: sudo $0 --set N"
  exit 0
fi
if [[ ${1:-} == "--set" ]]; then
  N=${2:-0}
  if [[ $EUID -ne 0 ]]; then echo "[hugepages] need sudo"; exit 1; fi
  echo $N > /proc/sys/vm/nr_hugepages
  echo "[hugepages] set nr_hugepages=$N"
  exit 0
fi
echo "Usage: $0 [--show] | --set N"
