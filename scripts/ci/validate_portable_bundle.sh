#!/usr/bin/env bash
set -euo pipefail

MANIFEST=${1:-core/config/default_manifest.yaml}
OUTPUT_DIR=${2:-build/portable}
FORMATS=${3:-"oci wasi tar"}

python tools/portable_builder.py --manifest "$MANIFEST" --output "$OUTPUT_DIR" $(for fmt in $FORMATS; do printf ' --format %s' "$fmt"; done)

cargo run -p noa_cicd --example validate_portable -- "$MANIFEST" "$OUTPUT_DIR" "$FORMATS"
