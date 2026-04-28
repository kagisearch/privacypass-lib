#!/bin/bash
# Copy built libraries to Flutter plugin
# Run this after building libraries with build_android.sh, build_ios.sh, etc.

set -e

echo "📋 Copying libraries to Flutter plugin..."
echo "=========================================="
echo ""

# Determine script location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGIN_DIR="$SCRIPT_DIR/../../../"

if [ ! -d "$PLUGIN_DIR" ]; then
    echo "⚠️  Warning: Flutter plugin directory not found at $PLUGIN_DIR"
    echo "   Run this after creating the plugin:"
    echo "   flutter create --template=plugin --platforms=android,ios privacypass_ffi"
    echo ""
    exit 1
fi

cd "$PLUGIN_DIR"

# Copy Android libraries
if [ -f "scripts/copy_android_libs.sh" ]; then
    echo "  • Copying Android libraries..."
    if bash scripts/copy_android_libs.sh; then
        echo "    ✅ Android libraries copied"
    else
        echo "    ❌ Failed to copy Android libraries"
    fi
else
    echo "  ⚠️  Android copy script not found"
fi

# Copy iOS libraries
if [ -f "scripts/copy_ios_libs.sh" ]; then
    echo "  • Copying iOS libraries..."
    if bash scripts/copy_ios_libs.sh; then
        echo "    ✅ iOS libraries copied"
    else
        echo "    ❌ Failed to copy iOS libraries"
    fi
else
    echo "  ⚠️  iOS copy script not found"
fi

# Copy macOS libraries
if [[ "$OSTYPE" == "darwin"* ]] && [ -f "scripts/copy_macos_libs.sh" ]; then
    echo "  • Copying macOS libraries..."
    if bash scripts/copy_macos_libs.sh; then
        echo "    ✅ macOS libraries copied"
    else
        echo "    ❌ Failed to copy macOS libraries"
    fi
fi

echo ""
echo "✅ Libraries copied to Flutter plugin!"
echo ""
