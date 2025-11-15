#!/bin/bash

# Generate Documentation for NOA ARK OS
# This script generates comprehensive documentation from Rust code comments

set -e

OPEN_BROWSER=false
CLEAN=false
OUTPUT_DIR="docs/api"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -OpenBrowser)
            OPEN_BROWSER=true
            shift
            ;;
        -Clean)
            CLEAN=true
            shift
            ;;
        -OutputDir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [-OpenBrowser] [-Clean] [-OutputDir <dir>]"
            exit 1
            ;;
    esac
done

echo "🔧 NOA ARK OS Documentation Generator"
echo "====================================="

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Clean previous docs if requested
if [ "$CLEAN" = true ]; then
    echo "🧹 Cleaning previous documentation..."
    rm -rf "$OUTPUT_DIR"
    rm -rf target/doc
fi

# Activate portable Cargo if available
CARGO_SCRIPT="./server/tools/activate-cargo-wsl.sh"
if [ -f "$CARGO_SCRIPT" ]; then
    echo "🔨 Activating portable Cargo..."
    source "$CARGO_SCRIPT"
fi

# Generate documentation
echo "📚 Generating Rust documentation..."
if cargo doc --workspace --no-deps --document-private-items; then
    echo "✅ Documentation generated successfully!"

    # Copy docs to output directory
    if [ ! -d "$OUTPUT_DIR" ]; then
        mkdir -p "$OUTPUT_DIR"
    fi

    echo "📋 Copying documentation to $OUTPUT_DIR..."
    cp -r target/doc/* "$OUTPUT_DIR"

    # Generate additional docs
    echo "📝 Generating additional documentation..."

    # Generate crate documentation index
    INDEX_PATH="$OUTPUT_DIR/index.html"
    cat > "$INDEX_PATH" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>NOA ARK OS API Documentation</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #2c3e50; }
        .crate-list { margin: 20px 0; }
        .crate-item { margin: 10px 0; padding: 10px; border-left: 4px solid #3498db; background: #f8f9fa; }
        .crate-link { color: #3498db; text-decoration: none; font-weight: bold; }
        .crate-link:hover { text-decoration: underline; }
        .timestamp { color: #7f8c8d; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>🚀 NOA ARK OS API Documentation</h1>
    <p>Comprehensive API documentation for the NOA ARK OS platform components.</p>

    <div class="crate-list">
        <h2>Core Components</h2>
        <div class="crate-item">
            <a href="noa_core/index.html" class="crate-link">noa_core</a>
            <p>Core OS functionality including kernel, gateway, and system services</p>
        </div>
        <div class="crate-item">
            <a href="noa_agents/index.html" class="crate-link">noa_agents</a>
            <p>Agent factory with hive mind and swarm capabilities</p>
        </div>
        <div class="crate-item">
            <a href="noa_crc/index.html" class="crate-link">noa_crc</a>
            <p>Continuous ReCode system for AI-supervised adaptation</p>
        </div>
        <div class="crate-item">
            <a href="noa_cicd/index.html" class="crate-link">noa_cicd</a>
            <p>CI/CD pipeline with CRC integration</p>
        </div>
    </div>

    <div class="timestamp">
        Generated on: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
    </div>
</body>
</html>
EOF

    echo "📖 Documentation index created at $INDEX_PATH"

    # Open browser if requested (only works in graphical environments)
    if [ "$OPEN_BROWSER" = true ]; then
        echo "🌐 Opening documentation in browser..."
        if command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$INDEX_PATH" &
        elif command -v open >/dev/null 2>&1; then
            open "$INDEX_PATH" &
        else
            echo "⚠️  Could not open browser automatically"
        fi
    fi

    echo ""
    echo "📚 Documentation Summary:"
    echo "  - Location: $OUTPUT_DIR"
    echo "  - Index: $INDEX_PATH"
    echo "  - Open in browser: ./scripts/generate-docs.sh -OpenBrowser"

else
    echo "❌ Documentation generation failed!"
    exit 1
fi

echo ""
echo "🎉 Documentation generation complete!"