# Registry Onboarding SOP

## Purpose

This SOP helps new contributors adopt the registry knowledge graph. It clarifies
what to document, how to validate entries, and how the runtime consumes the
metadata.

## Prerequisites

- Rust toolchain installed for building the core crate.
- Python 3.8+ available to run the automation scripts.
- Familiarity with the component(s) you plan to document.

## Steps

1. **Discover the schema**
   - Review `.workspace/registry/registry.schema.json` for the required fields.
   - Inspect existing documents such as `core.json` for idiomatic examples.

2. **Author metadata**
   - Create or update a JSON file inside `.workspace/registry/`.
   - Ensure each component lists the repository files it owns and any upstream
     component IDs in `dependencies`.
   - Reference owner IDs already defined in the same or other documents. Add new
     owners if necessary.

3. **Validate locally**
   - Run `scripts/automation/run_all.sh` to execute all registry checks.
   - Resolve reported issues before committing (duplicate IDs, missing files,
     invalid versions, etc.).

4. **Regenerate context**
   - Build or test the core crate (`cargo test -p noa_core`) to exercise the
     runtime ingestion path if desired.
   - Confirm the documentation in `docs/architecture/registry-knowledge-graph.md`
     reflects any significant conceptual changes.

5. **Submit changes**
   - Commit both the JSON updates and any associated code or docs.
   - Highlight the registry changes in the PR summary so reviewers can focus on
     validation and ownership alignment.

## Tips

- Keep component IDs stable; they act as graph node identifiers and appear in
  dependency edges.
- Leverage the `links` map to add dashboards, runbooks, or issue tracker URLs
  for quick follow-up.
- If a file belongs to multiple components, list it in each component's `files`
  array—runtime ingestion will merge the ownership metadata in the VFS layer.
