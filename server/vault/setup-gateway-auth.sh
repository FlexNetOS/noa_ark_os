#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
DATA_DIR="$SCRIPT_DIR/data"
LOG_DIR="$SCRIPT_DIR/logs"
CONF_FILE="$SCRIPT_DIR/vault.hcl"
AUTH_CONFIG="$SCRIPT_DIR/runtime/gateway_auth.json"

mkdir -p "$DATA_DIR" "$LOG_DIR"

echo "[vault-setup] Starting Vault initialization..."

# Set Vault address
export VAULT_ADDR="http://127.0.0.1:8200"

# Check if Vault is already initialized
if vault status >/dev/null 2>&1; then
    echo "[vault-setup] Vault is already running and initialized."
    echo "[vault-setup] Checking if gateway auth secrets are loaded..."

    # Check if secrets are already loaded
    if vault kv get secret/data/gateway/auth >/dev/null 2>&1; then
        echo "[vault-setup] Gateway auth secrets already loaded."
        exit 0
    fi
else
    echo "[vault-setup] Starting Vault server in background..."
    vault server -config="$CONF_FILE" >/dev/null 2>&1 &
    VAULT_PID=$!

    # Wait for Vault to start
    echo "[vault-setup] Waiting for Vault to start..."
    for i in {1..30}; do
        if vault status >/dev/null 2>&1; then
            break
        fi
        sleep 2
        if [ $i -eq 30 ]; then
            echo "[vault-setup] ERROR: Vault failed to start" >&2
            kill $VAULT_PID 2>/dev/null || true
            exit 1
        fi
    done

    echo "[vault-setup] Initializing Vault..."
    INIT_OUTPUT=$(vault operator init -key-shares=1 -key-threshold=1 -format=json)

    # Extract unseal key and root token
    UNSEAL_KEY=$(echo "$INIT_OUTPUT" | jq -r '.unseal_keys_b64[0]')
    ROOT_TOKEN=$(echo "$INIT_OUTPUT" | jq -r '.root_token')

    echo "[vault-setup] Unsealing Vault..."
    vault operator unseal "$UNSEAL_KEY"

    echo "[vault-setup] Authenticating with root token..."
    export VAULT_TOKEN="$ROOT_TOKEN"

    echo "[vault-setup] Enabling KV secrets engine..."
    vault secrets enable -path=secret kv-v2
fi

echo "[vault-setup] Loading gateway auth configuration..."
vault kv put secret/data/gateway/auth @"$AUTH_CONFIG"

echo "[vault-setup] Verifying gateway auth secrets..."
vault kv get secret/data/gateway/auth

echo "[vault-setup] Gateway auth secrets loaded successfully!"
echo "[vault-setup] VAULT_ADDR: $VAULT_ADDR"
echo "[vault-setup] VAULT_TOKEN: $VAULT_TOKEN"
echo "[vault-setup] Secret path: secret/data/gateway/auth"

# Save credentials for future use
cat > "$SCRIPT_DIR/.vault_credentials" << EOF
# Vault Credentials - Generated $(date)
# Keep this file secure and never commit to version control

VAULT_ADDR=$VAULT_ADDR
VAULT_TOKEN=$ROOT_TOKEN
GATEWAY_VAULT_SECRET_PATH=secret/data/gateway/auth

# To use these credentials:
# export VAULT_ADDR="$VAULT_ADDR"
# export VAULT_TOKEN="$ROOT_TOKEN"
# export GATEWAY_VAULT_SECRET_PATH="secret/data/gateway/auth"
EOF

chmod 600 "$SCRIPT_DIR/.vault_credentials"
echo "[vault-setup] Credentials saved to: $SCRIPT_DIR/.vault_credentials"

echo "[vault-setup] Setup complete! Vault is ready for gateway authentication."