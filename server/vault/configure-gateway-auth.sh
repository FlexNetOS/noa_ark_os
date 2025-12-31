#!/usr/bin/env bash
# Gateway Authentication Configuration Setup
# This script configures the gateway to use real secrets from either Vault or local config

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)
WORKSPACE_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
VAULT_DIR="$WORKSPACE_ROOT/server/vault"
AUTH_CONFIG_FILE="$VAULT_DIR/runtime/gateway_auth.json"

echo "🔐 Configuring Gateway Authentication..."
echo "========================================"

# Option 1: Use local config file (simpler, no Vault required)
if [ -f "$AUTH_CONFIG_FILE" ]; then
    echo "✅ Found local auth config: $AUTH_CONFIG_FILE"
    echo "📝 Setting GATEWAY_AUTH_CONFIG environment variable..."

    export GATEWAY_AUTH_CONFIG="$AUTH_CONFIG_FILE"
    echo "GATEWAY_AUTH_CONFIG=$GATEWAY_AUTH_CONFIG"

    # Test that the config is valid JSON
    if command -v jq >/dev/null 2>&1 && jq . "$AUTH_CONFIG_FILE" >/dev/null 2>&1; then
        echo "✅ Auth config is valid JSON"
    else
        echo "⚠️  jq not available or config validation skipped"
    fi
else
    echo "❌ Local auth config not found: $AUTH_CONFIG_FILE" >&2
    echo "💡 Run this script from the workspace root or create the config file first" >&2
    exit 1
fi

# Option 2: Use Vault (if available and configured)
if command -v vault >/dev/null 2>&1 && [ -n "${VAULT_ADDR:-}" ]; then
    echo ""
    echo "🔒 Vault detected, configuring Vault integration..."
    export GATEWAY_VAULT_SECRET_PATH="secret/data/gateway/auth"
    echo "GATEWAY_VAULT_SECRET_PATH=$GATEWAY_VAULT_SECRET_PATH"

    # Test Vault connectivity
    if vault status >/dev/null 2>&1; then
        echo "✅ Vault is accessible"

        # Check if gateway auth secrets exist in Vault
        if vault kv get "$GATEWAY_VAULT_SECRET_PATH" >/dev/null 2>&1; then
            echo "✅ Gateway auth secrets found in Vault"
            echo "🔄 Using Vault for gateway authentication"
        else
            echo "⚠️  Gateway auth secrets not found in Vault"
            echo "💡 Run './server/vault/setup-gateway-auth.sh' to load secrets into Vault"
        fi
    else
        echo "⚠️  Vault is not accessible"
        echo "💡 Make sure Vault is running and VAULT_ADDR/VAULT_TOKEN are set"
    fi
fi

echo ""
echo "🎯 Gateway Authentication Configuration Complete!"
echo "=================================================="
echo "The gateway will now use real secrets for authentication."
echo ""
echo "To run the gateway with this configuration:"
echo "  export GATEWAY_AUTH_CONFIG=\"$AUTH_CONFIG_FILE\""
if [ -n "${GATEWAY_VAULT_SECRET_PATH:-}" ]; then
    echo "  # OR use Vault:"
    echo "  export VAULT_ADDR=\"$VAULT_ADDR\""
    echo "  export VAULT_TOKEN=\"$VAULT_TOKEN\""
    echo "  export GATEWAY_VAULT_SECRET_PATH=\"$GATEWAY_VAULT_SECRET_PATH\""
fi
echo ""
echo "Test the configuration:"
echo "  cargo run --bin gateway -- --help"