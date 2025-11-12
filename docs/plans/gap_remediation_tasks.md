# Gap Remediation Task List

This document tracks actionable tasks to resolve outstanding gaps and quality issues identified during verification of the NOA ARK OS roadmap implementation.

## 1. Populate Workflow Blueprint Catalog
- [ ] Create the `workflow/blueprints` directory structure with subfolders for each blueprint category (e.g., `ci_cd/`, `data_ops/`, `agent_swarms/`).
- [ ] Implement sample blueprint definitions in the agreed manifest/schema format, including metadata and execution steps.
- [ ] Wire blueprint discovery into the UI/server workflow loaders so new assets are surfaced automatically.
- [ ] Add validation tests to ensure blueprint manifests conform to schema and load correctly.

## 2. Deliver Marketplace Assets & Tooling
- [ ] Establish the `apps/marketplace/catalog` structure with representative marketplace entries.
- [ ] Implement the `marketctl` tooling under `tools/marketctl/` to package, validate, and publish marketplace assets.
- [ ] Provide sample marketplace listings with licensing metadata and kernel compatibility notes.
- [ ] Document the marketplace submission and review workflow in `docs/community/`.

## 3. Implement Analytics Pipelines
- [ ] Create the `storage/analytics/pipelines`, `models`, and `views` directories with initial pipeline definitions.
- [ ] Implement data ingestion jobs that aggregate CRC throughput, deployment frequency, and agent efficiency metrics.
- [ ] Provide example dashboard configurations that consume the analytics outputs.
- [ ] Add automated tests to verify pipeline execution and data availability.

## 4. Seed Telemetry Storage Samples
- [ ] Add representative OpenTelemetry traces and metrics samples under `storage/telemetry` for validation and onboarding.
- [ ] Update gateway observability documentation to reference the sample datasets and validation steps.
- [ ] Integrate telemetry sample loading into local development scripts/tests.

## 5. Fix CICD Pipeline Struct Duplication
- [ ] Refactor `cicd/src/lib.rs` to eliminate duplicate struct fields and ensure the `Pipeline` struct compiles.
- [ ] Add unit tests covering serialization/deserialization of the `Pipeline` struct.
- [ ] Run `cargo fmt` and `cargo clippy` to validate code quality.

## 6. Extend GPU Detection Beyond NVIDIA
- [ ] Enhance hardware detection in `core/hardware` to recognize AMD, Intel, and Apple GPU capabilities.
- [ ] Add cross-platform detection logic using vendor-specific tools or APIs when available.
- [ ] Provide automated tests or mocks to validate detection across GPU vendors.
- [ ] Update runtime selection policies to leverage the expanded GPU profile data.

## 7. Deliver Value-Add Ecosystem Content
- [ ] Curate and publish initial blueprint bundles, marketplace items, and analytics datasets as part of a cohesive launch.
- [ ] Coordinate documentation updates across `docs/workflows`, `docs/community`, and `docs/analytics` to highlight value-add features.
- [ ] Announce the ecosystem release via the unified UI dashboard and release notes.
- [ ] Establish contribution guidelines and review processes to sustain ecosystem growth.
