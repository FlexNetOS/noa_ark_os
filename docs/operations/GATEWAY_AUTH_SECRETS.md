# Gateway Auth Secret Flow

This note captures how the gateway now sources authentication material from Vault and how to rotate
those secrets without downtime.

## Secret Layout

Store the entire authenticator config as a JSON document (see
`server/vault/gateway_auth.example.json`) and write it to Vault:

```bash
vault kv put secret/gateway/auth @server/vault/gateway_auth.example.json
```

Expected fields:

- `api_keys` – allowed automation keys.
- `mtls.allowed_fingerprints` – fingerprints for SAN-pinned certificates.
- `mtls.agent_san_prefix` / `mtls.control_plane_san` – SAN entries enforced during mTLS checks.
- `oidc.issuer` / `oidc.audience` / `oidc.hs256_secret` – OIDC validation contract.

## Runtime Wiring

1. Start Vault (see `server/vault/README.md`).
2. Export the following env vars before booting the gateway binary:

```bash
export VAULT_ADDR="http://127.0.0.1:8200"
export VAULT_TOKEN="$(vault login -token-only <approle-token>)"
export GATEWAY_VAULT_SECRET_PATH="secret/data/gateway/auth"
```

3. The gateway will fetch the JSON payload at startup. To operate offline, materialize the JSON to
`server/vault/runtime/gateway_auth.json` and point `GATEWAY_AUTH_CONFIG` to that path.

## Rotation Workflow

1. **Prepare new values** – update the JSON file with the new API key, fingerprint, or OIDC secret.
2. **Write to Vault** – `vault kv put secret/gateway/auth @gateway_auth.json`.
3. **Verify** – run `cargo test -p noa_gateway -- auth::` to execute the new rotation regression
   tests (`rejects_revoked_api_key`, `validates_oidc_claims`).
4. **Reload** – restart the gateway process or trigger a hot-reload hook so it re-reads the config.
5. **Revoke old material** – delete superseded keys/certificates from Vault once logs confirm no
   requests reference them.

## Evidence Trail

- Vault actions are already captured in its audit log.
- Gateway-level policy decisions referencing auth failures appear in
  `docs/verification/gateway_policy_audit.jsonl` once a request is denied.

Keeping the JSON contract in source control (example only) plus this workflow ensures we can rotate
credentials quickly while meeting the new roadmap requirements.
