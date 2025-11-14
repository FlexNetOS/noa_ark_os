SHELL := /bin/bash

DEV_CONFIG_JSON := tools/devshell/config.json
NODE_AVAILABLE := $(shell command -v node >/dev/null 2>&1 && echo 1)

ifeq ($(NODE_AVAILABLE),1)
PNPM_REQUIRED_VERSION := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs pnpm.requiredVersion; fi)
PNPM_STORE_DIR_OVERRIDE := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs pnpm.defaultStoreDir; fi)
DEV_PATH_PREFIX := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs env.pathPrefix --mode posix; fi)
DEV_CARGO_HOME := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs env.CARGO_HOME --mode posix; fi)
DEV_RUSTUP_HOME := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs env.RUSTUP_HOME --mode posix; fi)
DEV_PNPM_HOME := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs env.PNPM_HOME --mode posix; fi)
RUST_ANALYZER_CHECK_COMMAND := $(shell if [ -f $(DEV_CONFIG_JSON) ]; then node tools/devshell/read-config.cjs rustAnalyzer.checkCommand; fi)
endif

PNPM ?= pnpm
CARGO ?= cargo
PYTHON ?= python3

ifneq ($(PNPM_REQUIRED_VERSION),)
export NOA_PNPM_REQUIRED := $(PNPM_REQUIRED_VERSION)
endif

ifneq ($(PNPM_STORE_DIR_OVERRIDE),)
export PNPM_STORE_DIR := $(PNPM_STORE_DIR_OVERRIDE)
endif

ifneq ($(DEV_PATH_PREFIX),)
override PATH := $(DEV_PATH_PREFIX):$(PATH)
export PATH
endif

ifneq ($(DEV_CARGO_HOME),)
export CARGO_HOME := $(DEV_CARGO_HOME)
endif

ifneq ($(DEV_RUSTUP_HOME),)
export RUSTUP_HOME := $(DEV_RUSTUP_HOME)
endif

ifneq ($(DEV_PNPM_HOME),)
export PNPM_HOME := $(DEV_PNPM_HOME)
override PATH := $(PNPM_HOME):$(PATH)
endif

ifneq ($(RUST_ANALYZER_CHECK_COMMAND),)
export RUST_ANALYZER_CHECK_COMMAND := $(RUST_ANALYZER_CHECK_COMMAND)
endif

SNAPSHOT_ARCHIVE_ROOT ?= archive
SNAPSHOT_BUNDLE_PREFIX ?= noa-ark-os
SNAPSHOT_LEDGER_NAME ?= ledger.json
SNAPSHOT_TAR_COMPRESS ?= --zstd
SNAPSHOT_TAR_DECOMPRESS ?= --zstd
SNAPSHOT_BUNDLE_EXT ?= tar.zst

.PHONY: build test digest run ci-local ci\:local lint typecheck format
.PHONY: pipeline.local world-verify world-fix kernel snapshot rollback verify publish-audit setup

build:
	$(PNPM) build

test:
	$(PNPM) test
	$(CARGO) test -p noa_crc
	bash tests/shell/test_snapshot.sh

digest:
	$(CARGO) run -p noa_crc -- ingest

lint:
	$(PNPM) lint

format:
	$(PNPM) format
	$(CARGO) fmt --all

typecheck:
	$(PNPM) typecheck

ci-local: lint typecheck format test

ci\:local:
	@echo "⚠️  'ci:local' is deprecated; forwarding to 'ci-local'"
	@$(MAKE) ci-local

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
	@echo "🔍 Reconciling world model consistency..."
	@cargo run -p noa_core --bin noa_world -- verify

