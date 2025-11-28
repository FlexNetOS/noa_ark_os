#!/usr/bin/env bash
# NOA ARK OS - Portable PowerShell bootstrapper
# Downloads PowerShell bundles for multiple platforms and records a unified manifest.

set -euo pipefail

PWSH_VERSION="${PWSH_VERSION:-7.4.5}"
TARGET_INPUT="${PWSH_TARGET_PLATFORMS:-host}"
DEFAULT_PLATFORM="${PWSH_DEFAULT_PLATFORM:-}"
INCLUDE_DESKTOP="${PWSH_INCLUDE_DESKTOP:-0}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PWSH_ROOT="$SCRIPT_DIR/pwsh-portable"
PLATFORM_ROOT="$PWSH_ROOT/platforms"
DOWNLOAD_DIR="$PWSH_ROOT/downloads"
MANIFEST_DIR="$PWSH_ROOT/manifests"
BIN_DIR="$PWSH_ROOT/bin"
mkdir -p "$DOWNLOAD_DIR" "$MANIFEST_DIR" "$BIN_DIR" "$PLATFORM_ROOT"

SUPPORTED_PLATFORMS=("linux-x64" "linux-arm64" "osx-x64" "osx-arm64" "win-x64")
DESKTOP_PLATFORMS=("win-x64" "osx-x64" "osx-arm64")

usage() {
    cat <<'EOF'
Usage: ./server/tools/setup-portable-pwsh.sh [--platforms list|all] [--help]

Options:
  --platforms <csv>   Comma-separated list of platform suffixes to download (e.g. linux-x64,win-x64).
  --platforms all     Download every supported platform archive.
    --include-desktop   Append Windows + macOS platforms to whatever --platforms selects.
  --help              Show this help message.

Environment:
  PWSH_TARGET_PLATFORMS   Same as --platforms (default: host platform only).
  PWSH_DEFAULT_PLATFORM   Override which platform powers bin/current symlinks (default: host).
    PWSH_INCLUDE_DESKTOP    When set to 1, automatically append Windows/macOS bundles so manifests enumerate desktop targets.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --platforms)
            if [[ $# -lt 2 ]]; then
                echo "--platforms requires a value" >&2
                exit 2
            fi
            TARGET_INPUT="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        --include-desktop)
            INCLUDE_DESKTOP=1
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 2
            ;;
    esac
done

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

append_platforms() {
    local -n arr_ref=$1
    shift
    for entry in "$@"; do
        arr_ref+=("$entry")
    done
}

dedupe_platforms() {
    local -n arr_ref=$1
    declare -A seen=()
    local -a unique=()
    for entry in "${arr_ref[@]}"; do
        if [[ -z "$entry" ]]; then
            continue
        fi
        if [[ -n "${seen[$entry]:-}" ]]; then
            continue
        fi
        seen["$entry"]=1
        unique+=("$entry")
    done
    arr_ref=("${unique[@]}")
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

detect_host_platform() {
    local sys arch
    sys="$(uname -s 2>/dev/null || echo unknown)"
    arch="$(uname -m 2>/dev/null || echo unknown)"
    case "$sys" in
        Linux*)
            case "$arch" in
                x86_64|amd64) echo "linux-x64" ;;
                aarch64|arm64) echo "linux-arm64" ;;
                *) echo "" ;;
            esac
            ;;
        Darwin*)
            case "$arch" in
                x86_64) echo "osx-x64" ;;
                arm64) echo "osx-arm64" ;;
                *) echo "" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            echo "win-x64"
            ;;
        *) echo "" ;;
    esac
}

resolve_platforms() {
    local input="$1"
    local -n out_ref=$2
    if [[ "$input" == "host" ]]; then
        local host
        host="$(detect_host_platform)"
        if [[ -z "$host" ]]; then
            echo "Unable to detect host platform" >&2
            exit 2
        fi
        out_ref=("$host")
        return
    fi
    if [[ "$input" == "all" ]]; then
        out_ref=("${SUPPORTED_PLATFORMS[@]}")
        return
    fi
    IFS=',' read -r -a out_ref <<< "$input"
    for idx in "${!out_ref[@]}"; do
        out_ref[$idx]="$(normalize_platform_alias "${out_ref[$idx]}")"
    done
}

