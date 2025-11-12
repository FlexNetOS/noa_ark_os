# Relocation daemon checkpoints

The relocation daemon persists operational checkpoints, pending approval queues,
and historical execution reports in this directory. Files are written as JSON
with timestamped filenames to keep an immutable audit trail. The
`relocation_state.json` file contains the latest in-flight state used by both
the CLI and HTTP API.
