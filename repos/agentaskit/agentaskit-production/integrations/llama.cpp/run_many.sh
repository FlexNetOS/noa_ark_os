#!/usr/bin/env bash
set -euo pipefail
root_dir="$(cd "$(dirname "$0")" && pwd)"
conf="$root_dir/config/models.yaml"

if [ ! -d "$root_dir/llama.cpp" ]; then
  bash "$root_dir/fetch.sh"
  bash "$root_dir/build.sh"
fi

if ! command -v yq >/dev/null 2>&1; then
  echo "Please install yq to parse YAML (e.g., brew install yq or pipx install yq)" >&2
  exit 1
fi

primary_path=$(yq -r '.models.primary_3b.path' "$conf")
secondary_path=$(yq -r '.models.secondary_7b.path' "$conf")
count=$(yq -r '.stacks.count' "$conf")
prompt=$(yq -r '.stacks.prompt' "$conf")
threads=$(yq -r '.stacks.threads' "$conf")

exe="$root_dir/llama.cpp/main"
if [ ! -x "$exe" ]; then exe="$root_dir/llama.cpp/build/bin/main"; fi
if [ ! -x "$exe" ]; then echo "llama.cpp main executable not found" >&2; exit 1; fi

for i in $(seq 1 "$count"); do
  if [ -f "$primary_path" ]; then
    ("$exe" -m "$primary_path" -p "$prompt" -t "$threads" &)
  fi
  if [ -f "$secondary_path" ]; then
    ("$exe" -m "$secondary_path" -p "$prompt" -t "$threads" &)
  fi
done
wait
