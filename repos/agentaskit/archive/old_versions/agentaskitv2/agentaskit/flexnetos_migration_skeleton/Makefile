
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

# Default target
.PHONY: all
all: init gen-sbom sign verify contract-test tri-run merge anchor

.PHONY: init
init:
	@mkdir -p $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS)
	@echo "[init] Repo initialized."

# Generate CycloneDX SBOMs for core, wasm, and models (minimal but valid)
.PHONY: gen-sbom
gen-sbom: init
	@$(PY) $(TOOLS)/sbom_gen.py --root $(ROOT) --out $(SBOM_DIR)/sbom.cdx.json
	@echo "[gen-sbom] Wrote $(SBOM_DIR)/sbom.cdx.json"

# Sign artifacts and SBOM with local offline scheme (SHA256 manifest) and optional minisign/gpg
.PHONY: sign
sign: gen-sbom
	@$(PY) $(TOOLS)/signer.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --out $(ARTIFACTS)/MANIFEST.sha256
	@if command -v minisign >/dev/null 2>&1; then \
		minisign -Sm $(ARTIFACTS)/MANIFEST.sha256 -s $(ORCH)/keys/minisign.key || true; \
		echo "[sign] minisign signature created (if key present)."; \
	else echo "[sign] minisign not found; SHA256 manifest only."; fi

# Verify signatures and SBOM hashes
.PHONY: verify
verify: sign
	@$(PY) $(TOOLS)/verify.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256
	@echo "[verify] verification OK"

# Consumer-driven contract tests for the hot path IDL + golden samples
.PHONY: contract-test
contract-test:
	@$(PY) $(TOOLS)/contract_test.py --contracts $(CONTRACTS) --samples $(CONTRACTS)/samples
	@echo "[contract-test] contracts pass"

# Build core (optional; runs only if cargo present)
.PHONY: build-core
build-core:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/core && cargo build --release; \
		echo "[build-core] built with cargo."; \
	else echo "[build-core] cargo not found; skipping."; fi

# Tri-sandbox parallel run (A/B/C) over inputs/, recording outputs and traces
.PHONY: tri-run
tri-run:
	@$(PY) $(TOOLS)/tri_runner.py --root $(ROOT) --inputs $(SANDBOX)/inputs --tri $(TRI) --out $(SANDBOX)/outputs
	@echo "[tri-run] A/B/C outputs ready."

# Merge A/B/C → Model D using evolutionary majority + fitness
.PHONY: merge
merge: tri-run
	@$(PY) $(SANDBOX)/tri-sandbox/unifier/merge.py --tri $(SANDBOX)/outputs --parent $(PARENT) --report $(PARENT)/fitness-report.json
	@echo "[merge] Produced Model D and fitness report."

# Anchor release: compute Merkle root over artifacts+sbom+policies and write anchor file
.PHONY: anchor
anchor: verify
	@$(PY) $(TOOLS)/merkle_anchor.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256 --out $(ANCHORS)/anchor-$(shell date +%Y%m%d-%H%M%S).txt
	@echo "[anchor] Anchored Merkle root locally (ready for on-chain)."

# Promote Model D to execution plane (copy & attest)
.PHONY: promote
promote: merge verify
	@$(PY) $(TOOLS)/promote.py --parent $(PARENT) --exec $(EXEC)
	@echo "[promote] Model D promoted to execution plane."

.PHONY: clean
clean:
	@rm -rf $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(SANDBOX)/outputs $(PARENT)/model-D $(PARENT)/fitness-report.json
	@echo "[clean] ephemeral outputs removed."
