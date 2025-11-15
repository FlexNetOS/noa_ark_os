# Archived VS Code Settings Bundle

The `.vscode/` directory that previously lived at the workspace root is stored
as the Base64-encoded tarball `.vscode.tar.b64`. This mirrors the approach used
for the devcontainer bundle so GitHub can render diffs while we retain a
restorable snapshot of every IDE task and setting.

## Restoring the archive

Use the development environment CLI to rebuild the tarball or extract the
original files when you need to validate parity with the CLI workflows:

```bash
# Materialize the tarball under out/.vscode.tar
python server/tools/dev_env_cli.py archives restore --bundle vscode --mode tar

# Extract the archived files into ./out/.vscode
python server/tools/dev_env_cli.py archives restore --bundle vscode --mode extract
```

Supply `--output` to choose a different path. For example, pass
`--mode extract --output .vscode` when you must temporarily reload the legacy VS
Code workspace configuration. Remove the directory again once validation is
complete.

## Integrity

The ledger entry in `archive/2025/11/ledger.json` records the SHA-256 checksum
for `.vscode.tar.b64` and documents that it replaces the live `.vscode/`
directory. Run `python server/tools/dev_env_cli.py archives list` to review the
metadata before restoring a bundle.
