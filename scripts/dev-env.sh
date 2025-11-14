#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST_PATH="${REPO_ROOT}/tools/devshell/dev-env.manifest.toml"
DEFAULT_IMAGE_TAG="noa-ark-os/dev-env:latest"
RUNTIME="${CONTAINER_RUNTIME:-}"

usage() {
  cat <<'USAGE'
Usage: scripts/dev-env.sh <command> [args]

Commands:
  build        Build or rebuild the developer container image from the manifest.
  run [cmd]    Start an interactive shell (default) or run [cmd] inside the container.
  smoke        Run a smoke test that validates core tooling inside the container.
  manifest     Print the manifest in JSON form for inspection.
  help         Show this message.

Environment variables:
  CONTAINER_RUNTIME  Override container runtime (docker or podman). Defaults to podman if available.
  DEV_ENV_IMAGE_TAG  Override the image tag used for build/run commands.
USAGE
}

ensure_runtime() {
  if [[ -n "${RUNTIME}" ]]; then
    return
  fi
  if command -v podman >/dev/null 2>&1; then
    RUNTIME="podman"
  elif command -v docker >/dev/null 2>&1; then
    RUNTIME="docker"
  else
    echo "Error: neither podman nor docker is available in PATH." >&2
    exit 1
  fi
}

manifest_json() {
  python - <<'PY' "${MANIFEST_PATH}" || exit 1
import json
import sys
import tomllib
from pathlib import Path

path = Path(sys.argv[1])
if not path.exists():
    raise SystemExit(f"Manifest not found: {path}")
with path.open('rb') as fh:
    data = tomllib.load(fh)
json.dump(data, sys.stdout, indent=2)
PY
}

generate_dockerfile() {
  python - <<'PY' "${MANIFEST_PATH}" || exit 1
import tomllib
import textwrap
import sys
from pathlib import Path

manifest = tomllib.loads(Path(sys.argv[1]).read_text())
image = manifest['image']['name']
user = manifest['user']
node = manifest['node']
rust = manifest['rust']
packages = manifest.get('packages', {}).get('apt', [])
post_create = manifest.get('post_create', {}).get('script')

apt_packages = ' '.join(packages)
node_setup_script = 'setup_lts.x'
node_desc = node.get('version', 'lts')
if node_desc.lower() != 'lts':
    node_setup_script = f"setup_{node_desc}.x"

enable_pnpm = node.get('enable_pnpm', False)
rust_profile = rust.get('profile', 'minimal')
rust_toolchain = rust.get('toolchain', 'stable')

lines = [
    f"FROM {image}",
    "",
    f"ARG USERNAME={user['name']}",
    f"ARG USER_UID={user['uid']}",
    f"ARG USER_GID={user['gid']}",
    "",
    "RUN groupadd --gid ${USER_GID} ${USERNAME} \\",
    "    && useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME}",
    "",
    "RUN apt-get update \\",
    "    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \\",
    "       ca-certificates curl gnupg" + (f" {apt_packages}" if apt_packages else "") + " \\",
    "    && rm -rf /var/lib/apt/lists/*",
    "",
    f"RUN curl -fsSL https://deb.nodesource.com/{node_setup_script} | bash - \\",
    "    && apt-get update \\",
    "    && DEBIAN_FRONTEND=noninteractive apt-get install -y nodejs \\",
    "    && rm -rf /var/lib/apt/lists/*",
]

if enable_pnpm:
    lines.append("RUN corepack enable pnpm")
else:
    lines.append("RUN corepack enable")

lines.extend([
    textwrap.dedent(f"""
    RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \\
        sh -s -- -y --default-toolchain {rust_toolchain} --profile {rust_profile}
    ENV PATH="/home/${{USERNAME}}/.cargo/bin:$PATH"
    RUN /bin/bash -lc "rustup component add rustfmt clippy"
    """).strip(),
])

lines.append("RUN mkdir -p /opt/devcontainer")

if post_create:
    lines.extend([
        f"COPY {post_create} /opt/devcontainer/post-create.sh",
        "RUN chmod +x /opt/devcontainer/post-create.sh",
    ])

lines.extend([
    "COPY tools/devshell/entrypoint.sh /opt/devcontainer/entrypoint.sh",
    "RUN chmod +x /opt/devcontainer/entrypoint.sh",
    "USER ${USERNAME}",
    "WORKDIR /workspace",
    "ENTRYPOINT [\"/opt/devcontainer/entrypoint.sh\"]",
    "CMD [\"bash\"]",
])

print('\n'.join(lines))
PY
}

build_image() {
  ensure_runtime
  local dockerfile
  dockerfile="$(mktemp)"
  trap 'rm -f "${dockerfile}"' RETURN
  generate_dockerfile >"${dockerfile}"
  local tag="${DEV_ENV_IMAGE_TAG:-${DEFAULT_IMAGE_TAG}}"
  echo "[dev-env] Building image ${tag} with ${RUNTIME}"
  ${RUNTIME} build -f "${dockerfile}" -t "${tag}" "${REPO_ROOT}"
}

ensure_image() {
  ensure_runtime
  local tag="${DEV_ENV_IMAGE_TAG:-${DEFAULT_IMAGE_TAG}}"
  if ! ${RUNTIME} image inspect "${tag}" >/dev/null 2>&1; then
    echo "[dev-env] Image ${tag} not found, building first..."
    build_image
  fi
}

run_container() {
  ensure_image
  local tag="${DEV_ENV_IMAGE_TAG:-${DEFAULT_IMAGE_TAG}}"
  local command=("bash")
  if [[ $# -gt 0 ]]; then
    command=("$@")
  fi
  local uid gid
  uid=$(id -u)
  gid=$(id -g)
  echo "[dev-env] Starting container from ${tag}"
  ${RUNTIME} run --rm -it \
    -v "${REPO_ROOT}:/workspace:Z" \
    -w /workspace \
    -e HOST_UID="${uid}" \
    -e HOST_GID="${gid}" \
    "${tag}" "${command[@]}"
}

smoke_test() {
  ensure_image
  local tag="${DEV_ENV_IMAGE_TAG:-${DEFAULT_IMAGE_TAG}}"
  echo "[dev-env] Running smoke test"
  ${RUNTIME} run --rm \
    -v "${REPO_ROOT}:/workspace:Z" \
    -w /workspace \
    -e DEV_ENV_SKIP_POST_CREATE=1 \
    "${tag}" bash -lc 'node --version && pnpm --version && rustc --version && cargo --version'
}

command="${1:-help}"
shift || true

case "${command}" in
  build)
    build_image
    ;;
  run)
    run_container "$@"
    ;;
  smoke)
    smoke_test
    ;;
  manifest)
    manifest_json
    ;;
  help|--help|-h)
    usage
    ;;
  *)
    echo "Unknown command: ${command}" >&2
    usage >&2
    exit 1
    ;;
esac
