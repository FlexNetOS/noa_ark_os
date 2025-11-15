# FlexNetOS Migration Skeleton (UPGRADED6)

**Timestamp:** 20251005-020711

Additions (upgrade-only):
- **WASI-cap file access** demo: host can preopen a whitelist dir at `/cap` using `FLEX_PREOPEN_DIR`; connector `readfile.wat` reads `/cap/hello.txt`.
- **Capability scopes (namespaced)**: cap tokens must include `connector:<name>` in `scopes` (e.g., `connector:readfile`).
- **fs-verity policy allowlist**: update & enforce measured digests for `artifacts/MANIFEST.sha256`.
