# FlexNetOS Migration Skeleton (UPGRADED4)

**Timestamp:** 20251005-015423

New:
- **Minisign enforcement**: `FLEX_MINISIGN_PUB` + `.minisig` on `artifacts/MANIFEST.sha256`.
- **Sealed manifest**: `make seal-manifest` (immutable) and `make fs-verity-enable`/`fs-verity-verify` if supported. Server can enforce with `FLEX_ENFORCE_SEAL=1`.
- **Cap'n Proto Python client**: `tools/capnp_python_client.py` (uses `pycapnp` if installed).
- **Next-actions recommender** runs at the end of key targets.
- **Agent Map cross-walk** (`docs/agent_map_crosswalk.md`) binding agents → orchestrator/exec components.
