SHELL := /bin/bash

PNPM ?= pnpm
CARGO ?= cargo
PYTHON ?= python3

.PHONY: build test digest run ci:local lint typecheck format
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

ci:local: lint typecheck format test

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
pipeline.local: world-verify build sbom test package sign verify scorekeeper publish-audit
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
@$(PYTHON) -m tools.repro.audit_pipeline sbom

# Scorekeeper (trust calculation)
scorekeeper:
@echo "🎯 Calculating trust scores..."
@$(PYTHON) -m tools.repro.audit_pipeline score

# Package artifacts
package:
@echo "📦 Packaging artifacts..."
@$(PYTHON) -m tools.repro.audit_pipeline package

# Sign artifacts
sign:
@echo "✍️  Signing artifacts..."
@$(PYTHON) -m tools.repro.audit_pipeline sign

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
@$(PYTHON) -m tools.repro.audit_pipeline verify

# Publish audit bundle
publish-audit:
@echo "📤 Publishing audit bundle..."
@$(PYTHON) -m tools.repro.audit_pipeline publish

# Setup toolchain
setup:
	@echo "🔧 Setting up build environment..."
	@# Install Rust if needed
	@command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	@# Install pnpm if needed
	@command -v pnpm >/dev/null 2>&1 || npm install -g pnpm
	@echo "✅ Setup complete"
