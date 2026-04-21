#!/bin/bash
# Master build script for Privacy Pass FFI Plugin
# Builds Rust libraries for all platforms and copies them to Flutter plugin

set -e

echo "🦀 Building Privacy Pass FFI Plugin"
echo "===================================="
echo ""

# Determine script location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FFI_DIR="$SCRIPT_DIR"
PLUGIN_DIR="$SCRIPT_DIR/../../privacypass_ffi"

cd "$FFI_DIR"

# Step 1: Build for Android
echo "📦 Step 1/4: Building Rust FFI for Android..."
echo "---------------------------------------------"
if bash build_android.sh; then
    echo "✅ Android build complete"
else
    echo "❌ Android build failed"
    exit 1
fi
echo ""

# Step 2: Build for iOS
echo "📦 Step 2/4: Building Rust FFI for iOS..."
echo "-------------------------------------------"
if bash build_ios.sh; then
    echo "✅ iOS build complete"
else
    echo "❌ iOS build failed"
    exit 1
fi
echo ""

# Step 2.5: Build for macOS (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "📦 Step 2.5/5: Building Rust FFI for macOS..."
    echo "---------------------------------------------"
    if bash build_macos.sh; then
        echo "✅ macOS build complete"
    else
        echo "⚠️  macOS build failed (continuing anyway)"
    fi
    echo ""
fi

# Step 3: Generate C headers
echo "📄 Step 3/4: Generating C headers..."
echo "-------------------------------------"
if command -v cbindgen &> /dev/null; then
    if cbindgen --config cbindgen.toml --output include/kagipp_ffi.h; then
        echo "✅ C headers generated at include/kagipp_ffi.h"
    else
        echo "⚠️  Warning: cbindgen failed, but continuing..."
    fi
else
    echo "⚠️  Warning: cbindgen not installed, skipping header generation"
    echo "   Install with: cargo install cbindgen"
fi
echo ""

# Step 4: Copy to Flutter plugin
echo "📋 Step 4/4: Copying libraries to Flutter plugin..."
echo "----------------------------------------------------"
if bash copy_to_plugin.sh; then
    echo ""
    echo "============================================"
    echo "✅ Build complete! Privacy Pass FFI ready."
    echo "============================================"
    echo ""
    echo "📱 Next steps:"
    echo "   1. cd $PLUGIN_DIR"
    echo "   2. flutter pub get"
    echo "   3. cd example && flutter run"
    echo ""
else
    echo ""
    echo "⚠️  Libraries built but not copied to plugin"
    echo "   Run ./copy_to_plugin.sh manually when ready"
    echo ""
fi
