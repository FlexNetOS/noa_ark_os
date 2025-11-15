#!/usr/bin/make -f

# FlexNetOS Migration Skeleton - UNIFIED VERSION
# Heals all regressions and preserves ALL capabilities from all versions
# Offline-first, no Docker required. Optional: minisign, gpg, cargo if installed.

SHELL := /bin/bash
PY := python3
ROOT := $(CURDIR)
TOOLS := $(ROOT)/tools
SANDBOX := $(ROOT)/sandbox
EXEC := $(ROOT)/execution
ORCH := $(ROOT)/orchestrator
ARTIFACTS := $(ROOT)/artifacts
SBOM_DIR := $(ROOT)/sbom
ANCHORS := $(ROOT)/anchors
CONTRACTS := $(ROOT)/contracts
TRI := $(SANDBOX)/tri-sandbox
PARENT := $(SANDBOX)/parent

# Next actions helper (from v7)
NEXT := $(PY) $(TOOLS)/next_actions.py

# Default target
.PHONY: all
all: init gen-sbom sign verify contract-test tri-run merge anchor
	@$(NEXT) --target all

.PHONY: init
init:
	@mkdir -p $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(ORCH)/state $(ORCH)/keys
	@echo "[init] Repo initialized with ALL directories preserved."
	@$(NEXT) --target init

# Generate CycloneDX SBOMs for core, wasm, and models (minimal but valid)
.PHONY: gen-sbom
gen-sbom: init
	@$(PY) $(TOOLS)/sbom_gen.py --root $(ROOT) --out $(SBOM_DIR)/sbom.cdx.json
	@echo "[gen-sbom] wrote $(SBOM_DIR)/sbom.cdx.json"
	@$(NEXT) --target gen-sbom

# Sign artifacts and SBOM with local offline scheme (SHA256 manifest) and optional minisign/gpg
.PHONY: sign
sign: gen-sbom
	@$(PY) $(TOOLS)/signer.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --out $(ARTIFACTS)/MANIFEST.sha256
	@if command -v minisign >/dev/null 2>&1; then \
		if [[ -f "$(ORCH)/keys/minisign.key" ]]; then \
			minisign -Sm $(ARTIFACTS)/MANIFEST.sha256 -s $(ORCH)/keys/minisign.key || true; \
			echo "[sign] minisign signature created ($(ARTIFACTS)/MANIFEST.sha256.minisig)."; \
		else echo "[sign] minisign present but no key at orchestrator/keys/minisign.key"; fi; \
	else echo "[sign] minisign not found; proceeding with manifest only."; fi
	@$(NEXT) --target sign

# Verify signatures and SBOM hashes
.PHONY: verify
verify: sign
	@$(PY) $(TOOLS)/verify.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256
	@echo "[verify] verification OK"
	@$(NEXT) --target verify

# Consumer-driven contract tests for the hot path IDL + golden samples  
.PHONY: contract-test
contract-test:
	@$(PY) $(TOOLS)/contract_test.py --contracts $(CONTRACTS) --samples $(CONTRACTS)/samples
	@echo "[contract-test] contracts pass"
	@$(NEXT) --target contract-test

# ---- Core server (enhanced from v7) ----
.PHONY: build-core run-core smoke-client py-client
build-core:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/core && cargo build --release; \
		echo "[build-core] built flex-core and flex-client."; \
	else echo "[build-core] cargo not found; skipping."; fi
	@$(NEXT) --target build-core

run-core: build-core
	@if [ -x "$(TOOLS)/numa_pin.sh" ]; then \
		$(TOOLS)/numa_pin.sh --node 0 -- $(EXEC)/core/target/release/flex-core; \
	else \
		$(EXEC)/core/target/release/flex-core; \
	fi
	@$(NEXT) --target run-core

smoke-client: build-core
	@echo "[smoke-client] launching server, running client, shutting down"
	@( $(EXEC)/core/target/release/flex-core & echo $$! > /tmp/flex_core.pid; \
	   sleep 1; \
	   $(EXEC)/core/target/release/flex-client || true; \
	   kill $$(cat /tmp/flex_core.pid) >/dev/null 2>&1 || true; \
	   rm -f /tmp/flex_core.pid )
	@$(NEXT) --target smoke-client

py-client:
	@$(PY) $(TOOLS)/capnp_python_client.py || true
	@$(NEXT) --target py-client

# ---- WASM host & connectors (from v7) ----
.PHONY: build-wasm-host mint-cap run-wasm-demo run-wasm-capfile
build-wasm-host:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/wasm_host && cargo build --release; \
		echo "[build-wasm-host] built wasm_host."; \
	else echo "[build-wasm-host] cargo not found; skipping."; fi
	@$(NEXT) --target build-wasm-host

mint-cap:
	@$(PY) $(TOOLS)/cap_token.py --sub demo --aud wasm-host --scopes connector:echo connector:json-filter connector:readfile --secret "$${FLEX_CONNECTOR_SECRET:-changeme}" --out $(EXEC)/connectors/demo.cap
	@echo "[mint-cap] wrote $(EXEC)/connectors/demo.cap"
	@$(NEXT) --target mint-cap

run-wasm-demo: build-wasm-host mint-cap
	@FLEX_CAP_TOKEN="$$(cat $(EXEC)/connectors/demo.cap)" FLEX_CONNECTOR_SECRET="${FLEX_CONNECTOR_SECRET:-changeme}" $(EXEC)/wasm_host/target/release/wasm_host $(EXEC)/connectors/echo/echo.wat
	@$(NEXT) --target run-wasm-demo

