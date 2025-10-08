# Makefile Patterns • 9-Phase Merge Orchestration

**Source**: WSL noa_ark_os `consolidation_merger/Makefile`  
**Integration**: Cycle 5 - NOA Ark OS Selective Drop  
**Date**: 2025-01-XX  
**Purpose**: Build automation, security, CI simulation

---

## Overview

This document extracts reusable Makefile patterns for merge orchestration, build automation, testing, security scanning, and CI pipeline simulation. The patterns demonstrate defensive merge strategies and comprehensive validation workflows.

---

## 9-Phase Merge Orchestration

### Configuration
```makefile
MERGE_ID ?= demo_merge
TARGET   ?= workspace/noa_ark_os/consolidation_merger/.merged_out
SOURCES  ?=
OUT_DIR  := workspace/noa_ark_os/consolidation_merger/.merge/$(MERGE_ID)
SIM_PATH ?= /tmp/merge_sim_$(MERGE_ID)
```

---

### Phase 1: AUTHOR - Define Strategy

```makefile
phase1: ## Phase 1 - AUTHOR (Define strategy & intent; creates MERGE_STRATEGY.md)
	@mkdir -p $(OUT_DIR)
	@{
		echo "# Merge Strategy: $(MERGE_ID)";
		echo "";
		echo "## Context";
		echo "- Purpose: $(or $(PURPOSE),Consolidate and flatten sources)";
		echo "- Success Criteria: $(or $(SUCCESS),Single unified output, no data loss)";
		echo "- Constraints: $(or $(CONSTRAINTS),Upgrades only; archive before delete)";
		echo "- Scope: $(or $(SCOPE),$(SOURCES))";
		echo "";
		echo "## Merge Semantics";
		echo "- Type: $(or $(TYPE),CONSOLIDATION)";
		echo "- Output Structure: $(or $(STRUCTURE),FLAT)";
		echo "- Duplicate Strategy: $(or $(DUP_STRATEGY),LATEST_WINS)";
		echo "";
		echo "## Sources";
		for s in $(SOURCES); do echo "- $$s"; done;
		echo "";
		echo "## Target";
		echo "- Location: $(TARGET)";
		echo "";
	} > $(OUT_DIR)/MERGE_STRATEGY.md
	@echo "[phase1] Wrote $(OUT_DIR)/MERGE_STRATEGY.md"
```

**Purpose**: Document merge intent, constraints, and strategy  
**Output**: `MERGE_STRATEGY.md`  
**Parameters**: `PURPOSE`, `SUCCESS`, `CONSTRAINTS`, `SCOPE`, `TYPE`, `STRUCTURE`, `DUP_STRATEGY`

---

### Phase 2: INDEX/SIGN - Create Baseline Hashes

```makefile
phase2: ## Phase 2 - INDEX/SIGN (Create HASHES_PRE_MERGE.txt)
	@mkdir -p $(OUT_DIR)
	@echo "[phase2] Indexing sources: $(SOURCES)"
	@{
		for s in $(SOURCES); do \
			if [ -d "$$s" ]; then \
				find "$$s" -type f -printf '%p\n' | while read -r f; do sha256sum "$$f"; done; \
			elif [ -f "$$s" ]; then sha256sum "$$s"; \
			else echo "SKIP $$s" 1>&2; fi; \
		done; \
	} | sort -k2 > $(OUT_DIR)/HASHES_PRE_MERGE.txt
	@echo "[phase2] Wrote $(OUT_DIR)/HASHES_PRE_MERGE.txt"
```

**Purpose**: Create cryptographic baseline of all source files  
**Output**: `HASHES_PRE_MERGE.txt` (sorted by filename)  
**Algorithm**: SHA-256

---

### Phase 3: SEAL/POLICY - Create Policy Checklist

