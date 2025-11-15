#!/usr/bin/env bash
# Run the portable Cargo and UI make targets to validate regressions locally.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

printf '\n🔁 Running cargo-check via make...\n'
make cargo-check

printf '\n🧪 Running cargo-test (no args)...\n'
make cargo-test

printf '\n🏗️  Building UI workspace...\n'
make ui-build

printf '\n✅ devshell regression suite complete\n'
