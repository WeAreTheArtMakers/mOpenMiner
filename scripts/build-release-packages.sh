#!/bin/bash
# Build release packages for all platforms

set -e

echo "🚀 Building OpenMiner Release Packages for All Platforms"
echo "=========================================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get version from Cargo.toml
VERSION=$(grep '^version' apps/desktop/src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)
echo "📦 Version: $VERSION"
echo ""

# Create release directory
RELEASE_DIR="release-packages"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Build frontend first (shared by all platforms)
echo "${BLUE}🎨 Building frontend...${NC}"
pnpm install
pnpm build
echo "${GREEN}✅ Frontend built${NC}"
echo ""

# ============================================
# 1. Build for macOS (current platform)
# ============================================
echo "${BLUE}🍎 Building for macOS...${NC}"
pnpm tauri build

# Copy macOS artifacts
if [ -d "apps/desktop/src-tauri/target/release/bundle/dmg" ]; then
    cp apps/desktop/src-tauri/target/release/bundle/dmg/*.dmg "$RELEASE_DIR/" 2>/dev/null || true
    echo "${GREEN}✅ macOS DMG copied${NC}"
fi

if [ -d "apps/desktop/src-tauri/target/release/bundle/macos" ]; then
    cd apps/desktop/src-tauri/target/release/bundle/macos
    zip -r "$OLDPWD/$RELEASE_DIR/OpenMiner-macOS-${VERSION}.app.zip" *.app
    cd "$OLDPWD"
    echo "${GREEN}✅ macOS .app bundle zipped${NC}"
fi
echo ""

# ============================================
# 2. Build for Raspberry Pi 5 (ARM64 Linux)
# ============================================
echo "${BLUE}🥧 Building for Raspberry Pi 5...${NC}"

# Check if cross-compiler is installed
if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo "${RED}❌ ARM64 cross-compiler not found${NC}"
    echo "Install with: brew install aarch64-unknown-linux-gnu"
    echo "Skipping Pi5 build..."
else
    # Build for ARM64 Linux
    cd apps/desktop/src-tauri
    cargo build --release --target aarch64-unknown-linux-gnu 2>&1 | grep -v "warning:" || true
    cd ../../..
    
    # Create tarball
    cd apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release
    tar -czf "$OLDPWD/$RELEASE_DIR/OpenMiner-Pi5-${VERSION}-arm64.tar.gz" openminedash
    cd "$OLDPWD"
    
    echo "${GREEN}✅ Raspberry Pi 5 build complete${NC}"
fi
echo ""

# ============================================
# 3. Build for Windows (x86_64)
# ============================================
echo "${BLUE}🪟 Building for Windows...${NC}"

# Check if MinGW is installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "${RED}❌ MinGW cross-compiler not found${NC}"
    echo "Install with: brew install mingw-w64"
    echo "Skipping Windows build..."
else
    # Build for Windows
    cd apps/desktop/src-tauri
    cargo build --release --target x86_64-pc-windows-gnu 2>&1 | grep -v "warning:" || true
    cd ../../..
    
    # Create zip
    cd apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release
    zip "$OLDPWD/$RELEASE_DIR/OpenMiner-Windows-${VERSION}-x64.zip" openminedash.exe
    cd "$OLDPWD"
    
    echo "${GREEN}✅ Windows build complete${NC}"
fi
echo ""

# ============================================
# Summary
# ============================================
echo "=========================================================="
echo "${GREEN}✅ Build Complete!${NC}"
echo "=========================================================="
echo ""
echo "📦 Release packages created in: $RELEASE_DIR/"
echo ""
ls -lh "$RELEASE_DIR/"
echo ""
echo "📋 Next steps:"
echo "1. Test the packages on each platform"
echo "2. Create a GitHub release: gh release create v${VERSION}"
echo "3. Upload packages: gh release upload v${VERSION} $RELEASE_DIR/*"
echo ""
