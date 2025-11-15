#!/usr/bin/env bash
# NOA ARK OS - Portable Node + pnpm bootstrapper
# Downloads Node.js v20.19.5 and prepares pnpm 8.15.4 inside server/tools/node-portable

set -euo pipefail

NODE_VERSION="${NODE_VERSION:-20.19.5}"
PNPM_VERSION="${PNPM_VERSION:-8.15.4}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
NODE_ROOT="$SCRIPT_DIR/node-portable"
ARTIFACT_DIR="$NODE_ROOT/artifacts"
MANIFEST="$NODE_ROOT/manifest.json"
mkdir -p "$ARTIFACT_DIR"

PLATFORM="$(uname -s)"
case "$PLATFORM" in
    Linux*) ARCH_SUFFIX="linux-x64"; ARCHIVE_EXT="tar.xz" ;;
    Darwin*) ARCH_SUFFIX="darwin-x64"; ARCHIVE_EXT="tar.xz" ;;
    MINGW*|MSYS*|CYGWIN*) ARCH_SUFFIX="win-x64"; ARCHIVE_EXT="zip" ;;
    *)
        echo "Unsupported platform: $PLATFORM" >&2
        exit 1
        ;;
esac

ARCHIVE_NAME="node-v${NODE_VERSION}-${ARCH_SUFFIX}.${ARCHIVE_EXT}"
ARCHIVE_PATH="$ARTIFACT_DIR/$ARCHIVE_NAME"
NODE_URL="https://nodejs.org/dist/v${NODE_VERSION}/${ARCHIVE_NAME}"
EXTRACT_DIR="$NODE_ROOT/node-v${NODE_VERSION}-${ARCH_SUFFIX}"

if [[ ! -f "$ARCHIVE_PATH" ]]; then
    echo "⬇️  Downloading Node ${NODE_VERSION} for ${ARCH_SUFFIX}..."
    curl -fSL "$NODE_URL" -o "$ARCHIVE_PATH"
else
    echo "ℹ️  Node archive already present at $ARCHIVE_PATH"
fi

if [[ -d "$EXTRACT_DIR" ]]; then
    echo "ℹ️  Removing previous extracted directory $EXTRACT_DIR"
    rm -rf "$EXTRACT_DIR"
fi

if [[ "$ARCHIVE_EXT" == "tar.xz" ]]; then
    echo "📦 Extracting $ARCHIVE_NAME"
    tar -xf "$ARCHIVE_PATH" -C "$NODE_ROOT"
else
    echo "📦 Extracting $ARCHIVE_NAME"
    unzip -q "$ARCHIVE_PATH" -d "$NODE_ROOT"
fi

ln -sfn "$EXTRACT_DIR" "$NODE_ROOT/current"
NODE_BIN_DIR="$EXTRACT_DIR/bin"
COREPACK_HOME="$NODE_ROOT/corepack"
mkdir -p "$COREPACK_HOME"

if [[ ! -x "$NODE_BIN_DIR/node" && ! -x "$NODE_BIN_DIR/node.exe" ]]; then
    echo "❌ Node binary missing after extraction" >&2
    exit 1
fi

case "$ARCH_SUFFIX" in
    linux-x64)
        PNPM_ASSET="pnpm-linuxstatic-x64"
        PNPM_TARGET="$NODE_BIN_DIR/pnpm"
        PNPM_URL="https://github.com/pnpm/pnpm/releases/download/v${PNPM_VERSION}/${PNPM_ASSET}"
        ;;
    darwin-x64)
        PNPM_ASSET="pnpm-macos-x64"
        PNPM_TARGET="$NODE_BIN_DIR/pnpm"
        PNPM_URL="https://github.com/pnpm/pnpm/releases/download/v${PNPM_VERSION}/${PNPM_ASSET}"
        ;;
    win-x64)
        PNPM_ASSET="pnpm-win-x64.exe"
        PNPM_TARGET="$NODE_BIN_DIR/pnpm.exe"
        PNPM_URL="https://github.com/pnpm/pnpm/releases/download/v${PNPM_VERSION}/${PNPM_ASSET}"
        ;;
    *)
        echo "❌ No pnpm artifact mapping for $ARCH_SUFFIX" >&2
        exit 1
        ;;
esac

PNPM_ARCHIVE_PATH="$ARTIFACT_DIR/$PNPM_ASSET"
if [[ ! -f "$PNPM_ARCHIVE_PATH" ]]; then
    echo "⬇️  Downloading pnpm ${PNPM_VERSION} artifact ${PNPM_ASSET}"
    curl -fSL "$PNPM_URL" -o "$PNPM_ARCHIVE_PATH"
else
    echo "ℹ️  pnpm artifact already present"
fi

cp "$PNPM_ARCHIVE_PATH" "$PNPM_TARGET"
chmod +x "$PNPM_TARGET"

if [[ "$ARCH_SUFFIX" != "win-x64" ]]; then
    cat > "$NODE_BIN_DIR/pnpx" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
exec "$SCRIPT_DIR/pnpm" dlx "$@"
WRAPPER
    chmod +x "$NODE_BIN_DIR/pnpx"
fi

NODE_BIN="$NODE_BIN_DIR/node"
PNPM_BIN="$PNPM_TARGET"

if command -v sha256sum >/dev/null 2>&1; then
    NODE_HASH="$(sha256sum "$NODE_BIN" | awk '{print $1}')"
    PNPM_HASH="$(sha256sum "$PNPM_BIN" | awk '{print $1}')"
else
    NODE_HASH="unavailable"
    PNPM_HASH="unavailable"
fi

date -u +"%Y-%m-%dT%H:%M:%SZ" > "$NODE_ROOT/.last_sync"
cat > "$MANIFEST" <<JSON
{
  "node_version": "${NODE_VERSION}",
  "pnpm_version": "${PNPM_VERSION}",
  "platform": "${ARCH_SUFFIX}",
  "node_archive": "${ARCHIVE_NAME}",
  "node_sha256": "${NODE_HASH}",
  "pnpm_sha256": "${PNPM_HASH}",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
JSON

cp "$MANIFEST" "$SCRIPT_DIR/node-portable.manifest.json"

echo "✅ Portable Node toolchain ready at $NODE_ROOT/current"
