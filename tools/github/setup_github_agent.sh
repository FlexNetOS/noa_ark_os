#!/bin/bash
#
# Setup script for GitHub Coding Agent
# Installs and configures the background task automation agent

set -euo pipefail

echo "════════════════════════════════════════════════════════════════════════════"
echo "  GitHub Coding Agent Setup"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""

# Make scripts executable
echo "📝 Making scripts executable..."
chmod +x ~/workspace/tools/github_coding_agent.sh
chmod +x ~/workspace/tools/agent

# Create log directory
echo "📁 Creating log directory..."
mkdir -p ~/logs/development

# Create tools/bin directory and symlink
echo "🔗 Creating PATH symlink..."
mkdir -p ~/workspace/tools/bin
ln -sf ../agent ~/workspace/tools/bin/agent 2>/dev/null || true

# Register in applications registry
echo "📋 Registering in applications registry..."
REGISTRY="$HOME/config/applications/registry.jsonl"
mkdir -p "$(dirname "$REGISTRY")"
touch "$REGISTRY"

# Check if already registered
if ! grep -q "github_coding_agent" "$REGISTRY" 2>/dev/null; then
    cat >> "$REGISTRY" <<EOF
{"timestamp":"$(date -u +"%Y-%m-%dT%H:%M:%SZ")","event":"application_registered","application":"github_coding_agent","version":"1.0","type":"background_service","status":"installed","config":"config/applications/github_coding_agent.yaml","script":"workspace/tools/github_coding_agent.sh","auto_mode":true}
EOF
fi

echo ""
echo "════════════════════════════════════════════════════════════════════════════"
echo "✅ GitHub Coding Agent setup complete!"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""
echo "Configuration:"
echo "  Mode: FULL AUTO (enabled in config)"
echo "  Check interval: 5 minutes"
echo "  Config file: ~/config/applications/github_coding_agent.yaml"
echo "  Log file: ~/logs/development/github_agent.log"
echo ""
echo "Quick Start:"
echo "  1. Start agent:  agent start"
echo "  2. Check status: agent status"
echo "  3. View logs:    agent logs"
echo "  4. Stop agent:   agent stop"
echo ""
echo "Optional - Install as system service:"
echo "  agent install    # Requires sudo"
echo ""
echo "The agent will monitor ~/workspace/repos/task_exec_kit/agentask.todo"
echo "and create notifications for tasks ready to execute."
echo ""
