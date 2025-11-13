#!/usr/bin/env bash
set -euo pipefail

# Ensure corepack/pnpm are enabled and pinned from package.json
if command -v corepack >/dev/null 2>&1; then
  corepack enable || true
  # Respect package.json packageManager pin, if present
  if [ -f package.json ] && grep -q '"packageManager"\s*:\s*"pnpm@' package.json; then
    ver=$(grep -o '"packageManager"\s*:\s*"pnpm@[^"]*' package.json | sed 's/.*pnpm@//')
    corepack prepare "pnpm@${ver}" --activate || true
  fi
fi

# Print versions for diagnostics
node -v || true
pnpm -v || (corepack prepare pnpm@latest --activate && pnpm -v) || true
cargo --version || true
rustup show || true

# Workspace bootstrap (safe, idempotent)
if [ -f "pnpm-workspace.yaml" ]; then
  pnpm install -w || true
  pnpm dedupe -w || true
fi

# Generate Cargo.lock without building (safe if Rust present)
if command -v cargo >/dev/null 2>&1; then
  (cargo generate-lockfile || true)
  (cargo update --workspace || true)
fi

echo "Post-create complete."