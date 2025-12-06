# NOA ARK OS API Docs

This directory hosts generated API documentation for NOA ARK OS:

- Rust crate docs rendered via `rustdoc` for core crates (for example
  `noa_core`, `noa_agents`, `noa_crc`, `noa_gateway`, `noa_ui_api`, and
  others).
- The OpenAPI specification for the tooling APIs
  (`noa-tools.openapi.yaml`).

The top-level `index.html` page provides a human-friendly entrypoint that links
into the crate documentation.

## Regeneration

These artifacts are snapshots. When the Rust crates or APIs change, regenerate
the docs from the workspace root with a command such as:

```bash
cargo doc --workspace --no-deps --all-features
```

and copy or configure the output so that the crate documentation for the
relevant packages (for example `noa_core`, `noa_gateway`, `noa_ui_api`) is
available under `docs/api/`.

Keep this directory in sync with the active crate graph and OpenAPI schema so
that the UI and operators always have an accurate offline reference.