```makefile
phase3: ## Phase 3 - SEAL/POLICY (Create POLICY_CHECKLIST.yaml)
	@mkdir -p $(OUT_DIR)
	@{
		echo "merge_id: $(MERGE_ID)";
		echo "timestamp: $$(date -u +%Y-%m-%dT%H:%M:%SZ)";
		echo "baseline_version: baseline";
		echo "baseline_tests_passing: 0/0";
		echo "policies:";
		echo "  - name: No Content Loss";
		echo "    verification_method: SHA-256 coverage and semantic check";
		echo "    criteria: Baseline content present";
		echo "    status: pending";
		echo "  - name: No Downgrade";
		echo "    verification_method: Hash match vs known-good";
		echo "    criteria: Output >= baseline quality";
		echo "    status: pending";
	} > $(OUT_DIR)/POLICY_CHECKLIST.yaml
	@echo "[phase3] Wrote $(OUT_DIR)/POLICY_CHECKLIST.yaml"
```

**Purpose**: Establish governance policies for merge  
**Output**: `POLICY_CHECKLIST.yaml`  
**Policies**: No Content Loss, No Downgrade

---

### Phase 4: TRI-RUN - Simulate A/B/C Layouts

```makefile
phase4: ## Phase 4 - TRI-RUN (Simulate A/B/C layouts)
	@mkdir -p $(SIM_PATH)/{ModelA,ModelB,ModelC} $(SIM_PATH)/sources $(OUT_DIR)
	@for s in $(SOURCES); do \
		if [ -d "$$s" ]; then rsync -a "$$s"/ $(SIM_PATH)/sources/; \
		else [ -f "$$s" ] && rsync -a "$$s" $(SIM_PATH)/sources/ || true; fi; \
	done
	@{
		echo "# Merge Simulation Report: $(MERGE_ID)"; \
		echo "Sources: $(SOURCES)"; \
		echo "Simulation Path: $(SIM_PATH)"; \
		echo "Timestamp: $$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
		echo "\nModels prepared: ModelA, ModelB, ModelC"; \
	} > $(OUT_DIR)/SIMULATION_REPORT.md
	@echo "[phase4] Prepared tri-run skeleton at $(SIM_PATH)"
```

**Purpose**: Prepare tri-sandbox simulation environment  
**Output**: `SIMULATION_REPORT.md`, ModelA/B/C directories  
**Strategy**: Consolidate sources for parallel evaluation

---

### Phase 5: PRE-FLIGHT & MERGE(D)

#### Phase 5-Preflight: Review Proposed Changes

```makefile
phase5-preflight: ## Phase 5 - PRE-FLIGHT (Show proposed output; no file ops)
	@mkdir -p $(OUT_DIR)
	@echo "=== PHASE 5 PRE-FLIGHT CHECK ===" > $(OUT_DIR)/PREFLIGHT_CHECK.md
	@echo "Target: $(TARGET)" >> $(OUT_DIR)/PREFLIGHT_CHECK.md
	@echo "Merge Type: $(or $(TYPE),CONSOLIDATION)" >> $(OUT_DIR)/PREFLIGHT_CHECK.md
	@echo "Duplicate Strategy: $(or $(DUP_STRATEGY),LATEST_WINS)" >> $(OUT_DIR)/PREFLIGHT_CHECK.md
	@echo "[phase5-preflight] See $(OUT_DIR)/PREFLIGHT_CHECK.md; set APPROVE=yes to run phase5-run"
```

**Purpose**: Non-destructive review of proposed changes  
**Output**: `PREFLIGHT_CHECK.md`  
**Governance**: Requires manual review before execution

#### Phase 5-Run: Execute Defensive Merge

