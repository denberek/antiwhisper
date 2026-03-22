#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STAGE_DIR="$REPO_ROOT/src-tauri/vendor/llama-runtime"

PLATFORM="${TAURI_ENV_PLATFORM:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
if [[ "$PLATFORM" == "darwin" ]]; then
  PLATFORM="macos"
fi

if [[ "$PLATFORM" != "macos" ]]; then
  echo "Skipping llama runtime staging on non-macOS platform."
  exit 0
fi

ARCH="${TAURI_ENV_ARCH:-$(uname -m)}"
case "$ARCH" in
  arm64|aarch64)
    TARGET_TRIPLE="aarch64-apple-darwin"
    ;;
  x86_64)
    TARGET_TRIPLE="x86_64-apple-darwin"
    ;;
  *)
    echo "Unsupported macOS architecture: $ARCH" >&2
    exit 1
    ;;
esac

if [[ -n "${LLAMA_CPP_PREFIX:-}" ]]; then
  LLAMA_PREFIX="$LLAMA_CPP_PREFIX"
elif command -v brew >/dev/null 2>&1; then
  LLAMA_PREFIX="$(brew --prefix llama.cpp)"
else
  echo "Unable to locate llama.cpp. Install it with 'brew install llama.cpp' or set LLAMA_CPP_PREFIX." >&2
  exit 1
fi

if [[ -n "${GGML_PREFIX:-}" ]]; then
  GGML_PREFIX_RESOLVED="$GGML_PREFIX"
elif command -v brew >/dev/null 2>&1; then
  GGML_PREFIX_RESOLVED="$(brew --prefix ggml)"
else
  echo "Unable to locate ggml. Install it with 'brew install llama.cpp' or set GGML_PREFIX." >&2
  exit 1
fi

if [[ -n "${OPENSSL_PREFIX:-}" ]]; then
  OPENSSL_PREFIX_RESOLVED="$OPENSSL_PREFIX"
elif command -v brew >/dev/null 2>&1; then
  OPENSSL_PREFIX_RESOLVED="$(brew --prefix openssl@3)"
else
  echo "Unable to locate openssl@3. Install it with 'brew install openssl@3' or set OPENSSL_PREFIX." >&2
  exit 1
fi

LLAMA_SERVER_BIN="${LLAMA_SERVER_BIN:-$LLAMA_PREFIX/bin/llama-server}"
LLAMA_LIB_DIR="$LLAMA_PREFIX/lib"
GGML_LIB_DIR="$GGML_PREFIX_RESOLVED/lib"
GGML_LIBEXEC_DIR="$GGML_PREFIX_RESOLVED/libexec"
OPENSSL_LIB_DIR="$OPENSSL_PREFIX_RESOLVED/lib"

for required in \
  "$LLAMA_SERVER_BIN" \
  "$LLAMA_LIB_DIR/libllama.0.dylib" \
  "$LLAMA_LIB_DIR/libmtmd.0.dylib" \
  "$GGML_LIB_DIR/libggml.0.dylib" \
  "$GGML_LIB_DIR/libggml-base.0.dylib" \
  "$OPENSSL_LIB_DIR/libssl.3.dylib" \
  "$OPENSSL_LIB_DIR/libcrypto.3.dylib"; do
  if [[ ! -f "$required" ]]; then
    echo "Missing required runtime file: $required" >&2
    exit 1
  fi
done

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/lib" "$STAGE_DIR/libexec" "$STAGE_DIR/macos"

SERVER_STAGE="$STAGE_DIR/llama-server-$TARGET_TRIPLE"
cp "$LLAMA_SERVER_BIN" "$SERVER_STAGE"
chmod +x "$SERVER_STAGE"

cp "$OPENSSL_LIB_DIR/libssl.3.dylib" "$STAGE_DIR/lib/"
cp "$OPENSSL_LIB_DIR/libcrypto.3.dylib" "$STAGE_DIR/lib/"
cp "$LLAMA_LIB_DIR/libllama.0.dylib" "$STAGE_DIR/lib/"
cp "$LLAMA_LIB_DIR/libmtmd.0.dylib" "$STAGE_DIR/lib/"
cp "$GGML_LIB_DIR/libggml.0.dylib" "$STAGE_DIR/lib/"
cp "$GGML_LIB_DIR/libggml-base.0.dylib" "$STAGE_DIR/lib/"

