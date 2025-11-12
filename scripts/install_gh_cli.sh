#!/usr/bin/env bash
set -euo pipefail

if command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI already installed: $(command -v gh)"
  exit 0
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required to install the GitHub CLI. Please install curl and re-run this script." >&2
  exit 1
fi

echo "Installing GitHub CLI (gh)..."

if [ -f /etc/os-release ]; then
  . /etc/os-release
else
  echo "Unsupported environment: cannot determine OS information from /etc/os-release." >&2
  exit 1
fi

case "$ID" in
  ubuntu|debian)
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
      | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
    sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
      | sudo tee /etc/apt/sources.list.d/github-cli.list >/dev/null
    sudo apt-get update
    sudo apt-get install -y gh
    ;;
  *)
    cat <<'EOF' >&2
Unsupported distribution for automatic installation.
Please install the GitHub CLI manually:
  https://cli.github.com/manual/installation
EOF
    exit 1
    ;;
esac

echo "GitHub CLI installation complete."
