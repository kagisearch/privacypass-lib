#!/bin/bash
# Build Privacy Pass FFI for macOS (dylib via cdylib)

set -e

echo "🍎 Building Privacy Pass FFI for macOS..."

# Determine script location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Build for macOS (native architecture)
echo "🔨 Building for macOS (native - $(uname -m))..."
cargo build --release

# The cdylib will be at:
# ../target/release/libkagipp_ffi.dylib (yes, .dylib even though we specified cdylib!)

DYLIB_PATH="../target/release/libkagipp_ffi.dylib"

if [ -f "$DYLIB_PATH" ]; then
    echo "✅ macOS dylib built successfully!"
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
    echo "❌ Failed to build dylib"
    exit 1
fi

echo ""
echo "✅ macOS build complete!"
echo ""
echo "💡 To use in Flutter, either:"
echo "   1. Copy to your Flutter app directory"
echo "   2. Reference by absolute path in Dart FFI bindings"
echo ""
