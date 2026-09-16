#!/usr/bin/env bash
# dirtree-rdm.sh — platform dispatch wrapper for the dirtree-rdm binary.
# Usage: bash path/to/dirtree-rdm.sh <command> [args...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$SCRIPT_DIR/dirtree-rdm/bin"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  Darwin-arm64)  BIN="$BIN_DIR/dirtree-rdm-darwin-arm64" ;;
  Darwin-x86_64) BIN="$BIN_DIR/dirtree-rdm-darwin-x64"   ;;
  Linux-x86_64)  BIN="$BIN_DIR/dirtree-rdm-linux-x64"    ;;
  Linux-aarch64) BIN="$BIN_DIR/dirtree-rdm-linux-arm64"  ;;
  *)
    echo "dirtree-rdm: unsupported platform $OS-$ARCH" >&2
    echo "Build from source: cd $SCRIPT_DIR/dirtree-rdm && cargo build --release" >&2
    exit 1
    ;;
esac

if [[ ! -x "$BIN" ]]; then
  echo "dirtree-rdm: binary not found at $BIN" >&2
  echo "Build from source: cd $SCRIPT_DIR/dirtree-rdm && bash build.sh" >&2
  exit 1
fi

exec "$BIN" "$@"
