#!/bin/bash
# ARK-OS Production Setup Script
# Initializes the system for first-time use

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 ARK-OS Production Setup${NC}"
echo "=========================="

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_error "Do not run this script as root"
    exit 1
fi

# System requirements check
echo -e "${BLUE}Checking system requirements...${NC}"

# Check Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi
print_status "Rust/Cargo found"

# Check Node.js for desktop UI
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    print_status "Node.js found: $NODE_VERSION"
else
    print_warning "Node.js not found - desktop UI may not work"
fi

# Setup directories
echo -e "${BLUE}Setting up directories...${NC}"

DIRS=(
    "$HOME/.ark-os"
    "$HOME/.ark-os/data"
    "$HOME/.ark-os/logs" 
    "$HOME/.ark-os/configs"
    "$HOME/.ark-os/cache"
    "$HOME/.ark-os/plugins"
)

for dir in "${DIRS[@]}"; do
    mkdir -p "$dir"
    print_status "Created $dir"
done

# Generate default configuration
echo -e "${BLUE}Generating configuration...${NC}"

CONFIG_FILE="$HOME/.ark-os/configs/config.json"
if [[ ! -f "$CONFIG_FILE" ]]; then
    cat > "$CONFIG_FILE" << 'EOF'
{
  "system": {
    "name": "ark-os-production",
    "version": "0.1.0",
    "environment": "Development",
    "data_directory": "~/.ark-os/data",
    "log_directory": "~/.ark-os/logs",
    "config_directory": "~/.ark-os/configs",
    "max_threads": null,
    "shutdown_timeout_seconds": 30
  },
  "agents": {
    "max_agents": 1000,
    "default_health_check_interval": {
      "secs": 30,
      "nanos": 0
    },
    "agent_registry_capacity": 10000,
    "message_queue_capacity": 10000,
    "resource_allocation_strategy": "Balanced"
  },
  "orchestration": {
    "max_concurrent_executions": 100,
    "planning_cycle_duration": {
      "secs": 300,
      "nanos": 0
    },
    "execution_timeout": {
      "secs": 3600,
      "nanos": 0
    },
    "autonomous_mode": false,
    "triple_verification_enabled": true,
    "auto_healing_enabled": true,
    "scheduler_config": {
      "max_concurrency": 10,
      "priority_queues": 7,
      "load_balancing_strategy": "RoundRobin",
      "resource_constraints": {
        "max_cpu_usage": 0.8,
        "max_memory_usage": 8589934592,
        "max_storage_usage": 107374182400,
        "max_network_bandwidth": 1073741824
      }
    },
    "executor_config": {
      "max_workers": 16,
      "worker_timeout": {
        "secs": 300,
        "nanos": 0
      },
      "retry_attempts": 3,
      "parallel_execution": true
    }
  },
  "execution": {
    "max_concurrent_tasks": 100,
    "default_timeout": {
      "secs": 300,
      "nanos": 0
    },
    "retry_attempts": 3,
    "worker_pool_size": 16,
    "resource_limits": {
      "max_memory_mb": 4096,
      "max_cpu_cores": 4.0,
      "max_storage_mb": 10240,
      "max_network_mbps": 1000
    }
  },
  "ui": {
    "desktop_enabled": true,
    "web_enabled": true,
    "api_enabled": true,
    "desktop_config": {
      "window_title": "ARK-OS Production",
      "window_width": 1200,
      "window_height": 800,
      "resizable": true,
      "fullscreen": false,
      "system_tray": true,
      "auto_updater": false,
      "dev_tools": false
    },
    "web_config": {
      "host": "127.0.0.1",
      "port": 3000,
      "static_files_path": "static",
      "cors_enabled": true,
      "compression_enabled": true
    },
    "api_config": {
      "host": "127.0.0.1",
      "port": 8080,
      "api_prefix": "/api/v1",
      "rate_limiting": false,
      "authentication_required": false
    }
  },
  "logging": {
    "level": "info",
    "format": "Pretty",
    "output": "Both",
    "file_rotation": {
      "max_file_size_mb": 100,
      "keep_files": 5,
      "rotate_daily": true
    },
    "structured_logging": false
  },
  "security": {
    "encryption_enabled": true,
    "tls_enabled": false,
    "authentication_required": false,
    "session_timeout_minutes": 60,
    "max_failed_attempts": 5,
    "allowed_origins": ["*"]
  },
  "features": {
    "agents_enabled": true,
    "orchestration_enabled": true,
    "execution_enabled": true,
    "desktop_ui_enabled": true,
    "web_ui_enabled": true,
    "api_enabled": true,
    "autonomous_mode": false,
    "triple_verification": true,
    "auto_healing": true,
    "metrics_collection": true,
    "performance_monitoring": true
  }
}
EOF
    print_status "Generated default configuration"
