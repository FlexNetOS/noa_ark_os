#!/usr/bin/env bash
set -euo pipefail
# View or set hugepages; requires root to set.
if [[ ${1:-} == "--show" || -z ${1:-} ]]; then
  echo "Hugepages total: $(cat /proc/meminfo | grep -i hugepages_total)"
  echo "Hugepages free : $(cat /proc/meminfo | grep -i hugepages_free)"
  echo "Default size   : $(cat /proc/meminfo | grep -i hugepagesize)"
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
