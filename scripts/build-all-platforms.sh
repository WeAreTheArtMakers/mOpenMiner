#!/bin/bash
# Build script for all platforms: macOS, Linux (Pi5), and Windows

set -e

echo "🚀 Building OpenMiner for all platforms..."
echo ""

# Build for macOS (current platform)
echo "======================================"
echo "🍎 Building for macOS..."
echo "======================================"
pnpm install
pnpm tauri:build
echo "✅ macOS build complete!"
echo ""

# Build for Raspberry Pi 5 (ARM64 Linux)
echo "======================================"
echo "🥧 Building for Raspberry Pi 5..."
echo "======================================"
./scripts/build-pi5.sh
echo ""

# Build for Windows
echo "======================================"
echo "🪟 Building for Windows..."
echo "======================================"
./scripts/build-windows.sh
echo ""

echo "======================================"
echo "✅ All builds complete!"
echo "======================================"
echo ""
echo "📦 Build artifacts:"
echo "   macOS:   apps/desktop/src-tauri/target/release/bundle/"
echo "   Pi5:     apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash"
echo "   Windows: apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/openminedash.exe"