```makefile
phase5-run: ## Phase 5 - MERGE(D) (Execute consolidation overlay if APPROVE=yes)
	@[ "$(APPROVE)" = "yes" ] || (echo "Refusing to merge: set APPROVE=yes" && exit 1)
	@mkdir -p $(TARGET)_next $(OUT_DIR)
	@echo "[phase5-run] Consolidating: $(SOURCES) -> $(TARGET)_next"
	@for s in $(SOURCES); do \
		if [ -d "$$s" ]; then \
			rsync --archive --links --times --group --owner --checksum $$RSYNC_DELETE "$$s"/ $(TARGET)_next/; \
		elif [ -f "$$s" ]; then \
			rsync --archive --links --times --group --owner --checksum $$RSYNC_DELETE "$$s" $(TARGET)_next/; \
		fi; \
	done
	@ln -sfn $(TARGET)_next $(TARGET)
	@echo "$(MERGE_ID) $(SOURCES) -> $(TARGET) at $$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> $(OUT_DIR)/MERGE_LOG.txt
	@echo "[phase5-run] Overlay complete; symlink updated: $(TARGET) -> $(TARGET)_next"
```

**Purpose**: Defensive merge with approval gate  
**Output**: `$(TARGET)_next/`, `MERGE_LOG.txt`  
**Governance**: Requires `APPROVE=yes`, uses symlink for atomic switch  
**Strategy**: rsync with checksum verification

---

### Phase 6: VERIFY/CONTRACT - Post-Merge Hashes

```makefile
phase6: ## Phase 6 - VERIFY/CONTRACT (Hashes post-merge)
	@[ -d $(TARGET)_next ] || (echo "Missing $(TARGET)_next; run phase5-run" && exit 1)
	@find $(TARGET)_next -type f -exec sha256sum {} \; | sort -k2 > $(OUT_DIR)/HASHES_POST_MERGE.txt
	@echo "[phase6] Wrote $(OUT_DIR)/HASHES_POST_MERGE.txt"
```

**Purpose**: Verify merge integrity with post-merge hashes  
**Output**: `HASHES_POST_MERGE.txt`  
**Verification**: Compare with HASHES_PRE_MERGE.txt

---

### Phase 7: ANCHOR - Merkle Root

```makefile
phase7: ## Phase 7 - ANCHOR (Merkle root over target)
	@[ -f $(OUT_DIR)/HASHES_POST_MERGE.txt ] || (echo "Missing HASHES_POST_MERGE.txt; run phase6" && exit 1)
	@sha256sum $(OUT_DIR)/HASHES_POST_MERGE.txt | awk '{print $$1}' > $(OUT_DIR)/MERKLE_ROOT.txt
	@{
		echo '{'; \
		echo '  "merge_id": "$(MERGE_ID)",'; \
		echo '  "timestamp": "'$$(date -u +%Y-%m-%dT%H:%M:%SZ)'",'; \
		echo '  "merkle_root": "'$$(cat $(OUT_DIR)/MERKLE_ROOT.txt)'"'; \
		echo '}'; \
	} > $(OUT_DIR)/MERGE_ANCHOR.json
	@echo "[phase7] Wrote $(OUT_DIR)/MERGE_ANCHOR.json"
```

**Purpose**: Create immutable Merkle root for merge verification  
**Output**: `MERKLE_ROOT.txt`, `MERGE_ANCHOR.json`  
**Algorithm**: SHA-256 of HASHES_POST_MERGE.txt

---

### Phase 8: PROMOTE - Production Deployment

```makefile
phase8: ## Phase 8 - PROMOTE (No-op here; use prompt guidance for production)
	@echo "[phase8] Skipping actual production move. Use rsync + symlink pattern"
```

**Purpose**: Placeholder for production deployment  
**Note**: Actual deployment left to user discretion  
**Pattern**: Follow rsync + symlink pattern from Phase 5

---

### Phase 9: ARCHIVE/CLEANUP - Source Archival

```makefile
phase9: ## Phase 9 - ARCHIVE/CLEANUP (Archive sources, verify)
	@mkdir -p $(OUT_DIR)
	@ARCHIVE=$(OUT_DIR)/$$(echo $(MERGE_ID) | tr ' ' '_')_sources_$$(date -u +%Y%m%dT%H%M%SZ).tar.gz; \
	 echo "[phase9] Creating $$ARCHIVE"; \
	 tar -czf "$$ARCHIVE" $(SOURCES); \
	 sha256sum "$$ARCHIVE" | tee -a $(OUT_DIR)/ARCHIVE_HASHES.txt >/dev/null
	@echo "[phase9] Archive ready; hashes in $(OUT_DIR)/ARCHIVE_HASHES.txt"
```

