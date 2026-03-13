#!/usr/bin/env bash
set -e

TARGET=$1

if [ -z "$TARGET" ]; then
  echo "Usage: $0 <target>"
  exit 1
fi

echo "Installing cargo-zigbuild and pinning zig to a stable release..."
# Use pipx to install cargo-zigbuild quickly on CI avoiding PEP 668 errors on Ubuntu 24.04
if command -v pipx &> /dev/null; then
  pipx install cargo-zigbuild
  # Force downgrade of ziglang to a stable version to prevent nightly zig from breaking macOS linker
  pipx runpip cargo-zigbuild install "ziglang~=0.13.0"
else
  pip3 install "cargo-zigbuild" "ziglang~=0.13.0" || true
fi

echo "Adding rust target $TARGET..."
rustup target add $TARGET

ZIGBUILD_ARGS="--locked --release --target $TARGET"

# Determine the build command to use
BUILD_CMD="cargo zigbuild"

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
if [[ "$TARGET" == *"-pc-windows-gnu"* ]]; then
  echo "Windows target detected..."
fi

# Install zig if not present
if ! command -v zig &> /dev/null; then
  echo "Installing zig via npm..."
  if [ "$CI" = "true" ]; then
    npm install -g @ziglang/cli@0.13.0
  else
    echo "Please install zig manually for local testing."
  fi
fi

# Check if target is Android
if [[ "$TARGET" == *"-android"* ]]; then
  echo "Android target detected. Overriding zig cc with Android NDK..."
  if [ -n "$ANDROID_NDK_LATEST_HOME" ]; then
    export ANDROID_NDK_HOME="$ANDROID_NDK_LATEST_HOME"
  fi
  if [ -n "$ANDROID_NDK_HOME" ]; then
    if [[ "$TARGET" == "aarch64"* ]]; then
      NDK_TARGET="aarch64-linux-android"
    elif [[ "$TARGET" == "x86_64"* ]]; then
      NDK_TARGET="x86_64-linux-android"
    fi
    # Use the official NDK toolchain wrappers to cleanly compile ring and aws-lc-sys
    # bypassing zig cc which struggles with Android NDK sysroot headers natively.
    export TARGET_CC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/${NDK_TARGET}24-clang"
    export TARGET_CXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/${NDK_TARGET}24-clang++"
    export TARGET_AR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

    # Override the linker configured by cargo so rustc links against the correct NDK sysroot
    TARGET_ENV_VAR=$(echo $TARGET | tr '-' '_' | tr '[:lower:]' '[:upper:]')
    export CARGO_TARGET_${TARGET_ENV_VAR}_LINKER="$TARGET_CC"

    # Use standard cargo build for Android to prevent cargo-zigbuild from overwriting CC and LINKER variables
    BUILD_CMD="cargo build"
  fi
fi

echo "Running $BUILD_CMD for target $TARGET..."
$BUILD_CMD $ZIGBUILD_ARGS

# Strip binaries to match original action behavior
echo "Stripping binaries..."
if command -v llvm-strip &> /dev/null; then
  find "target/$TARGET/release/" -maxdepth 1 -type f -executable -exec llvm-strip {} + || true
else
  echo "llvm-strip not found, skipping strip to prevent format errors on cross-compiled binaries."
fi
