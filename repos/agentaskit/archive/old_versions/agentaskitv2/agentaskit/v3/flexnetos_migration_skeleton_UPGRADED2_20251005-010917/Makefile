
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

.PHONY: all
all: init gen-sbom sign verify contract-test tri-run merge anchor

.PHONY: init
init:
	@mkdir -p $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(ORCH)/state $(ORCH)/keys
	@echo "[init] Repo initialized."

.PHONY: gen-sbom
gen-sbom: init
	@$(PY) $(TOOLS)/sbom_gen.py --root $(ROOT) --out $(SBOM_DIR)/sbom.cdx.json
	@echo "[gen-sbom] wrote $(SBOM_DIR)/sbom.cdx.json"

.PHONY: sign
sign: gen-sbom
	@$(PY) $(TOOLS)/signer.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --out $(ARTIFACTS)/MANIFEST.sha256
	@if command -v minisign >/dev/null 2>&1; then \
		if [[ -f "$(ORCH)/keys/minisign.key" ]]; then \
			minisign -Sm $(ARTIFACTS)/MANIFEST.sha256 -s $(ORCH)/keys/minisign.key || true; \
			echo "[sign] minisign signature created."; \
		else echo "[sign] minisign present but no key at orchestrator/keys/minisign.key"; fi; \
	else echo "[sign] minisign not found; proceeding with manifest only."; fi

.PHONY: verify
verify: sign
	@$(PY) $(TOOLS)/verify.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256
	@echo "[verify] OK"

.PHONY: contract-test
contract-test:
	@$(PY) $(TOOLS)/contract_test.py --contracts $(CONTRACTS) --samples $(CONTRACTS)/samples
	@echo "[contract-test] pass"

.PHONY: build-core
build-core:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/core && cargo build --release; \
		echo "[build-core] built flex-core."; \
	else echo "[build-core] cargo not found; skipping."; fi

.PHONY: tri-run
tri-run:
	@$(PY) $(TOOLS)/tri_runner.py --root $(ROOT) --inputs $(SANDBOX)/inputs --tri $(TRI) --out $(SANDBOX)/outputs
	@echo "[tri-run] A/B/C outputs ready"

.PHONY: merge
merge: tri-run
	@$(PY) $(SANDBOX)/tri-sandbox/unifier/merge.py --tri $(SANDBOX)/outputs --parent $(PARENT) --report $(PARENT)/fitness-report.json
	@echo "[merge] Model D ready"

.PHONY: anchor
anchor: verify
	@$(PY) $(TOOLS)/merkle_anchor.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256 --out $(ANCHORS)/anchor-$(shell date +%Y%m%d-%H%M%S).json
	@echo "[anchor] anchor receipt written"

.PHONY: promote
promote: merge verify
	@$(PY) $(TOOLS)/promote.py --parent $(PARENT) --exec $(EXEC)
	@echo "[promote] Model D promoted"

.PHONY: hooks-install
hooks-install:
	@mkdir -p .git/hooks hooks
	@cp hooks/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "[hooks-install] pre-push hook installed"

.PHONY: orchestrator-sim
orchestrator-sim:
	@$(PY) $(ORCH)/agent_runtime/agent_orchestrator.py --demo

.PHONY: numa-pin hugepages
numa-pin:
	@$(TOOLS)/numa_pin.sh --help || true

hugepages:
	@$(TOOLS)/hugepages.sh --help || true

.PHONY: clean
clean:
	@rm -rf $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(SANDBOX)/outputs $(PARENT)/model-D $(PARENT)/fitness-report.json
	@echo "[clean] done"
