#!/usr/bin/env bash
# NOA ARK OS - Portable PowerShell bootstrapper
# Downloads PowerShell into server/tools/pwsh-portable for offline, cross-platform launches.

set -euo pipefail

PWSH_VERSION="${PWSH_VERSION:-7.4.5}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PWSH_ROOT="$SCRIPT_DIR/pwsh-portable"
DOWNLOAD_DIR="$PWSH_ROOT/downloads"
MANIFEST_DIR="$PWSH_ROOT/manifests"
BIN_DIR="$PWSH_ROOT/bin"
mkdir -p "$DOWNLOAD_DIR" "$MANIFEST_DIR" "$BIN_DIR"

UNAME_S="$(uname -s)"
UNAME_M="$(uname -m)"
ARCHIVE_EXT="tar.gz"
PLATFORM_SUFFIX=""
case "$UNAME_S" in
    Linux*)
        case "$UNAME_M" in
            x86_64) PLATFORM_SUFFIX="linux-x64" ;;
            aarch64|arm64) PLATFORM_SUFFIX="linux-arm64" ;;
            *) echo "Unsupported Linux architecture: $UNAME_M" >&2; exit 2 ;;
        esac
        ;;
    Darwin*)
        case "$UNAME_M" in
            x86_64) PLATFORM_SUFFIX="osx-x64" ;;
            arm64) PLATFORM_SUFFIX="osx-arm64" ;;
            *) echo "Unsupported macOS architecture: $UNAME_M" >&2; exit 2 ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM_SUFFIX="win-x64"
        ARCHIVE_EXT="zip"
        ;;
    *)
        echo "Unsupported platform: $UNAME_S" >&2
        exit 2
        ;;
esac

if [[ -z "$PLATFORM_SUFFIX" ]]; then
    echo "Unable to determine PowerShell artifact for platform" >&2
    exit 2
fi

if [[ "$PLATFORM_SUFFIX" == win-* ]]; then
    ARCHIVE_NAME="PowerShell-${PWSH_VERSION}-${PLATFORM_SUFFIX}.${ARCHIVE_EXT}"
else
    ARCHIVE_NAME="powershell-${PWSH_VERSION}-${PLATFORM_SUFFIX}.${ARCHIVE_EXT}"
fi
ARCHIVE_BASENAME="${ARCHIVE_NAME%%.${ARCHIVE_EXT}}"
DOWNLOAD_URL="https://github.com/PowerShell/PowerShell/releases/download/v${PWSH_VERSION}/${ARCHIVE_NAME}"
ARCHIVE_PATH="$DOWNLOAD_DIR/$ARCHIVE_NAME"
EXTRACT_DIR="$PWSH_ROOT/${ARCHIVE_BASENAME}"
TMP_DIR="$PWSH_ROOT/.extract-tmp"

archive_manifest_if_exists() {
    local manifest_rel="server/tools/pwsh-portable.manifest.json"
    local current_manifest="$SCRIPT_DIR/pwsh-portable.manifest.json"
    if [[ ! -f "$current_manifest" ]]; then
        return
    fi
    local year month timestamp archive_dir archive_file
    year="$(date -u +%Y)"
    month="$(date -u +%m)"
    timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
    archive_dir="$REPO_ROOT/archive/$year/$month/$(dirname "$manifest_rel")"
    mkdir -p "$archive_dir"
    archive_file="$REPO_ROOT/archive/$year/$month/${manifest_rel}.${timestamp}.tar.zst"
    tar --zstd -cf "$archive_file" -C "$REPO_ROOT" "$manifest_rel"
}

compute_sha() {
    local target="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$target" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$target" | awk '{print $1}'
    else
        echo "unavailable"
    fi
}

echo "📦 Targeting PowerShell ${PWSH_VERSION} (${PLATFORM_SUFFIX})"
if [[ ! -f "$ARCHIVE_PATH" ]]; then
    echo "⬇️  Downloading $DOWNLOAD_URL"
    curl -fSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
else
    echo "ℹ️  Using cached archive $ARCHIVE_PATH"
fi

rm -rf "$TMP_DIR" "$EXTRACT_DIR"
mkdir -p "$TMP_DIR"
if [[ "$ARCHIVE_EXT" == "zip" ]]; then
    unzip -q "$ARCHIVE_PATH" -d "$TMP_DIR"
else
    tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"
fi

mapfile -t TMP_ENTRIES < <(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -print)
if [[ ${#TMP_ENTRIES[@]} -eq 1 && -d "${TMP_ENTRIES[0]}" ]]; then
    mv "${TMP_ENTRIES[0]}" "$EXTRACT_DIR"
else
    mkdir -p "$EXTRACT_DIR"
    for entry in "${TMP_ENTRIES[@]}"; do
        mv "$entry" "$EXTRACT_DIR/"
    done
fi
rm -rf "$TMP_DIR"

ln -sfn "$EXTRACT_DIR" "$PWSH_ROOT/current"

PWSH_BIN="$EXTRACT_DIR/pwsh"
if [[ "$PLATFORM_SUFFIX" == win-* ]]; then
    if [[ -f "$EXTRACT_DIR/pwsh.exe" ]]; then
        PWSH_BIN="$EXTRACT_DIR/pwsh.exe"
    else
        PWSH_BIN="$EXTRACT_DIR/PowerShell.exe"
    fi
fi

if [[ ! -f "$PWSH_BIN" ]]; then
    echo "❌ pwsh binary missing after extraction" >&2
    exit 3
fi

chmod +x "$PWSH_BIN" 2>/dev/null || true
if [[ "$PLATFORM_SUFFIX" == win-* ]]; then
    ln -sfn "$PWSH_BIN" "$BIN_DIR/pwsh.exe"
else
    ln -sfn "$PWSH_BIN" "$BIN_DIR/pwsh"
fi

PWSH_SHA="$(compute_sha "$PWSH_BIN")"
MANIFEST_PATH="$PWSH_ROOT/manifest.json"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
cat > "$MANIFEST_PATH" <<JSON
{
  "pwsh_version": "${PWSH_VERSION}",
  "platform": "${PLATFORM_SUFFIX}",
  "archive": "${ARCHIVE_NAME}",
  "source_url": "${DOWNLOAD_URL}",
  "binary": "${PWSH_BIN#${SCRIPT_DIR}/}",
  "sha256": "${PWSH_SHA}",
  "generated_at": "${GENERATED_AT}"
}
JSON

cp "$MANIFEST_PATH" "$MANIFEST_DIR/pwsh-${PWSH_VERSION}-${PLATFORM_SUFFIX}.json"
archive_manifest_if_exists
cp "$MANIFEST_PATH" "$SCRIPT_DIR/pwsh-portable.manifest.json"

echo "✅ Portable PowerShell ready at $PWSH_ROOT/current"
