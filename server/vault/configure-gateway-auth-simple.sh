#!/usr/bin/env bash
# Gateway Authentication Configuration Setup
# This script configures the gateway to use real secrets

echo "🔐 Configuring Gateway Authentication..."
echo "========================================"

# Resolve base Vault home (defaults to ~/.noa/vault) and allow overrides via NOA_VAULT_HOME
NOA_VAULT_HOME="${NOA_VAULT_HOME:-"$HOME/.noa/vault"}"
export NOA_VAULT_HOME

echo "📁 Using Vault home: $NOA_VAULT_HOME"
echo "   (override by exporting NOA_VAULT_HOME=/custom/path before running this script)"

# Set the path to the auth config file under the managed Vault home
AUTH_CONFIG_FILE="$NOA_VAULT_HOME/runtime/gateway_auth.json"
REPO_FALLBACK_AUTH_CONFIG="$(dirname "$0")/runtime/gateway_auth.json"

# Seed the managed runtime file from the repository copy when needed
if [ ! -f "$AUTH_CONFIG_FILE" ] && [ -f "$REPO_FALLBACK_AUTH_CONFIG" ]; then
    mkdir -p "$(dirname "$AUTH_CONFIG_FILE")"
    cp "$REPO_FALLBACK_AUTH_CONFIG" "$AUTH_CONFIG_FILE"
    echo "📄 Seeded $AUTH_CONFIG_FILE from $REPO_FALLBACK_AUTH_CONFIG"
fi

if [ -f "$AUTH_CONFIG_FILE" ]; then
    echo "✅ Found local auth config: $AUTH_CONFIG_FILE"
    echo "📝 Setting GATEWAY_AUTH_CONFIG environment variable..."

    export GATEWAY_AUTH_CONFIG="$AUTH_CONFIG_FILE"
    echo "GATEWAY_AUTH_CONFIG=$GATEWAY_AUTH_CONFIG"

    echo "✅ Auth config is ready"
else
    echo "❌ Local auth config not found: $AUTH_CONFIG_FILE" >&2
    exit 1
fi

echo ""
echo "🎯 Gateway Authentication Configuration Complete!"
echo "=================================================="
echo "The gateway will now use real secrets for authentication."
echo ""
echo "To run the gateway with this configuration:"
echo "  export GATEWAY_AUTH_CONFIG=\"$AUTH_CONFIG_FILE\""
echo ""
echo "Test the configuration:"
echo "  cargo run --bin gateway -- --help"