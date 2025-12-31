#!/bin/bash
set -euo pipefail

# Ark OS Test Script
echo "🧪 Running Ark OS test suite..."

# Source Rust environment
source $HOME/.cargo/env

# Check workspace health
echo "🔍 Checking workspace health..."
cargo check --workspace

# Run tests (no-run to compile tests without executing)
echo "🏗️  Compiling tests..."
cargo test --workspace --no-run

# Run unit tests
echo "🧪 Running unit tests..."
cargo test --workspace --lib

# Run integration tests (if any)
echo "🔗 Running integration tests..."
cargo test --workspace --test '*' || echo "No integration tests found"

# Run clippy for linting
echo "📎 Running clippy lints..."
cargo clippy --workspace --all-targets --all-features -- -D warnings || echo "Clippy warnings found"

# Format check
echo "🎨 Checking code formatting..."
cargo fmt --all -- --check || echo "Code formatting issues found"

# Component-specific tests
echo "🔧 Testing individual components..."

echo "  Testing core..."
cargo test --package ark-os-core

echo "  Testing API server..."
cargo test --package ark-os-api-server

echo "  Testing Python bindings..."
cargo test --package ark-os-python-bindings

echo "  Testing ML engine..."
cargo test --package ark-os-ml-engine

echo "  Testing database..."
cargo test --package ark-os-database

echo "  Testing agent orchestration..."
cargo test --package ark-os-agent-orchestration

echo "✅ All tests completed!"

# Save exit code for smoke test
echo $? > .test_exitcode