ensure_platform_supported() {
    local platform="$1"
    local supported=0
    for entry in "${SUPPORTED_PLATFORMS[@]}"; do
        if [[ "$entry" == "$platform" ]]; then
            supported=1
            break
        fi
    done
    if [[ $supported -ne 1 ]]; then
        echo "Unsupported platform target: $platform" >&2
        exit 2
    fi
}

relative_to_script() {
    local target="$1"
    if command -v python3 >/dev/null 2>&1; then
        python3 - <<'PY' "$SCRIPT_DIR" "$target"
import sys
from pathlib import Path
base = Path(sys.argv[1]).resolve()
target = Path(sys.argv[2]).resolve()
print(target.relative_to(base))
PY
        return
    fi
    if command -v node >/dev/null 2>&1; then
        node - <<'JS' "$SCRIPT_DIR" "$target"
const path = require('path');
const base = path.resolve(process.argv[2]);
const tgt = path.resolve(process.argv[3]);
console.log(path.relative(base, tgt));
JS
        return
    fi
    echo "Error: python3 or node required to compute relative paths" >&2
    exit 2
}

normalize_platform_alias() {
    local token="$1"
    if [[ -z "$token" ]]; then
        echo ""
        return
    fi
    token="${token,,}"
    case "$token" in
        linux|linux64|linux-x86_64|linux-amd64)
            echo "linux-x64"
            ;;
        linux-arm|linux-arm64|linux-aarch64)
            echo "linux-arm64"
            ;;
        mac|macos|macosx|darwin|osx|osx64)
            echo "osx-x64"
            ;;
        mac-arm|macarm|mac-arm64|macos-arm64|darwin-arm64|osx-arm64)
            echo "osx-arm64"
            ;;
        win|windows|windows-x64|win64)
            echo "win-x64"
            ;;
        *)
            echo "$1"
            ;;
    esac
}

emit_manifest_entry() {
    local platform="$1"
    local archive="$2"
    local archive_sha="$3"
    local bundle_root="$4"
    local binary="$5"
    local binary_kind="$6"
    local source_url="$7"
    local binary_sha="$8"
    local generated_at="$9"
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$platform" "$archive" "$archive_sha" "$bundle_root" "$binary" "$binary_kind" "$source_url" "$binary_sha" "$generated_at" <<'PY'
import json
import sys
platform, archive, archive_sha, bundle_root, binary, binary_kind, source_url, binary_sha, generated_at = sys.argv[1:10]
print(json.dumps({
    "platform": platform,
    "archive": archive,
    "archive_sha256": archive_sha,
    "bundle_root": bundle_root,
    "binary": binary,
    "binary_kind": binary_kind,
    "source_url": source_url,
    "sha256": binary_sha,
    "generated_at": generated_at,
}))
PY
        return
    fi
    if command -v node >/dev/null 2>&1; then
        node - "$platform" "$archive" "$archive_sha" "$bundle_root" "$binary" "$binary_kind" "$source_url" "$binary_sha" "$generated_at" <<'JS'
const [,,platform, archive, archive_sha, bundle_root, binary, binary_kind, source_url, binary_sha, generated_at] = process.argv;
console.log(JSON.stringify({
  platform,
  archive,
  archive_sha256: archive_sha,
  bundle_root,
  binary,
  binary_kind,
  source_url,
  sha256: binary_sha,
  generated_at,
}));
JS
        return
    fi
    echo "Error: python3 or node required to build manifest entries" >&2
    exit 2
}

download_archive() {
    local url="$1" path="$2"
    if [[ -f "$path" ]]; then
        echo "ℹ️  Using cached archive $path"
        return
    fi
    echo "⬇️  Downloading $url"
    curl -fSL "$url" -o "$path"
}

extract_archive() {
    local archive="$1" dest="$2" ext="$3"
    mkdir -p "$dest"
    if [[ "$ext" == "zip" ]]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive" -d "$dest"
        else
            python3 - <<'PY' "$archive" "$dest"
import sys, zipfile
archive = sys.argv[1]
dest = sys.argv[2]
with zipfile.ZipFile(archive) as zf:
    zf.extractall(dest)
PY
        fi
    else
        tar -xzf "$archive" -C "$dest"
    fi
}

