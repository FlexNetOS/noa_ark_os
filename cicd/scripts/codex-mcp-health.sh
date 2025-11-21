#!/usr/bin/env bash
set -euo pipefail

if [[ "${NOA_MCP_HEALTH_DISABLED:-0}" == "1" ]]; then
  echo "[codex-mcp-health] skipping due to NOA_MCP_HEALTH_DISABLED=1"
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CODEX_BIN="${CODEX_BIN:-codex}"
CODEX_HOME="${CODEX_HOME:-${HOME}/.codex}"
PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  PYTHON_BIN="python"
fi

TOOLS_URL="${NOA_TOOLS_SERVER_URL:-http://127.0.0.1:8910/}"
REMOTE_ONLY="${NOA_MCP_REMOTE_ONLY:-0}"

MCP_LIST_OUTPUT="$(mktemp)"
agent_log=""
TOOLS_PID=""

cleanup() {
  if [[ -n "${TOOLS_PID:-}" ]] && ps -p "${TOOLS_PID}" >/dev/null 2>&1; then
    kill "${TOOLS_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${agent_log:-}" ]]; then
    rm -f "${agent_log}" || true
  fi
  rm -f "${MCP_LIST_OUTPUT}" || true
}
trap cleanup EXIT

if ! command -v "${CODEX_BIN}" >/dev/null 2>&1; then
  echo "codex executable not found. Run scripts/codex-bootstrap.sh first." >&2
  exit 1
fi

"${CODEX_BIN}" mcp list --json >"${MCP_LIST_OUTPUT}"
"${PYTHON_BIN}" - <<'PY' "${MCP_LIST_OUTPUT}"
import json
import sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
if not isinstance(data, list):
    raise SystemExit("codex mcp list --json did not return an array")
if not any(entry.get("name") == "noa" for entry in data):
    raise SystemExit("noa MCP server not configured in Codex")
PY

echo "[codex-mcp-health] codex mcp list --json reports noa server"

readarray -t _parsed <<<"$("${PYTHON_BIN}" - <<'PY' "${TOOLS_URL}"
import sys
from urllib.parse import urlparse

parsed = urlparse(sys.argv[1])
host = parsed.hostname or "127.0.0.1"
if parsed.port:
    port = parsed.port
else:
    port = 443 if parsed.scheme == "https" else 80
base = parsed._replace(path="", params="", query="", fragment="").geturl().rstrip("/") + "/"
print(f"{host}:{port}")
print(base)
PY
)"
NOA_ADDRESS_DEFAULT="${_parsed[0]}"
TOOLS_BASE_URL="${_parsed[1]}"
NOA_ADDRESS="${NOA_TOOLS_ADDRESS:-${NOA_ADDRESS_DEFAULT}}"

if [[ "${REMOTE_ONLY}" != "1" ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo command missing; cannot run noa-tools-agent for health check" >&2
    exit 1
  fi
  export NOA_WORKSPACE_ROOT="${REPO_ROOT}"
  export NOA_TOOLS_ADDRESS="${NOA_ADDRESS}"
  agent_log="$(mktemp)"
  cargo run -p noa-tools-agent --bin noa-tools-agent --quiet >"${agent_log}" 2>&1 &
  TOOLS_PID=$!
else
  echo "[codex-mcp-health] remote-only mode enabled; expecting NOA_TOOLS_SERVER_URL (${TOOLS_BASE_URL}) to be reachable"
fi

"${PYTHON_BIN}" - <<'PY' "${TOOLS_BASE_URL}"
import json
import sys
import time
import urllib.request

base = sys.argv[1].rstrip("/")
url = base + "/list_files"
payload = json.dumps({"path": "."}).encode()
deadline = time.time() + 20
while time.time() < deadline:
    try:
        req = urllib.request.Request(
            url,
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=5) as response:
            if 200 <= response.status < 300:
                sys.exit(0)
    except Exception:
        time.sleep(0.5)
raise SystemExit("timed out waiting for noa_tools_agent to accept requests")
PY

MCP_COMMAND=()
if command -v noa-mcp-server >/dev/null 2>&1; then
  MCP_COMMAND+=("$(command -v noa-mcp-server)")
else
  MCP_COMMAND+=("cargo" "run" "-p" "noa-mcp-server" "--quiet")
fi

"${PYTHON_BIN}" - <<'PY' "${REPO_ROOT}" "${TOOLS_BASE_URL}" -- "${MCP_COMMAND[@]}"
import json
import os
import select
import subprocess
import sys

args = sys.argv[1:]
if "--" not in args:
    raise SystemExit("missing MCP command delimiter")
sep = args.index("--")
repo_root = args[0]
server_url = args[1]
command = args[sep + 1 :]
if not command:
    raise SystemExit("missing MCP server command")

env = os.environ.copy()
env["NOA_TOOLS_SERVER_URL"] = server_url

proc = subprocess.Popen(
    command,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    cwd=repo_root,
    env=env,
)

def rpc_call(request):
    message = json.dumps(request)
    proc.stdin.write(message + "\n")
    proc.stdin.flush()
    ready, _, _ = select.select([proc.stdout], [], [], 15)
    if not ready:
        raise SystemExit(f"timeout waiting for response to {request['method']}")
    line = proc.stdout.readline().strip()
    if not line:
        raise SystemExit(f"empty response for {request['method']}")
    response = json.loads(line)
    if response.get("error"):
        raise SystemExit(f"{request['method']} failed: {response['error']}")
    return response

try:
    rpc_call({"jsonrpc": "2.0", "id": 1, "method": "initialize"})
    tools_resp = rpc_call({"jsonrpc": "2.0", "id": 2, "method": "list_tools"})
    tools = tools_resp.get("result", {}).get("tools", [])
    required = {
        "noa.list_files",
        "noa.read_file",
        "noa.apply_patch",
        "noa.edit_file",
        "noa.run_command",
        "noa.build",
        "noa.test",
    }
    names = {tool.get("name") for tool in tools}
    missing = sorted(required - names)
    if missing:
        raise SystemExit(f"Missing required tools: {', '.join(missing)}")
    call_resp = rpc_call({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "call_tool",
        "params": {"name": "noa.list_files", "arguments": {"path": "."}},
    })
    entries = call_resp.get("result", {}).get("entries", [])
    if not entries:
        raise SystemExit("noa.list_files returned no entries")
finally:
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

stderr = proc.stderr.read()
if stderr.strip():
    sys.stderr.write(stderr)
PY

echo "[codex-mcp-health] noa.list_files succeeded via MCP stdio bridge"
