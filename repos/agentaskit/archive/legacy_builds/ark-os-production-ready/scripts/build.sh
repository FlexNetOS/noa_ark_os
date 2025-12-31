#!/bin/bash
# ARK-OS Production Build Script
# Combines building capabilities from all three repositories

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build configuration
BUILD_TARGET="${ARK_OS_BUILD_TARGET:-}"
BUILD_PROFILE="${BUILD_PROFILE:-release}"
FEATURES="${FEATURES:-full}"
SKIP_TESTS="${SKIP_TESTS:-false}"

echo -e "${BLUE}🏗️  ARK-OS Production Build Script${NC}"
echo "======================================"

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust toolchain."
    exit 1
fi

if ! command -v node &> /dev/null; then
    print_warning "Node.js not found. Desktop UI may not build properly."
fi

print_status "Prerequisites checked"

# Set git hash for build info
export GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Clean previous builds if requested
if [[ "${1:-}" == "clean" ]]; then
    echo -e "${BLUE}Cleaning previous builds...${NC}"
    cargo clean
    print_status "Clean completed"
fi

# Create necessary directories
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p data logs configs static docs/api
print_status "Directories created"

# Build the project
echo -e "${BLUE}Building ARK-OS Production...${NC}"
echo "Profile: $BUILD_PROFILE"
echo "Features: $FEATURES"
echo "Target: ${BUILD_TARGET:-default}"

BUILD_ARGS=""
if [[ "$BUILD_PROFILE" == "release" ]]; then
    BUILD_ARGS="--release"
fi

if [[ "$FEATURES" != "default" ]]; then
    if [[ "$FEATURES" == "full" ]]; then
        BUILD_ARGS="$BUILD_ARGS --features full"
    else
        BUILD_ARGS="$BUILD_ARGS --features $FEATURES"
    fi
fi

if [[ -n "$BUILD_TARGET" ]]; then
    BUILD_ARGS="$BUILD_ARGS --target $BUILD_TARGET"
fi

# Build all binaries
echo -e "${BLUE}Building server binary...${NC}"
cargo build $BUILD_ARGS --bin ark-os-server

echo -e "${BLUE}Building desktop binary...${NC}"
if [[ "$FEATURES" == "full" ]] || [[ "$FEATURES" == *"desktop"* ]]; then
    cargo build $BUILD_ARGS --bin ark-os-desktop
else
    print_warning "Desktop features not enabled, skipping desktop binary"
fi

echo -e "${BLUE}Building CLI binary...${NC}"
cargo build $BUILD_ARGS --bin ark-os-cli

print_status "Build completed"

# Run tests unless skipped
if [[ "$SKIP_TESTS" != "true" ]]; then
    echo -e "${BLUE}Running tests...${NC}"
    cargo test $BUILD_ARGS
    print_status "Tests passed"
else
    print_warning "Tests skipped"
fi

# Generate documentation
echo -e "${BLUE}Generating documentation...${NC}"
cargo doc --no-deps --features $FEATURES
print_status "Documentation generated"

# Create deployment package structure
echo -e "${BLUE}Creating deployment structure...${NC}"

DEPLOY_DIR="target/deploy"
mkdir -p "$DEPLOY_DIR"/{bin,configs,docs,scripts}

# Copy binaries
if [[ "$BUILD_PROFILE" == "release" ]]; then
    BIN_DIR="target/release"
else
    BIN_DIR="target/debug"
fi

cp "$BIN_DIR/ark-os-server" "$DEPLOY_DIR/bin/" 2>/dev/null || print_warning "ark-os-server not found"
cp "$BIN_DIR/ark-os-desktop" "$DEPLOY_DIR/bin/" 2>/dev/null || print_warning "ark-os-desktop not found"
cp "$BIN_DIR/ark-os-cli" "$DEPLOY_DIR/bin/" 2>/dev/null || print_warning "ark-os-cli not found"

# Copy configuration templates
cp configs/* "$DEPLOY_DIR/configs/" 2>/dev/null || echo "# Default config placeholder" > "$DEPLOY_DIR/configs/config.json"

# Copy documentation
cp -r target/doc "$DEPLOY_DIR/docs/api" 2>/dev/null || print_warning "API documentation not generated"
cp README.md "$DEPLOY_DIR/docs/" 2>/dev/null || echo "# ARK-OS Production" > "$DEPLOY_DIR/docs/README.md"

# Copy scripts
cp scripts/* "$DEPLOY_DIR/scripts/" 2>/dev/null || print_warning "No scripts to copy"

print_status "Deployment structure created in $DEPLOY_DIR"

# Print build summary
echo ""
echo -e "${GREEN}🎉 Build Summary${NC}"
echo "================"
echo "Profile: $BUILD_PROFILE"
echo "Features: $FEATURES"
echo "Git Hash: $GIT_HASH"
echo "Build Time: $(date)"
echo "Deployment: $DEPLOY_DIR"

# Print next steps
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "1. Review configuration in $DEPLOY_DIR/configs/"
echo "2. Run: ./target/deploy/bin/ark-os-server --help"
echo "3. Initialize: ./target/deploy/bin/ark-os-server init"
echo "4. Start: ./target/deploy/bin/ark-os-server start"

echo -e "${GREEN}✓ Build completed successfully!${NC}"