else
    print_warning "Configuration already exists"
fi

# Create systemd service file (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${BLUE}Creating systemd service...${NC}"
    
    SERVICE_FILE="$HOME/.config/systemd/user/ark-os.service"
    mkdir -p "$(dirname "$SERVICE_FILE")"
    
    cat > "$SERVICE_FILE" << EOF
[Unit]
Description=ARK-OS Production System
After=network.target

[Service]
Type=exec
ExecStart=$(which ark-os-server || echo '/usr/local/bin/ark-os-server') start --config $CONFIG_FILE
ExecStop=$(which ark-os-server || echo '/usr/local/bin/ark-os-server') stop
Restart=always
RestartSec=10
User=%i
Group=%i
Environment=HOME=%h
Environment=ARK_OS_CONFIG_DIR=%h/.ark-os/configs
Environment=ARK_OS_DATA_DIR=%h/.ark-os/data
Environment=ARK_OS_LOG_DIR=%h/.ark-os/logs

[Install]
WantedBy=default.target
EOF
    
    print_status "Created systemd service (use 'systemctl --user enable ark-os' to enable)"
fi

# Setup environment variables
echo -e "${BLUE}Setting up environment...${NC}"

BASHRC_ADDITION="
# ARK-OS Production Environment
export ARK_OS_CONFIG_DIR=\"\$HOME/.ark-os/configs\"
export ARK_OS_DATA_DIR=\"\$HOME/.ark-os/data\"
export ARK_OS_LOG_DIR=\"\$HOME/.ark-os/logs\"
export PATH=\"\$PATH:\$HOME/.ark-os/bin\"
"

if ! grep -q "ARK-OS Production Environment" "$HOME/.bashrc" 2>/dev/null; then
    echo "$BASHRC_ADDITION" >> "$HOME/.bashrc"
    print_status "Added environment variables to ~/.bashrc"
else
    print_warning "Environment variables already in ~/.bashrc"
fi

# Create desktop entry (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]] && command -v ark-os-desktop &> /dev/null; then
    echo -e "${BLUE}Creating desktop entry...${NC}"
    
    DESKTOP_FILE="$HOME/.local/share/applications/ark-os.desktop"
    mkdir -p "$(dirname "$DESKTOP_FILE")"
    
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=ARK-OS Production
Comment=ARK-OS Production System
Exec=ark-os-desktop
Icon=ark-os
Terminal=false
Categories=Development;System;
StartupNotify=true
EOF
    
    print_status "Created desktop entry"
fi

# Final setup verification
echo -e "${BLUE}Verifying setup...${NC}"

if [[ -f "$CONFIG_FILE" ]]; then
    print_status "Configuration file exists"
else
    print_error "Configuration file missing"
fi

for dir in "${DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        print_status "Directory $dir exists"
    else
        print_error "Directory $dir missing"
    fi
done

echo ""
echo -e "${GREEN}🎉 Setup Complete!${NC}"
echo "=================="
echo "Configuration: $CONFIG_FILE"
echo "Data Directory: $HOME/.ark-os/data"
echo "Log Directory: $HOME/.ark-os/logs"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "1. Review and customize: $CONFIG_FILE"
echo "2. Build the project: ./scripts/build.sh"
echo "3. Run: ark-os-server init"
echo "4. Start: ark-os-server start"
echo ""
echo "For service management (Linux):"
echo "  systemctl --user enable ark-os"
echo "  systemctl --user start ark-os"
echo ""
echo -e "${YELLOW}Note:${NC} Restart your shell or run 'source ~/.bashrc' to use environment variables"