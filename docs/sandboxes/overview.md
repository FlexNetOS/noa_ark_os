# Sandbox Validation Overview

Sandbox environments provide safe, isolated deployment targets for validating workflow output before promoting changes downstream.

* **Trigger validations** from the AI Ops Studio by running commands like `validate sandbox`. The unified workflow API schedules the associated pipelines.
* **Observe telemetry** in the Workflow Command Center module, where validation stages emit live status and logs via WebSocket events.
* **Promote with confidence** once automated checks pass. The CI/CD Console enables fast handoffs into blue/green or canary rollout strategies.
