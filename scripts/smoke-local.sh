#!/usr/bin/env bash
set -euo pipefail
export OFFLINE_FIRST=true AI_PROVIDER=llama.cpp
. "$HOME/.cargo/env" 2>/dev/null || true
cargo test -p noa_crc --test cas_smoke --test digest_smoke -- --nocapture
cargo test -p noa_ui_api -- --nocapture
mkdir -p out/ci
cargo run -p noa_crc -- ingest --root . --report out/ci/ingest.json
 test -f out/ci/ingest.json && echo "Ingest OK: out/ci/ingest.json"
