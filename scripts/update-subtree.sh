#!/bin/bash
# update-subtree.sh - Update a specific subtree from upstream

set -e

if [ -z "$1" ]; then
    echo "Usage: ./scripts/update-subtree.sh <component-name>"
    echo ""
    echo "Available components:"
    echo "  - ark-os-noa"
    echo "  - ARK-OS"
    echo "  - agentaskit"
    echo "  - deflexnet-app"
    echo "  - deflex-ai-os"
    echo "  - MicroAgentStack"
    exit 1
fi

COMPONENT=$1

echo "🔄 Updating $COMPONENT from upstream..."

# Check if component exists
if [ ! -d "repos/$COMPONENT" ]; then
    echo "❌ Error: Component 'repos/$COMPONENT' does not exist"
    exit 1
fi

# Pull latest changes
git subtree pull --prefix=repos/$COMPONENT $COMPONENT main --squash

echo "✓ $COMPONENT updated successfully!"
echo ""
echo "Changes have been merged. Review with:"
echo "  git log --oneline -5"
echo "  git diff HEAD~1"
