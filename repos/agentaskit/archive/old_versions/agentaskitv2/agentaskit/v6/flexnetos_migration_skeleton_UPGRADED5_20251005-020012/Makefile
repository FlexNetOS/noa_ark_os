
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

NEXT := $(PY) $(TOOLS)/next_actions.py

.PHONY: all
all: init gen-sbom sign verify contract-test tri-run merge anchor
	@$(NEXT) --target all

.PHONY: init
init:
	@mkdir -p $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(ORCH)/state $(ORCH)/keys
	@echo "[init] Repo initialized."
	@$(NEXT) --target init

.PHONY: gen-sbom
gen-sbom: init
	@$(PY) $(TOOLS)/sbom_gen.py --root $(ROOT) --out $(SBOM_DIR)/sbom.cdx.json
	@echo "[gen-sbom] wrote $(SBOM_DIR)/sbom.cdx.json"
	@$(NEXT) --target gen-sbom

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

.PHONY: verify
verify: sign
	@$(PY) $(TOOLS)/verify.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256
	@echo "[verify] OK"
	@$(NEXT) --target verify

.PHONY: contract-test
contract-test:
	@$(PY) $(TOOLS)/contract_test.py --contracts $(CONTRACTS) --samples $(CONTRACTS)/samples
	@echo "[contract-test] pass"
	@$(NEXT) --target contract-test

.PHONY: build-core
build-core:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/core && cargo build --release; \
		echo "[build-core] built flex-core and flex-client."; \
	else echo "[build-core] cargo not found; skipping."; fi
	@$(NEXT) --target build-core

.PHONY: run-core
run-core: build-core
	@if [ -x "$(TOOLS)/numa_pin.sh" ]; then \
		$(TOOLS)/numa_pin.sh --node 0 -- $(EXEC)/core/target/release/flex-core; \
	else \
		$(EXEC)/core/target/release/flex-core; \
	fi
	@$(NEXT) --target run-core

.PHONY: smoke-client
smoke-client: build-core
	@echo "[smoke-client] launching server, running client, shutting down"
	@( $(EXEC)/core/target/release/flex-core & echo $$! > /tmp/flex_core.pid; \
	   sleep 1; \
	   $(EXEC)/core/target/release/flex-client || true; \
	   kill $$(cat /tmp/flex_core.pid) >/dev/null 2>&1 || true; \
	   rm -f /tmp/flex_core.pid )
	@$(NEXT) --target smoke-client

.PHONY: py-client
py-client:
	@$(PY) $(TOOLS)/capnp_python_client.py || true
	@$(NEXT) --target py-client

# ---- WASM host & connectors ----
.PHONY: build-wasm-host
build-wasm-host:
	@if command -v cargo >/dev/null 2>&1; then \
		cd $(EXEC)/wasm_host && cargo build --release; \
		echo "[build-wasm-host] built wasm_host."; \
	else echo "[build-wasm-host] cargo not found; skipping."; fi
	@$(NEXT) --target build-wasm-host

.PHONY: mint-cap
mint-cap:
	@$(PY) $(TOOLS)/cap_token.py --sub demo --aud wasm-host --scopes read echo --secret "$${FLEX_CONNECTOR_SECRET:-changeme}" --out $(EXEC)/connectors/demo.cap
	@echo "[mint-cap] wrote $(EXEC)/connectors/demo.cap"
	@$(NEXT) --target mint-cap

.PHONY: run-wasm-demo
run-wasm-demo: build-wasm-host mint-cap
	@FLEX_CAP_TOKEN="$$(cat $(EXEC)/connectors/demo.cap)" $(EXEC)/wasm_host/target/release/wasm_host $(EXEC)/connectors/echo/echo.wat
	@$(NEXT) --target run-wasm-demo

# ---- fs-verity signing ----
.PHONY: fs-verity-enable fs-verity-verify fs-verity-sign
fs-verity-enable:
	@sudo $(TOOLS)/fs_integrity.sh verity-enable $(ARTIFACTS)/MANIFEST.sha256
	@$(NEXT) --target fs-verity-enable

fs-verity-verify:
	@$(TOOLS)/fs_integrity.sh verity-status $(ARTIFACTS)/MANIFEST.sha256
	@$(NEXT) --target fs-verity-verify

fs-verity-sign:
	@$(TOOLS)/fs_integrity.sh verity-sign $(ARTIFACTS)/MANIFEST.sha256 $${FSV_KEY:-/path/key.pem} $${FSV_CERT:-/path/cert.pem}
	@$(NEXT) --target fs-verity-sign

.PHONY: anchor
anchor: verify
	@$(PY) $(TOOLS)/merkle_anchor.py --root $(ROOT) --sbom $(SBOM_DIR)/sbom.cdx.json --manifest $(ARTIFACTS)/MANIFEST.sha256 --out $(ANCHORS)/anchor-$(shell date +%Y%m%d-%H%M%S).json
	@echo "[anchor] anchor receipt written"
	@$(NEXT) --target anchor

.PHONY: promote
promote: merge verify
	@$(PY) $(TOOLS)/promote.py --parent $(PARENT) --exec $(EXEC)
	@echo "[promote] Model D promoted"
	@$(NEXT) --target promote

.PHONY: tri-run
tri-run:
	@$(PY) $(TOOLS)/tri_runner.py --root $(ROOT) --inputs $(SANDBOX)/inputs --tri $(TRI) --out $(SANDBOX)/outputs
	@echo "[tri-run] A/B/C outputs ready"
	@$(NEXT) --target tri-run

.PHONY: merge
merge: tri-run
	@$(PY) $(SANDBOX)/tri-sandbox/unifier/merge.py --tri $(SANDBOX)/outputs --parent $(PARENT) --report $(PARENT)/fitness-report.json
	@echo "[merge] Model D ready"
	@$(NEXT) --target merge

.PHONY: hooks-install
hooks-install:
	@mkdir -p .git/hooks hooks
	@cp hooks/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "[hooks-install] pre-push hook installed"
	@$(NEXT) --target hooks-install

.PHONY: orchestrator-sim
orchestrator-sim:
	@$(PY) $(ORCH)/agent_runtime/agent_orchestrator.py --demo
	@$(NEXT) --target orchestrator-sim

.PHONY: numa-pin hugepages
numa-pin:
	@$(TOOLS)/numa_pin.sh --help || true
	@$(NEXT) --target numa-pin

hugepages:
	@$(TOOLS)/hugepages.sh --show || true
	@$(NEXT) --target hugepages

.PHONY: clean
clean:
	@rm -rf $(ARTIFACTS) $(SBOM_DIR) $(ANCHORS) $(SANDBOX)/outputs $(PARENT)/model-D $(PARENT)/fitness-report.json
	@echo "[clean] done"
	@$(NEXT) --target clean
