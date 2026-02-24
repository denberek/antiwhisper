#!/usr/bin/env bash
# Download prebuilt llama-server binaries from llama.cpp GitHub releases
# and place them in src-tauri/binaries/ with Tauri sidecar naming.
#
# Usage:
#   ./scripts/download-llama-server.sh                    # Download all macOS targets
#   ./scripts/download-llama-server.sh --target aarch64-apple-darwin  # Single target
#   ./scripts/download-llama-server.sh --version b5240    # Specific version

set -euo pipefail

LLAMA_CPP_VERSION="${LLAMA_CPP_VERSION:-b8133}"
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --version) LLAMA_CPP_VERSION="$2"; shift 2 ;;
        --target)  TARGET="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$REPO_ROOT/src-tauri/binaries"
mkdir -p "$BIN_DIR"

GITHUB_BASE="https://github.com/ggml-org/llama.cpp/releases/download/${LLAMA_CPP_VERSION}"

download_binary() {
    local arch="$1"       # arm64 or x64
    local triple="$2"     # aarch64-apple-darwin or x86_64-apple-darwin
    local dest="$BIN_DIR/llama-server-${triple}"

    if [[ -f "$dest" ]]; then
        echo "Already exists: $dest"
        return
    fi

    local archive="llama-${LLAMA_CPP_VERSION}-bin-macos-${arch}.tar.gz"
    local url="${GITHUB_BASE}/${archive}"

    echo "Downloading ${archive}..."
    curl -fSL "$url" | tar xz --include='*/llama-server' -C /tmp

    mv "/tmp/llama-${LLAMA_CPP_VERSION}/llama-server" "$dest"
    chmod +x "$dest"
    echo "Installed: $dest ($(du -h "$dest" | cut -f1))"
}

if [[ -n "$TARGET" ]]; then
    case "$TARGET" in
        aarch64-apple-darwin) download_binary "arm64" "$TARGET" ;;
        x86_64-apple-darwin)  download_binary "x64"   "$TARGET" ;;
        *) echo "Unsupported target: $TARGET (only macOS targets supported)"; exit 0 ;;
    esac
else
    download_binary "arm64" "aarch64-apple-darwin"
    download_binary "x64"   "x86_64-apple-darwin"
fi

echo "Done."
