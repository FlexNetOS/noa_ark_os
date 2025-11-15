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
CARGO_PORTABLE ?= tools/devshell/cargo-portable.sh
UI_SCOPE ?= --filter vibe-kanban
CARGO ?= ./tools/devshell/portable-cargo.sh
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

.PHONY: build test digest run ci-local lint typecheck format
.PHONY: pipeline.local world-verify world-fix kernel snapshot rollback verify publish-audit rollback-sim setup
.PHONY: build test digest run ci-local lint typecheck format docs:check
PYTHON ?= python3
BASE_REF ?= origin/main

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

ACTIVATION_CHECK := \
	@if [ -z "$$NOA_CARGO_ENV" ] || [ -z "$$NOA_NODE_ENV" ]; then \
		echo "❌ Portable toolchain not activated. Run 'source ./server/tools/activate-cargo.sh' and 'source ./server/tools/activate-node.sh' first." >&2; \
		exit 1; \
	fi

.PHONY: deps build test digest run ci-local lint typecheck format

# Snapshot configuration (retained from local-first pipeline additions)
SNAPSHOT_ARCHIVE_ROOT ?= archive
SNAPSHOT_BUNDLE_PREFIX ?= noa-ark-os
SNAPSHOT_LEDGER_NAME ?= ledger.json
SNAPSHOT_TAR_COMPRESS ?= --zstd
SNAPSHOT_TAR_DECOMPRESS ?= --zstd
SNAPSHOT_BUNDLE_EXT ?= tar.zst

.PHONY: build test digest run ci-local ci\:local lint typecheck format \
	cargo-build cargo-check cargo-test cargo-run \
	ui-build ui-test ui-lint ui-typecheck ui-format ui-dev
.PHONY: pipeline.local world-verify world-fix kernel snapshot rollback verify publish-audit setup
.PHONY: provider-pointers archival-verify duplicate-check router-singleton conventional-commits export-roadmap
.PHONY: record-local-pipeline


deps:
	$(ACTIVATION_CHECK)
	$(PNPM) install --frozen-lockfile

build: deps
	$(PNPM) build

test: deps
	$(PNPM) test
	$(CARGO) test -p noa_crc
	bash tests/shell/test_snapshot.sh

digest:
	$(CARGO) run -p noa_crc -- ingest

lint: deps
	$(PNPM) lint

format: deps
	$(PNPM) format
	$(CARGO) fmt --all

typecheck: deps
	$(PNPM) typecheck

docs:check:
	$(PNPM) docs:lint

ci-local: lint typecheck format docs:check test
ci-core: lint typecheck format test

run: deps
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

cargo-build:
	$(CARGO_PORTABLE) build $(if $(ARGS),$(ARGS))

cargo-check:
	$(CARGO_PORTABLE) check $(if $(ARGS),$(ARGS))

cargo-test:
	$(CARGO_PORTABLE) test $(if $(ARGS),$(ARGS))

cargo-run:
	$(CARGO_PORTABLE) run $(if $(ARGS),$(ARGS))

ui-build:
	$(PNPM) $(UI_SCOPE) build $(if $(ARGS),$(ARGS))

ui-test:
	$(PNPM) $(UI_SCOPE) test $(if $(ARGS),$(ARGS))

ui-lint:
	$(PNPM) $(UI_SCOPE) lint $(if $(ARGS),$(ARGS))

ui-typecheck:
	$(PNPM) $(UI_SCOPE) typecheck $(if $(ARGS),$(ARGS))

ui-format:
	$(PNPM) $(UI_SCOPE) format $(if $(ARGS),$(ARGS))

ui-dev:
	$(PNPM) $(UI_SCOPE) dev $(if $(ARGS),$(ARGS))

# Machine-First Pipeline (authoritative local pipeline)
pipeline.local: world-verify build sbom test package sign verify scorekeeper publish-audit
	@echo "✅ Pipeline complete"
@echo "✅ Pipeline complete"
pipeline.local: world-verify build provider-pointers archival-verify duplicate-check router-singleton ci-local sbom scorekeeper package sign conventional-commits export-roadmap record-local-pipeline
	@echo "✅ Pipeline complete"

provider-pointers: deps
	$(ACTIVATION_CHECK)
	$(PNPM) exec tsx tools/ci/verify_provider_pointers.ts

archival-verify: deps
	$(ACTIVATION_CHECK)
	BASE_REF="$(BASE_REF)" $(PNPM) exec tsx tools/ci/verify_archival.ts

