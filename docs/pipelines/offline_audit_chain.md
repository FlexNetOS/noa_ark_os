# Offline Audit Chain

The machine-first pipeline promoted by `make pipeline.local` now executes an offline reproducibility chain that can be replayed without network access. This document explains the flow and how to verify artifacts.

## Workflow Overview

1. **`make sbom`** – Invokes `python -m tools.repro.audit_pipeline sbom` to generate deterministic CycloneDX SBOMs for the kernel (`audit/SBOM.kernel.cdx.json`) and workspace extensions (`audit/SBOM.extensions.cdx.json`). The generator prefers `cargo metadata --locked` so the dependency graph is reproducible.
2. **`make package`** – Builds the Rust workspace in release mode and produces reproducible tarballs (`dist/noa-ark-os-*.tar.zst`) using `tar --zstd --sort=name --mtime=@$SOURCE_DATE_EPOCH --owner=0 --group=0 --numeric-owner` (requires the `zstd` binary). A consolidated manifest is written to `audit/artifacts.manifest.json`.
3. **`make sign`** – Seals the manifest and SBOMs using the offline HMAC signing shim. Signatures and fingerprints are emitted to `audit/signatures.json` alongside `*.sig` sidecars. For production, point `--signing-key` at a hardened key and replace `audit/offline-dev-hmac.key`.
4. **`make verify`** – Recomputes digests, checks HMAC signatures, and fails fast on any drift. This is the hook CI uses to block tampered bundles or mismatched hashes.
5. **`make scorekeeper`** – After verification passes, the trust score is written to `metrics/trust_score.json`. The default scorer returns `1.0` when SBOMs and signatures are valid, `0.0` otherwise.
6. **`make publish-audit`** – Packages an audit bundle in `audit/bundles/` and appends a ledger entry to `audit/ledger.jsonl`. Each entry captures the bundle hash, workspace snapshot ID, SBOM and artifact digests, and the computed trust score.

The Makefile’s `pipeline.local` target chains these steps so local runs match CI shims exactly.

## Verifying an Audit Bundle

1. Run `make verify` (or `python -m tools.repro.audit_pipeline verify`) to ensure the manifest, SBOMs, and signatures match the recorded digests.
2. Inspect `metrics/trust_score.json` for the calculated score and criteria.
3. Recreate the ledger entry locally:
   ```bash
   python -m tools.repro.audit_pipeline publish --audit-dir audit --dist-dir dist --metrics-dir metrics
   ```
   The command refuses to publish if verification fails, guaranteeing that recorded bundles match the verified state.
4. Validate the bundle hash recorded in `audit/ledger.jsonl` using `sha256sum audit/bundles/<bundle>.tar.zst`.

## CI Shims

GitHub Actions workflows delegate to `make pipeline.local`. Any deviation in SBOMs, tarball hashes, or signatures causes the `verify` step to fail, which in turn blocks the CI job. This ensures remote runners reproduce the offline artifacts byte-for-byte before publishing a new audit bundle.

## Key Management

* The repository ships with a deterministic development key at `audit/offline-dev-hmac.key`. Replace it with an environment- or hardware-backed key for production.
* To override the key, pass `--signing-key /path/to/key` when invoking the Python module or set `NOA_SIGNING_KEY` to a hex string before running `make sign` / `make verify`.

## Ledger Format

Each line in `audit/ledger.jsonl` is a JSON object:

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "snapshot_id": "<git-hash>",
  "bundle": {
    "path": "audit/bundles/audit-2024-01-01T00-00-00Z-deadbeef.tar.zst",
    "sha256": "<hash>",
    "size": 1234
  },
  "artifacts": [...],
  "sboms": [...],
  "trust_score": 1.0
}
```

Consumers replay ledger entries chronologically to verify provenance and detect drift.