set_platform_metadata() {
    local platform="$1"
    case "$platform" in
        linux-*|osx-*)
            ARCHIVE_EXT="tar.gz"
            ARCHIVE_NAME="powershell-${PWSH_VERSION}-${platform}.${ARCHIVE_EXT}"
            BINARY_NAME="pwsh"
            ;;
        win-*)
            ARCHIVE_EXT="zip"
            ARCHIVE_NAME="PowerShell-${PWSH_VERSION}-${platform}.${ARCHIVE_EXT}"
            BINARY_NAME="pwsh.exe"
            ;;
        *)
            echo "Unsupported platform metadata request: $platform" >&2
            exit 2
            ;;
    esac
    SOURCE_URL="https://github.com/PowerShell/PowerShell/releases/download/v${PWSH_VERSION}/${ARCHIVE_NAME}"
}

declare -a REQUESTED_PLATFORMS
resolve_platforms "$TARGET_INPUT" REQUESTED_PLATFORMS
if [[ "$INCLUDE_DESKTOP" == "1" ]]; then
    append_platforms REQUESTED_PLATFORMS "${DESKTOP_PLATFORMS[@]}"
fi
dedupe_platforms REQUESTED_PLATFORMS
for platform in "${REQUESTED_PLATFORMS[@]}"; do
    ensure_platform_supported "$platform"
done

HOST_PLATFORM="$(detect_host_platform)"
if [[ -z "$DEFAULT_PLATFORM" ]]; then
    DEFAULT_PLATFORM="$HOST_PLATFORM"
fi

MANIFEST_ENTRIES=()

