#!/usr/bin/env bash
# Cross-compile dirtree-rdm for specified targets (or all targets if none given).
# Run from the dirtree-rdm/ directory (where Cargo.toml lives).
#
# Usage:
#   bash build.sh                              # build all targets
#   bash build.sh local                        # build only the local arch target
#   bash build.sh <triple> [<triple>...]       # build specific targets
#
# Requires: cargo, cross (cargo install cross) for Linux cross-compilation.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$SCRIPT_DIR/bin"
mkdir -p "$BIN_DIR"

ALL_TARGETS=(
  "aarch64-apple-darwin:dirtree-rdm-darwin-arm64"
  "x86_64-apple-darwin:dirtree-rdm-darwin-x64"
  "x86_64-unknown-linux-gnu:dirtree-rdm-linux-x64"
  "aarch64-unknown-linux-gnu:dirtree-rdm-linux-arm64"
  "x86_64-pc-windows-msvc:dirtree-rdm-windows-x64.exe"
)

# Resolve which targets to build
if [[ $# -eq 0 ]]; then
  TARGETS=("${ALL_TARGETS[@]}")
elif [[ "$1" == "local" ]]; then
  OS="$(uname -s)"
  ARCH="$(uname -m)"
  case "$OS-$ARCH" in
    Darwin-arm64)   TARGETS=("aarch64-apple-darwin:dirtree-rdm-darwin-arm64") ;;
    Darwin-x86_64)  TARGETS=("x86_64-apple-darwin:dirtree-rdm-darwin-x64") ;;
    Linux-x86_64)   TARGETS=("x86_64-unknown-linux-gnu:dirtree-rdm-linux-x64") ;;
    Linux-aarch64)  TARGETS=("aarch64-unknown-linux-gnu:dirtree-rdm-linux-arm64") ;;
    *)
      echo "Unsupported local platform: $OS-$ARCH"
      exit 1
      ;;
  esac
else
  # Build only the named triples; look up output names from ALL_TARGETS
  TARGETS=()
  for triple in "$@"; do
    found=0
    for entry in "${ALL_TARGETS[@]}"; do
      if [[ "${entry%%:*}" == "$triple" ]]; then
        TARGETS+=("$entry")
        found=1
        break
      fi
    done
    if [[ $found -eq 0 ]]; then
      echo "Unknown target triple: $triple"
      exit 1
    fi
  done
fi

for entry in "${TARGETS[@]}"; do
  TARGET="${entry%%:*}"
  OUT_NAME="${entry##*:}"

  echo "==> building $TARGET → bin/$OUT_NAME"

  # Add target if missing
  rustup target add "$TARGET" 2>/dev/null || true

  # Linux targets require `cross` (cargo install cross) when building on macOS
  if [[ "$TARGET" == *linux* ]]; then
    if command -v cross &>/dev/null; then
      cross build --release --target "$TARGET"
    else
      echo "  SKIP: 'cross' not installed; cannot cross-compile $TARGET on this host."
      echo "        Install with: cargo install cross"
      echo "        Or build in a Linux CI environment."
      continue
    fi
  else
    cargo build --release --target "$TARGET"
  fi

  cp "target/$TARGET/release/dirtree-rdm" "$BIN_DIR/$OUT_NAME"
  echo "  → $BIN_DIR/$OUT_NAME"
done

echo ""
echo "All done. Binaries in $BIN_DIR/"
ls -lh "$BIN_DIR/"
