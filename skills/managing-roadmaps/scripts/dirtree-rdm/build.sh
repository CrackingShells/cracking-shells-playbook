#!/usr/bin/env bash
# Cross-compile dirtree-rdm for all supported targets.
# Run from the dirtree-rdm/ directory (where Cargo.toml lives).
# Requires: cargo, cross (cargo install cross) or the target toolchains installed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$SCRIPT_DIR/bin"
mkdir -p "$BIN_DIR"

TARGETS=(
  "aarch64-apple-darwin:dirtree-rdm-darwin-arm64"
  "x86_64-apple-darwin:dirtree-rdm-darwin-x64"
  "x86_64-unknown-linux-gnu:dirtree-rdm-linux-x64"
  "aarch64-unknown-linux-gnu:dirtree-rdm-linux-arm64"
)

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
echo "All binaries written to $BIN_DIR/"
ls -lh "$BIN_DIR/"