shopt -s nullglob
backend_plugins=("$GGML_LIBEXEC_DIR"/*.so)
shopt -u nullglob

if [[ ${#backend_plugins[@]} -eq 0 ]]; then
  echo "No ggml backend plugins found in $GGML_LIBEXEC_DIR" >&2
  exit 1
fi

for plugin in "${backend_plugins[@]}"; do
  cp "$plugin" "$STAGE_DIR/libexec/"
  cp "$plugin" "$STAGE_DIR/macos/"
done

chmod -R u+w "$STAGE_DIR"

install_name_tool -change "$OPENSSL_LIB_DIR/libssl.3.dylib" "@loader_path/../lib/libssl.3.dylib" "$SERVER_STAGE"
install_name_tool -change "$OPENSSL_LIB_DIR/libcrypto.3.dylib" "@loader_path/../lib/libcrypto.3.dylib" "$SERVER_STAGE"
install_name_tool -change "@rpath/libmtmd.0.dylib" "@loader_path/../lib/libmtmd.0.dylib" "$SERVER_STAGE"
install_name_tool -change "@rpath/libllama.0.dylib" "@loader_path/../lib/libllama.0.dylib" "$SERVER_STAGE"
install_name_tool -change "$GGML_LIB_DIR/libggml.0.dylib" "@loader_path/../lib/libggml.0.dylib" "$SERVER_STAGE"
install_name_tool -change "$GGML_LIB_DIR/libggml-base.0.dylib" "@loader_path/../lib/libggml-base.0.dylib" "$SERVER_STAGE"

install_name_tool -id "@loader_path/libssl.3.dylib" "$STAGE_DIR/lib/libssl.3.dylib"
install_name_tool -change "$OPENSSL_LIB_DIR/libcrypto.3.dylib" "@loader_path/libcrypto.3.dylib" "$STAGE_DIR/lib/libssl.3.dylib"
install_name_tool -id "@loader_path/libcrypto.3.dylib" "$STAGE_DIR/lib/libcrypto.3.dylib"

install_name_tool -id "@loader_path/libllama.0.dylib" "$STAGE_DIR/lib/libllama.0.dylib"
install_name_tool -change "$GGML_LIB_DIR/libggml.0.dylib" "@loader_path/libggml.0.dylib" "$STAGE_DIR/lib/libllama.0.dylib"
install_name_tool -change "$GGML_LIB_DIR/libggml-base.0.dylib" "@loader_path/libggml-base.0.dylib" "$STAGE_DIR/lib/libllama.0.dylib"

install_name_tool -id "@loader_path/libmtmd.0.dylib" "$STAGE_DIR/lib/libmtmd.0.dylib"
install_name_tool -change "$GGML_LIB_DIR/libggml.0.dylib" "@loader_path/libggml.0.dylib" "$STAGE_DIR/lib/libmtmd.0.dylib"
install_name_tool -change "$GGML_LIB_DIR/libggml-base.0.dylib" "@loader_path/libggml-base.0.dylib" "$STAGE_DIR/lib/libmtmd.0.dylib"
install_name_tool -change "@rpath/libllama.0.dylib" "@loader_path/libllama.0.dylib" "$STAGE_DIR/lib/libmtmd.0.dylib"

install_name_tool -id "@loader_path/libggml.0.dylib" "$STAGE_DIR/lib/libggml.0.dylib"
install_name_tool -change "@rpath/libggml-base.0.dylib" "@loader_path/libggml-base.0.dylib" "$STAGE_DIR/lib/libggml.0.dylib"
install_name_tool -id "@loader_path/libggml-base.0.dylib" "$STAGE_DIR/lib/libggml-base.0.dylib"

if command -v codesign >/dev/null 2>&1; then
  codesign --force --sign - \
    "$SERVER_STAGE" \
    "$STAGE_DIR/lib/"*.dylib \
    "$STAGE_DIR/libexec/"*.so \
    "$STAGE_DIR/macos/"*.so >/dev/null
fi

echo "Prepared llama runtime in $STAGE_DIR"
