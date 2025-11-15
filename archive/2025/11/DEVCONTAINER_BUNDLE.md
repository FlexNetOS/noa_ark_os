# Archived Devcontainer Bundle

This bundle captures the `.devcontainer/` directory that existed before the
CLI-based tooling replaced IDE-managed tasks. Binary blobs are no longer stored
in the repository; instead, the tarball payload is Base64 encoded as
`.devcontainer.tar.b64` so GitHub can render and diff the contents while keeping
our archival ledger intact.

## Restoring the archive

Use the development environment CLI to rebuild the tarball or extract the
original files:

```bash
# Materialize the tarball under out/.devcontainer.tar
python server/tools/dev_env_cli.py archives restore --bundle devcontainer --mode tar

# Extract the archived files into ./out/.devcontainer
python server/tools/dev_env_cli.py archives restore --bundle devcontainer --mode extract
```

Pass a custom `--output` path to control where the tarball or extracted
directory is written. Use `--output .devcontainer` with `--mode extract` if you
need to temporarily re-enable the legacy VS Code configuration, then delete the
folder again after verification.

## Integrity

The ledger entry in `archive/2025/11/ledger.json` tracks the SHA-256 checksum of
the Base64 asset and records that it replaces the live `.devcontainer/`
directory. Use `python server/tools/dev_env_cli.py archives list` to review the
metadata before restoring a bundle.
