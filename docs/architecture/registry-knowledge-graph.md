# Registry Knowledge Graph Service

## Overview

The registry knowledge graph captures a unified view of components, their
source files, cross-component dependencies, and accountable owners across the
NOA Ark OS codebase. Every entry inside `.workspace/registry/*.json` conforms
to the shared `registry.schema.json` contract and is loaded at runtime by the
core memory subsystem.

The ingestion pipeline performs the following steps:

1. **Document discovery** – the memory subsystem enumerates every JSON document
   under `.workspace/registry` (excluding the schema file) and deserializes the
   `components` and `owners` collections.
2. **Graph consolidation** – each component becomes a node in the in-memory
   graph while owner records become node attributes. The graph indexes
   dependencies and reverse-dependency edges, and tracks every referenced
   repository file.
3. **Validation** – structural invariants (duplicate IDs, missing owners,
   orphaned dependencies) are enforced before the graph is published to the
   rest of the kernel.
4. **Distribution** – consumers query the graph through `core::memory` helper
   APIs or via the file-system metadata synchronisation exposed by
   `core::fs`.

The registry exists to make it trivial to answer questions such as “which
components own this file?” or “who maintains the feature that depends on the
filesystem sync layer?”. All answers are backed by a consistent knowledge
source with well defined ownership information.

## Data Model

Each component definition includes:

- **id** – globally unique node identifier used for dependency edges.
- **version** – semantic version for the metadata definition.
- **files** – list of repository-relative paths covered by the component.
- **dependencies** – upstream component IDs.
- **owners** – owner IDs resolved against the shared owner directory.
- **tags/links/description** – optional metadata for discoverability.

Owners describe teams or individuals responsible for one or more components and
carry contact channels for escalation.

The memory subsystem converts the raw JSON documents into a
`RegistryGraph` structure containing:

- component map (`RegistryNode` entries by ID),
- owner map (`RegistryOwner` entries by ID),
- dependency adjacency (dependencies + dependents), and
- file index mapping file paths to component IDs.

## Runtime APIs

`core::memory` exposes lightweight helper functions to interact with the graph:

- `registry_snapshot()` – clone of the current `RegistryGraph`.
- `registry_component(id)` – single component lookup.
- `registry_dependencies(id)` / `registry_dependents(id)` – traversing the
  dependency edges in either direction.
- `registry_components_for_file(path)` – lookup by repository path.

The filesystem layer calls `registry_snapshot()` during initialisation and
translates component records into `FileDescriptor` metadata via
`sync_registry_metadata()`. Each descriptor now carries the owning components,
their versions, and owner display names, allowing downstream tooling to surface
context directly from the virtual filesystem API.

## Automation Hooks

`scripts/automation/validate_registry.py` acts as a portable guardrail that is
executed via `scripts/automation/run_all.sh`. The validator checks for duplicate
IDs, missing owners, semantic-version formatting, dependency resolution, and
file existence before new changes reach main. Integrate the shell wrapper into
pre-commit hooks or CI pipelines to prevent drift between code and metadata.

## Extending the Graph

1. Add or update JSON documents inside `.workspace/registry/` following the
   schema.
2. Run `scripts/automation/run_all.sh` (or the Python validator directly) to
   confirm schema compliance.
3. Add any new documentation links or owner entries to keep the knowledge graph
   accurate.
4. Commit the changes; runtime ingestion will pick up the new documents on the
   next boot.
