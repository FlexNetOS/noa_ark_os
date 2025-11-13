SHELL := /bin/bash

PNPM ?= pnpm
CARGO ?= cargo

.PHONY: build test digest run ci:local lint typecheck format

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
trap '[[ -n "'"'"$$UI_PID'"'"'" ]] && kill $$UI_PID 2>/dev/null || true; \
      [[ -n "'"'"$$API_PID'"'"'" ]] && kill $$API_PID 2>/dev/null || true' EXIT INT TERM; \
$(PNPM) --filter vibe-kanban dev & \
UI_PID=$$!; \
$(CARGO) run -p noa_ui_api & \
API_PID=$$!; \
wait $$UI_PID $$API_PID