**Purpose**: Archive source files after successful merge  
**Output**: Timestamped `.tar.gz`, `ARCHIVE_HASHES.txt`  
**Governance**: "Heal, Don't Harm" - preserve sources before cleanup

---

### Complete Pipeline

```makefile
all-phases: phase1 phase2 phase3 phase4 phase5-preflight phase5-run phase6 phase7 phase8 phase9
	@echo "[all-phases] Completed for $(MERGE_ID)"
```

**Usage**:
```bash
make all-phases MERGE_ID=example SOURCES="src1 src2" TARGET=output APPROVE=yes
```

---

## Development Tools

### Installation

```makefile
install: ## Install development dependencies
	pip install -e ".[dev,docs]"

install-minimal: ## Install minimal runtime dependencies
	pip install -e .
```

---

### Testing

```makefile
test: ## Run complete test suite
	cd workspace && python tests/test_suite.py

test-unit: ## Run unit tests only
	pytest tests/ -v --cov=tools --cov-report=html --cov-report=term-missing

test-integration: ## Run integration tests
	pytest tests/ -v -k "integration"
```

---

### Code Quality

```makefile
lint: ## Run linting checks
	flake8 tools/ tests/ --count --select=E9,F63,F7,F82 --show-source --statistics
	flake8 tools/ tests/ --count --exit-zero --max-complexity=10 --max-line-length=100 --statistics

format: ## Format code with black
	black tools/ tests/

format-check: ## Check code formatting
	black --check --diff tools/ tests/

type-check: ## Run type checking with mypy
	mypy tools/ tests/ --ignore-missing-imports
```

---

### Security

```makefile
security-scan: ## Run security vulnerability scanning
	@echo "🔒 Running security scans..."
	@safety check --full-report || echo "⚠️ Safety check found issues"
	@bandit -r tools/ tests/ -f txt || echo "⚠️ Bandit found potential security issues"
	@echo "✅ Security scanning complete"

security-install: ## Install security dependencies
	pip install -e ".[security]"
```

**Tools**:
- **safety**: Check dependencies for known vulnerabilities
- **bandit**: Scan for security issues in Python code

---

### CI Simulation

```makefile
ci: lint format-check type-check test validate-schemas ## Run full CI pipeline locally
	@echo "✅ Local CI pipeline completed successfully"

dev: format type-check test ## Quick development check
```

**Purpose**: Run complete CI pipeline locally before push  
**Stages**: Linting → Format check → Type check → Tests → Schema validation

---

## Docker Support

```makefile
docker-build: ## Build Docker image
	docker build -f test.Dockerfile -t noa-deployment-kit:latest .

docker-run: ## Run Docker container
	docker run --rm -it noa-deployment-kit:latest

docker-test: ## Run tests in Docker
	docker run --rm noa-deployment-kit:latest make test
```

---

## Performance Monitoring

```makefile
performance-test: ## Run performance benchmarks
	@echo "⚡ Running performance tests..."
	@cd workspace && python -m pytest tests/ -k "performance" --benchmark-only
	@echo "✅ Performance testing complete"

performance-monitor: ## Start performance monitoring
	@echo "📊 Starting performance monitoring..."
	@bash deploy/performance-monitor.sh monitor

performance-check: ## Run single performance check
	@echo "📈 Running performance check..."
	@bash deploy/performance-monitor.sh check
```

---

## Logging and Auditing

