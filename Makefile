SHELL := /bin/bash

PNPM ?= pnpm
CARGO ?= cargo

.PHONY: build test digest run ci:local lint typecheck format docs:check
.PHONY: pipeline.local world-verify world-fix kernel snapshot rollback verify publish-audit setup

build:
	$(PNPM) build

test:
	$(PNPM) test
	$(CARGO) test -p noa_crc

digest:
	$(CARGO) run -p noa_crc -- ingest

lint:
	$(PNPM) lint

format:
	$(PNPM) format
	$(CARGO) fmt --all

typecheck:
	$(PNPM) typecheck

docs:check:
	$(PNPM) docs:lint

ci:local: lint typecheck format docs:check test

run:
	@set -euo pipefail; \
	UI_PID=""; \
	API_PID=""; \
	trap '[[ -n "'"'"$$UI_PID'"'"' ]] && kill $$UI_PID 2>/dev/null || true; \
	      [[ -n "'"'"$$API_PID'"'"' ]] && kill $$API_PID 2>/dev/null || true' EXIT INT TERM; \
	$(PNPM) --filter vibe-kanban dev & \
	UI_PID=$$!; \
	$(CARGO) run -p noa_ui_api & \
	API_PID=$$!; \
	wait $$UI_PID $$API_PID

# Machine-First Pipeline (authoritative local pipeline)
pipeline.local: world-verify build sbom test scorekeeper package sign
	@echo "✅ Pipeline complete"

# World model verification
world-verify:
	@echo "🔍 Verifying world model consistency..."
	@# TODO: Implement world graph validation
	@echo "⚠️  world-verify not yet implemented (Phase 3)"

world-fix:
	@echo "🔧 Auto-repairing world model..."
	@# TODO: Implement world model reconciliation
	@echo "⚠️  world-fix not yet implemented (Phase 3)"

# Kernel build
kernel:
	@echo "🔨 Building kernel independently..."
	$(CARGO) build -p noa_core --release
	@echo "✅ Kernel build complete"

# SBOM generation
sbom:
	@echo "📋 Generating SBOM..."
	@mkdir -p audit
	@# TODO: Implement SBOM generation with cargo-sbom or similar
	@echo "⚠️  SBOM generation not yet implemented (Phase 8)"
	@echo '{"placeholder": true, "timestamp": "'$$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > audit/SBOM.manifest.json

# Scorekeeper (trust calculation)
scorekeeper:
	@echo "🎯 Calculating trust scores..."
	@mkdir -p metrics
	@# TODO: Implement scorekeeper with integrity/reversibility/capability metrics
	@echo "⚠️  Scorekeeper not yet implemented (Phase 2)"
	@echo '{"trust_score": null, "timestamp": "'$$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > metrics/trust_score.json

# Package artifacts
package:
	@echo "📦 Packaging artifacts..."
	@mkdir -p dist
	@# TODO: Implement artifact packaging
	@echo "⚠️  Packaging not yet implemented (Phase 10)"

# Sign artifacts
sign:
	@echo "✍️  Signing artifacts..."
	@# TODO: Implement artifact signing with GPG or similar
	@echo "⚠️  Signing not yet implemented (Phase 8)"

# Snapshot creation
snapshot:
	@echo "📸 Creating system snapshot..."
	@mkdir -p .snapshots
	@# TODO: Implement snapshot creation with git tags or similar
	@echo "⚠️  Snapshot not yet implemented (Phase 0)"

# Rollback to previous snapshot
rollback:
	@echo "⏪ Rolling back to previous snapshot..."
	@# TODO: Implement rollback mechanism
	@echo "⚠️  Rollback not yet implemented (Phase 0)"

# Verify build reproducibility
verify:
	@echo "🔐 Verifying build reproducibility..."
	@# TODO: Implement reproducibility verification
	@echo "⚠️  Verify not yet implemented (Phase 8)"

# Publish audit bundle
publish-audit:
	@echo "📤 Publishing audit bundle..."
	@mkdir -p audit
	@# TODO: Package and publish audit artifacts
	@echo "⚠️  Publish-audit not yet implemented (Phase 10)"
	@echo '{"audit_bundle": "placeholder", "timestamp": "'$$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > audit/bundle_metadata.json

# Setup toolchain
setup:
	@echo "🔧 Setting up build environment..."
	@# Install Rust if needed
	@command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	@# Install pnpm if needed
	@command -v pnpm >/dev/null 2>&1 || npm install -g pnpm
	@echo "✅ Setup complete"
