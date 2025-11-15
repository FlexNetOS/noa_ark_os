#!/bin/bash
set -euo pipefail

# ARK-OS Setup and Verification Script
# Complete setup and verification of the ARK-OS ecosystem

echo "=== ARK-OS Setup and Verification Script ==="
echo "Starting setup process at $(date)"

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    log "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    log "Rust already installed: $(rustc --version)"
    source ~/.cargo/env
fi

# Verify Rust installation
log "Verifying Rust installation..."
log "Rust version: $(rustc --version)"
log "Cargo version: $(cargo --version)"

# Check workspace structure
log ""
log "Verifying workspace structure..."
if [ ! -f "Cargo.toml" ]; then
    log "ERROR: Cargo.toml not found in current directory"
    exit 1
fi

# List workspace members
log "Workspace members:"
grep -A 20 "members = \[" Cargo.toml | grep -E '^\s*"' | sed 's/[",]//g' | sed 's/^[[:space:]]*/  - /'

# Create necessary directories
log ""
log "Creating necessary directories..."
mkdir -p logs target docs

# Run build script
log ""
log "Running build script..."
chmod +x scripts/build.sh
./scripts/build.sh debug

# Run test script
log ""
log "Running test script..."
chmod +x scripts/test.sh
./scripts/test.sh no-run

# Run smoke tests
log ""
log "Running smoke tests..."
chmod +x scripts/smoke.sh
./scripts/smoke.sh

# Generate project documentation
log ""
log "Generating project documentation..."
cargo doc --workspace --no-deps

# Create environment file
log ""
log "Creating environment configuration..."
cat > .env << EOF
# ARK-OS Environment Configuration
RUST_LOG=info
RUST_BACKTRACE=1

# Database configurations
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=postgres
POSTGRES_USER=postgres

REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_DB=0

QDRANT_URL=http://127.0.0.1:6333

# Application settings
ARK_OS_VERSION=0.1.0
ARK_OS_ENV=development
EOF

# Generate project summary
log ""
log "Generating project summary..."
cat > PROJECT_SUMMARY.md << EOF
# ARK-OS Project Summary

## Overview
ARK-OS is a comprehensive Rust ecosystem integrating multiple AI/ML and system components.

## Components
- **ark-os-core**: Core library and utilities
- **rustpython-example**: Python interpreter integration
- **pyo3-bindings**: Python-Rust bindings
- **axum-api**: Web API server
- **candle-ml**: ML inference engine
- **burn-training**: ML training framework
- **rig-orchestration**: Agent orchestration
- **qdrant-client**: Vector database client
- **postgres-client**: PostgreSQL database client
- **redis-client**: Redis cache client
- **tauri-app**: Desktop application

## Quick Start
1. Build: \`./scripts/build.sh\`
2. Test: \`./scripts/test.sh\`
3. Smoke test: \`./scripts/smoke.sh\`

## Generated at
$(date)
EOF

# System information
log ""
log "System information:"
log "OS: $(uname -a)"
log "CPU cores: $(nproc)"
log "Memory: $(free -h | awk '/^Mem:/ {print $3 "/" $2}')"
log "Disk usage: $(df -h . | awk 'NR==2 {print $5 " used"}')"

# Final verification
log ""
log "Final verification..."
if cargo check --workspace &> logs/final_check.log; then
    log "✓ Final workspace check PASSED"
else
    log "✗ Final workspace check FAILED"
    exit 1
fi

log ""
log "=== SETUP COMPLETED SUCCESSFULLY ==="
log "Project is ready for development and deployment"
log "Setup completed at $(date)"

# Create success marker
touch .setup_complete
echo "$(date)" > .setup_timestamp
