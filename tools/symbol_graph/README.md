# Symbol Graph Builder

`noa_symbol_graph` provides an offline Tree-sitter powered indexer for NOA Ark OS
repositories. It extracts semantic nodes (functions, structs, classes) from Rust
and TypeScript sources, generates stable symbol IDs using
`core::symbols::stable_symbol_id`, and persists the resulting node/edge tables to
`.workspace/indexes/symbol_graph/`.

## Usage

```rust
use noa_symbol_graph::SymbolGraphBuilder;

let graph = SymbolGraphBuilder::new(".").index()?;
for node in graph.nodes.values() {
    println!("{} -> {}", node.name, node.stable_id);
}
```

The store automatically performs incremental updates: existing entries are
merged with new scans and edges are de-duplicated. Tests cover stable IDs across
file moves to guarantee consistent references for refactoring tools.

## Notebook metadata watcher

Run the notebook-aware watcher to stream symbol changes into
`.workspace/notebook_sync/diffs/`:

```bash
cargo run -p noa_symbol_graph --bin notebook_watcher -- --once .
```

Omit `--once` to keep the watcher running. Each diff file is a JSON payload the
notebook automation agent consumes to refresh headers, strip outputs, and emit
analytics.
