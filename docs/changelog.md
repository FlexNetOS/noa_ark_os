# NOA Ark OS Changelog

## 2025-11-15 — Normalize Archived IDE Bundles

### Summary
- Converted the archived `.devcontainer` payload into a Base64-encoded tarball to remove binary blobs from Git storage.
- Converted the archived `.vscode` payload into a Base64-encoded tarball so both IDE bundles comply with repository policies.
- Added CLI helpers for listing and restoring archive bundles so the legacy configuration can still be auto-loaded when needed.
- Documented the new archive format and restoration workflow in `archive/2025/11/DEVCONTAINER_BUNDLE.md`.
- Documented the `.vscode` archive restoration workflow in `archive/2025/11/VSCODE_BUNDLE.md`.

### Impact
- Keeps the repository compliant with GitHub's binary storage policies while preserving reversible history.
- Developers can regenerate tarballs or extract the legacy `.devcontainer/` directory with a single CLI command.
- Developers can likewise restore the legacy `.vscode/` directory alongside the devcontainer assets when necessary.
- Archival documentation now points directly at the new text-based asset for auditability.

## 2025-11-14 — Standardize Workspace Configuration on CLI

### Summary
- Introduced `server/tools/dev_env_cli.py` to manage multi-platform activation, diagnostics, and summaries.
- Updated all portable tooling guides to reference the CLI instead of `.vscode`-scoped instructions.
- Published a roadmap checklist for migrating IDE configurations and tracking archival gates.
- Prepared `.vscode` and `.devcontainer` tarballs per archival policy and logged their metadata.

### Impact
- Developers follow a consistent CLI-first workflow across Windows, WSL, and Linux.
- Documentation is decoupled from editor-specific settings, reducing maintenance overhead.
- Archival readiness is documented, enabling future removal of IDE assets once remaining gates close.
