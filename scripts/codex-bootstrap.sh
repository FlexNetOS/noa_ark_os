#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "${BIN_DIR}"

PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  PYTHON_BIN="python"
fi
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  echo "[codex-bootstrap] python interpreter not found; install python3 or set PYTHON_BIN" >&2
  exit 1
fi

log() {
  echo "[codex-bootstrap] $*" >&2
}

install_codex_cli() {
  if command -v codex >/dev/null 2>&1; then
    log "codex already installed at $(command -v codex)"
    return 0
  fi

  local os arch asset url tmp extracted
  os="$(uname -s)"
  arch="$(uname -m)"
  case "${os}:${arch}" in
    Linux:x86_64) asset="codex-x86_64-unknown-linux-musl.tar.gz" ;;
    Linux:aarch64 | Linux:arm64) asset="codex-aarch64-unknown-linux-musl.tar.gz" ;;
    Darwin:x86_64) asset="codex-x86_64-apple-darwin.tar.gz" ;;
    Darwin:arm64) asset="codex-aarch64-apple-darwin.tar.gz" ;;
    *)
      log "unsupported platform ${os}/${arch}; install Codex CLI manually"
      return 1
      ;;
  esac

  url="https://github.com/openai/codex/releases/latest/download/${asset}"
  tmp="$(mktemp -d)"
  trap "rm -rf '${tmp}'" RETURN
  log "downloading Codex CLI from ${url}"
  curl -fsSL "${url}" -o "${tmp}/codex.tar.gz"
  tar -xzf "${tmp}/codex.tar.gz" -C "${tmp}"
  extracted="$(find "${tmp}" -maxdepth 1 -type f -name "codex*" | head -n1)"
  if [[ -z "${extracted}" ]]; then
    log "failed to locate extracted Codex binary in ${tmp}"
    return 1
  fi
  if command -v install >/dev/null 2>&1; then
    install -m 0755 "${extracted}" "${BIN_DIR}/codex"
  else
    cp "${extracted}" "${BIN_DIR}/codex"
    chmod +x "${BIN_DIR}/codex"
  fi
  log "installed codex to ${BIN_DIR}/codex"
}

install_noa_mcp_server() {
  if command -v noa-mcp-server >/dev/null 2>&1; then
    log "noa-mcp-server already installed at $(command -v noa-mcp-server)"
    return 0
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    log "cargo not found; cannot build noa-mcp-server"
    return 1
  fi
  log "building noa-mcp-server via cargo install"
  cargo install --locked --path "${REPO_ROOT}/server/mcp" --root "${HOME}/.local"
  log "installed noa-mcp-server to ${BIN_DIR}/noa-mcp-server"
}

write_codex_config() {
  local codex_home config snippet
  codex_home="${CODEX_HOME:-${HOME}/.codex}"
  config="${codex_home}/config.toml"
  mkdir -p "${codex_home}"

  snippet="$(mktemp)"
  cat >"${snippet}" <<EOF
[mcp_servers.noa]
command = "${BIN_DIR}/noa-mcp-server"
cwd = "${REPO_ROOT}"
env = { NOA_TOOLS_SERVER_URL = "${NOA_TOOLS_SERVER_URL:-http://127.0.0.1:8910/}" }
startup_timeout_sec = 20
tool_timeout_sec = 60
EOF

  "${PYTHON_BIN}" - <<'PY' "${config}" "${snippet}"
import sys
from pathlib import Path

config_path = Path(sys.argv[1])
snippet_path = Path(sys.argv[2])
snippet = snippet_path.read_text().strip()

if config_path.exists():
    lines = config_path.read_text().splitlines()
else:
    lines = []

result = []
skip = False
target = "[mcp_servers.noa]"

for line in lines:
    stripped = line.strip()
    if stripped.startswith("[") and not stripped.startswith("[["):
        if skip:
            skip = False
        if stripped == target:
            skip = True
            continue
    if not skip:
        result.append(line.rstrip())

if result and result[-1] != "":
    result.append("")

result.append(snippet)
result.append("")

config_path.write_text("\n".join(result), encoding="utf-8")
PY

  rm -f "${snippet}"
  log "updated ${config} with noa MCP entry"
}

main() {
  install_codex_cli
  install_noa_mcp_server
  write_codex_config
  if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    log "PATH is missing ${BIN_DIR}; add 'export PATH=${BIN_DIR}:\$PATH' to your shell profile"
  fi
  log "Codex CLI bootstrap complete"
}

main "$@"
