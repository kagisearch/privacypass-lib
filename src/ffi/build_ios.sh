#!/bin/bash
# Build Privacy Pass FFI library for iOS (device + simulator)
# Requires: Xcode Command Line Tools

set -e

echo "🍎 Building Privacy Pass FFI for iOS..."

cd "$(dirname "$0")"

# Verify Xcode is installed
if ! command -v xcodebuild &> /dev/null; then
    echo "❌ Error: Xcode not found. Please install Xcode from the App Store"
    exit 1
fi

echo "🔨 Building for iOS device (arm64)..."
cargo build --release --target aarch64-apple-ios

echo "🔨 Building for iOS simulator (x86_64 - Intel Macs)..."
cargo build --release --target x86_64-apple-ios

echo "🔨 Building for iOS simulator (arm64 - Apple Silicon)..."
cargo build --release --target aarch64-apple-ios-sim

echo "🔗 Creating universal simulator library..."
mkdir -p ../target/universal-ios-sim

# Create universal binary for simulator (supports both Intel and Apple Silicon)
lipo -create \
    ../target/x86_64-apple-ios/release/libkagipp_ffi.a \
    ../target/aarch64-apple-ios-sim/release/libkagipp_ffi.a \
    -output ../target/universal-ios-sim/libkagipp_ffi.a

echo ""
echo "✅ iOS build complete!"
echo ""
echo "📦 Libraries built:"
echo "  • Device (arm64):      ../target/aarch64-apple-ios/release/libkagipp_ffi.a"
echo "  • Simulator (universal): ../target/universal-ios-sim/libkagipp_ffi.a"
echo ""
echo "💡 Use the device library for real iPhones/iPads"
echo "💡 Use the universal simulator library for Xcode simulator"
echo ""
