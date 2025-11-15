# CLI Configuration Migration Checklist

**Status Date:** 2025-11-14
**Owners:** Workspace Tooling Guild

---

## 🎯 Objective

Track the transition from IDE-coupled configuration to the repository-standard CLI workflows.
This checklist consolidates all stubs from the portable tooling docs and records the
verification gates required before the legacy `.vscode/` and `.devcontainer/` directories can
be relocated into `archive/`.

---

## ✅ Completed Items

- [x] `server/tools/dev_env_cli.py` provides platform activation guidance and diagnostics.
- [x] `server/tools/README.md`, `QUICK_START.md`, and `SETTINGS_SUMMARY.md` reference the CLI instead of `.vscode` profiles.
- [x] `server/tools/MULTI_PLATFORM.md` documents CLI-first workflows across Windows, WSL, and Linux.
- [x] `server/tools/rustup-portable/README.md` routes rust-analyzer troubleshooting through the CLI.
- [x] Archive tarballs for `.vscode/` and `.devcontainer/` converted to Base64-encoded bundles (pending relocation).

---

## ⏳ Outstanding Tasks

- [ ] Run `python server/tools/dev_env_cli.py doctor` on Windows, WSL, and native Linux hosts and record the outputs in `storage/db/evidence/ledger.jsonl`.
- [ ] Capture automated smoke tests that invoke the CLI activation commands within CI.
- [ ] Publish shell profile snippets for macOS PowerShell and zsh users in `server/tools/MULTI_PLATFORM.md`.
- [ ] Secure sign-off from the IDE Extension maintainers confirming no remaining dependency on workspace `.vscode` files.

---

## 🚦 Archival Gate for IDE Directories

Move `.vscode/` and `.devcontainer/` into `archive/` only after all outstanding tasks above are checked.
When relocation is approved:

1. Update this checklist with the completion date and approver initials.
2. Run `python server/tools/dev_env_cli.py summary --format json` and attach the output to the Evidence Ledger.
3. Compress the directories via `tar --zstd -cf archive/YYYY/MM/<name>.tar.zst <name>` (already staged for 2025-11) and
   confirm SHA-256 hashes are recorded in `archive/2025/11/ledger.json`.
4. Remove live references to `.vscode/` and `.devcontainer/` from any remaining documentation.

---

## 🔁 Review Cadence

- **Weekly:** Workspace Tooling Guild reviews outstanding tasks.
- **Monthly:** Update this checklist with new findings or additional gates discovered by CI.
- **Quarterly:** Confirm archived tarballs still decompress successfully and the CLI remains in parity with prior IDE flows.

---

**Contact:** tooling@noa-ark-os.example.com