world-fix:
	@echo "🛠️ Generating remediation plan for world model drift..."
	@cargo run -p noa_core --bin noa_world -- fix

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
	@set -euo pipefail; \
	TS="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	TS_SAFE="$$(date -u +%Y%m%dT%H%M%SZ)"; \
	YEAR="$$(date -u +%Y)"; \
	MONTH="$$(date -u +%m)"; \
	ARCHIVE_DIR="$(SNAPSHOT_ARCHIVE_ROOT)/$$YEAR/$$MONTH"; \
	SNAPSHOT_DIR="$$ARCHIVE_DIR/snapshots"; \
	mkdir -p "$$SNAPSHOT_DIR"; \
        BUNDLE_NAME="$(SNAPSHOT_BUNDLE_PREFIX)-$$TS_SAFE.$(SNAPSHOT_BUNDLE_EXT)"; \
        BUNDLE_PATH="$$SNAPSHOT_DIR/$$BUNDLE_NAME"; \
        echo "🧾 Bundling tracked files into $$BUNDLE_PATH"; \
        git rev-parse HEAD >/dev/null; \
        FILE_LIST="$$SNAPSHOT_DIR/.snapshot-files-$$TS_SAFE"; \
        git ls-files -z > "$$FILE_LIST"; \
        tar --force-local $(SNAPSHOT_TAR_COMPRESS) -cf "$$BUNDLE_PATH" --null -T "$$FILE_LIST"; \
	rm -f "$$FILE_LIST"; \
	SHA="$$(sha256sum "$$BUNDLE_PATH" | awk '{print $$1}')"; \
	LEDGER="$$ARCHIVE_DIR/$(SNAPSHOT_LEDGER_NAME)"; \
	if [[ ! -f "$$LEDGER" ]]; then echo "[]" > "$$LEDGER"; fi; \
	COMMIT="$$(git rev-parse HEAD)"; \
	python3 tools/snapshot_ledger.py snapshot "$$LEDGER" "$$BUNDLE_PATH" "$$SHA" "$$COMMIT" "$$TS" "$(SNAPSHOT_BUNDLE_PREFIX)"; \
	echo "✅ Snapshot recorded at $$BUNDLE_PATH"

# Rollback to previous snapshot
rollback:
	@set -euo pipefail; \
        echo "⏪ Rolling back to previous snapshot..."; \
        BUNDLE_VALUE="$(BUNDLE)"; \
        if [[ -z "$$BUNDLE_VALUE" ]]; then BUNDLE_VALUE="${BUNDLE:-}"; fi; \
        if [[ -z "$$BUNDLE_VALUE" ]]; then echo "BUNDLE variable is required, e.g. make rollback BUNDLE=$(SNAPSHOT_ARCHIVE_ROOT)/YYYY/MM/snapshots/<file>.tar.zst" >&2; exit 2; fi; \
        if [[ ! -f "$$BUNDLE_VALUE" ]]; then echo "Bundle $$BUNDLE_VALUE not found" >&2; exit 3; fi; \
        SHA="$$(sha256sum "$$BUNDLE_VALUE" | awk '{print $$1}')"; \
        TS="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
        BUNDLE_DIR="$$(dirname "$$BUNDLE_VALUE")"; \
	MONTH_DIR="$$(dirname "$$BUNDLE_DIR")"; \
	YEAR_DIR="$$(dirname "$$MONTH_DIR")"; \
	MONTH="$$(basename "$$MONTH_DIR")"; \
	YEAR="$$(basename "$$YEAR_DIR")"; \
	LEDGER="$(SNAPSHOT_ARCHIVE_ROOT)/$$YEAR/$$MONTH/$(SNAPSHOT_LEDGER_NAME)"; \
	if [[ ! -f "$$LEDGER" ]]; then echo "Ledger $$LEDGER not found for bundle" >&2; exit 4; fi; \
        python3 tools/snapshot_ledger.py rollback "$$LEDGER" "$$BUNDLE_VALUE" "$$SHA" "$$TS"; \
        TAR_DECOMPRESS="$(SNAPSHOT_TAR_DECOMPRESS)"; \
        if [[ "$$TAR_DECOMPRESS" == "" ]]; then TAR_DECOMPRESS=""; fi; \
        tar --force-local $$TAR_DECOMPRESS -xf "$$BUNDLE_VALUE"; \
        echo "✅ Rollback complete from $$BUNDLE_VALUE"

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
