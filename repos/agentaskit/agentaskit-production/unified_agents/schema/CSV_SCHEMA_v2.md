# CSV Schema v2.0 (Extended Metadata)

*Status: draft — backwards compatible with v1*

The original CSV schema (v1.0) mandates a minimal set of columns
(`agent_name`, `role`, `layer`, `scope`, `tools`, `inputs`, `outputs`,
`guardrails`, `escalation_to`, `stack`) and ignores any additional
columns【928610657846884†L2-L8】.  Version 2.0 retains those mandatory
columns but allows a richer set of optional fields that align with
the columns present in the **All Inclusive Agent Directory v6+** file.
Adding these fields does **not** break existing tooling — unrecognised
columns are ignored by older normalisers — but enables more detailed
manifests when processed by `normalize_csv_v2.py`.

## Mandatory fields (unchanged)

```
agent_name, role, layer, scope, tools, inputs, outputs,
guardrails, escalation_to, stack
```

## Additional optional fields

Below is the full list of additional columns recognised by
`normalize_csv_v2.py`.  Fields may be left blank.  Lists should be
semicolon‑, comma‑ or pipe‑separated; the normaliser will convert
them into arrays internally.

```
agent_code, agent_id, display_name, aka, type,
category, subcategory, epic, operations_domain,
owner_role, raci_r, raci_a, raci_c, raci_i,
human_approval_required, escalation_policy, court_gate,
risk_class, purpose, capabilities, capability_pack,
plane, triggers, artifacts, actions, tools_stack,
data_sources, connectors, endpoints, commands,
code_paths, manifests, policies, models,
embedding_models, memory_scope, state_store, cache,
scheduling, sla, metrics, telemetry,
dependencies, depends_on_agents, provides_capabilities,
failure_modes, auto_remediation, audit_logging,
pii_handling, security_level, export_control,
license_category, maturity_level, autonomy_level,
operational_readiness_score, status, version,
created_at, last_updated, source_file,
source_row_index, source_rows_json, governance_role,
scheduler_owner, spawn_policy, court_policy_id,
capability_pack_id, efg_requirements, cost_center,
budget_cap, telemetry_topic
```

### Layer enumeration

The `layer` column must still map into one of the six canonical
hierarchy levels: `cecca` → `board` → `executive` → `stack_chief`
→ `specialist` → `micro`【254663532232176†L56-L57】.  Free‑form values in
`layer` or `role` will be normalised via heuristic matching
(`micro_agent`, `orchestrator`, `vp`, etc.).  Missing `role` values
will default based on the derived layer.

### Compatibility

Older tooling that only understands v1.0 can safely ignore these
additional columns.  The `normalize_csv_v2.py` script will output
all present columns in snake‑case order, ensuring both mandatory and
optional fields are available for downstream processing.