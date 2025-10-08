#!/usr/bin/env bash
# NOA ARK OS - Quick Start Script

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         NOA ARK OS - Quick Start                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}This script will:${NC}"
echo "  1. Verify prerequisites"
echo "  2. Build all components"
echo "  3. Run tests"
echo "  4. Execute the full system demo"
echo ""
read -p "Press Enter to continue..."
echo ""

# Check Rust
echo -e "${BLUE}[1/4]${NC} Checking prerequisites..."
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust not found. Please install from: https://rustup.rs/${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Rust toolchain found: $(cargo --version)"
echo ""

# Build
echo -e "${BLUE}[2/4]${NC} Building NOA ARK OS..."
cargo build --release --workspace
echo -e "${GREEN}✓${NC} Build completed"
echo ""

# Test
echo -e "${BLUE}[3/4]${NC} Running tests..."
cargo test --workspace --release
echo -e "${GREEN}✓${NC} All tests passed"
echo ""

# Run demo
echo -e "${BLUE}[4/4]${NC} Running full system demo..."
echo ""
cargo run --release --example full_system_demo
echo ""

echo "╔════════════════════════════════════════════════════════════╗"
echo "║                   Quick Start Complete!                    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  • Run the kernel: cargo run --release --bin noa_kernel"
echo "  • Read the docs: cat docs/GETTING_STARTED.md"
echo "  • Explore examples: ls examples/"
echo ""