duplicate-check: deps
	$(ACTIVATION_CHECK)
	$(PNPM) exec tsx tools/ci/check_duplicate_content.ts

router-singleton: deps
	$(ACTIVATION_CHECK)
	$(PNPM) exec tsx tools/ci/check_router_singleton.ts

conventional-commits: deps
	$(ACTIVATION_CHECK)
	$(PNPM) exec tsx tools/commit_copilot/cli.ts enforce

export-roadmap: deps
	$(ACTIVATION_CHECK)
	$(PNPM) export:roadmap

record-local-pipeline:
	$(ACTIVATION_CHECK)
	scripts/pipeline/record_local_pipeline.sh "make pipeline.local"

# World model verification
world-verify:
        @echo "🔍 Reconciling world model consistency..."
        @$(CARGO) run -p noa_core --bin noa_world -- verify

world-fix:
        @echo "🛠️ Generating remediation plan for world model drift..."
        @$(CARGO) run -p noa_core --bin noa_world -- fix

# Kernel build
kernel:
	@echo "🔨 Building kernel crate..."
	$(CARGO) build -p noa_core
	@echo "✅ Kernel crate compiled"

image: kernel
	@echo "🛠️ Producing standalone kernel image..."
	$(CARGO) build -p noa_core --bin noa_kernel --bin noa_host_control --release
	@mkdir -p dist/kernel
	@cp target/release/noa_kernel dist/kernel/
	@cp target/release/noa_host_control dist/kernel/
	@cp core/config/default_manifest.yaml dist/kernel/manifest.yaml
	@printf "# NOA ARK Kernel Image\n\nThis directory contains the release-built kernel binaries and manifest for controlled execution.\n" > dist/kernel/README.md
	@set -euo pipefail; $(CARGO) test -p noa_core --tests -- --nocapture | tee dist/kernel/test-results.log
	@echo "✅ Kernel image staged under dist/kernel"
	$(ACTIVATION_CHECK)
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
        @echo "🎯 Calculating trust scores..."
        @mkdir -p metrics
        @cargo run -p noa_workflow --bin reward_report -- --json > metrics/reward_summary.json
        @cargo run -p noa_workflow --bin reward_report -- > metrics/reward_summary.txt
        @echo "✅ Scorekeeper report generated at metrics/reward_summary.json"
        @TARGET=$${NOA_TRUST_METRICS_PATH:-metrics/trust_score.json}; \
            NOA_TRUST_METRICS_PATH=$$TARGET $(CARGO) run -p noa_core --bin noa_scorekeeper -- \
                --integrity-pass $${TRUST_INTEGRITY_PASS:-120} \
                --integrity-fail $${TRUST_INTEGRITY_FAIL:-0} \
                --reversibility-pass $${TRUST_REVERSIBILITY_PASS:-96} \
                --reversibility-fail $${TRUST_REVERSIBILITY_FAIL:-4} \
                --capability-pass $${TRUST_CAPABILITY_PASS:-80} \
                --capability-fail $${TRUST_CAPABILITY_FAIL:-20} \
            || { echo "❌ Scorekeeper failed"; exit 1; }; \
            echo "✅ Trust snapshot stored at $$TARGET"
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
	@mkdir -p audit
	@cargo run --manifest-path cicd/Cargo.toml --bin publish_audit -- --repo . --output audit --ledger audit/ledger.jsonl
	@latest=$$(ls -d audit/bundle-* 2>/dev/null | tail -n 1); \
	if [ -n "$$latest" ]; then \
		echo "🔍 Verifying $$latest"; \
		audit/verify.sh "$$latest"; \
	else \
		echo "⚠️  No bundle produced"; \
	fi

# Run rollback simulation locally
rollback-sim:
	@echo "⏱️  Running rollback simulation..."
	@cargo run --manifest-path cicd/Cargo.toml --bin rollback_simulation -- --repo . --ledger audit/ledger.jsonl --output audit/rollbacks
@echo "📤 Publishing audit bundle..."
@$(PYTHON) -m tools.repro.audit_pipeline publish

# Setup toolchain
setup:
        @echo "🔧 Setting up build environment..."
        @# Install Rust if needed
        @command -v rustc >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        @# Install pnpm if needed
        @command -v pnpm >/dev/null 2>&1 || npm install -g pnpm
        @echo "🔍 Capturing Cargo environment snapshot..."
        @$(CARGO) --version
        @echo "✅ Setup complete"
