# Rustup Portable Settings

## Purpose

Provide a normalized path for the portable rustup configuration usable in Linux codespaces and Windows portable environments.

## File

`settings.toml` (current version = 12)

## Overrides

Add per-target or toolchain overrides under the `[overrides]` section, e.g.

```toml
[overrides]
"wasm32-unknown-unknown" = { profile = "minimal" }
```

## Git Add

Use the relative path when staging:

```bash
git add server/tools/rustup-portable/settings.toml
```

Do NOT use Windows drive prefixes inside the Linux container (e.g. `D:/dev/...`).

## Rust-Analyzer Diagnostics

If you saw errors like:

```
invalid config values:
/diagnostics/warningsAsHint: invalid type: boolean `true`, expected a string;
/diagnostics/warningsAsInfo: invalid type: boolean `true`, expected a string;
```

Use the CLI diagnostics helper instead of editing IDE settings directly:

```bash
python server/tools/dev_env_cli.py diagnostics
```

If you still need custom overrides, add them to `settings.toml` under `[diagnostics]`.
