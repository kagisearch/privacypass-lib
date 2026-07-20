#!/bin/bash
# Build Privacy Pass FFI for macOS (universal dylib via cdylib: arm64 + x86_64)

set -e

echo "🍎 Building Privacy Pass FFI for macOS (universal)..."

# Determine script location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔨 Building for macOS (arm64 - Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin

echo "🔨 Building for macOS (x86_64 - Intel)..."
cargo build --release --target x86_64-apple-darwin

echo "🔗 Creating universal macOS dylib..."
mkdir -p ../target/universal-macos

# Combine both architectures into a single fat binary so the library runs
# natively on Apple Silicon and Intel Macs.
lipo -create \
    ../target/aarch64-apple-darwin/release/libkagipp_ffi.dylib \
    ../target/x86_64-apple-darwin/release/libkagipp_ffi.dylib \
    -output ../target/universal-macos/libkagipp_ffi.dylib

DYLIB_PATH="../target/universal-macos/libkagipp_ffi.dylib"

if [ -f "$DYLIB_PATH" ]; then
    echo "✅ macOS universal dylib built successfully!"
    echo ""
    echo "📦 Library location:"
    echo "   $DYLIB_PATH"

    # Print file info
    echo ""
    echo "📊 Library info:"
    file "$DYLIB_PATH"
    ls -lh "$DYLIB_PATH"

    # Show architecture
    echo ""
    echo "🏗️  Architecture:"
    lipo -info "$DYLIB_PATH"

else
    echo "❌ Failed to build universal dylib"
    exit 1
fi

echo ""
echo "✅ macOS build complete!"
echo ""
echo "💡 To use in Flutter, either:"
echo "   1. Copy to your Flutter app directory"
echo "   2. Reference by absolute path in Dart FFI bindings"
echo ""
