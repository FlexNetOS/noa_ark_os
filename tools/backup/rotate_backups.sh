#!/usr/bin/env bash
set -euo pipefail

# rotate_backups.sh - Keep only the most recent N backup files.
# Dry-run by default; use --apply to actually delete.
# Matches files by prefix and typical archive extensions.

usage() {
  cat <<EOF
Usage: $(basename "$0") --dir DIR [--prefix PREFIX] [--keep N] [--apply]

Options:
  --dir DIR        Directory containing backup files (required)
  --prefix PREFIX  Filename prefix to match (default: noa_projects_)
  --keep N         Number of most recent files to keep (default: 7)
  --apply          Apply deletions (default: dry-run)
  -h, --help       Show this help

Notes:
- Recognized extensions: .tar.zst, .tgz, .tar.gz, .zip
- Sorts by modification time (newest first)
- Prints actions and a summary
EOF
}

DIR=""
PREFIX="noa_projects_"
KEEP=7
APPLY=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir) DIR="$2"; shift 2;;
    --prefix) PREFIX="$2"; shift 2;;
    --keep) KEEP="$2"; shift 2;;
    --apply) APPLY=true; shift;;
    -h|--help) usage; exit 0;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1;;
  esac
done

if [[ -z "$DIR" ]]; then
  echo "ERROR: --dir is required" >&2
  usage
  exit 1
fi
if [[ ! -d "$DIR" ]]; then
  echo "ERROR: directory not found: $DIR" >&2
  exit 1
fi

mapfile -t FILES < <(find "$DIR" -maxdepth 1 -type f \
  \( -name "${PREFIX}*.tar.zst" -o -name "${PREFIX}*.tgz" -o -name "${PREFIX}*.tar.gz" -o -name "${PREFIX}*.zip" \) \
  -printf "%T@ %p\n" | sort -r -n | awk '{print $2}')

TOTAL=${#FILES[@]}
if (( TOTAL <= KEEP )); then
  echo "Nothing to rotate: total=$TOTAL, keep=$KEEP"
  exit 0
fi

TO_DELETE=("${FILES[@]:KEEP}")

echo "Rotation plan (dry-run=${APPLY:false}):"
printf "  Keep (%d):\n" "$KEEP"
printf "    %s\n" "${FILES[@]:0:KEEP}"
printf "  Delete (%d):\n" "${#TO_DELETE[@]}"
printf "    %s\n" "${TO_DELETE[@]}"

if $APPLY; then
  for f in "${TO_DELETE[@]}"; do
    if [[ -f "$f" ]]; then
      rm -f -- "$f"
      echo "Deleted: $f"
      # remove checksum file if present
      for ext in .sha256 .sha256sum .sha256.txt; do
        [[ -f "$f$ext" ]] && rm -f -- "$f$ext" && echo "Deleted: $f$ext" || true
      done
    fi
  done
  echo "Rotation applied: kept $KEEP, deleted ${#TO_DELETE[@]}"
else
  echo "Dry-run: no files deleted. Re-run with --apply to execute."
fi
