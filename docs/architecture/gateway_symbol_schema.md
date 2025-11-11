# Gateway Symbol Schema Contract

The gateway normalizes every connector (API, plugin, feature flag, etc.) into a shared contract so
routing, verification, and compliance checks can be automated. This document captures the canonical
schema that powers the Rust catalog in `core/src/gateway.rs`.

## Schema Fields

| Field | Type | Description |
| --- | --- | --- |
| `schema_id` | String | Stable identifier for the schema entry (`<domain>.<capability>.<kind>`). |
| `kind` | `SymbolKind` | Normalized connector class (e.g., `api`, `plugin`, `feature_flag`). Custom kinds are supported via `SymbolKind::Custom`. |
| `version` | SemVer string | Schema evolution marker; changes must be additive and backward compatible unless lifecycle is `deprecated`/`retired`. |
| `capability_taxonomy` | Set<String> | Capabilities exposed to routing (e.g., `stream`, `analytics`). Used for capability matching and coverage scoring. |
| `lifecycle` | `LifecycleStage` | Operational status: `prototype`, `active`, `deprecated`, or `retired`. |
| `recommended_zones` | Set<String> | Preferred deployment or execution zones (e.g., `edge`, `global`). Enforced against intent constraints. |
| `compliance_tags` | Set<String> | Compliance or certification tags (e.g., `pii_safe`, `audited`). Useful for guardrails and audit evidence. |
| `compatibility` | List<`CompatibilityWindow`> | Declares compatible peer connectors. Each window binds another `SymbolKind` to a supported version range. |
| `schema_hash` | String | Content-derived hash that connectors reference to prove schema alignment. |

## Lifecycle Governance

- `prototype`: Experimental. Requires explicit opt-in and cannot be default routed.
- `active`: Fully supported. Gateways will consider these schemas during automatic routing.
- `deprecated`: Functional but scheduled for removal. Intents receive warnings and lower health
  weighting.
- `retired`: No longer accepted. Connectors referencing retired schemas are rejected.

## Catalog Operations

The gateway exposes a registry API:

- `register_schema(schema)` – Adds a new schema, rejecting conflicting hashes.
- `ingest_schema_catalog(schemas)` – Bulk loads schema packs (used during bootstrap).
- `catalog_snapshot()` – Produces counts per lifecycle stage and capability indexes for reporting.

## Connector Requirements

Connectors call `register_symbol` with metadata that must align with a registered schema:

1. `schema_hash` must match the schema's `schema_hash` field.
2. `kind` and `version` must align with the schema definition.
3. Declared capabilities must be a subset of `capability_taxonomy`.

Violations fail fast with `GatewayError::PolicyViolation`, protecting the topology from drift.

## Intent Compilation Interface

Automation layers describe routing intents in YAML. Example:

```yaml
intents:
  - name: replicate analytics stream
    target: api
    capabilities: ["stream", "analytics"]
    constraints:
      max_latency_ms: 20
      min_trust_score: 0.7
      encryption: true
      zones: ["global"]
```

`IntentCompiler::compile` converts the manifest into strongly typed `Intent` instances with
constrained defaults (latency ≤ 250ms, trust ≥ 0.6, encrypted, global zone) so higher layers can
avoid manual boilerplate.

## Telemetry Feed

Every registry, scan, route, and self-heal action emits `TelemetryEvent`s. Consumers should drain
telemetry via `Gateway::drain_telemetry` and forward it to observability pipelines. Events include
timestamp, kind (`SchemaRegistered`, `ConnectorRegistered`, `ScanCompleted`, `RouteCompiled`,
`SelfHealSuggested`), and free-form context strings that reference connector IDs or counts.

By codifying this schema contract the gateway delivers the ontology, automation, and observability
pillars of the roadmap while staying auditable and predictable.
