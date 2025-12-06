# Kernel-first manifest

This directory hosts the canonical machine-readable manifest that captures the kernel-first dependency graph for AgentOS.
It is paired with tooling in `scripts/packaging` and `scripts/ci` that consumes the manifest to produce reproducible build
artifacts and enforce interface compliance across services.

The manifest is intentionally deterministic so that packaging outputs are portable between Linux, macOS, and containerized
runtimes. Each service entry includes interface contracts, boot ordering, optional dependencies, and health semantics that are
referenced by the single-host orchestration profile.

Consumers should use the helper APIs in `core.kernel.manifest.loader` to read the manifest instead of parsing the JSON file
directly. The loader performs schema validation and provides convenience helpers for querying dependency relationships.
