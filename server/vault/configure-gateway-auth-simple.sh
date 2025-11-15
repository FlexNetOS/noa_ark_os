#!/usr/bin/env bash
# Gateway Authentication Configuration Setup
# This script configures the gateway to use real secrets

echo "🔐 Configuring Gateway Authentication..."
echo "========================================"

# Set the path to the auth config file
AUTH_CONFIG_FILE="/home/noa/dev/workspace/noa_ark_os/noa_ark_os/server/vault/runtime/gateway_auth.json"

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