for platform in "${REQUESTED_PLATFORMS[@]}"; do
    set_platform_metadata "$platform"
    ARCHIVE_PATH="$DOWNLOAD_DIR/$ARCHIVE_NAME"
    PLATFORM_DIR="$PLATFORM_ROOT/$platform"
    TMP_DIR="$PWSH_ROOT/.extract-${platform}-$$"
    rm -rf "$TMP_DIR" "$PLATFORM_DIR"
    mkdir -p "$TMP_DIR"

    echo "📦 Preparing PowerShell ${PWSH_VERSION} (${platform})"
    download_archive "$SOURCE_URL" "$ARCHIVE_PATH"
    extract_archive "$ARCHIVE_PATH" "$TMP_DIR" "$ARCHIVE_EXT"
    ARCHIVE_SHA="$(compute_sha "$ARCHIVE_PATH")"

    mapfile -t TMP_ENTRIES < <(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -print)
    mkdir -p "$PLATFORM_DIR"
    if [[ ${#TMP_ENTRIES[@]} -eq 1 && -d "${TMP_ENTRIES[0]}" ]]; then
        mv "${TMP_ENTRIES[0]}" "$PLATFORM_DIR/"
        BUNDLE_DIR="$PLATFORM_DIR/$(basename "${TMP_ENTRIES[0]}")"
    else
        BUNDLE_DIR="$PLATFORM_DIR/${ARCHIVE_NAME%.*}"
        BUNDLE_DIR="${BUNDLE_DIR%.tar}"
        mkdir -p "$BUNDLE_DIR"
        for entry in "${TMP_ENTRIES[@]}"; do
            mv "$entry" "$BUNDLE_DIR/"
        done
    fi
    rm -rf "$TMP_DIR"

    if [[ "$platform" == win-* ]]; then
        if [[ -f "$BUNDLE_DIR/pwsh.exe" ]]; then
            PWSH_BIN="$BUNDLE_DIR/pwsh.exe"
        elif [[ -f "$BUNDLE_DIR/PowerShell.exe" ]]; then
            PWSH_BIN="$BUNDLE_DIR/PowerShell.exe"
        else
            echo "❌ pwsh.exe missing for $platform" >&2
            exit 3
        fi
    else
        PWSH_BIN="$BUNDLE_DIR/pwsh"
        if [[ ! -f "$PWSH_BIN" && -f "$BUNDLE_DIR/pwsh-preview" ]]; then
            PWSH_BIN="$BUNDLE_DIR/pwsh-preview"
        fi
        if [[ ! -f "$PWSH_BIN" ]]; then
            echo "❌ pwsh binary missing for $platform" >&2
            exit 3
        fi
        chmod +x "$PWSH_BIN" 2>/dev/null || true
    fi

    PLATFORM_BIN_DIR="$BIN_DIR/$platform"
    mkdir -p "$PLATFORM_BIN_DIR"
    if [[ "$platform" == win-* ]]; then
        ln -sfn "$PWSH_BIN" "$PLATFORM_BIN_DIR/pwsh.exe"
    else
        ln -sfn "$PWSH_BIN" "$PLATFORM_BIN_DIR/pwsh"
    fi

    if [[ "$platform" == "$DEFAULT_PLATFORM" ]]; then
        ln -sfn "$BUNDLE_DIR" "$PWSH_ROOT/current"
        if [[ "$platform" == win-* ]]; then
            ln -sfn "$PWSH_BIN" "$BIN_DIR/pwsh.exe"
        else
            ln -sfn "$PWSH_BIN" "$BIN_DIR/pwsh"
        fi
    fi

        PWSH_SHA="$(compute_sha "$PWSH_BIN")"
        BINARY_REL="$(relative_to_script "$PWSH_BIN")"
        BUNDLE_REL="$(relative_to_script "$BUNDLE_DIR")"
        ENTRY_GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
        BINARY_KIND=$( [[ "$platform" == win-* ]] && echo "pwsh.exe" || echo "pwsh" )
        cat > "$MANIFEST_DIR/pwsh-${PWSH_VERSION}-${platform}.json" <<JSON
{
    "pwsh_version": "${PWSH_VERSION}",
    "platform": "${platform}",
    "archive": "${ARCHIVE_NAME}",
    "archive_sha256": "${ARCHIVE_SHA}",
    "source_url": "${SOURCE_URL}",
    "bundle_root": "${BUNDLE_REL}",
    "binary_kind": "${BINARY_KIND}",
    "binary": "${BINARY_REL}",
    "sha256": "${PWSH_SHA}",
    "generated_at": "${ENTRY_GENERATED_AT}"
}
JSON

        manifest_entry=$(emit_manifest_entry "$platform" "$ARCHIVE_NAME" "$ARCHIVE_SHA" "$BUNDLE_REL" "$BINARY_REL" "$BINARY_KIND" "$SOURCE_URL" "$PWSH_SHA" "$ENTRY_GENERATED_AT")
        MANIFEST_ENTRIES+=("$manifest_entry")
done

if [[ -z "$DEFAULT_PLATFORM" ]]; then
    echo "DEFAULT_PLATFORM unresolved" >&2
    exit 2
fi

if [[ ! -e "$PWSH_ROOT/current" ]]; then
    echo "Default platform '$DEFAULT_PLATFORM' was not provisioned. Set PWSH_DEFAULT_PLATFORM to one of: ${REQUESTED_PLATFORMS[*]}" >&2
    exit 2
fi

MANIFEST_PATH="$PWSH_ROOT/manifest.json"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
{
    echo "{"
    echo "  \"pwsh_version\": \"${PWSH_VERSION}\"," 
    echo "  \"generated_at\": \"${GENERATED_AT}\"," 
    echo "  \"default_platform\": \"${DEFAULT_PLATFORM}\"," 
    echo "  \"platforms\": ["
    for idx in "${!MANIFEST_ENTRIES[@]}"; do
        if [[ $idx -gt 0 ]]; then
            printf ',\n'
        fi
        printf '    %s' "${MANIFEST_ENTRIES[$idx]}"
    done
    echo
    echo "  ]"
    echo "}"
} > "$MANIFEST_PATH"

archive_manifest_if_exists
cp "$MANIFEST_PATH" "$SCRIPT_DIR/pwsh-portable.manifest.json"

echo "✅ Portable PowerShell bundles ready. Default platform: ${DEFAULT_PLATFORM}"
