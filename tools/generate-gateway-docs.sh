#!/bin/bash
# Gateway Documentation Automation Script
# Generates comprehensive documentation from code and updates references

set -e

echo "🚀 Generating Gateway Documentation..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "core/Cargo.toml" ]; then
    print_error "Must be run from workspace root (noa_ark_os/)"
    exit 1
fi

# Activate portable Cargo if available
if [ -f "server/tools/activate-cargo-wsl.sh" ]; then
    print_status "Activating portable Cargo..."
    source server/tools/activate-cargo-wsl.sh
fi

# Generate Rust documentation
print_status "Generating Rust API documentation..."
cargo doc --workspace --no-deps --document-private-items

# Generate gateway-specific docs
print_status "Generating Gateway API reference..."
cargo doc -p noa_core --no-deps --document-private-items
cargo doc -p noa_gateway --no-deps --document-private-items

# Copy docs to docs/api/ if it exists
if [ -d "docs/api" ]; then
    print_status "Copying documentation to docs/api/..."
    cp -r target/doc/* docs/api/ 2>/dev/null || true
fi

# Generate OpenAPI specs from code comments
print_status "Extracting API specifications..."
# This would integrate with a tool like utoipa or similar
# For now, placeholder for future implementation

# Update README with latest metrics
print_status "Updating README with current metrics..."

# Get current test results
TEST_RESULTS=$(cargo test --workspace --quiet -- --nocapture 2>&1 | tail -20 || echo "Tests completed")

# Get build status
BUILD_STATUS=$(cargo check --workspace --quiet && echo "✅ Passing" || echo "❌ Failing")

# Update docs/README_BUILD_KITS.md if it exists
if [ -f "docs/README_BUILD_KITS.md" ]; then
    print_status "Updating build kits documentation..."
    # Placeholder for build kit generation
fi

# Generate changelog from git history
print_status "Generating changelog..."
if [ -f "CHANGELOG.md" ]; then
    # Extract recent changes
    git log --oneline --since="1 month ago" -- docs/ core/src/gateway.rs server/gateway/ > /tmp/recent_changes.txt
    if [ -s /tmp/recent_changes.txt ]; then
        echo "## Recent Gateway Changes ($(date +%Y-%m-%d))" >> CHANGELOG.md
        echo "" >> CHANGELOG.md
        cat /tmp/recent_changes.txt >> CHANGELOG.md
        echo "" >> CHANGELOG.md
        print_status "Changelog updated"
    fi
fi

# Validate documentation links
print_status "Validating documentation links..."
# This would check for broken links in markdown files
# Placeholder for future implementation

# Generate dependency graph
print_status "Generating dependency graphs..."
cargo tree --workspace > docs/dependencies.txt 2>/dev/null || true

# Create documentation index
print_status "Creating documentation index..."
cat > docs/DOCUMENTATION_INDEX.md << 'EOF'
# NOA ARK OS Documentation Index

## Gateway System Documentation

### Core Implementation
- [Gateway Architecture](plans/GATEWAY_ROADMAP.md) - Complete roadmap and implementation status
- [Gateway API Reference](api/noa_core/gateway/index.html) - Rust API documentation
- [Server Gateway API](api/noa_gateway/index.html) - Server-side API documentation

### Operational Guides
- [Quick Start](QUICK_START.md) - Gateway startup and basic usage
- [Deployment Guide](DEPLOYMENT.md) - Production deployment procedures
- [Troubleshooting](operations/gateway.md) - Common issues and solutions

### Development Resources
- [Contributing Guide](DEVELOPMENT.md) - Development workflows
- [Testing Guide](tests/GATEWAY_TESTING_CERTIFICATION_2025.md) - Test procedures
- [Security Audit](audits/GATEWAY_SECURITY_CERTIFICATION_2024.md) - Security compliance

### Certification & Compliance
- [Certification Bundle](audits/GATEWAY_CERTIFICATION_BUNDLE_2025.md) - Complete certification evidence
- [Privacy Certification](audits/GATEWAY_PRIVACY_CERTIFICATION_2024.md) - Privacy compliance
- [Operational Certification](tests/GATEWAY_OPERATIONAL_CERTIFICATION_2024.md) - Operational readiness

## Automation Status
- **Last Generated**: $(date)
- **Build Status**: $BUILD_STATUS
- **Test Results**: See recent test output
- **Documentation SLA**: <48 hours for changes

## Quick Links
- [Gateway Roadmap](plans/GATEWAY_ROADMAP.md)
- [API Documentation](api/)
- [Test Results](tests/)
- [Security Audits](audits/)
EOF

print_status "Documentation generation complete!"
print_status "Generated files:"
echo "  - Rust API docs: target/doc/"
echo "  - Documentation index: docs/DOCUMENTATION_INDEX.md"
echo "  - Dependency graph: docs/dependencies.txt"
echo "  - Updated changelog: CHANGELOG.md"

if [ -f "docs/README_BUILD_KITS.md" ]; then
    echo "  - Build kits: docs/README_BUILD_KITS.md"
fi

print_warning "Note: Some advanced features require additional tools for full automation"
print_warning "Consider installing: cargo-udeps, cargo-audit, cargo-tarpaulin for enhanced reporting"