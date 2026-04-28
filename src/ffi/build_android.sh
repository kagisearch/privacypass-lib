#!/bin/bash
# Build Privacy Pass FFI library for all Android architectures
# Requires: Android NDK installed via Android Studio

set -e

echo "🤖 Building Privacy Pass FFI for Android..."

# Determine NDK path
if [ -z "$ANDROID_NDK_HOME" ]; then
    # Try common NDK locations
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        # Find the latest NDK version
        NDK_VERSION=$(ls -1 "$HOME/Library/Android/sdk/ndk" | sort -V | tail -n 1)
        export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/$NDK_VERSION"
        echo "📍 Found Android NDK at: $ANDROID_NDK_HOME"
    else
        echo "❌ Error: ANDROID_NDK_HOME not set and couldn't find NDK"
        echo "   Please install Android NDK via Android Studio or set ANDROID_NDK_HOME"
        exit 1
    fi
fi

# Verify NDK exists
if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "❌ Error: Android NDK not found at $ANDROID_NDK_HOME"
    exit 1
fi

# Check if cargo-ndk is installed
if ! command -v cargo-ndk &> /dev/null; then
    echo "📦 Installing cargo-ndk..."
    cargo install cargo-ndk
fi

cd "$(dirname "$0")"

echo "🔨 Building for Android targets..."
cargo ndk \
    --target aarch64-linux-android \
    --target armv7-linux-androideabi \
    --target x86_64-linux-android \
    --platform 24 \
    -- build --release

echo ""
echo "✅ Android build complete!"
echo ""
echo "📦 Libraries built:"
echo "  • arm64-v8a:    target/aarch64-linux-android/release/libkagipp_ffi.so"
echo "  • armeabi-v7a:  target/armv7-linux-androideabi/release/libkagipp_ffi.so"
echo "  • x86_64:       target/x86_64-linux-android/release/libkagipp_ffi.so"
echo ""