```makefile
logs-show: ## Show recent application logs
	@echo "📋 Recent application logs:"
	@tail -50 /var/log/noa-deployment.log 2>/dev/null || echo "No logs found"

logs-errors: ## Show recent error logs
	@echo "❌ Recent errors:"
	@grep -i "error\|critical\|fatal" /var/log/noa-deployment.log 2>/dev/null | tail -10 || echo "No errors found"

audit-trail: ## Show deployment audit trail
	@echo "📊 Deployment audit trail:"
	@ls -la /opt/noa-deployment-kit/backups/ 2>/dev/null || echo "No backups found"
```

---

## Schema Validation

```makefile
validate-schemas: ## Validate all unified schemas
	python -c "
	import json
	import jsonschema
	import yaml

	# Load schemas
	with open('schema/capsule.schema.unified.json', 'r') as f:
	    capsule_schema = json.load(f)

	with open('schema/manifest.schema.unified.json', 'r') as f:
	    manifest_schema = json.load(f)

	# Validate manifest
	with open('workspace/stack.manifest.unified.json', 'r') as f:
	    manifest_data = json.load(f)

	try:
	    jsonschema.validate(manifest_data, manifest_schema)
	    print('✅ Manifest schema validation passed')
	except jsonschema.ValidationError as e:
	    print(f'❌ Manifest schema validation failed: {e}')
	    exit(1)

	print('✅ All schema validations passed')
	"

validate-manifest: ## Validate unified manifest integrity
	python tools/normalize_csv_unified.py --validate-manifest workspace/stack.manifest.unified.json
```

---

## Help System

```makefile
help: ## Show this help message
	@echo "NOA Deployment Kit Build System"
	@echo "==============================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
```

**Output Format**:
```
target-name          Description of the target
```

**Color Coding**: Target names in cyan for visibility

---

## Adaptation for NoaArkOS

### 9-Phase Merge Pattern

```makefile
# NoaArkOS-specific configuration
CRC_MERGE_ID ?= crc_drop_$(shell date +%Y%m%d)
CRC_TARGET   ?= crc/drop-in/processing/adaptation
CRC_SOURCES  ?= crc/drop-in/incoming/repos/*
CRC_OUT_DIR  := crc/drop-in/.merge_logs/$(CRC_MERGE_ID)

# Import 9-phase targets
include workflow/makefiles/merge-phases.mk

# CRC-specific wrapper
crc-merge: phase1 phase2 phase3 phase4 phase5-preflight
	@echo "Review $(CRC_OUT_DIR)/PREFLIGHT_CHECK.md"
	@echo "Run: make crc-merge-execute APPROVE=yes"

crc-merge-execute: phase5-run phase6 phase7 phase8 phase9
	@echo "Merge complete: $(CRC_MERGE_ID)"
	@echo "Merkle root: $$(cat $(CRC_OUT_DIR)/MERKLE_ROOT.txt)"
```

---

### CI Integration

```makefile
# NoaArkOS CI targets
.PHONY: ci-check ci-test ci-security ci-full

ci-check: ## Quick CI check (format, lint, type)
	cargo fmt -- --check
	cargo clippy --all-targets --workspace
	cargo check --workspace

ci-test: ## Run all tests
	cargo test --workspace --all-targets

ci-security: ## Security audit
	cargo audit
	cargo deny check

ci-full: ci-check ci-test ci-security ## Complete CI pipeline
	@echo "✅ Full CI pipeline passed"
```

---

## References

### Original Source
- WSL noa_ark_os: `consolidation_merger/Makefile`
- Cycle 5 Integration: `NOA_ARK_OS_SELECTIVE_DROP_MANIFEST.md`

### Related NoaArkOS Documentation
- Consolidation Workflow: `workflow/consolidation_merger_guide.md`
- CRC System: `crc/README.md`
- CI/CD: `cicd/README.md`

### External Tools
- **rsync**: File synchronization with checksum verification
- **sha256sum**: Cryptographic hashing
- **tar/gzip**: Archive creation
- **safety**: Python security auditing
- **bandit**: Python security linting
- **pytest**: Testing framework
- **black**: Code formatting
- **mypy**: Type checking

---

*Document v1.0 - Cycle 5 Integration - Makefile Patterns*
