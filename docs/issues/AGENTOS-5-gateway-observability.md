# AGENTOS-5 — Develop advanced gateway with observability

## Description
Modernize the gateway to deliver policy enforcement, traffic shaping, and deep observability linked to platform telemetry expectations.

## Checklist
- [ ] Define ingress, policy, and observability requirements aligned with kernel-first services.
- [ ] Implement gateway routing, authentication, and rate-limiting features with tests.
- [ ] Integrate telemetry export (metrics, traces, logs) into the observability pipeline.
- [ ] Provide dashboards and alert templates for gateway health.
- [ ] Document operator runbooks for incident response scenarios.

## Acceptance criteria
- Gateway enforces policy and routing rules validated by automated integration tests.
- Observability data streams into the standard telemetry stack with dashboards for latency, errors, and throughput.
- Runbooks detail detection and mitigation steps for at least three incident classes.
- Gateway configuration references AGENTOS-2 baseline without duplicating definitions.

## Meta
- Owner: TBA
- Priority: P1
- Status: Proposed
- Depends on: AGENTOS-2
- Watchers: @gateway-team
- Linked workstreams:
  - [Gateway services](../../services/gateway)
  - [Telemetry dashboards](../../services/observability)

## Coordination
- Work with observability owners to align dashboards and replace the roadmap anchor when an external issue URL becomes available.
