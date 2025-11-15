# Notebook Sync Automation Agent

The notebook sync automation consumes symbol graph diffs emitted by
`tools/symbol_graph` and keeps notebooks reproducible by refreshing
metadata, citations, and analytics after every code change.

## Responsibilities

- Load metadata diff batches from `.workspace/notebook_sync/diffs`.
- Update notebook-level metadata (`metadata.noa`) with the latest symbol
  changes and citation stubs.
- Strip outputs and execution counters from every cell to keep commits
  deterministic.
- Append analytics runs to `metrics/notebook_analytics.json` so local and
  automated pipelines share the same history.
- Archive processed diff files under
  `.workspace/notebook_sync/diffs/processed` for auditability.

## Running locally

The agent is exposed through a CLI so developers can replay notebook
updates offline:

```bash
make notebooks-sync
```

This target rebuilds the symbol graph, generates diff batches (if any),
invokes the automation agent, and leaves the processed diffs archived.
