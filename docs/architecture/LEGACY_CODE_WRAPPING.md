# Legacy Code Wrapping Strategy

This document defines how to retain dead / deprecated code that references past projects while preventing it from polluting active builds and triggering noisy diagnostics.

## Goals

- Preserve historical logic and structure for future re-integration.
- Eliminate accidental usage of stale code paths.
- Make revival plans explicit and searchable.
- Keep Rust build fast and warnings clean (`dead_code`, `unused_imports`).

## Core Patterns

### 1. Dedicated `legacy` Module Per Crate

Create a `legacy` module (directory) inside the crate root (e.g. `core/src/legacy/`). All archived code lives there.

Structure:

```
crate_root/
  src/
    legacy/
      mod.rs               # entry point
      project_x.rs         # isolated legacy unit
      project_y/
        mod.rs
```

### 2. Feature Gating (Optional Activation)

Add a `legacy` feature to crates that contain legacy code. Example in a crate `Cargo.toml`:

```toml
[features]
legacy = []
default = []
```

Then gate the module in `lib.rs`:

```rust
#[cfg(feature = "legacy")]
pub mod legacy;
```

No workspace-wide feature is required; enable per crate using:

```bash
cargo build -p core --features legacy
```

### 3. Suppressing Dead Code Warnings Intentionally

Inside `legacy/mod.rs` or per file:

```rust
#![allow(dead_code)]
#![allow(unused_imports)]
```

Do NOT apply these crate-wide. Keep the scope minimal.

### 4. Revival Metadata Header

Each legacy file begins with a structured comment:

```rust
// LEGACY BLOCK
// Origin: project-x (commit abc123)
// Reason: Deferred during migration phase 2.
// Dependencies: metrics, gateway
// Revival Plan: Refactor into async task scheduler (Q1 2026)
// Status: Archived
```

### 5. Wrapper Macro (Simple Tagging)

If you need lightweight tagging without a feature gate, use the `legacy_item!` macro defined once in `legacy/mod.rs`:

```rust
macro_rules! legacy_item {
    ($item:item) => {
        #[allow(dead_code)]
        #[allow(unused_imports)]
        $item
    };
}
```

Usage:

```rust
legacy_item! {
    pub fn old_scheduler_logic() {
        // ... preserved algorithm ...
    }
}
```

### 6. Explicit Dependency Isolation

Avoid importing from `legacy` into active modules. The dependency direction should be one-way: active code may call a _revived_ item only after moving it out of `legacy`.

### 7. Tracking and Audit

List all legacy units in `docs/architecture/LEGACY_INVENTORY.md` (create when needed). Columns:
`module | file | origin | revival_target | status | notes`

### 8. Removal Workflow

When reviving:

1. Copy or move code out of `legacy`.
2. Remove allowances and ensure it compiles cleanly.
3. Delete original legacy file after migration.
4. Update inventory status to `revived`.

## rust-analyzer Diagnostics Mapping

In `.vscode/settings.json` or user settings, configure more granular hints instead of global suppression:

```json
"rust-analyzer.diagnostics.warningsAsHint": ["unused-imports"],
"rust-analyzer.diagnostics.warningsAsInfo": ["dead_code"]
```

This keeps legacy warnings visible but lower severity.

## Example Layout Added in `core`

See `core/src/legacy/mod.rs` for the starter scaffold.

## Next Steps

- Add `legacy` feature flags only to crates that actually need them.
- Populate `LEGACY_INVENTORY.md` once first legacy blocks are moved.
- Keep this doc updated when policy changes.
