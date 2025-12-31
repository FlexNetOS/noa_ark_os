#!/bin/bash
set -euo pipefail

# Ark OS Build Script
echo "🚀 Building Ark OS workspace..."

# Source Rust environment
source $HOME/.cargo/env

# Check system resources
echo "📊 System Resources:"
echo "CPU: $(nproc) cores"
echo "Memory: $(free -h | awk '/^Mem:/ {print $3 "/" $2}')"
echo "Disk: $(df -h . | awk 'NR==2 {print $5 " used"}')"
echo ""

# Build workspace
echo "🔨 Building workspace components..."
cargo build --workspace --release

# Build individual components for verification
echo "🔧 Building individual components..."
cargo build --package ark-os-core --release
cargo build --package ark-os-api-server --release
cargo build --package ark-os-python-bindings --release
cargo build --package ark-os-ml-engine --release
cargo build --package ark-os-database --release
cargo build --package ark-os-agent-orchestration --release

# Note: Tauri desktop app requires additional setup
echo "📱 Tauri desktop app build requires additional setup (Node.js, etc.)"
echo "   Use 'cargo build --package ark-os-desktop-app' after setting up frontend dependencies"

echo "✅ Build completed successfully!"
echo "📦 Built artifacts are in target/release/"
