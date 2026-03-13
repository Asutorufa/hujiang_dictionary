#!/usr/bin/env bash
set -e

TARGET=$1

if [ -z "$TARGET" ]; then
  echo "Usage: $0 <target>"
  exit 1
fi

echo "Installing cargo-zigbuild..."
cargo install cargo-zigbuild

echo "Adding rust target $TARGET..."
rustup target add $TARGET

ZIGBUILD_ARGS="--release --target $TARGET"

# Check if target is macOS
if [[ "$TARGET" == *"-apple-darwin" ]]; then
  echo "Setting up macOS SDK..."
  SDK_DIR="$PWD/MacOSX11.3.sdk"
  if [ ! -d "$SDK_DIR" ]; then
    curl -L https://github.com/joseluisq/macosx-sdks/releases/download/11.3/MacOSX11.3.sdk.tar.xz | tar xJ
  fi
  export SDKROOT="$SDK_DIR"
fi

# Check if target is Windows
if [[ "$TARGET" == *"-pc-windows-gnu" ]]; then
  echo "Windows target detected..."
fi

# Install zig if not present
if ! command -v zig &> /dev/null; then
  echo "Installing zig via npm..."
  npm install -g @ziglang/cli
fi

echo "Building with cargo-zigbuild for target $TARGET..."
cargo zigbuild $ZIGBUILD_ARGS
