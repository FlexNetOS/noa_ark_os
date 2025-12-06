#!/bin/bash

# Import starred repositories script
# Usage: ./scripts/import-repos.sh

set -e

echo "🚀 Starting repository import process..."

# Create directories if they don't exist
mkdir -p vendor external

# Function to add a subtree
add_subtree() {
    local repo=$1
    local path=$2
    local branch=${3:-main}
    
    if [ ! -d "$path" ]; then
        echo "Adding subtree: $repo -> $path"
        git subtree add --prefix="$path" "https://github.com/$repo.git" "$branch" --squash
    else
        echo "Subtree already exists: $path"
    fi
}

# Function to add a submodule
add_submodule() {
    local repo=$1
    local path=$2
    
    if [ ! -d "$path" ]; then
        echo "Adding submodule: $repo -> $path"
        git submodule add "https://github.com/$repo.git" "$path"
    else
        echo "Submodule already exists: $path"
    fi
}

# Priority imports - Core infrastructure
echo "📦 Importing core infrastructure..."
add_subtree "cloudflare/pingora" "vendor/pingora" "main"
add_subtree "modelcontextprotocol/rust-sdk" "vendor/mcp-rust" "main"

# ML/AI frameworks
echo "🤖 Importing ML/AI frameworks..."
add_submodule "huggingface/candle" "external/candle"
add_submodule "tracel-ai/burn" "external/burn"
add_submodule "ggml-org/llama.cpp" "external/llama-cpp"

# Agent frameworks
echo "🎯 Importing agent frameworks..."
add_submodule "letta-ai/letta" "external/letta"
add_submodule "browser-use/browser-use" "external/browser-use"
add_submodule "All-Hands-AI/OpenHands" "external/openhands"

# Data tools
echo "💾 Importing data tools..."
add_subtree "apache/datafusion" "vendor/datafusion" "main"
add_submodule "duckdb/duckdb" "external/duckdb"
add_submodule "qdrant/rust-client" "external/qdrant-client"

# Development tools
echo "🛠️ Importing development tools..."
add_submodule "Aider-AI/aider" "external/aider"
add_submodule "firecrawl/firecrawl" "external/firecrawl"

echo "✅ Repository import complete!"
echo "Run 'git submodule update --init --recursive' to initialize all submodules"