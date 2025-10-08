#!/bin/bash
set -euo pipefail

# ARK-OS Build Script
# Builds all components in the Cargo workspace

echo "=== ARK-OS Build Script ==="
echo "Starting build process at $(date)"

# Source Rust environment
source ~/.cargo/env

# Check Rust installation
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Set build configuration
BUILD_MODE="${1:-debug}"
echo "Build mode: $BUILD_MODE"

# Build workspace
echo ""
echo "Building Cargo workspace..."
if [ "$BUILD_MODE" = "release" ]; then
    cargo build --workspace --release
else
    cargo build --workspace
fi

echo ""
echo "Build completed successfully!"

# Build individual components for verification
echo ""
echo "Verifying individual component builds..."

COMPONENTS=(
    "ark-os-core"
    "rustpython-example"
    "pyo3-bindings"
    "axum-api"
    "candle-ml"
    "burn-training"
    "rig-orchestration"
    "qdrant-client"
    "postgres-client"
    "redis-client"
    "tauri-app"
)

for component in "${COMPONENTS[@]}"; do
    echo "Building $component..."
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --package "$component" --release
    else
        cargo build --package "$component"
    fi
done

echo ""
echo "All components built successfully!"

# Generate build artifacts summary
echo ""
echo "Build artifacts:"
if [ "$BUILD_MODE" = "release" ]; then
    find target/release -name "*.rlib" -o -name "*.so" -o -name "*.dylib" -o -name "*.dll" | head -20
    echo "Binaries:"
    find target/release -type f -executable | grep -E "(axum-server|tauri-app)" || true
else
    find target/debug -name "*.rlib" -o -name "*.so" -o -name "*.dylib" -o -name "*.dll" | head -20
    echo "Binaries:"
    find target/debug -type f -executable | grep -E "(axum-server|tauri-app)" || true
fi

echo ""
echo "Build script completed at $(date)"
echo "=== Build Summary ==="
echo "Mode: $BUILD_MODE"
echo "Components: ${#COMPONENTS[@]}"
echo "Status: SUCCESS"
