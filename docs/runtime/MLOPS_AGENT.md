# MLOps Agent Operating Procedure

The MLOps agent stack coordinates automated training, evaluation, and promotion
for NOA ARK OS model lifecycles. This document describes how operators bootstrap
the orchestration runtime, monitor agent activity, and respond to drift gates.

## Architecture Overview

- **Agent Controller:** Implemented by `AgentLifecycleController` under
  `agents/src/implementations/ml_controller.rs`. It provisions specialist
  trainers through the `AgentFactory`, drives execution with the shared
  `RuntimeManager`, and ensures metric thresholds are enforced before
  promotion.
- **Runtime Orchestrator:** Provided by `ai/runtime`. The
  `TrainingOrchestrator` exposes a lifecycle API and emits
  [`LifecycleEvent`](../../ai/runtime/src/orchestrator.rs) notifications for
  downstream automation.
- **Storage Gateway:** Model binaries are staged inside
  `storage/artifacts/workspaces/` and promoted into
  `storage/artifacts/<lifecycle>/<profile>/` via the `FilesystemArtifactStore`.
  Verification summaries are appended to `storage/db/evidence/ledger.jsonl`.
- **Registry Integration:** The `CapabilityRegistry` ensures the
  `ml.lifecycle.controller` capability is registered and mirrors artifact
  metadata to `registry/ml_artifacts.log` for CI consumption.

## Bootstrap Steps

1. Instantiate the orchestrator using the helper:
   ```rust
   use ai_runtime::bootstrap_default_orchestrator;
   use agents::{AgentFactory, runtime::RuntimeManager};
   use std::sync::Arc;

   let factory = Arc::new(AgentFactory::new());
   let runtime = Arc::new(RuntimeManager::new());
   let orchestrator = bootstrap_default_orchestrator(
       factory,
       runtime,
       "storage/artifacts".into(),
       "storage/db/evidence/ledger.jsonl".into(),
       "registry/capabilities.json".into(),
       "ml.lifecycle.controller",
   );
   ```
2. Submit a `TrainingPlan` parsed from configuration (YAML/JSON) and await the
   pipeline report to confirm artifact promotion.
3. Subscribe to the broadcast channel via `orchestrator.subscribe()` to receive
   lifecycle events in dashboards or CLI tooling.

## Drift Detection and Rollout Gates

- CI jobs defined under `cicd/ml/` consume `registry/ml_artifacts.log` to detect
  performance drift and pause rollouts when thresholds regress.
- When drift is detected, the MLOps agent raises a `PromotionReady` event with a
  `status: blocked` annotation in the ledger entry. Operators must investigate
  the dataset diffs and retrain before re-enabling rollout.

## Incident Response

- **Training Failures:** Check workspace payloads under
  `storage/artifacts/workspaces/<agent_id>/` and review runtime logs emitted by
  the controller.
- **Threshold Violations:** Metrics are written alongside thresholds in the
  evidence ledger; ensure evaluation datasets match the declared plan and re-run
  verification before overriding.
- **Registry Conflicts:** If the capability manifest cannot be updated, validate
  `registry/capabilities.json` has writable permissions and re-run the
  orchestrator with administrative credentials.

## Audit Checklist

- [ ] Artifact manifest includes lifecycle identifier, checksum, and timestamp.
- [ ] Ledger entry recorded with verifier signature metadata.
- [ ] Registry log updated with evaluation metrics and promotion decision.
- [ ] Drift detection CI job executed on the latest artifact revision.
