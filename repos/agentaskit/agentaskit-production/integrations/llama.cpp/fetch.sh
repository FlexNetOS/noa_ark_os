#!/usr/bin/env bash
set -euo pipefail
root_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$root_dir"
if [ ! -d llama.cpp ]; then
  git clone https://github.com/ggerganov/llama.cpp.git
fi
cd llama.cpp
# Pin to a known good commit (update as needed)
COMMIT=${LLAMACPP_COMMIT:-"master"}
git fetch --all
git checkout "$COMMIT"
echo "Checked out llama.cpp @ $(git rev-parse --short HEAD)"
