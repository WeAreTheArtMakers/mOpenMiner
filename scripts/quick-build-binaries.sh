#!/bin/bash
# Quick build script - just binaries, no Tauri bundling

set -e

echo "🚀 Quick Build - Binaries Only"
echo "================================"
echo ""

# Create release directory
RELEASE_DIR="quick-release"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Build frontend
echo "🎨 Building frontend..."
pnpm install > /dev/null 2>&1
pnpm build > /dev/null 2>&1
echo "✅ Frontend built"
echo ""

# Build Raspberry Pi 5
echo "🥧 Building for Raspberry Pi 5..."
cd apps/desktop/src-tauri
cargo build --release --target aarch64-unknown-linux-gnu 2>&1 | tail -5
cd ../../..

if [ -f "apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash" ]; then
    cd apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release
    tar -czf "$OLDPWD/$RELEASE_DIR/OpenMiner-Pi5-arm64.tar.gz" openminedash
    cd "$OLDPWD"
    echo "✅ Pi5 binary packaged"
else
    echo "❌ Pi5 build failed"
fi
echo ""

# Build Windows
echo "🪟 Building for Windows..."
cd apps/desktop/src-tauri
cargo build --release --target x86_64-pc-windows-gnu 2>&1 | tail -5
cd ../../..

if [ -f "apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/openminedash.exe" ]; then
    cd apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release
    zip "$OLDPWD/$RELEASE_DIR/OpenMiner-Windows-x64.zip" openminedash.exe
    cd "$OLDPWD"
    echo "✅ Windows binary packaged"
else
    echo "❌ Windows build failed"
fi
echo ""

# Build macOS
echo "🍎 Building for macOS..."
cd apps/desktop/src-tauri
cargo build --release 2>&1 | tail -5
cd ../../..

if [ -f "apps/desktop/src-tauri/target/release/openminedash" ]; then
    cd apps/desktop/src-tauri/target/release
    tar -czf "$OLDPWD/$RELEASE_DIR/OpenMiner-macOS-arm64.tar.gz" openminedash
    cd "$OLDPWD"
    echo "✅ macOS binary packaged"
else
    echo "❌ macOS build failed"
fi
echo ""

echo "================================"
echo "✅ Build Complete!"
echo "================================"
echo ""
echo "📦 Packages in: $RELEASE_DIR/"
ls -lh "$RELEASE_DIR/"
echo ""
