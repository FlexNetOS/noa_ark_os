#!/usr/bin/env bash
# Portable Caddy bootstrapper (Linux/macOS/WSL)
set -euo pipefail

CADDY_VERSION="${CADDY_VERSION:-2.8.4}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
CADDY_ROOT="$SCRIPT_DIR/caddy-portable"
ARTIFACT_DIR="$CADDY_ROOT/artifacts"
mkdir -p "$ARTIFACT_DIR"

UNAME_S="$(uname -s)"
case "$UNAME_S" in
    Linux*) TARGET_OS="linux" ; ARCHIVE_EXT="tar.gz" ;;
    Darwin*) TARGET_OS="mac" ; ARCHIVE_EXT="tar.gz" ;;
    MINGW*|MSYS*|CYGWIN*) TARGET_OS="windows" ; ARCHIVE_EXT="zip" ;;
    *) echo "Unsupported platform: $UNAME_S" >&2; exit 1 ;;
esac

ARCHIVE_NAME="caddy_${CADDY_VERSION}_${TARGET_OS}_amd64.${ARCHIVE_EXT}"
ARCHIVE_PATH="$ARTIFACT_DIR/$ARCHIVE_NAME"
DOWNLOAD_URL="https://github.com/caddyserver/caddy/releases/download/v${CADDY_VERSION}/${ARCHIVE_NAME}"
EXTRACT_DIR="$CADDY_ROOT/caddy_${CADDY_VERSION}_${TARGET_OS}_amd64"

if [[ ! -f "$ARCHIVE_PATH" ]]; then
    echo "⬇️  Downloading Caddy ${CADDY_VERSION} for ${TARGET_OS}..."
    curl -fSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
else
    echo "ℹ️  Reusing $ARCHIVE_PATH"
fi

rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"
if [[ "$ARCHIVE_EXT" == "tar.gz" ]]; then
    tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"
else
    unzip -q "$ARCHIVE_PATH" -d "$EXTRACT_DIR"
fi

if [[ -d "$EXTRACT_DIR/caddy_${CADDY_VERSION}_${TARGET_OS}_amd64" ]]; then
    mv "$EXTRACT_DIR/caddy_${CADDY_VERSION}_${TARGET_OS}_amd64"/* "$EXTRACT_DIR/" 2>/dev/null || true
    rmdir "$EXTRACT_DIR/caddy_${CADDY_VERSION}_${TARGET_OS}_amd64" 2>/dev/null || true
fi

ln -sfn "$EXTRACT_DIR" "$CADDY_ROOT/current"

CADDY_BIN="$EXTRACT_DIR/caddy"
if [[ "$TARGET_OS" == "windows" ]]; then
    CADDY_BIN="$EXTRACT_DIR/caddy.exe"
fi

if [[ ! -f "$CADDY_BIN" ]]; then
    echo "❌ Caddy binary missing after extraction" >&2
    exit 1
fi

chmod +x "$CADDY_BIN"

declare -A HASHES
if command -v sha256sum >/dev/null 2>&1; then
    HASHES["$CADDY_BIN"]="$(sha256sum "$CADDY_BIN" | awk '{print $1}')"
fi

MANIFEST_PATH="$CADDY_ROOT/manifest.json"
cat > "$MANIFEST_PATH" <<JSON
{
  "version": "${CADDY_VERSION}",
  "target_os": "${TARGET_OS}",
  "archive": "${ARCHIVE_NAME}",
  "archive_path": "${ARCHIVE_PATH#${SCRIPT_DIR}/}",
  "caddy_binary": "${CADDY_BIN#${SCRIPT_DIR}/}",
  "sha256": "${HASHES["$CADDY_BIN"]:-unknown}",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
JSON

cp "$MANIFEST_PATH" "$SCRIPT_DIR/caddy-portable.manifest.json"

echo "✅ Portable Caddy ready at $CADDY_ROOT/current"