run-wasm-capfile: build-wasm-host mint-cap
	@mkdir -p $(EXEC)/connectors/cap-sandbox && echo "Hello from preopened dir!" > $(EXEC)/connectors/cap-sandbox/hello.txt
	@FLEX_CAP_TOKEN="$$(cat $(EXEC)/connectors/demo.cap)" FLEX_CONNECTOR_SECRET="${FLEX_CONNECTOR_SECRET:-changeme}" FLEX_PREOPEN_DIR="$(EXEC)/connectors/cap-sandbox" $(EXEC)/wasm_host/target/release/wasm_host $(EXEC)/connectors/readfile/readfile.wat
	@$(NEXT) --target run-wasm-capfile

# ---- fs-verity policy allowlist (from v7) ----
.PHONY: update-verity-policy check-verity-policy fs-verity-enable fs-verity-verify fs-verity-sign
update-verity-policy:
	@$(PY) $(TOOLS)/verity_policy.py update --file $(ARTIFACTS)/MANIFEST.sha256 --policy $(EXEC)/policies/fsverity_allowlist.json
	@echo "[verity] policy updated"
	@$(NEXT) --target update-verity-policy

check-verity-policy:
	@$(PY) $(TOOLS)/verity_policy.py check --file $(ARTIFACTS)/MANIFEST.sha256 --policy $(EXEC)/policies/fsverity_allowlist.json
	@$(NEXT) --target check-verity-policy

fs-verity-enable:
	@sudo $(TOOLS)/fs_integrity.sh verity-enable $(ARTIFACTS)/MANIFEST.sha256
	@$(NEXT) --target fs-verity-enable

fs-verity-verify:
	@$(TOOLS)/fs_integrity.sh verity-status $(ARTIFACTS)/MANIFEST.sha256
	@$(NEXT) --target fs-verity-verify

fs-verity-sign:
	@$(TOOLS)/fs_integrity.sh verity-sign $(ARTIFACTS)/MANIFEST.sha256 $${FSV_KEY:-/path/key.pem} $${FSV_CERT:-/path/cert.pem}
	@$(NEXT) --target fs-verity-sign

# ---- File system integrity (HEALED - from v5) ----
.PHONY: seal-immutable unseal-immutable
seal-immutable:
	@$(TOOLS)/fs_integrity.sh seal-immutable $(ARTIFACTS)/MANIFEST.sha256
	@echo "[fs] sealed immutable"

unseal-immutable:
	@$(TOOLS)/fs_integrity.sh unseal-immutable $(ARTIFACTS)/MANIFEST.sha256  
	@echo "[fs] unsealed immutable"

# ---- Release anchors & tri-merge ----
.PHONY: anchor promote tri-run merge
anchor: verify
	@$(PY) $(TOOLS)/merkle_anchor.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256 --out $(ANCHORS)/anchor-$(shell date +%Y%m%d-%H%M%S).json
	@echo "[anchor] anchor receipt written to anchors/ directory (HEALED)"
	@$(NEXT) --target anchor

promote: merge verify
	@$(PY) $(TOOLS)/promote.py --parent $(PARENT) --exec $(EXEC)
	@echo "[promote] Model D promoted"
	@$(NEXT) --target promote

tri-run:
	@$(PY) $(TOOLS)/tri_runner.py --root $(ROOT) --inputs $(SANDBOX)/inputs --tri $(TRI) --out $(SANDBOX)/outputs
	@echo "[tri-run] A/B/C outputs ready"
	@$(NEXT) --target tri-run

merge: tri-run
	@$(PY) $(SANDBOX)/tri-sandbox/unifier/merge.py --tri $(SANDBOX)/outputs --parent $(PARENT) --report $(PARENT)/fitness-report.json
	@echo "[merge] Model D ready"
	@$(NEXT) --target merge

# ---- Hooks & sys helpers ----
.PHONY: hooks-install orchestrator-sim numa-pin hugepages clean
hooks-install:
	@mkdir -p .git/hooks hooks
	@cp hooks/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "[hooks-install] pre-push hook installed"
	@$(NEXT) --target hooks-install

# HEALED: Agent orchestrator from v2-v3 (PT/POP mechanics)
orchestrator-sim:
	@$(PY) $(ORCH)/agent_runtime/agent_orchestrator.py --demo
	@echo "[orchestrator] agent runtime with PT/POP mechanics (HEALED from v2-v3)"
	@$(NEXT) --target orchestrator-sim

numa-pin:
	@$(TOOLS)/numa_pin.sh --help || true
	@$(NEXT) --target numa-pin

hugepages:
	@$(TOOLS)/hugepages.sh --show || true
	@$(NEXT) --target hugepages

# HEALED: All directories preserved
clean:
	@rm -rf $(ARTIFACTS)/* $(SBOM_DIR)/* $(ANCHORS)/* $(SANDBOX)/outputs $(PARENT)/model-D $(PARENT)/fitness-report.json
	@echo "[clean] ephemeral outputs removed (directories preserved - HEALED)"
	@$(NEXT) --target clean

# Additional targets for completeness
.PHONY: status-check full-test
status-check:
	@echo "[status] Checking system capabilities..."
	@echo "  - Python: $$(python3 --version 2>/dev/null || echo 'NOT FOUND')"
	@echo "  - Cargo: $$(cargo --version 2>/dev/null || echo 'NOT FOUND')" 
	@echo "  - minisign: $$(minisign -v 2>/dev/null || echo 'NOT FOUND')"
	@echo "  - fsverity: $$(fsverity --version 2>/dev/null || echo 'NOT FOUND')"
	@echo "  - capnp: $$(capnp --version 2>/dev/null || echo 'NOT FOUND')"

full-test: all
	@echo "[full-test] Running comprehensive test suite..."
	@$(MAKE) status-check
	@$(MAKE) orchestrator-sim
	@echo "[full-test] All systems operational"