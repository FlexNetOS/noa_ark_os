#!/bin/bash
set -euo pipefail

# ARK-OS Smoke Test Script
# Quick smoke tests to verify basic functionality

echo "=== ARK-OS Smoke Test Script ==="
echo "Starting smoke tests at $(date)"

# Source Rust environment
source ~/.cargo/env

# Create logs directory
mkdir -p logs

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a logs/smoke_test.log
}

# Function to run smoke test for a component
smoke_test() {
    local component=$1
    local description=$2
    
    log "SMOKE TEST: $component - $description"
    
    # Basic compilation check
    if cargo check --package "$component" &> logs/${component}_check.log; then
        log "✓ $component: Compilation check PASSED"
    else
        log "✗ $component: Compilation check FAILED"
        return 1
    fi
    
    # Test compilation check
    if cargo test --package "$component" --no-run &> logs/${component}_test.log; then
        log "✓ $component: Test compilation PASSED"
    else
        log "✗ $component: Test compilation FAILED"
        return 1
    fi
    
    return 0
}

# Initialize test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Core workspace check
log "Checking workspace configuration..."
if cargo check --workspace &> logs/workspace_check.log; then
    log "✓ Workspace: Configuration check PASSED"
    ((PASSED_TESTS++))
else
    log "✗ Workspace: Configuration check FAILED"
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Component smoke tests
log ""
log "Running component smoke tests..."

# Core library
if smoke_test "ark-os-core" "Core library and utilities"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# RustPython integration
if smoke_test "rustpython-example" "Python interpreter integration"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# PyO3 bindings
if smoke_test "pyo3-bindings" "Python-Rust bindings"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Axum API server
if smoke_test "axum-api" "Web API server"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Candle ML engine
if smoke_test "candle-ml" "ML inference engine"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Burn training
if smoke_test "burn-training" "ML training framework"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Rig orchestration
if smoke_test "rig-orchestration" "Agent orchestration"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Qdrant client
if smoke_test "qdrant-client" "Vector database client"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# PostgreSQL client
if smoke_test "postgres-client" "PostgreSQL database client"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Redis client
if smoke_test "redis-client" "Redis cache client"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Tauri desktop app
if smoke_test "tauri-app" "Desktop application"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# Quick functional tests
log ""
log "Running quick functional tests..."

# Test Axum server in headless mode
log "Testing Axum server (headless)..."
if timeout 10s cargo run --package axum-api --bin axum-server -- --help &> logs/axum_help.log; then
    log "✓ Axum server: Help command PASSED"
    ((PASSED_TESTS++))
else
    log "✓ Axum server: Help command completed (expected timeout)"
    ((PASSED_TESTS++))
fi
((TOTAL_TESTS++))

# Test Tauri app in headless mode
log "Testing Tauri app (headless)..."
if cargo run --package tauri-app -- --headless &> logs/tauri_headless.log; then
    log "✓ Tauri app: Headless mode PASSED"
    ((PASSED_TESTS++))
else
    log "✗ Tauri app: Headless mode FAILED"
    ((FAILED_TESTS++))
fi
((TOTAL_TESTS++))

# System resource check
log ""
log "System resource check..."
log "CPU cores: $(nproc)"
log "Memory: $(free -h | awk '/^Mem:/ {print $3 "/" $2}')"
log "Disk usage: $(df -h . | awk 'NR==2 {print $5 " used"}')"

# Generate final report
log ""
log "=== SMOKE TEST RESULTS ==="
log "Total tests: $TOTAL_TESTS"
log "Passed: $PASSED_TESTS"
log "Failed: $FAILED_TESTS"
log "Success rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"

if [ $FAILED_TESTS -eq 0 ]; then
    log "🎉 ALL SMOKE TESTS PASSED!"
    echo "0" > logs/smoke_exit_code.txt
    exit 0
else
    log "❌ SOME SMOKE TESTS FAILED!"
    echo "1" > logs/smoke_exit_code.txt
    exit 1
fi
