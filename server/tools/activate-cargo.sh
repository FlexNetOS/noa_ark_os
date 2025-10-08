#!/bin/bash
# NOA ARK OS - Activate Cargo (WSL/Linux)
# Usage: source ./server/tools/activate-cargo.sh

echo -e "\n🔧 Activating Cargo for WSL/Linux..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if Rust is installed in WSL
if [ -f "$HOME/.cargo/env" ]; then
    # Load Cargo environment
    source "$HOME/.cargo/env"
    
    echo -e "\n✅ Cargo Activated Successfully!"
    echo -e "\nEnvironment:"
    echo "  CARGO_HOME   = $CARGO_HOME"
    echo "  RUSTUP_HOME  = $RUSTUP_HOME"
    
    echo -e "\nVersions:"
    cargo --version
    rustc --version
    
    echo -e "\n💡 Tips:"
    echo "  • Run 'cargo build' to build projects"
    echo "  • Run 'cargo run' to run projects"
    echo "  • Run 'cargo test' to run tests"
    echo ""
else
    echo -e "\n⚠️  Rust not found in WSL!"
    echo -e "\nTo install Rust in WSL, run:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo -e "\nThen reload this script:"
    echo "  source ./server/tools/activate-cargo.sh"
    echo ""
    
    # Check if Windows portable Cargo can be accessed
    WIN_CARGO="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin/cargo.exe"
    if [ -f "$WIN_CARGO" ]; then
        echo -e "ℹ️  Windows portable Cargo detected at:"
        echo "  $WIN_CARGO"
        echo -e "\nYou can use Windows Cargo from WSL (slower):"
        echo "  export CARGO_HOME=/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
        echo "  export RUSTUP_HOME=/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
        echo "  export PATH=\"/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:\$PATH\""
        echo ""
        
        read -p "Would you like to use Windows Cargo from WSL? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            export CARGO_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable"
            export RUSTUP_HOME="/mnt/d/dev/workspaces/noa_ark_os/server/tools/rustup-portable"
            export PATH="/mnt/d/dev/workspaces/noa_ark_os/server/tools/cargo-portable/bin:$PATH"
            
            echo -e "\n✅ Windows Cargo activated in WSL!"
            cargo.exe --version
            rustc.exe --version
            echo ""
        fi
    fi
fi
