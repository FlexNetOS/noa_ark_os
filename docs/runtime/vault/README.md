# Vault Runtime Contract

This document describes how the `server/vault` assets consume gateway-managed
configuration so local developers and CI runners share the same layout.

## NOA_VAULT_HOME

- **Purpose:** Anchors the Vault data directory, Raft state, and runtime files
  (such as `runtime/gateway_auth.json`).
- **Source of truth:** `.workspace/registry/environment.vault.json` documents the
  variable and default fallback.
- **Default:** `~/.noa/vault` when the variable is not set.
- **Usage:**
  - `server/vault/vault.hcl` resolves the Raft path via `{{env "NOA_VAULT_HOME"}}/data`.
  - `server/vault/configure-gateway-auth-simple.sh` exports `NOA_VAULT_HOME`,
    seeds `$NOA_VAULT_HOME/runtime/gateway_auth.json`, and sets
    `GATEWAY_AUTH_CONFIG`.

Override the location by exporting `NOA_VAULT_HOME=/custom/path` before running
Vault scripts or CLI commands. Ensure `data` and `runtime` subdirectories exist.

## Bootstrap Checklist

1. `export NOA_VAULT_HOME=${NOA_VAULT_HOME:-"$HOME/.noa/vault"}`
2. `mkdir -p "$NOA_VAULT_HOME/data" "$NOA_VAULT_HOME/runtime"`
3. `./server/vault/configure-gateway-auth-simple.sh` (seeds runtime auth file)
4. `./server/vault/start.sh` (or `vault server -config=server/vault/vault.hcl`)

Recording this contract in the documentation + registry keeps every gateway and
operator aligned without relying on hard-coded absolute paths.
