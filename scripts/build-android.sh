#!/bin/bash
set -euo pipefail

# Build Flovenet Android APK
# Prerequisites:
#   - Android SDK at /home/x/android-sdk
#   - Android NDK 27+
#   - Rust aarch64-linux-android target installed
#
# This script:
#   1. Compiles flovenet-core for Android (arm64-v8a)
#   2. Copies the .so to the Android project
#   3. Builds the APK with Gradle

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ANDROID_DIR="$PROJECT_DIR/android"
JNILIBS_DIR="$ANDROID_DIR/app/src/main/jniLibs/arm64-v8a"
ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-/home/x/android-sdk/ndk/27.0.12077973}"

echo "==> Building Flovenet Android APK"

# Step 1: Ensure the jniLibs directory exists
mkdir -p "$JNILIBS_DIR"

# Step 2: Compile flovenet-core for Android
echo "==> Compiling flovenet-core for aarch64-linux-android..."
export ANDROID_NDK_HOME
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"

cd "$PROJECT_DIR"
cargo build --target aarch64-linux-android -p flovenet-core --release

# Step 3: Copy the .so to jniLibs
echo "==> Copying .so to jniLibs..."
cp "target/aarch64-linux-android/release/libflovenet_core.so" "$JNILIBS_DIR/"
ls -lh "$JNILIBS_DIR/libflovenet_core.so"

# Step 4: Build APK with Gradle
echo "==> Building APK..."
cd "$ANDROID_DIR"
export ANDROID_HOME="${ANDROID_HOME:-/home/x/android-sdk}"
./gradlew assembleDebug

echo "==> Done!"
echo "    APK: $ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
