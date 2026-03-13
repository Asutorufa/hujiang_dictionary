#!/usr/bin/env bash
set -e

TARGET=$1

if [ -z "$TARGET" ]; then
  echo "Usage: $0 <target>"
  exit 1
fi

echo "Installing cargo-zigbuild..."
# Use pip to install cargo-zigbuild quickly on CI
pip3 install cargo-zigbuild

echo "Adding rust target $TARGET..."
rustup target add $TARGET

ZIGBUILD_ARGS="--locked --release --target $TARGET"

# Check if target is macOS
if [[ "$TARGET" == *"-apple-darwin" ]]; then
  echo "Setting up macOS SDK..."
  SDK_DIR="$PWD/MacOSX11.3.sdk"
  if [ ! -d "$SDK_DIR" ]; then
    curl -L https://github.com/joseluisq/macosx-sdks/releases/download/11.3/MacOSX11.3.sdk.tar.xz | tar xJ
  fi
  export SDKROOT="$SDK_DIR"
  ZIGBUILD_ARGS="$ZIGBUILD_ARGS --sysroot $SDK_DIR"
fi

# Check if target is Windows
if [[ "$TARGET" == *"-pc-windows-gnu"* ]]; then
  echo "Windows target detected..."
fi

# Install zig if not present
if ! command -v zig &> /dev/null; then
  echo "Installing zig via npm..."
  if [ "$CI" = "true" ]; then
    npm install -g @ziglang/cli
  else
    echo "Please install zig manually for local testing."
  fi
fi

# Check if target is Android
if [[ "$TARGET" == *"-android"* ]]; then
  echo "Android target detected. Using Android NDK for C dependencies if available..."
  # Tell cmake/cc-rs to use the Android NDK for C dependencies like aws-lc-sys if needed
  if [ -n "$ANDROID_NDK_LATEST_HOME" ]; then
    export ANDROID_NDK_HOME="$ANDROID_NDK_LATEST_HOME"
  fi
  if [ -n "$ANDROID_NDK_HOME" ]; then
    export CFLAGS="$CFLAGS --sysroot=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot"
    export CXXFLAGS="$CXXFLAGS --sysroot=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot"
  fi
fi

echo "Building with cargo-zigbuild for target $TARGET..."
cargo zigbuild $ZIGBUILD_ARGS

# Strip binaries to match original action behavior
echo "Stripping binaries..."
if command -v llvm-strip &> /dev/null; then
  STRIP_CMD="llvm-strip"
else
  STRIP_CMD="strip"
fi

find "target/$TARGET/release/" -maxdepth 1 -type f -executable -exec $STRIP_CMD {} + || true
