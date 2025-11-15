#!/usr/bin/env bash
set -euo pipefail
root_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$root_dir/llama.cpp"
# Basic CPU build
make -j$(nproc) || make
